use std::sync::Arc;

use dashmap::DashMap;
use futures::channel::mpsc::UnboundedSender as Sender;
use crate::gateway::{
    ChunkGuildFilter,
    ShardManagerMessage,
    ShardRunnerInfo,
    ShardRunnerMessage,
};
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;

/// A general utility struct provided on event dispatches.
///
/// The [`Context`] helps with dealing with the current "context" of the event dispatch. It also
/// acts as a general high-level interface over the low-level [`http`] module, plus the associated
/// [`Shard`] which received the event.
///
/// The context contains "shortcuts", like for interacting with the shard. Methods like
/// [`Self::set_activity`] will unlock the shard and perform an update for you to save a bit of
/// work.
///
/// A context will only live for the event it was dispatched for. After the event handler finished,
/// it is destroyed and will not be re-used.
///
/// [`Shard`]: crate::gateway::Shard
/// [`http`]: crate::http
#[derive(Clone)]
pub struct Context {
    /// A clone of [`Client::data`]. Refer to its documentation for more information.
    ///
    /// [`Client::data`]: super::Client::data
    pub(crate) data: Arc<dyn std::any::Any + Send + Sync>,
    /// The channel to communicate with the shard runner.
    pub(crate) shard: Sender<ShardRunnerMessage>,
    /// The channel to communicate with the shard manager.
    pub(crate) manager: Sender<ShardManagerMessage>,
    /// The ID of the shard this context is related to.
    pub shard_id: ShardId,
    pub http: Arc<Http>,
    /// Metadata about the initialised shards, and their control channels.
    pub runners: Arc<DashMap<ShardId, (ShardRunnerInfo, Sender<ShardRunnerMessage>)>>,
}

impl CacheHttp for Context {
    fn http(&self) -> &Http {
        &self.http
    }
}

impl Context {
    /// A container for a data type that can be used across contexts.
    ///
    /// The purpose of the data field is to be accessible and persistent across contexts; that is,
    /// data can be modified by one context, and will persist through the future and be accessible
    /// through other contexts. This is useful for anything that should "live" through the program:
    /// counters, database connections, custom user caches, etc.
    ///
    /// # Panics
    /// Panics if the generic provided is not equal to the type provided in [`ClientBuilder::data`].
    ///
    /// [`ClientBuilder::data`]: super::ClientBuilder::data
    #[must_use]
    pub fn data<Data: Send + Sync + 'static>(&self) -> Arc<Data> {
        Arc::clone(&self.data)
            .downcast()
            .expect("Type provided to Context should be the same as ClientBuilder::data.")
    }

    /// A version of [`Self::data`] which returns a reference to the Data.
    ///
    /// This is useful if you need to borrow `Data` with the lifetime of `Context`, but otherwise
    /// [`Self::data`] should be used.
    #[must_use]
    #[expect(clippy::needless_lifetimes, reason = "Easier to understand when explicitly written")]
    pub fn data_ref<'a, Data: Send + Sync + 'static>(&'a self) -> &'a Data {
        self.data
            .downcast_ref()
            .expect("Type provided to Context should be the same as ClientBuilder::data.")
    }

    /// Requests that one or multiple [`Guild`]s be chunked.
    ///
    /// This will ask the gateway to start sending member chunks for large guilds. If a guild is
    /// large enough, then a full member list will not be provided upon connection, and must
    /// instead be requested directly. The full list will be sent in "chunks" until all members
    /// matching the request have been sent.
    ///
    /// Member chunks are sent as the [`Event::GuildMembersChunk`] event. Each chunk only contains
    /// a partial amount of the total members.
    ///
    /// # Examples
    ///
    /// Chunk a single guild, limiting to 2000 [`Member`]s, and not specifying a query
    /// parameter:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// # use serenity::gateway::client::FullEvent;
    /// # use serenity::gateway::{ChunkGuildFilter, Shard};
    /// # use serenity::all::GuildId;
    ///
    /// # struct Handler;
    /// #
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
    ///         match event {
    ///             FullEvent::Ready {
    ///                 ..
    ///             } => {
    ///                 ctx.chunk_guild(
    ///                     GuildId::new(81384788765712384),
    ///                     Some(2000),
    ///                     false,
    ///                     ChunkGuildFilter::None,
    ///                     None,
    ///                 );
    ///             },
    ///             _ => {},
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Chunk a single guild by Id, limiting to 20 members, specifying a query parameter of `"do"`
    /// and a nonce of `"request"`:
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::*;
    /// # use serenity::model::gateway::Ready;
    /// # use serenity::gateway::client::FullEvent;
    /// # use serenity::gateway::{ChunkGuildFilter, Shard};
    /// # use serenity::all::GuildId;
    ///
    /// # struct Handler;
    /// #
    ///
    /// #[serenity::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
    ///         match event {
    ///             FullEvent::Ready {
    ///                 ..
    ///             } => {
    ///                 ctx.chunk_guild(
    ///                     GuildId::new(81384788765712384),
    ///                     Some(20),
    ///                     false,
    ///                     ChunkGuildFilter::Query("do".to_owned()),
    ///                     Some("request".to_string()),
    ///                 );
    ///             },
    ///             _ => {},
    ///         }
    ///     }
    /// }
    /// ```
    pub fn chunk_guild(
        &self,
        guild_id: GuildId,
        limit: Option<u16>,
        presences: bool,
        filter: ChunkGuildFilter,
        nonce: Option<String>,
    ) {
        self.send_to_shard(ShardRunnerMessage::ChunkGuild {
            guild_id,
            limit,
            presences,
            filter,
            nonce,
        });
    }

    /// Indicates to the gateway that the client wants to join, move, or disconnect from a voice
    /// channel.
    #[cfg(feature = "voice")]
    pub fn update_voice_state(
        &self,
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        self_mute: bool,
        self_deaf: bool,
    ) {
        self.send_to_shard(ShardRunnerMessage::UpdateVoiceState {
            guild_id,
            channel_id,
            self_mute,
            self_deaf,
        });
    }

    /// Sends a message to the shard.
    fn send_to_shard(&self, msg: ShardRunnerMessage) {
        if let Err(e) = self.shard.unbounded_send(msg) {
            tracing::warn!("failed to send ShardRunnerMessage to shard: {}", e);
        }
    }

    /// Sends a message back to the shard manager to shutdown all currently running shards,
    /// including this one.
    pub fn shutdown_all(&self) {
        if let Err(e) = self.manager.unbounded_send(ShardManagerMessage::Quit(Ok(()))) {
            tracing::warn!("failed to send shutdown request to shard manager: {}", e);
        }
    }
}
