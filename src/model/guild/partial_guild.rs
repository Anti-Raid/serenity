use std::collections::HashMap;

use nonmax::NonMaxU64;
use serde::Serialize;

use crate::model::prelude::*;

/// Partial information about a [`Guild`]. This does not include information like member data.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object).
#[bool_to_bitflags::bool_to_bitflags]

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Self")]
#[non_exhaustive]
pub struct PartialGuild {
    // ======
    // These fields are copy-pasted from the top part of Guild, and the omitted fields filled in
    // ======
    /// The unique Id identifying the guild.
    ///
    /// This is equivalent to the Id of the default role (`@everyone`).
    pub id: GuildId,
    /// The name of the guild.
    pub name: FixedString,
    // Omitted `owner` field because only Http::get_guilds uses it, which returns GuildInfo
    /// The Id of the [`User`] who owns the guild.
    pub owner_id: UserId,
    /// Indicator of the current verification level of the guild.
    pub verification_level: VerificationLevel,
    /// Default explicit content filter level.
    pub explicit_content_filter: ExplicitContentFilter,
    /// A mapping of the guild's roles.
    pub roles: ExtractMap<RoleId, Role>,
    /// Indicator of whether the guild requires multi-factor authentication for [`Role`]s or
    /// [`User`]s with moderation permissions.
    pub mfa_level: MfaLevel,
    /// The maximum number of presences for the guild. The default value is currently 25000.
    ///
    /// **Note**: It is in effect when it is `None`.
    pub max_presences: Option<NonMaxU64>,
    /// The maximum number of members for the guild.
    pub max_members: Option<NonMaxU64>,
    /// The server's premium boosting level.
    pub premium_tier: PremiumTier,
    /// The total number of users currently boosting this server.
    pub premium_subscription_count: Option<NonMaxU64>,
    /// Approximate number of members in this guild.
    pub approximate_member_count: Option<NonMaxU64>,
    /// Approximate number of non-offline members in this guild.
    pub approximate_presence_count: Option<NonMaxU64>,
    /// The guild NSFW state. See [`discord support article`].
    ///
    /// [`discord support article`]: https://support.discord.com/hc/en-us/articles/1500005389362-NSFW-Server-Designation
    pub nsfw_level: NsfwLevel,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
}

#[cfg(feature = "model")]
impl PartialGuild {
    /// Calculate a [`Member`]'s permissions in the guild.
    ///
    /// You likely want to use PartialGuild::user_permissions_in instead as this function does not
    /// consider permission overwrites.
    #[must_use]
    pub fn member_permissions(&self, member: &Member) -> Permissions {
        Self::user_permissions_in_(
            None,
            member.user.id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

    /// Gets the highest role a [`Member`] of this Guild has.
    ///
    /// Returns None if the member has no roles or the member from this guild.
    #[must_use]
    pub fn member_highest_role(&self, member: &Member) -> Option<&Role> {
        Self::_member_highest_role_in(&self.roles, member)
    }

    /// Helper function for portability
    pub(crate) fn _member_highest_role_in<'a>(
        roles: &'a ExtractMap<RoleId, Role>,
        member: &Member,
    ) -> Option<&'a Role> {
        let mut highest: Option<&Role> = None;

        for role_id in &member.roles {
            if let Some(role) = roles.get(role_id) {
                // Skip this role if this role in iteration has:
                // - a position less than the recorded highest
                // - a position equal to the recorded, but a higher ID
                if let Some(highest) = highest
                    && (role.position < highest.position
                        || (role.position == highest.position && role.id > highest.id))
                {
                    continue;
                }

                highest = Some(role);
            }
        }

        highest
    }

    /// Returns which of two [`User`]s has a higher [`Member`] hierarchy.
    ///
    /// Hierarchy is essentially who has the [`Role`] with the highest [`position`].
    ///
    /// Returns [`None`] if at least one of the given users' member instances is not present.
    /// Returns [`None`] if the users have the same hierarchy, as neither are greater than the
    /// other.
    ///
    /// If both user IDs are the same, [`None`] is returned. If one of the users is the guild
    /// owner, their ID is returned.
    ///
    /// [`position`]: Role::position
    #[must_use]
    pub fn greater_member_hierarchy(&self, lhs: &Member, rhs: &Member) -> Option<UserId> {
        let lhs_highest_role = self.member_highest_role(lhs);
        let rhs_highest_role = self.member_highest_role(rhs);

        Self::_greater_member_hierarchy_in(
            lhs_highest_role,
            rhs_highest_role,
            self.owner_id,
            lhs,
            rhs,
        )
    }

