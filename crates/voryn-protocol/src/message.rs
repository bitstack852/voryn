//! Wire message format for Voryn protocol messages.

use serde::{Deserialize, Serialize};

/// Encrypted message as transmitted over the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Sender's Ed25519 public key.
    pub sender_pubkey: Vec<u8>,
    /// Encryption nonce.
    pub nonce: Vec<u8>,
    /// Encrypted message payload.
    pub ciphertext: Vec<u8>,
    /// Ed25519 signature over (nonce || ciphertext).
    pub signature: Vec<u8>,
    /// Unix timestamp (milliseconds).
    pub timestamp: u64,
    /// Unique message identifier.
    pub message_id: String,
}

/// Delivery acknowledgment message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAck {
    /// ID of the message being acknowledged.
    pub message_id: String,
    /// Acknowledger's public key.
    pub ack_pubkey: Vec<u8>,
    /// Unix timestamp of receipt.
    pub received_at: u64,
    /// Signature over (message_id || received_at).
    pub signature: Vec<u8>,
}

/// Message delivery status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// Queued locally, not yet sent.
    Pending,
    /// Sent to network, awaiting ACK.
    Sent,
    /// Delivery confirmed by recipient.
    Delivered,
    /// Failed to deliver after retries.
    Failed,
}
