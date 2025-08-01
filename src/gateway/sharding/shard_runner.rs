use std::sync::Arc;

use dashmap::DashMap;
use dashmap::try_result::TryResult;
use futures::channel::mpsc::{self, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, error, trace, warn};

use super::{Shard, ShardAction, ShardManagerMessage, ShardRunnerInfo};
use crate::gateway::client::dispatch::dispatch_model;
use crate::gateway::client::{Context, EventHandler};
use crate::gateway::{ChunkGuildFilter, GatewayError};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::model::event::GatewayEvent;
use crate::model::id::{GuildId, ShardId};

/// A runner for managing a [`Shard`] and its respective WebSocket client.
pub struct ShardRunner {
    data: Arc<dyn std::any::Any + Send + Sync>,
    event_handler: Option<Arc<dyn EventHandler>>,
    runners: Arc<DashMap<ShardId, (ShardRunnerInfo, Sender<ShardRunnerMessage>)>>,
    // channel to send messages back to the shard manager
    manager_tx: Sender<ShardManagerMessage>,
    // channel to receive messages from the shard manager and dispatches
    runner_rx: Receiver<ShardRunnerMessage>,
    // channel to send messages to the shard runner from the shard manager
    runner_tx: Sender<ShardRunnerMessage>,
    pub(crate) shard: Shard,
    pub http: Arc<Http>,
}

impl ShardRunner {
    /// Creates a new runner for a Shard.
    pub fn new(opt: ShardRunnerOptions) -> Self {
        let (tx, rx) = mpsc::unbounded();

        Self {
            data: opt.data,
            event_handler: opt.event_handler,
            runners: opt.runners,
            manager_tx: opt.manager_tx,
            runner_rx: rx,
            runner_tx: tx,
            shard: opt.shard,
            http: opt.http,
        }
    }

