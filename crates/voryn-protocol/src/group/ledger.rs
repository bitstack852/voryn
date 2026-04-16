//! Cryptographic group ledger — append-only, hash-chained event log.
//!
//! The ledger is the source of truth for group membership, admin actions,
//! and key resharing events. Each entry is signed by the actor and
//! hash-chained to the previous entry for tamper detection.

use super::events::{GroupEvent, LedgerEntry};
use crate::exchange;

/// An append-only group ledger.
#[derive(Debug, Clone)]
pub struct GroupLedger {
    /// Group identifier.
    pub group_id: String,
    /// All ledger entries in order.
    entries: Vec<LedgerEntry>,
}

impl GroupLedger {
    /// Create a new ledger with a genesis event (GroupCreated).
    pub fn new(group_id: String, admin_pubkey: Vec<u8>, group_name: String) -> Self {
        let event = GroupEvent::GroupCreated {
            group_id: group_id.clone(),
            admin_pubkey: admin_pubkey.clone(),
            group_name,
        };

        let entry = LedgerEntry {
            index: 0,
            previous_hash: vec![], // Genesis has no previous
            event,
            timestamp: exchange::current_timestamp_ms(),
            signer_pubkey: admin_pubkey,
            signature: vec![], // Signature added externally
        };

        GroupLedger {
            group_id,
            entries: vec![entry],
        }
    }

    /// Append a new event to the ledger.
    pub fn append(
        &mut self,
        event: GroupEvent,
        signer_pubkey: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<&LedgerEntry, String> {
        let previous_hash = self.entries.last()
            .map(|e| e.hash())
            .unwrap_or_default();

        let entry = LedgerEntry {
            index: self.entries.len() as u64,
            previous_hash,
            event,
            timestamp: exchange::current_timestamp_ms(),
            signer_pubkey,
            signature,
        };

        self.entries.push(entry);
        Ok(self.entries.last().unwrap())
    }

    /// Validate the hash chain integrity of the entire ledger.
    pub fn validate_chain(&self) -> Result<(), String> {
        for i in 1..self.entries.len() {
            let expected_hash = self.entries[i - 1].hash();
            if self.entries[i].previous_hash != expected_hash {
                return Err(format!(
                    "Hash chain broken at index {}: expected {:?}, got {:?}",
                    i, expected_hash, self.entries[i].previous_hash
                ));
            }
        }
        Ok(())
    }

    /// Get all entries.
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// Get the latest entry.
    pub fn latest(&self) -> Option<&LedgerEntry> {
        self.entries.last()
    }

    /// Get the current member list (from events).
    pub fn current_members(&self) -> Vec<Vec<u8>> {
        let mut members = std::collections::HashSet::new();

        for entry in &self.entries {
            match &entry.event {
                GroupEvent::GroupCreated { admin_pubkey, .. } => {
                    members.insert(admin_pubkey.clone());
                }
                GroupEvent::MemberAdded { member_pubkey, .. } => {
                    members.insert(member_pubkey.clone());
                }
                GroupEvent::MemberRemoved { member_pubkey, .. } => {
                    members.remove(member_pubkey);
                }
                GroupEvent::GroupDissolved { .. } => {
                    members.clear();
                }
                _ => {}
            }
        }

        members.into_iter().collect()
    }

    /// Get current admins.
    pub fn current_admins(&self) -> Vec<Vec<u8>> {
        let mut admins = std::collections::HashSet::new();

        for entry in &self.entries {
            match &entry.event {
                GroupEvent::GroupCreated { admin_pubkey, .. } => {
                    admins.insert(admin_pubkey.clone());
                }
                GroupEvent::AdminPromoted { new_admin_pubkey, .. } => {
                    admins.insert(new_admin_pubkey.clone());
                }
                _ => {}
            }
        }

        admins.into_iter().collect()
    }

    /// Check if a pubkey is an admin.
    pub fn is_admin(&self, pubkey: &[u8]) -> bool {
        self.current_admins().iter().any(|a| a == pubkey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ledger() {
        let ledger = GroupLedger::new(
            "group-1".into(),
            vec![1u8; 32],
            "Test Group".into(),
        );
        assert_eq!(ledger.entries().len(), 1);
        assert_eq!(ledger.current_members().len(), 1);
        assert!(ledger.is_admin(&[1u8; 32]));
    }

    #[test]
    fn test_append_and_validate() {
        let mut ledger = GroupLedger::new(
            "group-2".into(),
            vec![1u8; 32],
            "Test".into(),
        );

        ledger.append(
            GroupEvent::MemberAdded {
                group_id: "group-2".into(),
                member_pubkey: vec![2u8; 32],
                added_by: vec![1u8; 32],
            },
            vec![1u8; 32],
            vec![0; 64], // Mock signature
        ).unwrap();

        assert_eq!(ledger.entries().len(), 2);
        assert_eq!(ledger.current_members().len(), 2);
        assert!(ledger.validate_chain().is_ok());
    }

    #[test]
    fn test_member_removal() {
        let mut ledger = GroupLedger::new(
            "group-3".into(),
            vec![1u8; 32],
            "Test".into(),
        );

        ledger.append(
            GroupEvent::MemberAdded {
                group_id: "group-3".into(),
                member_pubkey: vec![2u8; 32],
                added_by: vec![1u8; 32],
            },
            vec![1u8; 32],
            vec![],
        ).unwrap();

        assert_eq!(ledger.current_members().len(), 2);

        ledger.append(
            GroupEvent::MemberRemoved {
                group_id: "group-3".into(),
                member_pubkey: vec![2u8; 32],
                removed_by: vec![1u8; 32],
                reason: "test".into(),
            },
            vec![1u8; 32],
            vec![],
        ).unwrap();

        assert_eq!(ledger.current_members().len(), 1);
    }

    #[test]
    fn test_tampered_chain_detected() {
        let mut ledger = GroupLedger::new(
            "group-4".into(),
            vec![1u8; 32],
            "Test".into(),
        );

        ledger.append(
            GroupEvent::MemberAdded {
                group_id: "group-4".into(),
                member_pubkey: vec![2u8; 32],
                added_by: vec![1u8; 32],
            },
            vec![1u8; 32],
            vec![],
        ).unwrap();

        // Tamper with the chain
        ledger.entries.get_mut(1).unwrap().previous_hash = vec![0xFF; 32];

        assert!(ledger.validate_chain().is_err());
    }
}
