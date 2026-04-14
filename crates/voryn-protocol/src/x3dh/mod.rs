//! Extended Triple Diffie-Hellman (X3DH) — initial key agreement protocol.
//!
//! X3DH establishes a shared secret between two parties for initializing
//! a Double Ratchet session. It provides mutual authentication and
//! forward secrecy even before the first message is sent.

pub mod prekeys;

pub use prekeys::{PreKeyBundle, X3DHResult};
