//! Group ledger event types — all actions recorded as signed, hash-chained entries.

use serde::{Deserialize, Serialize};

/// All possible group ledger events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupEvent {
    GroupCreated {
        group_id: String,
        admin_pubkey: Vec<u8>,
        group_name: String,
    },
    MemberAdded {
        group_id: String,
        member_pubkey: Vec<u8>,
        added_by: Vec<u8>,
    },
    MemberRemoved {
        group_id: String,
        member_pubkey: Vec<u8>,
        removed_by: Vec<u8>,
        reason: String,
    },
    AdminPromoted {
        group_id: String,
        new_admin_pubkey: Vec<u8>,
        promoted_by: Vec<u8>,
    },
    KeyReshared {
        group_id: String,
        epoch: u64,
        threshold: u32,
        share_count: u32,
    },
    GroupDissolved {
        group_id: String,
        dissolved_by: Vec<u8>,
    },
    PolicyChanged {
        group_id: String,
        changed_by: Vec<u8>,
        policy_type: String,
        policy_value: String,
    },
}

/// A signed, hash-chained ledger entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Sequential entry index.
    pub index: u64,
    /// Hash of the previous entry (SHA-256, 32 bytes). Empty for genesis.
    pub previous_hash: Vec<u8>,
    /// The event recorded in this entry.
    pub event: GroupEvent,
    /// Unix timestamp (milliseconds).
    pub timestamp: u64,
    /// Public key of the signer.
    pub signer_pubkey: Vec<u8>,
    /// Ed25519 signature over (index || previous_hash || event_bytes || timestamp).
    pub signature: Vec<u8>,
}

impl LedgerEntry {
    /// Compute the hash of this entry for chaining.
    pub fn hash(&self) -> Vec<u8> {
        let data = self.signable_data();
        let mut hash = [0u8; 32];
        unsafe {
            libsodium_sys::crypto_hash_sha256(
                hash.as_mut_ptr(),
                data.as_ptr(),
                data.len() as u64,
            );
        }
        hash.to_vec()
    }

    /// Get the data that should be signed.
    pub fn signable_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.index.to_be_bytes());
        data.extend_from_slice(&self.previous_hash);
        data.extend_from_slice(&bincode::serialize(&self.event).unwrap_or_default());
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data
    }

    /// Serialize for storage/transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| e.to_string())
    }
}
