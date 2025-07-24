//! Models pertaining to the gateway.

use std::num::{NonZeroU16, NonZeroU64};

use serde::ser::SerializeSeq;
use url::Url;

use super::prelude::*;
use super::utils::*;

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

/// Representation of an activity that a [`User`] is performing.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#activity-object-activity-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Activity {
    /// The ID of the application for the activity.
    #[serde(default)]
    pub application_id: Option<ApplicationId>,
    /// Images for the presence and their texts.
    pub assets: Option<ActivityAssets>,
    /// What the user is doing.
    pub details: Option<FixedString>,
    /// Activity flags describing what the payload includes.
    pub flags: Option<ActivityFlags>,
    /// Whether or not the activity is an instanced game session.
    pub instance: Option<bool>,
    /// The type of activity being performed
    #[serde(rename = "type")]
    pub kind: ActivityType,
    /// The name of the activity.
    pub name: FixedString,
    /// Information about the user's current party.
    pub party: Option<ActivityParty>,
    /// Secrets for Rich Presence joining and spectating.
    pub secrets: Option<ActivitySecrets>,
    /// The user's current party status.
    pub state: Option<FixedString>,
    /// Emoji currently used in custom status
    pub emoji: Option<ActivityEmoji>,
    /// Unix timestamps for the start and/or end times of the activity.
    pub timestamps: Option<ActivityTimestamps>,
    /// The sync ID of the activity. Mainly used by the Spotify activity type which uses this
    /// parameter to store the track ID.
    #[cfg(feature = "unstable")]
    pub sync_id: Option<FixedString>,
    /// The session ID of the activity. Reserved for specific activity types, such as the Activity
    /// that is transmitted when a user is listening to Spotify.
    #[cfg(feature = "unstable")]
    pub session_id: Option<FixedString>,
    /// The Stream URL if [`Self::kind`] is [`ActivityType::Streaming`].
    pub url: Option<Url>,
    /// The buttons of this activity.
    ///
    /// **Note**: There can only be up to 2 buttons.
    #[serde(default, deserialize_with = "deserialize_buttons")]
    pub buttons: FixedArray<ActivityButton>,
    /// Unix timestamp (in milliseconds) of when the activity was added to the user's session
    pub created_at: u64,
}

/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-buttons).

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ActivityButton {
    /// The text shown on the button.
    pub label: FixedString,
    /// The url opened when clicking the button.
    ///
    /// **Note**: Bots cannot access activity button URL.
    #[serde(default)]
    pub url: FixedString,
}

/// The assets for an activity.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-assets).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityAssets {
    /// The ID for a large asset of the activity, usually a snowflake.
    pub large_image: Option<FixedString>,
    /// Text displayed when hovering over the large image of the activity.
    pub large_text: Option<FixedString>,
    /// The ID for a small asset of the activity, usually a snowflake.
    pub small_image: Option<FixedString>,
    /// Text displayed when hovering over the small image of the activity.
    pub small_text: Option<FixedString>,
}

bitflags! {
    /// A set of flags defining what is in an activity's payload.
    ///
    /// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-flags).
    
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct ActivityFlags: u64 {
        /// Whether the activity is an instance activity.
        const INSTANCE = 1 << 0;
        /// Whether the activity is joinable.
        const JOIN = 1 << 1;
        /// Whether the activity can be spectated.
        const SPECTATE = 1 << 2;
        /// Whether a request can be sent to join the user's party.
        const JOIN_REQUEST = 1 << 3;
        /// Whether the activity can be synced.
        const SYNC = 1 << 4;
        /// Whether the activity can be played.
        const PLAY = 1 << 5;
        /// Whether the activity party is friend only.
        const PARTY_PRIVACY_FRIENDS = 1 << 6;
        /// Whether the activity party is in a voice channel.
        const PARTY_PRIVACY_VOICE_CHANNEL = 1 << 7;
        /// Whether the activity can be embedded.
        const EMBEDDED = 1 << 8;
    }
}

/// Information about an activity's party.
///
/// [Discord docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activityparty-struct).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityParty {
    /// The ID of the party.
    pub id: Option<FixedString>,
    /// Used to show the party's current and maximum size.
    pub size: Option<[u32; 2]>,
}

