use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::*;
use crate::internal::prelude::*;

impl ThreadId {
    /// Converts the type of this Id to [`GenericChannelId`].
    ///
    /// This allows you to call methods which are shared between channels and threads, and does not
    /// change the inner value at all.
    #[must_use]
    pub fn widen(self) -> GenericChannelId {
        self.into()
    }
}


#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildThread {
    /// The shared fields between [`GuildChannel`] and [`GuildThread`].
    #[serde(flatten)]
    pub base: BaseGuildChannel,
    /// The Id of the thread.
    pub id: ThreadId,
    /// The Id of the parent text channel.
    pub parent_id: ChannelId,
    /// The Id of the user who created this thread
    pub owner_id: UserId,
    /// An approximate count of users in a thread, stops counting at 50.
    pub member_count: u8,
    /// An approximate count of messages in the thread.
    pub message_count: u32,
    /// The thread metadata.
    pub thread_metadata: ThreadMetadata,
    /// Thread member object for the current user, if they have joined the thread.
    ///
    /// This is only included on certain API endpoints.
    pub member: Option<PartialThreadMember>,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
}

impl ExtractKey<ThreadId> for GuildThread {
    fn extract_key(&self) -> &ThreadId {
        &self.id
    }
}

/// A thread data.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#thread-metadata-object).
#[bool_to_bitflags::bool_to_bitflags]

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct ThreadMetadata {
    /// Whether the thread is archived.
    pub archived: bool,
    /// Duration in minutes to automatically archive the thread after recent activity.
    pub auto_archive_duration: AutoArchiveDuration,
    /// When a thread is locked, only users with `MANAGE_THREADS` permission can unarchive it.
    #[serde(default)]
    pub locked: bool,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
}

/// A partial guild thread.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object),
/// [subset description](https://discord.com/developers/docs/topics/gateway#thread-delete)

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialGuildThread {
    /// The thread Id.
    pub id: ThreadId,
    /// The thread guild Id.
    pub guild_id: GuildId,
    /// The parent text channel Id.
    pub parent_id: ChannelId,
    /// The channel type.
    #[serde(rename = "type")]
    pub kind: ChannelType,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialThreadMember {
    /// The time the current user last joined the thread.
    pub join_timestamp: DateTime<Utc>,
    /// Any user-thread settings, currently only used for notifications
    pub flags: ThreadMemberFlags,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
}

/// A model representing a user in a Guild Thread.
///
/// [Discord docs], [extra fields].
///
/// [Discord docs]: https://discord.com/developers/docs/resources/channel#thread-member-object,
/// [extra fields]: https://discord.com/developers/docs/topics/gateway-events#thread-member-update-thread-member-update-event-extra-fields

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadMember {
    #[serde(flatten)]
    pub inner: PartialThreadMember,
    /// The id of the thread.
    pub id: ThreadId,
    /// The id of the user.
    pub user_id: UserId,
    /// Additional information about the user.
    ///
    /// This field is only present when `with_member` is set to `true` when calling
    /// List Thread Members or Get Thread Member, or inside [`ThreadMembersUpdateEvent`].
    pub member: Option<Member>,
    /// ID of the guild.
    ///
    /// Always present in [`ThreadMemberUpdateEvent`], otherwise `None`.
    pub guild_id: Option<GuildId>,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
}

bitflags! {
    /// Describes extra features of the message.
    ///
    /// Discord docs: flags field on [Thread Member](https://discord.com/developers/docs/resources/channel#thread-member-object).
    
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct ThreadMemberFlags: u64 {
        // Not documented.
        const NOTIFICATIONS = 1 << 0;
    }
}
