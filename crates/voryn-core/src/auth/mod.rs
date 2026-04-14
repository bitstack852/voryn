//! Authentication module — passcode, biometrics, attempt limiting, duress.

pub mod attempt_limiter;
pub mod duress;
pub mod timelock;

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
