//! Time lock — re-authentication after inactivity.
//!
//! The app transitions to the passcode screen when the user has been
//! inactive for a configurable period.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Time lock configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLockConfig {
    /// Inactivity timeout before requiring re-authentication.
    pub timeout: TimeLockTimeout,
    /// Whether time lock is enabled.
    pub enabled: bool,
}

/// Predefined timeout intervals.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeLockTimeout {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
}

impl TimeLockTimeout {
    pub fn as_duration(&self) -> Duration {
        match self {
            Self::OneMinute => Duration::from_secs(60),
            Self::FiveMinutes => Duration::from_secs(300),
            Self::FifteenMinutes => Duration::from_secs(900),
            Self::ThirtyMinutes => Duration::from_secs(1800),
            Self::OneHour => Duration::from_secs(3600),
        }
    }
}

impl Default for TimeLockConfig {
    fn default() -> Self {
        Self {
            timeout: TimeLockTimeout::FiveMinutes,
            enabled: true,
        }
    }
}

/// Runtime state for time lock tracking.
pub struct TimeLockState {
    last_activity: Instant,
    config: TimeLockConfig,
}

impl TimeLockState {
    pub fn new(config: TimeLockConfig) -> Self {
        Self {
            last_activity: Instant::now(),
            config,
        }
    }

    /// Record user activity (resets the timer).
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check if the time lock has expired (needs re-authentication).
    pub fn is_locked(&self) -> bool {
        if !self.config.enabled {
            return false;
        }
        self.last_activity.elapsed() >= self.config.timeout.as_duration()
    }

    /// Get remaining time before lock (in seconds).
    pub fn remaining_secs(&self) -> u64 {
        if !self.config.enabled {
            return u64::MAX;
        }
        let elapsed = self.last_activity.elapsed();
        let timeout = self.config.timeout.as_duration();
        if elapsed >= timeout {
            0
        } else {
            (timeout - elapsed).as_secs()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_not_locked() {
        let state = TimeLockState::new(TimeLockConfig::default());
        assert!(!state.is_locked());
    }

    #[test]
    fn test_disabled_never_locks() {
        let config = TimeLockConfig {
            timeout: TimeLockTimeout::OneMinute,
            enabled: false,
        };
        let state = TimeLockState::new(config);
        // Even without touching, should not lock when disabled
        assert!(!state.is_locked());
    }

    #[test]
    fn test_touch_resets() {
        let mut state = TimeLockState::new(TimeLockConfig::default());
        state.touch();
        assert!(!state.is_locked());
        assert!(state.remaining_secs() > 0);
    }
}
