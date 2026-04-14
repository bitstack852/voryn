//! Duress passcode — shows empty/decoy state when entered under coercion.
//!
//! The duress passcode is a separate passcode that, when entered, presents
//! the app as if it were freshly installed or contains decoy data.
//! The real database remains encrypted and inaccessible.

use serde::{Deserialize, Serialize};

/// Duress mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuressConfig {
    /// Whether duress mode is enabled.
    pub enabled: bool,
    /// Hash of the duress passcode (for comparison without storing plaintext).
    /// Stored separately from the real passcode.
    pub duress_passcode_hash: Option<Vec<u8>>,
    /// What to show in duress mode.
    pub mode: DuressMode,
    /// Whether to silently trigger a remote wipe signal.
    pub trigger_remote_wipe: bool,
}

/// What the app shows when the duress passcode is entered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DuressMode {
    /// Show a completely empty state (no contacts, no messages).
    Empty,
    /// Show pre-configured decoy contacts and messages.
    Decoy,
}

impl Default for DuressConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            duress_passcode_hash: None,
            mode: DuressMode::Empty,
            trigger_remote_wipe: false,
        }
    }
}

impl DuressConfig {
    /// Check if a given passcode hash matches the duress passcode.
    pub fn is_duress_passcode(&self, passcode_hash: &[u8]) -> bool {
        self.enabled
            && self
                .duress_passcode_hash
                .as_ref()
                .map_or(false, |h| h == passcode_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        let config = DuressConfig::default();
        assert!(!config.enabled);
        assert!(!config.is_duress_passcode(b"anything"));
    }

    #[test]
    fn test_duress_detection() {
        let config = DuressConfig {
            enabled: true,
            duress_passcode_hash: Some(vec![0xAA; 32]),
            mode: DuressMode::Empty,
            trigger_remote_wipe: false,
        };

        assert!(config.is_duress_passcode(&vec![0xAA; 32]));
        assert!(!config.is_duress_passcode(&vec![0xBB; 32]));
    }
}