    /// A wrapper around [`ShardRunner::start_loop`] that starts the runner's main loop and
    /// monitors for errors.
    ///
    /// If a fatal error occurs, this will send a message to the shard manager to close the
    /// connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn run(&mut self) {
        if let Err(Error::Gateway(e)) = self.start_loop().await
            && let Err(why) = self.manager_tx.unbounded_send(ShardManagerMessage::Quit(Err(e)))
        {
            warn!("Failed to send return value: {why}");
        }
        debug!("[ShardRunner {:?}] Stopping", self.shard.shard_info());
    }

    /// Starts the runner's loop to receive events.
    ///
    /// This runs a loop that performs the following in each iteration:
    ///
    /// 1. Checks the receiver for [`ShardRunnerMessage`]s from the [`ShardManager`] and acts on any
    ///    that are received.
    ///
    /// 2. Checks if a heartbeat should be sent to the discord Gateway and sends one if needed.
    ///
    /// 3. Attempts to retrieve a message from the WebSocket, processing it into a [`GatewayEvent`].
    ///    This will block for at most 500ms before assuming there is no message available.
    ///
    /// 4. Checks to determine if the received gateway response is specifying an action to take
    ///    (e.g. resuming, reconnecting, heartbeating), or if it contains an [`Event`] to dispatch
    ///    via the client, then performs said action.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails or the intents contained disallowed values.
    ///
    /// [`ShardManager`]: super::ShardManager
    /// [`Event`]: crate::model::event::Event
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn start_loop(&mut self) -> Result<()> {
        debug!("[ShardRunner {:?}] Running", self.shard.shard_info());

        loop {
            trace!("[ShardRunner {:?}] loop iteration started.", self.shard.shard_info());
            if !self.recv().await {
                return Ok(());
            }

            // check heartbeat
            if !self.shard.do_heartbeat().await {
                warn!("[ShardRunner {:?}] Error heartbeating", self.shard.shard_info(),);

                self.restart().await;
                return Ok(());
            }

            let pre = self.shard.stage();
            let action = self.recv_event().await?;
            let post = self.shard.stage();

            if post != pre {
                self.update_runner_info();
            }

            if let Some(action) = action {
                match action {
                    ShardAction::Reconnect => {
                        if !self.reconnect().await {
                            return Ok(());
                        }
                    },
                    ShardAction::Heartbeat => {
                        if let Err(e) = self.shard.heartbeat().await {
                            debug!(
                                "[ShardRunner {:?}] Reconnecting due to error while heartbeating: {:?}",
                                self.shard.shard_info(),
                                e
                            );
                            if !self.reconnect().await {
                                return Ok(());
                            }
                        }
                    },
                    ShardAction::Identify => {
                        if let Err(e) = self.shard.identify().await {
                            debug!(
                                "[ShardRunner {:?}] Reconnecting due to error while identifying: {:?}",
                                self.shard.shard_info(),
                                e
                            );
                            if !self.reconnect().await {
                                return Ok(());
                            }
                        }
                    },
                    ShardAction::Dispatch(event) => {
                        let context = self.make_context();
                        if !self
                            .event_handler
                            .as_ref()
                            .is_none()
                        {
                            dispatch_model(
                                event,
                                context,
                                self.event_handler.clone(),
                            )
                            .await;
                        }
                    },
                }
            }

            trace!("[ShardRunner {:?}] loop iteration reached the end.", self.shard.shard_info());
        }
    }

    /// Shuts down the WebSocket client.
    ///
    /// The Shard will be in an indeterminate state after this call, especially if called after
    /// error.
    ///
    /// Therefore, the only correct code path is to exit out of the ShardRunner loop and discard the
    /// WebSocket client entirely.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn shutdown(&mut self, close_code: u16) {
        debug!("[ShardRunner {:?}] Shutting down.", self.shard.shard_info());
        // Send a Close Frame to Discord, which allows a bot to "log off"
        drop(
            self.shard
                .client
                .close(Some(CloseFrame {
                    code: close_code.into(),
                    reason: "".into(),
                }))
                .await,
        );
    }

    // Receives messages over the internal `runner_rx` channel and handles them. Will loop over all
    // queued messages until the channel is empty. Requests a restart if handling a message fails.
    //
    // Returns whether the shard runner can continue executing its main loop.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn recv(&mut self) -> bool {
        while let Ok(msg) = self.runner_rx.try_next() {
            let Some(msg) = msg else {
                // This should never happen, because `self.runner_tx` always holds a copy of the
                // other end of the channel.
                warn!(
                    "[ShardRunner {:?}] Internal channel tx dropped; restarting",
                    self.shard.shard_info(),
                );

                self.restart().await;
                return false;
            };

            let res = match msg {
                ShardRunnerMessage::Restart => {
                    self.restart().await;
                    return false;
                },
                ShardRunnerMessage::Shutdown(code) => {
                    self.shutdown(code).await;
                    return false;
                },
                ShardRunnerMessage::ChunkGuild {
                    guild_id,
                    limit,
                    presences,
                    filter,
                    nonce,
                } => {
                    self.shard
                        .chunk_guild(guild_id, limit, presences, filter, nonce.as_deref())
                        .await
                },
            };

            if let Err(why) = res {
                warn!("[ShardRunner {:?}] Websocket error: {:?}", self.shard.shard_info(), why);

                self.restart().await;
                return false;
            }
        }

        true
    }

    /// Returns a received event, as well as whether reading the potentially present event was
    /// successful.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn recv_event(&mut self) -> Result<Option<ShardAction>> {
        let gateway_event = match self.shard.client.recv_json().await {
            Ok(Some(inner)) => Ok(inner),
            Ok(None) => {
                return Ok(None);
            },
            Err(Error::Tungstenite(tung_err)) if matches!(*tung_err, TungsteniteError::Io(_)) => {
                debug!("Attempting to auto-reconnect");

                return Ok(Some(ShardAction::Reconnect));
            },
            Err(why) => Err(why),
        };

        let is_ack = matches!(gateway_event, Ok(GatewayEvent::HeartbeatAck));
        let action = match self.shard.handle_event(gateway_event) {
            Ok(action) => action,
            Err(Error::Gateway(
                why @ (GatewayError::InvalidAuthentication
                | GatewayError::InvalidApiVersion
                | GatewayError::InvalidGatewayIntents
                | GatewayError::DisallowedGatewayIntents),
            )) => {
                error!("Shard handler received fatal err: {why:?}");

                return Err(Error::Gateway(why));
            },
            Err(why) => {
                error!("Shard handler recieved err: {why:?}");
                return Ok(None);
            },
        };

        if is_ack {
            self.update_runner_info();
        }

        Ok(action)
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn reconnect(&mut self) -> bool {
        if self.shard.session_id().is_some() {
            match self.shard.resume().await {
                Ok(()) => true,
                Err(why) => {
                    warn!(
                        "[ShardRunner {:?}] Resume failed, reidentifying: {:?}",
                        self.shard.shard_info(),
                        why,
                    );

                    // Don't spam reattempts on internet connection loss
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                    self.restart().await;
                    false
                },
            }
        } else {
            self.restart().await;
            false
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    async fn restart(&mut self) {
        let shard_id = self.shard.shard_info().id;

        self.shutdown(4000).await;

        if let Err(why) = self.manager_tx.unbounded_send(ShardManagerMessage::Boot(shard_id)) {
            warn!(
                "[ShardRunner {:?}] Failed to send boot request back to shard manager: {why:?}",
                self.shard.shard_info(),
            );
        }
    }

    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    fn update_runner_info(&self) {
        if let TryResult::Present(mut entry) = self.runners.try_get_mut(&self.shard.info.id) {
            let (runner_info, _) = entry.value_mut();

            runner_info.latency = self.shard.latency();
            runner_info.stage = self.shard.stage();
        }
    }

    fn make_context(&self) -> Context {
        Context {
            data: Arc::clone(&self.data),
            shard: self.runner_tx.clone(),
            manager: self.manager_tx.clone(),
            shard_id: self.shard.shard_info().id,
            http: Arc::clone(&self.http),
            runners: Arc::clone(&self.runners),
        }
    }

    pub(super) fn runner_tx(&self) -> Sender<ShardRunnerMessage> {
        self.runner_tx.clone()
    }
}

/// Options to be passed to [`ShardRunner::new`].
pub struct ShardRunnerOptions {
    pub data: Arc<dyn std::any::Any + Send + Sync>,
    pub event_handler: Option<Arc<dyn EventHandler>>,
    pub runners: Arc<DashMap<ShardId, (ShardRunnerInfo, Sender<ShardRunnerMessage>)>>,
    pub manager_tx: Sender<ShardManagerMessage>,
    pub shard: Shard,
    pub http: Arc<Http>,
}

/// A message to send from a shard over a WebSocket.
#[derive(Debug)]
pub enum ShardRunnerMessage {
    /// Indicator that a shard should be restarted.
    Restart,
    /// Indicator that a shard should be shutdown with a specific WebSocket close code. Sending a
    /// code of 1000 or 1001 will invalidate the session and show the current user as logged off.
    /// Any other code will keep the session active until it times out.
    ///
    /// See the [Discord docs].
    ///
    /// [Discord docs]: https://discord.com/developers/docs/events/gateway#initiating-a-disconnect
    Shutdown(u16),
    /// Indicates that the client is to send a member chunk message.
    ChunkGuild {
        /// The IDs of the [`Guild`] to chunk.
        ///
        /// [`Guild`]: crate::model::guild::Guild
        guild_id: GuildId,
        /// The maximum number of members to receive [`GuildMembersChunkEvent`]s for.
        ///
        /// [`GuildMembersChunkEvent`]: crate::model::event::GuildMembersChunkEvent
        limit: Option<u16>,
        /// Used to specify if we want the presences of the matched members.
        ///
        /// Requires [`crate::model::gateway::GatewayIntents::GUILD_PRESENCES`].
        presences: bool,
        /// A filter to apply to the returned members.
        filter: ChunkGuildFilter,
        /// Optional nonce to identify [`GuildMembersChunkEvent`] responses.
        ///
        /// [`GuildMembersChunkEvent`]: crate::model::event::GuildMembersChunkEvent
        nonce: Option<String>,
    },
}
