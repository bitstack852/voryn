//! Safety numbers — out-of-band verification of contact identity.
//!
//! Safety numbers are derived from both parties' identity keys and are
//! identical when computed by either party. Users compare safety numbers
//! (displayed as numeric strings or QR codes) to verify they're communicating
//! with the intended person.

use serde::{Deserialize, Serialize};

/// A safety number for verifying a contact's identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyNumber {
    /// Numeric string representation (e.g., "12345 67890 12345 67890 12345 67890").
    pub display: String,
    /// Raw bytes (32 bytes) for QR code encoding.
    pub bytes: Vec<u8>,
}

/// Compute a safety number from two identity public keys.
///
/// The computation is symmetric: compute(A, B) == compute(B, A).
/// This is achieved by sorting the keys before hashing.
pub fn compute_safety_number(
    our_pubkey: &[u8; 32],
    their_pubkey: &[u8; 32],
) -> SafetyNumber {
    // Sort keys to ensure symmetry
    let (first, second) = if our_pubkey < their_pubkey {
        (our_pubkey, their_pubkey)
    } else {
        (their_pubkey, our_pubkey)
    };

    // Hash: SHA-256(first || second)
    let mut input = [0u8; 64];
    input[..32].copy_from_slice(first);
    input[32..].copy_from_slice(second);

    let mut hash = [0u8; 32];
    unsafe {
        libsodium_sys::crypto_hash_sha256(
            hash.as_mut_ptr(),
            input.as_ptr(),
            input.len() as u64,
        );
    }

    // Convert to numeric display: take each byte mod 100000 in groups of 5
    let display = format_safety_number(&hash);

    SafetyNumber {
        display,
        bytes: hash.to_vec(),
    }
}

/// Format 32 bytes as a human-readable safety number.
/// Groups of 5 digits separated by spaces.
fn format_safety_number(hash: &[u8; 32]) -> String {
    let mut groups = Vec::new();
    for chunk in hash.chunks(4) {
        let n = u32::from_be_bytes([
            chunk.first().copied().unwrap_or(0),
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
            chunk.get(3).copied().unwrap_or(0),
        ]);
        groups.push(format!("{:05}", n % 100000));
    }
    groups.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_number_symmetric() {
        let pk_a = [1u8; 32];
        let pk_b = [2u8; 32];

        let sn_ab = compute_safety_number(&pk_a, &pk_b);
        let sn_ba = compute_safety_number(&pk_b, &pk_a);

        assert_eq!(sn_ab.display, sn_ba.display);
        assert_eq!(sn_ab.bytes, sn_ba.bytes);
    }

    #[test]
    fn test_different_keys_different_numbers() {
        let pk_a = [1u8; 32];
        let pk_b = [2u8; 32];
        let pk_c = [3u8; 32];

        let sn_ab = compute_safety_number(&pk_a, &pk_b);
        let sn_ac = compute_safety_number(&pk_a, &pk_c);

        assert_ne!(sn_ab.display, sn_ac.display);
    }

    #[test]
    fn test_safety_number_format() {
        let pk_a = [0xAA; 32];
        let pk_b = [0xBB; 32];

        let sn = compute_safety_number(&pk_a, &pk_b);
        // Should be groups of 5 digits separated by spaces
        let parts: Vec<&str> = sn.display.split(' ').collect();
        assert_eq!(parts.len(), 8); // 32 bytes / 4 bytes per group = 8 groups
        for part in parts {
            assert_eq!(part.len(), 5);
            assert!(part.chars().all(|c| c.is_ascii_digit()));
        }
    }
}
