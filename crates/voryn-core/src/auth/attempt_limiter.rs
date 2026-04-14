//! Passcode attempt limiting — configurable failed attempt counter with data wipe.
//!
//! The attempt counter is stored outside SQLCipher (in a simple file or
//! platform keychain) since SQLCipher requires the passcode to open.

use serde::{Deserialize, Serialize};

/// Configuration for passcode attempt limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptConfig {
    /// Maximum failed attempts before wipe (default: 10).
    pub max_attempts: u32,
    /// Current failed attempt count.
    pub failed_count: u32,
    /// Progressive delay enabled (exponential backoff between attempts).
    pub progressive_delay: bool,
}

impl Default for AttemptConfig {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            failed_count: 0,
            progressive_delay: true,
        }
    }
}

impl AttemptConfig {
    /// Record a failed attempt. Returns true if wipe threshold reached.
    pub fn record_failure(&mut self) -> AttemptResult {
        self.failed_count += 1;

        if self.failed_count >= self.max_attempts {
            return AttemptResult::WipeTriggered;
        }

        let remaining = self.max_attempts - self.failed_count;
        let delay_ms = if self.progressive_delay {
            // Exponential backoff: 1s, 2s, 4s, 8s, 16s, 32s...
            (1000 * 2u64.pow(self.failed_count.min(10) - 1)).min(60_000)
        } else {
            0
        };

        AttemptResult::RetryAfter {
            remaining,
            delay_ms,
        }
    }

    /// Record a successful attempt (reset counter).
    pub fn record_success(&mut self) {
        self.failed_count = 0;
    }

    /// Get remaining attempts.
    pub fn remaining(&self) -> u32 {
        self.max_attempts.saturating_sub(self.failed_count)
    }
}

/// Result of a passcode attempt.
#[derive(Debug, PartialEq)]
pub enum AttemptResult {
    /// Wipe threshold reached — destroy all data.
    WipeTriggered,
    /// Retry allowed after delay.
    RetryAfter {
        remaining: u32,
        delay_ms: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AttemptConfig::default();
        assert_eq!(config.max_attempts, 10);
        assert_eq!(config.failed_count, 0);
        assert_eq!(config.remaining(), 10);
    }

    #[test]
    fn test_record_failure_counts_down() {
        let mut config = AttemptConfig::default();
        let result = config.record_failure();
        assert!(matches!(result, AttemptResult::RetryAfter { remaining: 9, .. }));
        assert_eq!(config.remaining(), 9);
    }

    #[test]
    fn test_wipe_triggered_at_limit() {
        let mut config = AttemptConfig {
            max_attempts: 3,
            failed_count: 0,
            progressive_delay: false,
        };

        config.record_failure();
        config.record_failure();
        let result = config.record_failure();
        assert_eq!(result, AttemptResult::WipeTriggered);
    }

    #[test]
    fn test_success_resets_counter() {
        let mut config = AttemptConfig::default();
        config.record_failure();
        config.record_failure();
        assert_eq!(config.remaining(), 8);

        config.record_success();
        assert_eq!(config.remaining(), 10);
    }

    #[test]
    fn test_progressive_delay() {
        let mut config = AttemptConfig::default();
        if let AttemptResult::RetryAfter { delay_ms, .. } = config.record_failure() {
            assert_eq!(delay_ms, 1000); // 2^0 * 1000
        }
        if let AttemptResult::RetryAfter { delay_ms, .. } = config.record_failure() {
            assert_eq!(delay_ms, 2000); // 2^1 * 1000
        }
        if let AttemptResult::RetryAfter { delay_ms, .. } = config.record_failure() {
            assert_eq!(delay_ms, 4000); // 2^2 * 1000
        }
    }
}
