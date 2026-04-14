//! Remote wipe protocol — trusted contact can trigger data destruction.
//!
//! A designated trusted contact can send a signed wipe command over the
//! P2P network. The target device verifies the signature and timestamp,
//! then performs a full data wipe.

use serde::{Deserialize, Serialize};

/// Remote wipe command sent over the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeCommand {
    /// The command type.
    pub command: String, // "wipe"
    /// Public key of the device to be wiped.
    pub target_pubkey: Vec<u8>,
    /// Public key of the trusted contact sending the command.
    pub sender_pubkey: Vec<u8>,
    /// Unix timestamp (milliseconds).
    pub timestamp: u64,
    /// Random nonce to prevent replay attacks.
    pub nonce: Vec<u8>,
    /// Ed25519 signature over (command || target_pubkey || timestamp || nonce).
    pub signature: Vec<u8>,
}

/// Maximum age of a wipe command (5 minutes) to prevent replay.
const MAX_COMMAND_AGE_MS: u64 = 5 * 60 * 1000;

/// Grace period before wipe executes (5 minutes), allowing cancellation.
pub const WIPE_GRACE_PERIOD_MS: u64 = 5 * 60 * 1000;

impl WipeCommand {
    /// Get the data that should be signed for this command.
    pub fn signed_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.command.as_bytes());
        data.extend_from_slice(&self.target_pubkey);
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data.extend_from_slice(&self.nonce);
        data
    }

    /// Validate the command's timestamp (within MAX_COMMAND_AGE_MS of now).
    pub fn is_timestamp_valid(&self, current_time_ms: u64) -> bool {
        if self.timestamp > current_time_ms {
            return false; // Future timestamp
        }
        current_time_ms - self.timestamp <= MAX_COMMAND_AGE_MS
    }

    /// Serialize for network transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }

    /// Deserialize from network bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| e.to_string())
    }
}

/// Nonce tracker to prevent replay attacks.
pub struct NonceTracker {
    seen_nonces: std::collections::HashSet<Vec<u8>>,
}

impl Default for NonceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl NonceTracker {
    pub fn new() -> Self {
        Self {
            seen_nonces: std::collections::HashSet::new(),
        }
    }

    /// Check if a nonce has been seen before. Returns true if it's new.
    pub fn check_and_record(&mut self, nonce: &[u8]) -> bool {
        self.seen_nonces.insert(nonce.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wipe_command_serialization() {
        let cmd = WipeCommand {
            command: "wipe".to_string(),
            target_pubkey: vec![1u8; 32],
            sender_pubkey: vec![2u8; 32],
            timestamp: 1713100000000,
            nonce: vec![3u8; 16],
            signature: vec![4u8; 64],
        };

        let bytes = cmd.to_bytes().unwrap();
        let recovered = WipeCommand::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.command, "wipe");
        assert_eq!(recovered.target_pubkey, vec![1u8; 32]);
    }

    #[test]
    fn test_timestamp_validation() {
        let now = 1713100000000u64;
        let cmd = WipeCommand {
            command: "wipe".to_string(),
            target_pubkey: vec![],
            sender_pubkey: vec![],
            timestamp: now - 60_000, // 1 minute ago
            nonce: vec![],
            signature: vec![],
        };

        assert!(cmd.is_timestamp_valid(now));

        // 10 minutes ago — too old
        let old_cmd = WipeCommand {
            timestamp: now - 10 * 60 * 1000,
            ..cmd.clone()
        };
        assert!(!old_cmd.is_timestamp_valid(now));
    }

    #[test]
    fn test_nonce_replay_prevention() {
        let mut tracker = NonceTracker::new();
        assert!(tracker.check_and_record(b"nonce1")); // New
        assert!(!tracker.check_and_record(b"nonce1")); // Replay
        assert!(tracker.check_and_record(b"nonce2")); // New
    }
}
