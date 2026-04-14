//! Admin-controlled group auto-delete policies.
//!
//! The group admin sets a deletion policy that applies to all messages.
//! Policy changes are recorded in the group ledger.
//! Each member's device independently enforces the timer.

use serde::{Deserialize, Serialize};
use super::timer::{AutoDeleteConfig, DeleteInterval};

/// Group-level auto-delete policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDeletePolicy {
    /// Group ID this policy applies to.
    pub group_id: String,
    /// The delete configuration.
    pub config: AutoDeleteConfig,
    /// Who set this policy.
    pub set_by: Vec<u8>,
    /// When the policy was set.
    pub set_at: u64,
}

impl GroupDeletePolicy {
    /// Create a new policy.
    pub fn new(group_id: String, interval: DeleteInterval, admin_pubkey: Vec<u8>) -> Self {
        Self {
            group_id,
            config: AutoDeleteConfig {
                enabled: true,
                interval,
            },
            set_by: admin_pubkey,
            set_at: crate::exchange::current_timestamp_ms(),
        }
    }

    /// Create a disabled policy (no auto-delete).
    pub fn disabled(group_id: String, admin_pubkey: Vec<u8>) -> Self {
        Self {
            group_id,
            config: AutoDeleteConfig::default(),
            set_by: admin_pubkey,
            set_at: crate::exchange::current_timestamp_ms(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_policy() {
        let policy = GroupDeletePolicy::new(
            "group-1".into(),
            DeleteInterval::TwentyFourHours,
            vec![1u8; 32],
        );
        assert!(policy.config.enabled);
        assert_eq!(policy.config.interval, DeleteInterval::TwentyFourHours);
    }

    #[test]
    fn test_disabled_policy() {
        let policy = GroupDeletePolicy::disabled("group-1".into(), vec![1u8; 32]);
        assert!(!policy.config.enabled);
    }
}
