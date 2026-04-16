//! Duress passcode — shows empty/decoy state when entered under coercion.

use serde::{Deserialize, Serialize};

/// Duress mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuressConfig {
    pub enabled: bool,
    pub duress_passcode_hash: Option<Vec<u8>>,
    pub mode: DuressMode,
    pub trigger_remote_wipe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DuressMode {
    Empty,
    Decoy,
}

impl Default for DuressConfig {
    fn default() -> Self {
        Self { enabled: false, duress_passcode_hash: None, mode: DuressMode::Empty, trigger_remote_wipe: false }
    }
}

impl DuressConfig {
    pub fn is_duress_passcode(&self, passcode_hash: &[u8]) -> bool {
        self.enabled && self.duress_passcode_hash.as_ref().is_some_and(|h| h == passcode_hash)
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

        assert!(config.is_duress_passcode(&[0xAA; 32]));
        assert!(!config.is_duress_passcode(&[0xBB; 32]));
    }
}
