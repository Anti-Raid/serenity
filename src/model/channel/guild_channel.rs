use std::{collections::HashMap, fmt};

use nonmax::{NonMaxU16, NonMaxU32};

use crate::model::prelude::*;

/// Represents the shared fields between [`GuildChannel`] and [`GuildThread`].
///
/// [Discord docs](https://discord.com/developers/docs/topics/threads#thread-fields)

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BaseGuildChannel {
    /// The Id of the guild the channel is located in.
    #[serde(default)]
    pub guild_id: GuildId,
    /// The type of the channel.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The name of the channel. (1-100 characters)
    pub name: FixedString<u16>,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
}

/// Represents a channel in a [`Guild`], excluding thread information.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object).

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GuildChannel {
    /// The shared fields between [`GuildChannel`] and [`GuildThread`].
    #[serde(flatten)]
    pub base: BaseGuildChannel,
    /// The unique ID of the channel.
    pub id: ChannelId,
    /// The Id of the parent category for a channel.
    ///
    /// **Note**: This is only available for channels in a category.
    // Technically shared, but for different purposes.
    pub parent_id: Option<ChannelId>,
    /// The bitrate of the channel.
    ///
    /// **Note**: This is only available for voice and stage channels.
    pub bitrate: Option<NonMaxU32>,
    /// Permission overwrites for [`Member`]s and for [`Role`]s.
    #[serde(default)]
    pub permission_overwrites: FixedArray<PermissionOverwrite>,
    /// The position of the channel.
    ///
    /// The default text channel will _almost always_ have a position of `0`.
    #[serde(default)]
    pub position: u16,
    /// The topic of the channel.
    ///
    /// **Note**: This is only available for text, forum and stage channels.
    pub topic: Option<FixedString<u16>>,
    /// The maximum number of members allowed in the channel.
    ///
    /// This is max 99 for voice channels and 10,000 for stage channels (0 refers to no limit).
    pub user_limit: Option<NonMaxU16>,
    /// Used to tell if the channel is not safe for work.
    // This field can or can not be present sometimes, but if it isn't default to `false`.
    #[serde(default)]
    pub nsfw: bool,
    /// The region override.
    ///
    /// **Note**: This is only available for voice and stage channels. [`None`] for voice and stage
    /// channels means automatic region selection.
    pub rtc_region: Option<FixedString<u8>>,
    /// The video quality mode for a voice channel.
    pub video_quality_mode: Option<VideoQualityMode>,
    /// Default duration for newly created threads, in minutes, to automatically archive the thread
    /// after recent activity.
    pub default_auto_archive_duration: Option<AutoArchiveDuration>,
    /// Computed permissions for the invoking user in the channel, including overwrites.
    ///
    /// Only included inside [`CommandDataResolved`].
    pub permissions: Option<Permissions>,
    /// Extra information about the channel
    ///
    /// **Note**: This is only available in forum channels.
    #[serde(default)]
    pub flags: ChannelFlags,
    /// The set of available tags.
    ///
    /// **Note**: This is only available in forum channels.
    #[serde(default)]
    pub available_tags: FixedArray<ForumTag>,
    /// The emoji to show in the add reaction button
    ///
    /// **Note**: This is only available in a forum.
    pub default_reaction_emoji: Option<ForumEmoji>,
    /// The initial `rate_limit_per_user` to set on newly created threads in a channel. This field
    /// is copied to the thread at creation time and does not live update.
    ///
    /// **Note**: This is only available in a forum or text channel.
    pub default_thread_rate_limit_per_user: Option<NonMaxU16>,
    /// The status of a voice channel.
    ///
    /// **Note**: This is only available in voice channels.
    pub status: Option<FixedString<u16>>,
    /// The default sort order type used to order posts
    ///
    /// **Note**: This is only available in a forum.
    pub default_sort_order: Option<SortOrder>,
    /// The default forum layout view used to display posts in a forum. Defaults to 0, which
    /// indicates a layout view has not been set by a channel admin.
    ///
    /// **Note**: This is only available in a forum.
    pub default_forum_layout: Option<ForumLayoutType>,
}

enum_number! {
    /// See [`GuildChannel::default_forum_layout`].
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-object-forum-layout-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    
    #[non_exhaustive]
    pub enum ForumLayoutType {
        /// No default has been set for forum channel.
        NotSet = 0,
        /// Display posts as a list.
        ListView = 1,
        /// Display posts as a collection of tiles.
        GalleryView = 2,
        _ => Unknown(u8),
    }
}

#[cfg(feature = "model")]
impl GuildChannel {
    /// Whether or not this channel is text-based, meaning that it is possible to send messages.
    #[must_use]
    pub fn is_text_based(&self) -> bool {
        matches!(
            self.base.kind,
            ChannelType::Text
                | ChannelType::News
                | ChannelType::Voice
                | ChannelType::Stage
                | ChannelType::PublicThread
                | ChannelType::PrivateThread
                | ChannelType::NewsThread
        )
    }
}

impl fmt::Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl ExtractKey<ChannelId> for GuildChannel {
    fn extract_key(&self) -> &ChannelId {
        &self.id
    }
}
