use std::fmt;
use crate::model::prelude::*;

/// Information about a member of a guild.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object),
/// [extra fields](https://discord.com/developers/docs/topics/gateway-events#guild-member-add-guild-member-add-extra-fields).
#[bool_to_bitflags::bool_to_bitflags]

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Member {
    /// Attached User struct.
    pub user: User,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<FixedString<u8>>,
    /// The guild avatar hash
    pub avatar: Option<ImageHash>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: FixedArray<RoleId>,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<Timestamp>,
    /// Timestamp representing the date since the member is boosting the guild.
    pub premium_since: Option<Timestamp>,
    /// Indicator of whether the member can hear in voice channels.
    pub deaf: bool,
    /// Indicator of whether the member can speak in voice channels.
    pub mute: bool,
    /// Guild member flags.
    pub flags: GuildMemberFlags,
    /// Indicator that the member hasn't accepted the rules of the guild yet.
    #[serde(default)]
    pub pending: bool,
    /// The total permissions of the member in a channel, including overrides.
    ///
    /// This is only [`Some`] when returned in an [`Interaction`] object.
    ///
    /// [`Interaction`]: crate::model::application::Interaction
    pub permissions: Option<Permissions>,
    /// When the user's timeout will expire and the user will be able to communicate in the guild
    /// again.
    ///
    /// Will be None or a time in the past if the user is not timed out.
    pub communication_disabled_until: Option<Timestamp>,
    /// The unique Id of the guild that the member is a part of.
    #[serde(default)]
    pub guild_id: GuildId,
    /// If the member is currently flagged for sending excessive DMs to non-friend server members
    /// in the last 24 hours.
    ///
    /// Will be None or a time in the past if the user is not flagged.
    pub unusual_dm_activity_until: Option<Timestamp>,
}

bitflags! {
    /// Flags for a guild member.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object-guild-member-flags).
    
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct GuildMemberFlags: u32 {
        /// Member has left and rejoined the guild. Not editable
        const DID_REJOIN = 1 << 0;
        /// Member has completed onboarding. Not editable
        const COMPLETED_ONBOARDING = 1 << 1;
        /// Member is exempt from guild verification requirements. Editable
        const BYPASSES_VERIFICATION = 1 << 2;
        /// Member has started onboarding. Not editable
        const STARTED_ONBOARDING = 1 << 3;
        /// Member is a guest and can only access the voice channel they were invited to. Not
        /// editable
        const IS_GUEST = 1 << 4;
        /// Member has started Server Guide new member actions. Not editable
        const STARTED_HOME_ACTIONS = 1 << 5;
        /// Member has completed Server Guide new member actions. Not editable
        const COMPLETED_HOME_ACTIONS = 1 << 6;
        /// Member's username, display name, or nickname is blocked by AutoMod. Not editable
        const AUTOMOD_QUARANTINED_USERNAME = 1 << 7;
        /// Member has dismissed the DM settings upsell. Not editable
        const DM_SETTINGS_UPSELL_ACKNOWLEDGED = 1 << 9;
    }
}

impl fmt::Display for Member {
    /// Mentions the user so that they receive a notification.
    ///
    /// This is in the format of `<@USER_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.user.mention(), f)
    }
}

impl ExtractKey<UserId> for Member {
    fn extract_key(&self) -> &UserId {
        &self.user.id
    }
}

/// A partial amount of data for a member.
///
/// This is used in [`Message`]s from [`Guild`]s.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object),
/// subset specification unknown (field type "partial member" is used in
/// [link](https://discord.com/developers/docs/topics/gateway-events#message-create),
/// [link](https://discord.com/developers/docs/resources/invite#invite-stage-instance-object),
/// [link](https://discord.com/developers/docs/topics/gateway-events#message-create),
/// [link](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-resolved-data-structure),
/// [link](https://discord.com/developers/docs/interactions/receiving-and-responding#message-interaction-object))
#[bool_to_bitflags::bool_to_bitflags]

#[derive(Clone, Debug, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct PartialMember {
    /// Indicator of whether the member can hear in voice channels.
    #[serde(default)]
    pub deaf: bool,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<Timestamp>,
    /// Indicator of whether the member can speak in voice channels
    #[serde(default)]
    pub mute: bool,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<FixedString<u8>>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: FixedArray<RoleId>,
    /// Indicator that the member hasn't accepted the rules of the guild yet.
    #[serde(default)]
    pub pending: bool,
    /// Timestamp representing the date since the member is boosting the guild.
    pub premium_since: Option<Timestamp>,
    /// The unique Id of the guild that the member is a part of.
    ///
    /// Manually inserted in [`Reaction::deserialize`].
    pub guild_id: Option<GuildId>,
    /// Attached User struct.
    pub user: Option<User>,
    /// The total permissions of the member in a channel, including overrides.
    ///
    /// This is only [`Some`] when returned in an [`Interaction`] object.
    ///
    /// [`Interaction`]: crate::model::application::Interaction
    pub permissions: Option<Permissions>,
    /// If the member is currently flagged for sending excessive DMs to non-friend server members
    /// in the last 24 hours.
    ///
    /// Will be None or a time in the past if the user is not flagged.
    pub unusual_dm_activity_until: Option<Timestamp>,
    /// The guild avatar hash
    pub avatar: Option<ImageHash>,
}

impl From<PartialMember> for Member {
    fn from(partial: PartialMember) -> Self {
        let (pending, deaf, mute) = (partial.pending(), partial.deaf(), partial.mute());
        let mut member = Member {
            __generated_flags: MemberGeneratedFlags::empty(),
            user: partial.user.unwrap_or_default(),
            nick: partial.nick,
            avatar: partial.avatar,
            roles: partial.roles,
            joined_at: partial.joined_at,
            premium_since: partial.premium_since,
            flags: GuildMemberFlags::default(),
            permissions: partial.permissions,
            communication_disabled_until: None,
            guild_id: partial.guild_id.unwrap_or_default(),
            unusual_dm_activity_until: partial.unusual_dm_activity_until,
        };

        member.set_pending(pending);
        member.set_deaf(deaf);
        member.set_mute(mute);
        member
    }
}

impl From<Member> for PartialMember {
    fn from(member: Member) -> Self {
        let (pending, deaf, mute) = (member.pending(), member.deaf(), member.mute());
        let mut partial = PartialMember {
            __generated_flags: PartialMemberGeneratedFlags::empty(),
            joined_at: member.joined_at,
            nick: member.nick,
            roles: member.roles,
            premium_since: member.premium_since,
            guild_id: Some(member.guild_id),
            user: Some(member.user),
            permissions: member.permissions,
            unusual_dm_activity_until: member.unusual_dm_activity_until,
            avatar: member.avatar,
        };

        partial.set_deaf(deaf);
        partial.set_mute(mute);
        partial.set_pending(pending);
        partial
    }
}
