use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, WebSocketConfig};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async_with_config};
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, trace};
use url::Url;

use super::{ChunkGuildFilter, GatewayError};
use crate::constants::{Opcode};
use crate::internal::prelude::*;
use crate::model::event::GatewayEvent;
use crate::model::gateway::ShardInfo;
use crate::model::id::{GuildId, UserId};
use crate::{Error, Result};

#[derive(Serialize)]
struct ChunkGuildMessage<'a> {
    guild_id: GuildId,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<&'a str>,
    limit: u16,
    presences: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_ids: Option<Vec<UserId>>,
    nonce: &'a str,
}

#[derive(Serialize)]
struct SandwichExt<'a> {
    app_name: &'a str,
}

#[derive(Serialize)]
#[serde(untagged)]
enum WebSocketMessageData<'a> {
    Heartbeat(Option<u64>),
    ChunkGuild(ChunkGuildMessage<'a>),
    Identify {
        token: &'a str,
        shard: &'a ShardInfo,
        #[serde(rename = "__sandwich_ext")]
        sandwich_ext: SandwichExt<'a>
    },
    Resume {
        session_id: &'a str,
        token: &'a str,
        seq: u64,
        #[serde(rename = "__sandwich_ext")]
        sandwich_ext: SandwichExt<'a>
    },
}

#[derive(Serialize)]
struct WebSocketMessage<'a> {
    op: Opcode,
    d: WebSocketMessageData<'a>,
}

pub struct WsClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

const TIMEOUT: Duration = Duration::from_millis(500);

impl WsClient {
    pub(crate) async fn connect(
        url: Url,
    ) -> StdResult<Self, WsError> {
        let config = {
            let mut config = WebSocketConfig::default();
            config.max_message_size = None;
            config.max_frame_size = None;

            config
        };

        let (stream, _) = connect_async_with_config(url, Some(config), false).await?;

        Ok(Self {
            stream,
        })
    }

    pub(crate) async fn recv_json(&mut self) -> Result<Option<GatewayEvent>> {
        let message = match timeout(TIMEOUT, self.stream.next()).await {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(None) | Err(_) => return Ok(None),
        };

        let json_bytes = match message {
            Message::Text(ref payload) => payload.as_bytes(),
            Message::Close(Some(frame)) => {
                return Err(Error::Gateway(GatewayError::Closed(Some(frame))));
            },
            _ => return Ok(None),
        };

        match serde_json::from_slice(json_bytes) {
            Ok(event) => Ok(Some(event)),
            Err(err) => {
                debug!("Failing text: {}", String::from_utf8_lossy(json_bytes));
                Err(Error::Json(err))
            },
        }
    }

    pub(crate) async fn send_json(&mut self, value: &impl serde::Serialize) -> Result<()> {
        let message = Message::Text(serde_json::to_string(value)?.into());

        self.stream.send(message).await?;
        Ok(())
    }

    /// Delegate to `WebSocketStream::close`
    pub(crate) async fn close(&mut self, msg: Option<CloseFrame>) -> StdResult<(), WsError> {
        self.stream.close(msg).await?;
        Ok(())
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    pub async fn send_chunk_guild(
        &mut self,
        guild_id: GuildId,
        shard_info: &ShardInfo,
        limit: Option<u16>,
        presences: bool,
        filter: ChunkGuildFilter,
        nonce: Option<&str>,
    ) -> Result<()> {
        debug!("[{:?}] Requesting member chunks", shard_info);

        let (query, user_ids) = match filter {
            ChunkGuildFilter::None => (Some(String::new()), None),
            ChunkGuildFilter::Query(query) => (Some(query), None),
            ChunkGuildFilter::UserIds(user_ids) => (None, Some(user_ids)),
        };

        self.send_json(&WebSocketMessage {
            op: Opcode::RequestGuildMembers,
            d: WebSocketMessageData::ChunkGuild(ChunkGuildMessage {
                guild_id,
                query: query.as_deref(),
                limit: limit.unwrap_or(0),
                presences,
                user_ids,
                nonce: nonce.unwrap_or(""),
            }),
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn send_heartbeat(&mut self, shard_info: &ShardInfo, seq: Option<u64>) -> Result<()> {
        trace!("[{:?}] Sending heartbeat d: {:?}", shard_info, seq);

        self.send_json(&WebSocketMessage {
            op: Opcode::Heartbeat,
            d: WebSocketMessageData::Heartbeat(seq),
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self, token)))]
    pub async fn send_identify(
        &mut self,
        shard: &ShardInfo,
        token: &str,
    ) -> Result<()> {
        debug!("[{:?}] Identifying", shard);

        let msg = WebSocketMessage {
            op: Opcode::Identify,
            d: WebSocketMessageData::Identify {
                token,
                shard,
                sandwich_ext: SandwichExt {
                    app_name: "serenity", // TODO: Support custom app names
                }
            },
        };

        self.send_json(&msg).await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self, token)))]
    pub async fn send_resume(
        &mut self,
        shard_info: &ShardInfo,
        session_id: &str,
        seq: u64,
        token: &str,
    ) -> Result<()> {
        debug!("[{:?}] Sending resume; seq: {}", shard_info, seq);

        self.send_json(&WebSocketMessage {
            op: Opcode::Resume,
            d: WebSocketMessageData::Resume {
                session_id,
                token,
                seq,
                sandwich_ext: SandwichExt {
                    app_name: "serenity", // TODO: Support custom app names
                }
            },
        })
        .await
    }
}
