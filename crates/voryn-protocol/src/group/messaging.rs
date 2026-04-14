//! Group message encryption — symmetric key encryption for group messages.

use serde::{Deserialize, Serialize};

/// Encrypted group message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    /// Group identifier.
    pub group_id: String,
    /// Sender's Ed25519 public key.
    pub sender_pubkey: Vec<u8>,
    /// Key epoch (which group key was used).
    pub epoch: u64,
    /// Encryption nonce.
    pub nonce: Vec<u8>,
    /// Encrypted payload.
    pub ciphertext: Vec<u8>,
    /// Ed25519 signature over (group_id || epoch || nonce || ciphertext).
    pub signature: Vec<u8>,
    /// Unix timestamp (milliseconds).
    pub timestamp: u64,
    /// Unique message identifier.
    pub message_id: String,
}

impl GroupMessage {
    /// Get the data that should be signed.
    pub fn signable_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.group_id.as_bytes());
        data.extend_from_slice(&self.epoch.to_be_bytes());
        data.extend_from_slice(&self.nonce);
        data.extend_from_slice(&self.ciphertext);
        data
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| e.to_string())
    }
}
