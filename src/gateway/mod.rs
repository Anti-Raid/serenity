//! Contains the necessary plumping for maintaining a connection with Discord.
//! The primary building blocks are the [`Client`] and the [`Shard`].
//!
//! The [`Client`] is a high-level interface that takes care of communicating with Discord's REST
//! API as well as receiving and dispatching events from the gateway using a WebSocket client.
//!
//! On the other hand, the [`Shard`] is a low-level receiver and sender representing a single
//! connection to Discord. The client will handle shard management automatically for you, so you
//! should only care about using it directly if you really need to. See the [`sharding`] module for
//! details and documentation.
//!
//! [`Client`]: client::Client

pub mod client;
mod error;
pub mod sharding;
mod ws;

pub use self::error::Error as GatewayError;
pub use self::sharding::*;
pub use self::ws::WsClient;
use crate::model::id::UserId;

/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#request-guild-members).
#[derive(Clone, Debug)]
pub enum ChunkGuildFilter {
    /// Returns all members of the guilds specified. Requires GUILD_MEMBERS intent.
    None,
    /// A common username prefix filter for the members returned.
    ///
    /// Will return a maximum of 100 members.
    Query(String),
    /// A set of exact user IDs to query for.
    ///
    /// Will return a maximum of 100 members.
    UserIds(Vec<UserId>),
}