    /// Helper function for portability
    #[must_use]
    pub(crate) fn _greater_member_hierarchy_in(
        lhs_highest_role: Option<&Role>,
        rhs_highest_role: Option<&Role>,
        owner_id: UserId,
        lhs: &Member,
        rhs: &Member,
    ) -> Option<UserId> {
        // Check that the IDs are the same. If they are, neither is greater.
        if lhs.user.id == rhs.user.id {
            return None;
        }

        // Check if either user is the guild owner.
        if lhs.user.id == owner_id {
            return Some(lhs.user.id);
        } else if rhs.user.id == owner_id {
            return Some(rhs.user.id);
        }

        let lhs_role = lhs_highest_role.map_or((RoleId::new(1), 0), |r| (r.id, r.position));

        let rhs_role = rhs_highest_role.map_or((RoleId::new(1), 0), |r| (r.id, r.position));

        // If LHS and RHS both have no top position or have the same role ID, then no one wins.
        if (lhs_role.1 == 0 && rhs_role.1 == 0) || (lhs_role.0 == rhs_role.0) {
            return None;
        }

        // If LHS's top position is higher than RHS, then LHS wins.
        if lhs_role.1 > rhs_role.1 {
            return Some(lhs.user.id);
        }

        // If RHS's top position is higher than LHS, then RHS wins.
        if rhs_role.1 > lhs_role.1 {
            return Some(rhs.user.id);
        }

        // If LHS and RHS both have the same position, but LHS has the lower role ID, then LHS
        // wins.
        //
        // If RHS has the higher role ID, then RHS wins.
        if lhs_role.1 == rhs_role.1 && lhs_role.0 < rhs_role.0 {
            Some(lhs.user.id)
        } else {
            Some(rhs.user.id)
        }
    }

    /// Calculate a [`Member`]'s permissions in a given channel in the guild.
    #[must_use]
    pub fn user_permissions_in(&self, channel: &GuildChannel, member: &Member) -> Permissions {
        Self::user_permissions_in_(
            Some(channel),
            member.user.id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }

        /// Helper function that can also be used from [`PartialGuild`].
    pub(crate) fn user_permissions_in_(
        channel: Option<&GuildChannel>,
        member_user_id: UserId,
        member_roles: &[RoleId],
        guild_id: GuildId,
        guild_roles: &ExtractMap<RoleId, Role>,
        guild_owner_id: UserId,
    ) -> Permissions {
        let mut everyone_allow_overwrites = Permissions::empty();
        let mut everyone_deny_overwrites = Permissions::empty();
        let mut roles_allow_overwrites = Vec::new();
        let mut roles_deny_overwrites = Vec::new();
        let mut member_allow_overwrites = Permissions::empty();
        let mut member_deny_overwrites = Permissions::empty();

        if let Some(channel) = channel {
            for overwrite in &channel.permission_overwrites {
                match overwrite.kind {
                    PermissionOverwriteType::Member(user_id) => {
                        if member_user_id == user_id {
                            member_allow_overwrites = overwrite.allow;
                            member_deny_overwrites = overwrite.deny;
                        }
                    },
                    PermissionOverwriteType::Role(role_id) => {
                        if role_id.get() == guild_id.get() {
                            everyone_allow_overwrites = overwrite.allow;
                            everyone_deny_overwrites = overwrite.deny;
                        } else if member_roles.contains(&role_id) {
                            roles_allow_overwrites.push(overwrite.allow);
                            roles_deny_overwrites.push(overwrite.deny);
                        }
                    },
                }
            }
        }

        calculate_permissions(CalculatePermissions {
            is_guild_owner: member_user_id == guild_owner_id,
            everyone_permissions: if let Some(role) = guild_roles.get(&RoleId::new(guild_id.get()))
            {
                role.permissions
            } else {
                use tracing::error;

                error!("@everyone role missing in {}", guild_id);
                Permissions::empty()
            },
            user_roles_permissions: member_roles
                .iter()
                .map(|role_id| {
                    if let Some(role) = guild_roles.get(role_id) {
                        role.permissions
                    } else {
                        use tracing::warn;

                        warn!(
                            "{} on {} has non-existent role {:?}",
                            member_user_id, guild_id, role_id
                        );
                        Permissions::empty()
                    }
                })
                .collect(),
            everyone_allow_overwrites,
            everyone_deny_overwrites,
            roles_allow_overwrites,
            roles_deny_overwrites,
            member_allow_overwrites,
            member_deny_overwrites,
        })
    }
}

// Manual impl needed to insert guild_id into Role's
impl<'de> Deserialize<'de> for PartialGuildGeneratedOriginal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut guild = Self::deserialize(deserializer)?; // calls #[serde(remote)]-generated inherent method
        guild.roles.iter_mut().for_each(|r| r.guild_id = guild.id);
        Ok(guild)
    }
}

