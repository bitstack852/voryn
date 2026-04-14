//! Signed, single-use invite tokens.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// A cryptographic invite token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteToken {
    /// Inviter's Ed25519 public key.
    pub inviter_pubkey: Vec<u8>,
    /// Unique token identifier.
    pub token_id: String,
    /// Unix timestamp when created (milliseconds).
    pub created_at: u64,
    /// Unix timestamp when it expires (milliseconds).
    pub expires_at: u64,
    /// Ed25519 signature over (inviter_pubkey || token_id || created_at || expires_at).
    pub signature: Vec<u8>,
}

/// Default token validity: 48 hours.
const DEFAULT_EXPIRY_MS: u64 = 48 * 60 * 60 * 1000;

impl InviteToken {
    /// Create a new unsigned invite token.
    pub fn new(inviter_pubkey: Vec<u8>) -> Self {
        let now = crate::exchange::current_timestamp_ms();
        Self {
            inviter_pubkey,
            token_id: generate_token_id(),
            created_at: now,
            expires_at: now + DEFAULT_EXPIRY_MS,
            signature: Vec::new(),
        }
    }

    /// Create a token with custom expiry (in milliseconds from now).
    pub fn with_expiry(inviter_pubkey: Vec<u8>, expiry_ms: u64) -> Self {
        let now = crate::exchange::current_timestamp_ms();
        Self {
            inviter_pubkey,
            token_id: generate_token_id(),
            created_at: now,
            expires_at: now + expiry_ms,
            signature: Vec::new(),
        }
    }

    /// Get the data that should be signed.
    pub fn signable_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.inviter_pubkey);
        data.extend_from_slice(self.token_id.as_bytes());
        data.extend_from_slice(&self.created_at.to_be_bytes());
        data.extend_from_slice(&self.expires_at.to_be_bytes());
        data
    }

    /// Check if the token has expired.
    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        current_time_ms > self.expires_at
    }

    /// Encode as a Base58-like string for human sharing.
    pub fn encode(&self) -> Result<String, String> {
        let bytes = bincode::serialize(self).map_err(|e| e.to_string())?;
        Ok(base58_encode(&bytes))
    }

    /// Decode from an encoded string.
    pub fn decode(encoded: &str) -> Result<Self, String> {
        let bytes = base58_decode(encoded)?;
        bincode::deserialize(&bytes).map_err(|e| e.to_string())
    }
}

/// Generate a random token ID (16 hex chars).
fn generate_token_id() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 8] = rng.gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Simple Base58 encoding (Bitcoin-style alphabet).
fn base58_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    if data.is_empty() {
        return String::new();
    }

    // Convert bytes to base58
    let mut digits: Vec<u8> = vec![0];
    for &byte in data {
        let mut carry = byte as u32;
        for digit in digits.iter_mut() {
            carry += (*digit as u32) * 256;
            *digit = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }

    // Add leading zeros
    for &byte in data {
        if byte == 0 {
            digits.push(0);
        } else {
            break;
        }
    }

    digits.iter().rev().map(|&d| ALPHABET[d as usize] as char).collect()
}

/// Simple Base58 decoding.
fn base58_decode(encoded: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    let mut bytes: Vec<u8> = vec![0];
    for ch in encoded.chars() {
        let val = ALPHABET.find(ch).ok_or(format!("Invalid base58 character: {}", ch))? as u32;
        let mut carry = val;
        for byte in bytes.iter_mut() {
            carry += (*byte as u32) * 58;
            *byte = (carry % 256) as u8;
            carry /= 256;
        }
        while carry > 0 {
            bytes.push((carry % 256) as u8);
            carry /= 256;
        }
    }

    // Add leading zeros
    for ch in encoded.chars() {
        if ch == '1' {
            bytes.push(0);
        } else {
            break;
        }
    }

    bytes.reverse();
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_token() {
        let token = InviteToken::new(vec![1u8; 32]);
        assert!(!token.token_id.is_empty());
        assert!(token.expires_at > token.created_at);
        assert!(!token.is_expired(token.created_at + 1000));
    }

    #[test]
    fn test_token_expiry() {
        let token = InviteToken::with_expiry(vec![1u8; 32], 1000); // 1 second
        assert!(!token.is_expired(token.created_at + 500));
        assert!(token.is_expired(token.created_at + 2000));
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let token = InviteToken::new(vec![1u8; 32]);
        let encoded = token.encode().unwrap();
        assert!(!encoded.is_empty());
        let decoded = InviteToken::decode(&encoded).unwrap();
        assert_eq!(decoded.token_id, token.token_id);
        assert_eq!(decoded.inviter_pubkey, token.inviter_pubkey);
    }
}
