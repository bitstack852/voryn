//! Identity revocation — permanent exclusion from the network.

use serde::{Deserialize, Serialize};

/// Identity revocation notice broadcast to the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationNotice {
    /// Public key of the revoked identity.
    pub revoked_pubkey: Vec<u8>,
    /// Public key of the admin who issued the revocation.
    pub revoked_by: Vec<u8>,
    /// Reason for revocation.
    pub reason: String,
    /// Timestamp.
    pub timestamp: u64,
    /// Signature by the revoking admin.
    pub signature: Vec<u8>,
}

impl RevocationNotice {
    /// Get signable data.
    pub fn signable_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.revoked_pubkey);
        data.extend_from_slice(&self.revoked_by);
        data.extend_from_slice(self.reason.as_bytes());
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| e.to_string())
    }
}

/// Local revocation list — tracks which identities have been revoked.
pub struct RevocationList {
    revoked: std::collections::HashSet<Vec<u8>>,
}

impl RevocationList {
    pub fn new() -> Self {
        Self {
            revoked: std::collections::HashSet::new(),
        }
    }

    /// Add a revoked identity.
    pub fn revoke(&mut self, pubkey: Vec<u8>) {
        self.revoked.insert(pubkey);
    }

    /// Check if an identity is revoked.
    pub fn is_revoked(&self, pubkey: &[u8]) -> bool {
        self.revoked.contains(pubkey)
    }

    /// Get count of revoked identities.
    pub fn count(&self) -> usize {
        self.revoked.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revocation_list() {
        let mut list = RevocationList::new();
        let pk = vec![1u8; 32];

        assert!(!list.is_revoked(&pk));
        list.revoke(pk.clone());
        assert!(list.is_revoked(&pk));
        assert_eq!(list.count(), 1);
    }

    #[test]
    fn test_revocation_notice_serialization() {
        let notice = RevocationNotice {
            revoked_pubkey: vec![1u8; 32],
            revoked_by: vec![2u8; 32],
            reason: "compromised".to_string(),
            timestamp: 1713100000000,
            signature: vec![3u8; 64],
        };

        let bytes = notice.to_bytes().unwrap();
        let recovered = RevocationNotice::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.reason, "compromised");
    }
}
