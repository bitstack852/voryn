//! Admin role management and authorized actions.

use super::events::GroupEvent;

/// Actions that only admins can perform.
#[derive(Debug, Clone, PartialEq)]
pub enum AdminAction {
    InviteMember,
    RemoveMember,
    PromoteToAdmin,
    SetGroupPolicy,
    DissolveGroup,
}

/// Check if an action is authorized for the given role.
pub fn is_authorized(action: &AdminAction, is_admin: bool) -> bool {
    match action {
        AdminAction::InviteMember => is_admin,
        AdminAction::RemoveMember => is_admin,
        AdminAction::PromoteToAdmin => is_admin,
        AdminAction::SetGroupPolicy => is_admin,
        AdminAction::DissolveGroup => is_admin,
    }
}

/// Validate that a group event is authorized given the current ledger state.
pub fn validate_event_authorization(
    event: &GroupEvent,
    signer_pubkey: &[u8],
    is_signer_admin: bool,
) -> Result<(), String> {
    match event {
        GroupEvent::GroupCreated { .. } => Ok(()), // Always allowed for genesis
        GroupEvent::MemberAdded { added_by, .. } => {
            if added_by != signer_pubkey {
                return Err("Signer mismatch".into());
            }
            if !is_signer_admin {
                return Err("Only admins can add members".into());
            }
            Ok(())
        }
        GroupEvent::MemberRemoved { removed_by, .. } => {
            if removed_by != signer_pubkey {
                return Err("Signer mismatch".into());
            }
            if !is_signer_admin {
                return Err("Only admins can remove members".into());
            }
            Ok(())
        }
        GroupEvent::AdminPromoted { promoted_by, .. } => {
            if promoted_by != signer_pubkey {
                return Err("Signer mismatch".into());
            }
            if !is_signer_admin {
                return Err("Only admins can promote".into());
            }
            Ok(())
        }
        GroupEvent::GroupDissolved { dissolved_by, .. } => {
            if dissolved_by != signer_pubkey {
                return Err("Signer mismatch".into());
            }
            if !is_signer_admin {
                return Err("Only admins can dissolve".into());
            }
            Ok(())
        }
        GroupEvent::KeyReshared { .. } => Ok(()), // System event
        GroupEvent::PolicyChanged { changed_by, .. } => {
            if changed_by != signer_pubkey {
                return Err("Signer mismatch".into());
            }
            if !is_signer_admin {
                return Err("Only admins can change policies".into());
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_authorization() {
        assert!(is_authorized(&AdminAction::InviteMember, true));
        assert!(!is_authorized(&AdminAction::InviteMember, false));
        assert!(!is_authorized(&AdminAction::DissolveGroup, false));
    }

    #[test]
    fn test_event_authorization() {
        let admin = vec![1u8; 32];
        let member = vec![2u8; 32];

        let add_event = GroupEvent::MemberAdded {
            group_id: "g1".into(),
            member_pubkey: vec![3u8; 32],
            added_by: admin.clone(),
        };

        assert!(validate_event_authorization(&add_event, &admin, true).is_ok());
        assert!(validate_event_authorization(&add_event, &admin, false).is_err());
        assert!(validate_event_authorization(&add_event, &member, true).is_err());
    }
}
