use std::{collections::HashMap, fmt};
use crate::model::prelude::*;

/// Information about a member of a guild.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object),
/// [extra fields](https://discord.com/developers/docs/topics/gateway-events#guild-member-add-guild-member-add-extra-fields).
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Member {
    /// Attached User struct.
    pub user: User,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: FixedArray<RoleId>,
    /// Guild member flags.
    pub flags: GuildMemberFlags,
    /// The unique Id of the guild that the member is a part of.
    #[serde(default)]
    pub guild_id: GuildId,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
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
