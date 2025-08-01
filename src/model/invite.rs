//! Models for server and channel invites.

use nonmax::NonMaxU64;

use super::prelude::*;

/// Information about an invite code.
///
/// Information can not be accessed for guilds the current user is banned from.
///
/// [Discord docs](https://discord.com/developers/docs/resources/invite#invite-object).
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Invite {
    /// The approximate number of [`Member`]s in the related [`Guild`].
    pub approximate_member_count: Option<NonMaxU64>,
    /// The approximate number of [`Member`]s with an active session in the related [`Guild`].
    ///
    /// An active session is defined as an open, heartbeating WebSocket connection.
    /// These include [invisible][`OnlineStatus::Invisible`] members.
    pub approximate_presence_count: Option<NonMaxU64>,
    /// The unique code for the invite.
    pub code: FixedString,
    /// A representation of the minimal amount of information needed about the [`GuildChannel`]
    /// being invited to.
    pub channel: InviteChannel,
    /// A representation of the minimal amount of information needed about the [`Guild`] being
    /// invited to.
    pub guild: Option<InviteGuild>,
    /// A representation of the minimal amount of information needed about the [`User`] that
    /// created the invite.
    ///
    /// This can be [`None`] for invites created by Discord such as invite-widgets or vanity invite
    /// links.
    pub inviter: Option<User>,
}

/// A minimal amount of information about the channel an invite points to.
///
/// [Discord docs](https://discord.com/developers/docs/resources/invite#invite-object-example-invite-object).
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize)]
pub struct InviteChannel {
    pub id: ChannelId,
    pub name: FixedString,
    #[serde(rename = "type")]
    pub kind: ChannelType,
}

/// Subset of [`Guild`] used in [`Invite`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/invite#invite-object-example-invite-object).
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct InviteGuild {
    pub id: GuildId,
}
