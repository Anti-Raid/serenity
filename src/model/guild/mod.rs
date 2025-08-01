//! Models relating to guilds and types that it owns.

pub mod automod;
mod guild_id;
mod member;
mod partial_guild;
mod premium_tier;
mod role;
mod system_channel;
mod welcome_screen;

pub use self::guild_id::*;
pub use self::member::*;
pub use self::partial_guild::*;
pub use self::premium_tier::*;
pub use self::role::*;
pub use self::system_channel::*;
pub use self::welcome_screen::*;
use crate::model::prelude::*;

/// A representation of a banning of a user.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#ban-object).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Ban {
    /// The reason given for this ban.
    pub reason: Option<FixedString>,
    /// The user that was banned.
    pub user: User,
}

/// The response from [`GuildId::bulk_ban`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#bulk-guild-ban).

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BulkBanResponse {
    /// The users that were successfully banned.
    pub banned_users: Vec<UserId>,
    /// The users that were not successfully banned.
    pub failed_users: Vec<UserId>,
}


#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AfkMetadata {
    /// Id of a voice channel that's considered the AFK channel.
    pub afk_channel_id: ChannelId,
    /// The amount of seconds a user can not show any activity in a voice channel before being
    /// moved to an AFK channel -- if one exists.
    pub afk_timeout: AfkTimeout,
}

/// Data for an unavailable guild.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#unavailable-guild-object).

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UnavailableGuild {
    /// The Id of the [`Guild`] that may be unavailable.
    pub id: GuildId,
    /// Indicator of whether the guild is unavailable.
    #[serde(default)]
    pub unavailable: bool,
}

enum_number! {
    /// Default message notification level for a guild.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-default-message-notification-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    
    #[non_exhaustive]
    pub enum DefaultMessageNotificationLevel {
        /// Receive notifications for everything.
        All = 0,
        /// Receive only mentions.
        Mentions = 1,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Setting used to filter explicit messages from members.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-explicit-content-filter-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    
    #[non_exhaustive]
    pub enum ExplicitContentFilter {
        /// Don't scan any messages.
        None = 0,
        /// Scan messages from members without a role.
        WithoutRole = 1,
        /// Scan messages sent by all members.
        All = 2,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Multi-Factor Authentication level for guild moderators.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-mfa-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    
    #[non_exhaustive]
    pub enum MfaLevel {
        /// MFA is disabled.
        None = 0,
        /// MFA is enabled.
        Elevated = 1,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The level to set as criteria prior to a user being able to send
    /// messages in a [`Guild`].
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-verification-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    
    #[non_exhaustive]
    pub enum VerificationLevel {
        /// Does not require any verification.
        None = 0,
        /// Must have a verified email on the user's Discord account.
        Low = 1,
        /// Must also be a registered user on Discord for longer than 5 minutes.
        Medium = 2,
        /// Must also be a member of the guild for longer than 10 minutes.
        High = 3,
        /// Must have a verified phone on the user's Discord account.
        Higher = 4,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The [`Guild`] nsfw level.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-guild-nsfw-level).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    
    #[non_exhaustive]
    pub enum NsfwLevel {
        /// The nsfw level is not specified.
        Default = 0,
        /// The guild is considered as explicit.
        Explicit = 1,
        /// The guild is considered as safe.
        Safe = 2,
        /// The guild is age restricted.
        AgeRestricted = 3,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// The [`Guild`] AFK timeout length.
    ///
    /// See [AfkMetadata::afk_timeout].
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    
    #[non_exhaustive]
    pub enum AfkTimeout {
        OneMinute = 60,
        FiveMinutes = 300,
        FifteenMinutes = 900,
        ThirtyMinutes = 1800,
        OneHour = 3600,
        _ => Unknown(u16),
    }
}