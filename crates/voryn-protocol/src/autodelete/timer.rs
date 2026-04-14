//! Per-conversation auto-delete timers.
//!
//! Timer starts on delivery confirmation (not on send).
//! When expired, message is securely deleted from SQLCipher.

use serde::{Deserialize, Serialize};

/// Auto-delete timer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDeleteConfig {
    /// Whether auto-delete is enabled for this conversation.
    pub enabled: bool,
    /// Timer interval.
    pub interval: DeleteInterval,
}

/// Predefined delete intervals.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeleteInterval {
    OneHour,
    TwentyFourHours,
    SevenDays,
    ThirtyDays,
    /// Custom interval in seconds.
    Custom(u64),
}

impl DeleteInterval {
    /// Get the interval in milliseconds.
    pub fn as_millis(&self) -> u64 {
        match self {
            Self::OneHour => 60 * 60 * 1000,
            Self::TwentyFourHours => 24 * 60 * 60 * 1000,
            Self::SevenDays => 7 * 24 * 60 * 60 * 1000,
            Self::ThirtyDays => 30 * 24 * 60 * 60 * 1000,
            Self::Custom(secs) => secs * 1000,
        }
    }

    /// Human-readable display.
    pub fn display(&self) -> String {
        match self {
            Self::OneHour => "1 hour".into(),
            Self::TwentyFourHours => "24 hours".into(),
            Self::SevenDays => "7 days".into(),
            Self::ThirtyDays => "30 days".into(),
            Self::Custom(secs) => {
                if *secs < 3600 {
                    format!("{} minutes", secs / 60)
                } else if *secs < 86400 {
                    format!("{} hours", secs / 3600)
                } else {
                    format!("{} days", secs / 86400)
                }
            }
        }
    }
}

impl Default for AutoDeleteConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: DeleteInterval::TwentyFourHours,
        }
    }
}

/// A message's delete schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeleteSchedule {
    /// Message ID.
    pub message_id: String,
    /// Timestamp when the message was delivered.
    pub delivered_at: u64,
    /// Timestamp when the message should be deleted.
    pub delete_at: u64,
}

impl MessageDeleteSchedule {
    /// Create a schedule for a delivered message.
    pub fn new(message_id: String, delivered_at: u64, interval: &DeleteInterval) -> Self {
        Self {
            message_id,
            delivered_at,
            delete_at: delivered_at + interval.as_millis(),
        }
    }

    /// Check if this message should be deleted now.
    pub fn is_due(&self, current_time_ms: u64) -> bool {
        current_time_ms >= self.delete_at
    }

    /// Get remaining time in milliseconds.
    pub fn remaining_ms(&self, current_time_ms: u64) -> u64 {
        self.delete_at.saturating_sub(current_time_ms)
    }
}

/// Protocol message for synchronizing timer settings between peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSettingSync {
    /// Conversation ID.
    pub conversation_id: String,
    /// Sender's public key.
    pub sender_pubkey: Vec<u8>,
    /// New timer configuration.
    pub config: AutoDeleteConfig,
    /// Timestamp.
    pub timestamp: u64,
    /// Signature.
    pub signature: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_interval_millis() {
        assert_eq!(DeleteInterval::OneHour.as_millis(), 3_600_000);
        assert_eq!(DeleteInterval::TwentyFourHours.as_millis(), 86_400_000);
        assert_eq!(DeleteInterval::Custom(3600).as_millis(), 3_600_000);
    }

    #[test]
    fn test_message_schedule() {
        let now = 1713100000000u64;
        let schedule = MessageDeleteSchedule::new(
            "msg-1".into(),
            now,
            &DeleteInterval::OneHour,
        );

        assert!(!schedule.is_due(now + 1000)); // 1 second later
        assert!(schedule.is_due(now + 3_600_001)); // 1 hour + 1ms later
        assert_eq!(schedule.remaining_ms(now + 1_800_000), 1_800_000); // 30 min remaining
    }

    #[test]
    fn test_display_intervals() {
        assert_eq!(DeleteInterval::OneHour.display(), "1 hour");
        assert_eq!(DeleteInterval::SevenDays.display(), "7 days");
        assert_eq!(DeleteInterval::Custom(7200).display(), "2 hours");
    }
}
