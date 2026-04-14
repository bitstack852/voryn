//! KDF chains — root chain, sending chain, receiving chain.
//!
//! Implements the symmetric-key ratchet component of the Double Ratchet protocol.
//! Each chain maintains a chain key from which message keys are derived.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A KDF chain key (32 bytes). Used to derive message keys and the next chain key.
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct ChainKey {
    key: [u8; 32],
}

/// A message key derived from a chain key (32 bytes). Used once for encryption.
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct MessageKey {
    key: [u8; 32],
}

/// A KDF chain that produces message keys and advances its chain key.
#[derive(Clone, Serialize, Deserialize)]
pub struct Chain {
    /// Current chain key.
    chain_key: ChainKey,
    /// Number of message keys derived so far (chain index).
    index: u32,
}

impl ChainKey {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }

    /// Derive the next chain key using HMAC-SHA256 with constant 0x02.
    pub fn next_chain_key(&self) -> ChainKey {
        let derived = hmac_sha256(&self.key, &[0x02]);
        ChainKey { key: derived }
    }

    /// Derive a message key using HMAC-SHA256 with constant 0x01.
    pub fn message_key(&self) -> MessageKey {
        let derived = hmac_sha256(&self.key, &[0x01]);
        MessageKey { key: derived }
    }
}

impl MessageKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

impl Chain {
    /// Create a new chain from an initial chain key.
    pub fn new(chain_key: ChainKey) -> Self {
        Self {
            chain_key,
            index: 0,
        }
    }

    /// Get the current chain index.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Derive the next message key and advance the chain.
    pub fn next_message_key(&mut self) -> MessageKey {
        let mk = self.chain_key.message_key();
        self.chain_key = self.chain_key.next_chain_key();
        self.index += 1;
        mk
    }

    /// Advance the chain without producing a message key (skip).
    pub fn advance(&mut self) {
        self.chain_key = self.chain_key.next_chain_key();
        self.index += 1;
    }
}

/// HMAC-SHA256 using libsodium's crypto_auth_hmacsha256.
fn hmac_sha256(key: &[u8; 32], data: &[u8]) -> [u8; 32] {
    use sodiumoxide::crypto::auth::hmacsha256;
    let auth_key = hmacsha256::Key::from_slice(key).unwrap();
    let tag = hmacsha256::authenticate(data, &auth_key);
    let mut result = [0u8; 32];
    result.copy_from_slice(tag.as_ref());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_key_derivation_deterministic() {
        let ck = ChainKey::new([0xAA; 32]);
        let mk1 = ck.message_key();
        let mk2 = ck.message_key();
        // Same chain key -> same message key
        assert_eq!(mk1.as_bytes(), mk2.as_bytes());
    }

    #[test]
    fn test_chain_key_advances() {
        let ck = ChainKey::new([0xAA; 32]);
        let next = ck.next_chain_key();
        // Different chain key after advance
        assert_ne!(ck.as_bytes(), next.as_bytes());
    }

    #[test]
    fn test_chain_produces_unique_message_keys() {
        let mut chain = Chain::new(ChainKey::new([0xBB; 32]));
        let mk1 = chain.next_message_key();
        let mk2 = chain.next_message_key();
        let mk3 = chain.next_message_key();
        // Each message key should be unique
        assert_ne!(mk1.as_bytes(), mk2.as_bytes());
        assert_ne!(mk2.as_bytes(), mk3.as_bytes());
        assert_eq!(chain.index(), 3);
    }

    #[test]
    fn test_chain_index_increments() {
        let mut chain = Chain::new(ChainKey::new([0; 32]));
        assert_eq!(chain.index(), 0);
        chain.next_message_key();
        assert_eq!(chain.index(), 1);
        chain.advance();
        assert_eq!(chain.index(), 2);
    }
}
