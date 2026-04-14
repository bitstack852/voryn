//! Double Ratchet message headers.
//!
//! Headers contain the sender's current DH ratchet public key,
//! the chain index (message number), and previous chain length.

use serde::{Deserialize, Serialize};

/// Unencrypted header sent alongside each Double Ratchet message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    /// Sender's current DH ratchet public key (32 bytes, X25519).
    pub dh_public_key: [u8; 32],
    /// Previous sending chain length (number of messages sent before last DH ratchet).
    pub previous_chain_length: u32,
    /// Message number within the current sending chain.
    pub message_number: u32,
}

impl Header {
    pub fn new(dh_public_key: [u8; 32], previous_chain_length: u32, message_number: u32) -> Self {
        Self {
            dh_public_key,
            previous_chain_length,
            message_number,
        }
    }

    /// Serialize header for inclusion in authenticated data.
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Header serialization should not fail")
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_roundtrip() {
        let header = Header::new([0xAA; 32], 5, 10);
        let bytes = header.to_bytes();
        let recovered = Header::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.dh_public_key, [0xAA; 32]);
        assert_eq!(recovered.previous_chain_length, 5);
        assert_eq!(recovered.message_number, 10);
    }
}
