//! Dummy/chaff traffic generation to mask real message patterns.
//!
//! Sends encrypted dummy messages at random intervals that are
//! indistinguishable from real messages on the wire.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for chaff traffic generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaffConfig {
    /// Whether chaff traffic is enabled.
    pub enabled: bool,
    /// Minimum interval between chaff messages (seconds).
    pub min_interval_secs: u64,
    /// Maximum interval between chaff messages (seconds).
    pub max_interval_secs: u64,
    /// Reduce rate when on battery power.
    pub adaptive_rate: bool,
}

impl Default for ChaffConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_interval_secs: 30,
            max_interval_secs: 120,
            adaptive_rate: true,
        }
    }
}

impl ChaffConfig {
    /// Generate a random interval until the next chaff message.
    pub fn next_interval(&self) -> Duration {
        if !self.enabled {
            return Duration::from_secs(u64::MAX);
        }

        let mut rng = rand::thread_rng();
        let secs = rng.gen_range(self.min_interval_secs..=self.max_interval_secs);
        Duration::from_secs(secs)
    }

    /// Generate random chaff payload (looks like an encrypted message).
    pub fn generate_chaff_payload(&self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        // Random size matching typical message bucket sizes
        let sizes = [256, 1024, 4096];
        let size = sizes[rng.gen_range(0..sizes.len())];
        (0..size).map(|_| rng.gen::<u8>()).collect()
    }
}

/// Marker in chaff messages (only detectable by the recipient who can decrypt).
/// Chaff messages contain this magic byte sequence at the start after decryption.
pub const CHAFF_MAGIC: &[u8; 9] = b"VORNCHAFF";

/// Check if decrypted content is chaff (should be silently dropped).
pub fn is_chaff(decrypted: &[u8]) -> bool {
    decrypted.len() >= CHAFF_MAGIC.len() && &decrypted[..CHAFF_MAGIC.len()] == CHAFF_MAGIC.as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaff_detection() {
        let mut chaff = CHAFF_MAGIC.to_vec();
        chaff.extend_from_slice(b"random padding data");
        assert!(is_chaff(&chaff));

        assert!(!is_chaff(b"real message content"));
        assert!(!is_chaff(b"VORN")); // Too short
    }

    #[test]
    fn test_chaff_payload_generation() {
        let config = ChaffConfig::default();
        let payload = config.generate_chaff_payload();
        assert!(!payload.is_empty());
        assert!([256, 1024, 4096].contains(&payload.len()));
    }

    #[test]
    fn test_disabled_infinite_interval() {
        let config = ChaffConfig {
            enabled: false,
            ..Default::default()
        };
        assert_eq!(config.next_interval(), Duration::from_secs(u64::MAX));
    }
}
