//! New member onboarding flow.

use serde::{Deserialize, Serialize};

/// Join request sent by a new member to the inviter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    /// The invite token ID being redeemed.
    pub token_id: String,
    /// New member's Ed25519 public key.
    pub new_member_pubkey: Vec<u8>,
    /// New member's display name (optional).
    pub display_name: Option<String>,
    /// Timestamp.
    pub timestamp: u64,
    /// Signature over (token_id || new_member_pubkey || timestamp).
    pub signature: Vec<u8>,
}

/// Identity broadcast — inviter vouches for the new member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityBroadcast {
    /// New member's public key.
    pub member_pubkey: Vec<u8>,
    /// New member's display name.
    pub display_name: Option<String>,
    /// Inviter's public key (voucher).
    pub vouched_by: Vec<u8>,
    /// Inviter's signature over (member_pubkey || display_name || timestamp).
    pub voucher_signature: Vec<u8>,
    /// Timestamp.
    pub timestamp: u64,
}

impl JoinRequest {
    pub fn signable_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.token_id.as_bytes());
        data.extend_from_slice(&self.new_member_pubkey);
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data
    }
}
