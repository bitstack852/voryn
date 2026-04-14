//! Time lock — re-authentication after inactivity.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLockConfig {
    pub timeout: TimeLockTimeout,
    pub enabled: bool,
}

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
        Self { timeout: TimeLockTimeout::FiveMinutes, enabled: true }
    }
}

pub struct TimeLockState {
    last_activity: Instant,
    config: TimeLockConfig,
}

impl TimeLockState {
    pub fn new(config: TimeLockConfig) -> Self {
        Self { last_activity: Instant::now(), config }
    }

    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn is_locked(&self) -> bool {
        if !self.config.enabled {
            return false;
        }
        self.last_activity.elapsed() >= self.config.timeout.as_duration()
    }

    pub fn remaining_secs(&self) -> u64 {
        if !self.config.enabled {
            return u64::MAX;
        }
        let elapsed = self.last_activity.elapsed();
        let timeout = self.config.timeout.as_duration();
        if elapsed >= timeout { 0 } else { (timeout - elapsed).as_secs() }
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
        let config = TimeLockConfig { timeout: TimeLockTimeout::OneMinute, enabled: false };
        let state = TimeLockState::new(config);
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