impl Serialize for PartialGuildGeneratedOriginal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        Self::serialize(self, serializer) // calls #[serde(remote)]-generated inherent method
    }
}

/// Translated from the pseudo code at https://discord.com/developers/docs/topics/permissions#permission-overwrites
///
/// The comments within this file refer to the above link
#[cfg(feature = "model")]
fn calculate_permissions(data: CalculatePermissions) -> Permissions {
    if data.is_guild_owner {
        return Permissions::all();
    }

    // 1. Base permissions given to @everyone are applied at a guild level
    let mut permissions = data.everyone_permissions;
    // 2. Permissions allowed to a user by their roles are applied at a guild level
    for role_permission in data.user_roles_permissions {
        permissions |= role_permission;
    }

    if permissions.contains(Permissions::ADMINISTRATOR) {
        return Permissions::all();
    }

    // 3. Overwrites that deny permissions for @everyone are applied at a channel level
    permissions &= !data.everyone_deny_overwrites;
    // 4. Overwrites that allow permissions for @everyone are applied at a channel level
    permissions |= data.everyone_allow_overwrites;

    // 5. Overwrites that deny permissions for specific roles are applied at a channel level
    let mut role_deny_permissions = Permissions::empty();
    for p in data.roles_deny_overwrites {
        role_deny_permissions |= p;
    }
    permissions &= !role_deny_permissions;

    // 6. Overwrites that allow permissions for specific roles are applied at a channel level
    let mut role_allow_permissions = Permissions::empty();
    for p in data.roles_allow_overwrites {
        role_allow_permissions |= p;
    }
    permissions |= role_allow_permissions;

    // 7. Member-specific overwrites that deny permissions are applied at a channel level
    permissions &= !data.member_deny_overwrites;
    // 8. Member-specific overwrites that allow permissions are applied at a channel level
    permissions |= data.member_allow_overwrites;

    permissions
}

struct CalculatePermissions {
    /// Whether the guild member is the guild owner
    pub is_guild_owner: bool,
    /// Base permissions given to @everyone (guild level)
    pub everyone_permissions: Permissions,
    /// Permissions allowed to a user by their roles (guild level)
    pub user_roles_permissions: Vec<Permissions>,
    /// Overwrites that deny permissions for @everyone (channel level)
    pub everyone_allow_overwrites: Permissions,
    /// Overwrites that allow permissions for @everyone (channel level)
    pub everyone_deny_overwrites: Permissions,
    /// Overwrites that deny permissions for specific roles (channel level)
    pub roles_allow_overwrites: Vec<Permissions>,
    /// Overwrites that allow permissions for specific roles (channel level)
    pub roles_deny_overwrites: Vec<Permissions>,
    /// Member-specific overwrites that deny permissions (channel level)
    pub member_allow_overwrites: Permissions,
    /// Member-specific overwrites that allow permissions (channel level)
    pub member_deny_overwrites: Permissions,
}

#[cfg(feature = "model")]
impl Default for CalculatePermissions {
    fn default() -> Self {
        Self {
            is_guild_owner: false,
            everyone_permissions: Permissions::empty(),
            user_roles_permissions: Vec::new(),
            everyone_allow_overwrites: Permissions::empty(),
            everyone_deny_overwrites: Permissions::empty(),
            roles_allow_overwrites: Vec::new(),
            roles_deny_overwrites: Vec::new(),
            member_allow_overwrites: Permissions::empty(),
            member_deny_overwrites: Permissions::empty(),
        }
    }
}
