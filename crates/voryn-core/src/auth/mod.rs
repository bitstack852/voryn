//! Authentication module — passcode, biometrics, attempt limiting, duress.
//!
//! Implementations will be added in Phase 2 (passcode) and Phase 3 (duress, time lock).

/// Authentication state for the current session.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    /// App is locked, requires passcode entry.
    Locked,
    /// App is unlocked and active.
    Unlocked,
    /// App entered duress mode (shows decoy data).
    Duress,
}
