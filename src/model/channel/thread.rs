use super::*;
use crate::internal::prelude::*;
use crate::model::utils::is_false;

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
    /// The number of messages ever sent in a thread, it's similar to `message_count` on message
    /// creation, but will not decrement the number when a message is deleted.
    pub total_message_sent: u32,
    /// The set of applied tags.
    ///
    /// **Note**: This is only available in a thread in a forum.
    #[serde(default)]
    pub applied_tags: FixedArray<ForumTagId>,
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

#[derive(Clone, Copy, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct ThreadMetadata {
    /// Whether the thread is archived.
    pub archived: bool,
    /// Duration in minutes to automatically archive the thread after recent activity.
    pub auto_archive_duration: AutoArchiveDuration,
    /// The last time the thread's archive status was last changed; used for calculating recent
    /// activity.
    pub archive_timestamp: Option<Timestamp>,
    /// When a thread is locked, only users with `MANAGE_THREADS` permission can unarchive it.
    #[serde(default)]
    pub locked: bool,
    /// Timestamp when the thread was created.
    ///
    /// **Note**: only populated for threads created after 2022-01-09
    pub create_timestamp: Option<Timestamp>,
    /// Whether non-moderators can add other non-moderators to a thread.
    ///
    /// **Note**: Only available on private threads.
    #[serde(default, skip_serializing_if = "is_false")]
    pub invitable: bool,
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
}


#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialThreadMember {
    /// The time the current user last joined the thread.
    pub join_timestamp: Timestamp,
    /// Any user-thread settings, currently only used for notifications
    pub flags: ThreadMemberFlags,
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
    // According to https://discord.com/developers/docs/topics/gateway-events#thread-members-update,
    // > the thread member objects will also include the guild member and nullable presence objects
    // > for each added thread member
    // Which implies that ThreadMember has a presence field. But https://discord.com/developers/docs/resources/channel#thread-member-object
    // says that's not true. I'm not adding the presence field here for now
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