/// Secrets for an activity.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#activity-object-activity-secrets).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivitySecrets {
    /// The secret for joining a party.
    pub join: Option<FixedString>,
    /// The secret for a specific instanced match.
    #[serde(rename = "match")]
    pub match_: Option<FixedString>,
    /// The secret for spectating an activity.
    pub spectate: Option<FixedString>,
}

/// Representation of an emoji used in a custom status
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#activity-object-activity-emoji).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActivityEmoji {
    /// The name of the emoji.
    pub name: FixedString,
    /// The id of the emoji.
    pub id: Option<EmojiId>,
    /// Whether this emoji is animated.
    pub animated: Option<bool>,
}

enum_number! {
    /// [Discord docs](https://discord.com/developers/docs/topics/gateway-events#activity-object-activity-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    
    #[non_exhaustive]
    pub enum ActivityType {
        /// An indicator that the user is playing a game.
        Playing = 0,
        /// An indicator that the user is streaming to a service.
        Streaming = 1,
        /// An indicator that the user is listening to something.
        Listening = 2,
        /// An indicator that the user is watching something.
        Watching = 3,
        /// An indicator that the user uses custom statuses
        Custom = 4,
        /// An indicator that the user is competing somewhere.
        Competing = 5,
        _ => Unknown(u8),
    }
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

/// Information about the user of a [`Presence`] event.
///
/// Fields should be identical to those of [`User`], except that every field but `id` is
/// optional. This is currently not implemented fully.
///
/// [Discord docs](https://discord.com/developers/docs/resources/user#user-object),
/// [modification description](https://discord.com/developers/docs/topics/gateway-events#presence-update).
#[bool_to_bitflags::bool_to_bitflags]

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct PresenceUser {
    pub id: UserId,
    pub avatar: Option<ImageHash>,
    pub bot: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "discriminator")]
    pub discriminator: Option<NonZeroU16>,
    pub email: Option<FixedString>,
    pub mfa_enabled: Option<bool>,
    #[serde(rename = "username")]
    pub name: Option<FixedString<u8>>,
    pub verified: Option<bool>,
    pub public_flags: Option<UserPublicFlags>,
}

impl PresenceUser {
    /// Attempts to convert this [`PresenceUser`] instance into a [`User`].
    ///
    /// If one of [`User`]'s required fields is None in `self`, None is returned.
    #[must_use]
    pub fn into_user(self) -> Option<User> {
        let (bot, verified, mfa_enabled) = (self.bot()?, self.verified(), self.mfa_enabled());
        let mut user = User {
            avatar: self.avatar,
            discriminator: self.discriminator,
            global_name: None,
            id: self.id,
            name: self.name?,
            public_flags: self.public_flags,
            banner: None,
            accent_colour: None,
            member: None,
            locale: None,
            email: self.email,
            flags: self.public_flags.unwrap_or_default(),
            premium_type: PremiumType::None,
            __generated_flags: UserGeneratedFlags::empty(),
        };

        user.set_bot(bot);
        user.set_verified(verified);
        user.set_mfa_enabled(mfa_enabled.unwrap_or_default());

        Some(user)
    }

    /// Attempts to convert this [`PresenceUser`] instance into a [`User`].
    ///
    /// Will clone individual fields if needed.
    ///
    /// If one of [`User`]'s required fields is None in `self`, None is returned.
    #[must_use]
    pub fn to_user(&self) -> Option<User> {
        self.clone().into_user()
    }
}

/// Information detailing the current online status of a [`User`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#presence-update-presence-update-event-fields).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Presence {
    /// Data about the associated user.
    pub user: PresenceUser,
    /// The `GuildId` the presence update is coming from.
    pub guild_id: Option<GuildId>,
    /// The user's online status.
    pub status: OnlineStatus,
    /// [`User`]'s current activities.
    #[serde(default)]
    pub activities: FixedArray<Activity>,
    /// The devices a user are currently active on, if available.
    pub client_status: Option<ClientStatus>,
}

impl ExtractKey<UserId> for Presence {
    fn extract_key(&self) -> &UserId {
        &self.user.id
    }
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