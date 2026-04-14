//! Group permission model — defines what admins vs members can do.

use serde::{Deserialize, Serialize};

/// Role within a group.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GroupRole {
    Admin,
    Member,
}

/// Permissions for a group role.
#[derive(Debug, Clone)]
pub struct RolePermissions {
    pub can_send_messages: bool,
    pub can_invite_members: bool,
    pub can_remove_members: bool,
    pub can_promote_admin: bool,
    pub can_set_policies: bool,
    pub can_dissolve_group: bool,
}

impl RolePermissions {
    pub fn for_role(role: GroupRole) -> Self {
        match role {
            GroupRole::Admin => Self {
                can_send_messages: true,
                can_invite_members: true,
                can_remove_members: true,
                can_promote_admin: true,
                can_set_policies: true,
                can_dissolve_group: true,
            },
            GroupRole::Member => Self {
                can_send_messages: true,
                can_invite_members: false,
                can_remove_members: false,
                can_promote_admin: false,
                can_set_policies: false,
                can_dissolve_group: false,
            },
        }
    }
}
