//! Group message synchronization for offline members.

use serde::{Deserialize, Serialize};

/// Request to sync messages since a known point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Group ID to sync.
    pub group_id: String,
    /// Requester's public key.
    pub requester_pubkey: Vec<u8>,
    /// Last known message ID (sync everything after this).
    pub last_known_message_id: Option<String>,
    /// Requester's join timestamp (cannot receive messages before this).
    pub join_timestamp: u64,
}

/// Response containing missed messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Group ID.
    pub group_id: String,
    /// Missed messages (serialized GroupMessage bytes).
    pub messages: Vec<Vec<u8>>,
    /// Whether there are more messages to sync.
    pub has_more: bool,
}

impl SyncRequest {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| e.to_string())
    }
}

impl SyncResponse {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| e.to_string())
    }
}
