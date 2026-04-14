//! Double Ratchet Algorithm — forward-secret messaging protocol.
//!
//! Implements the Signal Double Ratchet specification for forward-secret
//! one-to-one messaging. Each message uses a unique encryption key;
//! compromising one key does not reveal past or future messages.
//!
//! References:
//! - https://signal.org/docs/specifications/doubleratchet/

pub mod chain;
pub mod session;
pub mod header;

pub use chain::{Chain, ChainKey, MessageKey};
pub use header::Header;
pub use session::Session;
