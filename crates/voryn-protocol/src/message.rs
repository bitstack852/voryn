//! Wire message format for Voryn protocol messages.

use serde::{Deserialize, Serialize};

/// Encrypted message as transmitted over the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Sender's Ed25519 public key (32 bytes).
    pub sender_pubkey: Vec<u8>,
    /// Encryption nonce (24 bytes for XSalsa20-Poly1305).
    pub nonce: Vec<u8>,
    /// Encrypted message payload.
    pub ciphertext: Vec<u8>,
    /// Ed25519 signature over (nonce || ciphertext).
    pub signature: Vec<u8>,
    /// Unix timestamp (milliseconds).
    pub timestamp: u64,
    /// Unique message identifier (UUID v4).
    pub message_id: String,
}

impl EncryptedMessage {
    /// Serialize to bytes for network transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>, crate::ProtocolError> {
        bincode::serialize(self)
            .map_err(|e| crate::ProtocolError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes received over network.
    pub fn from_bytes(data: &[u8]) -> Result<Self, crate::ProtocolError> {
        bincode::deserialize(data)
            .map_err(|e| crate::ProtocolError::SerializationError(e.to_string()))
    }

    /// Get the data that was signed (nonce || ciphertext) for verification.
    pub fn signed_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(self.nonce.len() + self.ciphertext.len());
        payload.extend_from_slice(&self.nonce);
        payload.extend_from_slice(&self.ciphertext);
        payload
    }
}

/// Delivery acknowledgment message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAck {
    /// ID of the message being acknowledged.
    pub message_id: String,
    /// Acknowledger's public key.
    pub ack_pubkey: Vec<u8>,
    /// Unix timestamp of receipt (milliseconds).
    pub received_at: u64,
    /// Signature over (message_id || received_at as big-endian bytes).
    pub signature: Vec<u8>,
}

impl DeliveryAck {
    pub fn to_bytes(&self) -> Result<Vec<u8>, crate::ProtocolError> {
        bincode::serialize(self)
            .map_err(|e| crate::ProtocolError::SerializationError(e.to_string()))
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, crate::ProtocolError> {
        bincode::deserialize(data)
            .map_err(|e| crate::ProtocolError::SerializationError(e.to_string()))
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization_roundtrip() {
        let msg = EncryptedMessage {
            sender_pubkey: vec![1u8; 32],
            nonce: vec![2u8; 24],
            ciphertext: vec![3u8; 100],
            signature: vec![4u8; 64],
            timestamp: 1713100000000,
            message_id: "test-msg-001".to_string(),
        };

        let bytes = msg.to_bytes().unwrap();
        let recovered = EncryptedMessage::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.message_id, "test-msg-001");
        assert_eq!(recovered.sender_pubkey, vec![1u8; 32]);
        assert_eq!(recovered.ciphertext, vec![3u8; 100]);
    }

    #[test]
    fn test_signed_payload() {
        let msg = EncryptedMessage {
            sender_pubkey: vec![0; 32],
            nonce: vec![0xAA; 24],
            ciphertext: vec![0xBB; 10],
            signature: vec![0; 64],
            timestamp: 0,
            message_id: String::new(),
        };

        let payload = msg.signed_payload();
        assert_eq!(payload.len(), 34); // 24 + 10
        assert_eq!(&payload[..24], &[0xAA; 24]);
        assert_eq!(&payload[24..], &[0xBB; 10]);
    }
}
