//! Models pertaining to the gateway.

use std::num::{NonZeroU16, NonZeroU64};

use serde::ser::SerializeSeq;

use super::prelude::*;

/// A representation of the data retrieved from the bot gateway endpoint.
///
/// This is different from the [`Gateway`], as this includes the number of shards that Discord
/// recommends to use for a bot user.
///
/// This is only applicable to bot users.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#get-gateway-bot-json-response).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BotGateway {
    /// The gateway to connect to.
    pub url: FixedString,
    /// The number of shards that is recommended to be used by the current bot user.
    pub shards: NonZeroU16,
    /// Information describing how many gateway sessions you can initiate within a ratelimit
    /// period.
    pub session_start_limit: SessionStartLimit,
}

/// A representation of the data retrieved from the gateway endpoint.
///
/// For the bot-specific gateway, refer to [`BotGateway`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#get-gateway-example-response).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Gateway {
    /// The gateway to connect to.
    pub url: FixedString,
}

/// Information detailing the current active status of a [`User`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#client-status-object).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ClientStatus {
    pub desktop: Option<OnlineStatus>,
    pub mobile: Option<OnlineStatus>,
    pub web: Option<OnlineStatus>,
}

/// An initial set of information given after IDENTIFYing to the gateway.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#ready-ready-event-fields).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Ready {
    /// API version
    #[serde(rename = "v")]
    pub version: u8,
    /// Information about the user including email
    pub user: CurrentUser,
    /// Guilds the user is in
    pub guilds: FixedArray<UnavailableGuild>,
    /// Used for resuming connections
    pub session_id: FixedString,
    /// Gateway URL for resuming connections
    pub resume_gateway_url: FixedString,
    /// Shard information associated with this session, if sent when identifying
    pub shard: Option<ShardInfo>,
    /// Contains id and flags
    pub application: PartialCurrentApplicationInfo,
}

/// Information describing how many gateway sessions you can initiate within a ratelimit period.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#session-start-limit-object-session-start-limit-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SessionStartLimit {
    /// The number of sessions that you can still initiate within the current ratelimit period.
    pub remaining: u64,
    /// The number of milliseconds until the ratelimit period resets.
    pub reset_after: u64,
    /// The total number of session starts within the ratelimit period allowed.
    pub total: u64,
    /// The number of identify requests allowed per 5 seconds.
    ///
    /// This is almost always 1, but for large bots (in more than 150,000 servers) it can be
    /// larger.
    pub max_concurrency: NonZeroU16,
}


#[derive(Clone, Copy, Debug)]
pub struct ShardInfo {
    pub id: ShardId,
    pub total: NonZeroU16,
}

impl Default for ShardInfo {
    fn default() -> Self {
        Self {
            id: ShardId(1),
            total: NonZeroU16::MIN,
        }
    }
}

impl<'de> serde::Deserialize<'de> for ShardInfo {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        <(u16, NonZeroU16)>::deserialize(deserializer).map(|(id, total)| ShardInfo {
            id: ShardId(id),
            total,
        })
    }
}

impl serde::Serialize for ShardInfo {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.id.0)?;
        seq.serialize_element(&self.total)?;
        seq.end()
    }
}

/// Timestamps of when a user started and/or is ending their activity.
///
/// [Discord docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activitytimestamps-struct).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityTimestamps {
    pub end: Option<NonZeroU64>,
    pub start: Option<NonZeroU64>,
}