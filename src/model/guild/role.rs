use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use crate::model::prelude::*;

/// Information about a role within a guild.
///
/// A role represents a set of permissions, and can be attached to one or multiple users. A role
/// has various miscellaneous configurations, such as being assigned a colour. Roles are unique per
/// guild and do not cross over to other guilds in any way, and can have channel-specific
/// permission overrides in addition to guild-level permissions.
///
/// [Discord docs](https://discord.com/developers/docs/topics/permissions#role-object).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Role {
    /// The Id of the role. Can be used to calculate the role's creation date.
    pub id: RoleId,
    /// The Id of the Guild the Role is in.
    #[serde(default)]
    pub guild_id: GuildId,
    /// The name of the role.
    pub name: FixedString,
    /// A set of permissions that the role has been assigned.
    ///
    /// See the [`permissions`] module for more information.
    ///
    /// [`permissions`]: crate::model::permissions
    pub permissions: Permissions,
    /// The role's position in the position list. Roles are considered higher in hierarchy if their
    /// position is higher.
    ///
    /// The `@everyone` role is usually either `-1` or `0`.
    pub position: i16,

    #[serde(flatten)]
    pub extra_info: HashMap<String, serde_json::Value>,
}

#[cfg(feature = "model")]
impl Role {
    /// Check that the role has the given permission.
    #[must_use]
    pub fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.contains(permission)
    }

    /// Checks whether the role has all of the given permissions.
    ///
    /// The 'precise' argument is used to check if the role's permissions are precisely equivalent
    /// to the given permissions. If you need only check that the role has at least the given
    /// permissions, pass `false`.
    #[must_use]
    pub fn has_permissions(&self, permissions: Permissions, precise: bool) -> bool {
        if precise {
            self.permissions == permissions
        } else {
            self.permissions.contains(permissions)
        }
    }
}

impl fmt::Display for Role {
    /// Format a mention for the role, pinging its members.
    // This is in the format of: `<@&ROLE_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl ExtractKey<RoleId> for Role {
    fn extract_key(&self) -> &RoleId {
        &self.id
    }
}

impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Role) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Role {
    fn cmp(&self, other: &Role) -> Ordering {
        // Discord does position DESC, id ASC so:
        if self.position == other.position {
            other.id.cmp(&self.id)
        } else {
            self.position.cmp(&other.position)
        }
    }
}

impl From<Role> for RoleId {
    /// Gets the Id of a role.
    fn from(role: Role) -> RoleId {
        role.id
    }
}

impl From<&Role> for RoleId {
    /// Gets the Id of a role.
    fn from(role: &Role) -> RoleId {
        role.id
    }
}
