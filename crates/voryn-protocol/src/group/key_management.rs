//! Group key lifecycle — generation, distribution via Shamir, resharing.

use super::shamir::{self, Share};
use serde::{Deserialize, Serialize};

/// A group key epoch — one generation of the group encryption key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupKeyEpoch {
    /// Epoch number (increments on every reshare).
    pub epoch: u64,
    /// Threshold required for key reconstruction.
    pub threshold: u32,
    /// Total number of shares distributed.
    pub share_count: u32,
    /// Our share of this epoch's key.
    pub our_share: Share,
    /// The reconstructed group key (cached after first reconstruction).
    /// This is encrypted at rest via SQLCipher.
    pub cached_key: Option<Vec<u8>>,
}

/// Generate a new group key, split it, and return shares for distribution.
pub fn generate_group_key(
    threshold: u8,
    member_count: u8,
) -> Result<(Vec<u8>, Vec<Share>), String> {
    // Generate random 32-byte group key
    let mut key = vec![0u8; 32];
    sodiumoxide::randombytes::randombytes_into(&mut key);

    // Split into shares
    let shares = shamir::split_secret(&key, threshold, member_count)?;

    Ok((key, shares))
}

/// Reconstruct the group key from collected shares.
pub fn reconstruct_group_key(shares: &[Share]) -> Result<Vec<u8>, String> {
    shamir::reconstruct_secret(shares)
}

/// Calculate the default threshold for a group size.
/// T = ceil(N * 2/3), minimum 2.
pub fn default_threshold(member_count: u8) -> u8 {
    let threshold = ((member_count as f64 * 2.0 / 3.0).ceil()) as u8;
    threshold.max(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_reconstruct() {
        sodiumoxide::init().ok();
        let (original_key, shares) = generate_group_key(3, 5).unwrap();
        assert_eq!(shares.len(), 5);
        assert_eq!(original_key.len(), 32);

        let reconstructed = reconstruct_group_key(&shares[..3]).unwrap();
        assert_eq!(reconstructed, original_key);
    }

    #[test]
    fn test_default_threshold() {
        assert_eq!(default_threshold(3), 2);
        assert_eq!(default_threshold(5), 4); // ceil(5 * 2/3) = 4
        assert_eq!(default_threshold(10), 7); // ceil(10 * 2/3) = 7
        assert_eq!(default_threshold(2), 2);
    }
}
