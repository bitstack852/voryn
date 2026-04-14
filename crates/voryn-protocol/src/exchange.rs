//! Message exchange — encrypt, sign, verify, decrypt using Phase 1 basic encryption.
//!
//! Phase 1 uses X25519 DH + HKDF + XSalsa20-Poly1305 for encryption.
//! This will be replaced by the Double Ratchet protocol in Phase 2.

use serde::{Deserialize, Serialize};

use crate::message::EncryptedMessage;

/// Parameters needed to construct an encrypted message.
/// In Phase 1, the symmetric key is derived from DH(our_sk, their_pk).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// The encrypted message ready for network transmission.
    pub encrypted: EncryptedMessage,
    /// The plaintext (only kept in memory for storage, never transmitted).
    #[serde(skip)]
    pub plaintext: Option<Vec<u8>>,
}

/// Generate a unique message ID.
pub fn generate_message_id() -> String {
    // Simple UUID-like ID using random bytes
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        // Last 6 bytes as a single hex block
        u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]),
    )
}

/// Get current timestamp in milliseconds.
pub fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_message_id_unique() {
        let id1 = generate_message_id();
        let id2 = generate_message_id();
        assert_ne!(id1, id2);
        // Should look UUID-ish
        assert!(id1.contains('-'));
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp_ms();
        // Should be after 2024
        assert!(ts > 1704067200000);
    }
}
