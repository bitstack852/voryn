//! Random send delays to prevent timing analysis.
//!
//! Adds a configurable random delay before each message send to prevent
//! an observer from correlating send events with user actions.

use rand::Rng;
use std::time::Duration;

/// Configuration for timing obfuscation.
#[derive(Debug, Clone)]
pub struct TimingConfig {
    /// Minimum delay in milliseconds (default: 0).
    pub min_delay_ms: u64,
    /// Maximum delay in milliseconds (default: 2000).
    pub max_delay_ms: u64,
    /// Whether timing obfuscation is enabled.
    pub enabled: bool,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 0,
            max_delay_ms: 2000,
            enabled: true,
        }
    }
}

impl TimingConfig {
    /// Generate a random delay duration based on the configuration.
    pub fn random_delay(&self) -> Duration {
        if !self.enabled || self.max_delay_ms == 0 {
            return Duration::ZERO;
        }

        let mut rng = rand::thread_rng();
        let delay_ms = rng.gen_range(self.min_delay_ms..=self.max_delay_ms);
        Duration::from_millis(delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_returns_zero() {
        let config = TimingConfig {
            enabled: false,
            ..Default::default()
        };
        assert_eq!(config.random_delay(), Duration::ZERO);
    }

    #[test]
    fn test_delay_within_range() {
        let config = TimingConfig {
            min_delay_ms: 100,
            max_delay_ms: 500,
            enabled: true,
        };

        for _ in 0..100 {
            let delay = config.random_delay();
            assert!(delay >= Duration::from_millis(100));
            assert!(delay <= Duration::from_millis(500));
        }
    }
}
