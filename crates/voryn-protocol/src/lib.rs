//! Voryn Protocol — Messaging protocols and cryptographic constructs.
//!
//! - Double Ratchet Algorithm for forward-secret 1-to-1 messaging
//! - X3DH for initial key agreement
//! - Shamir's Secret Sharing for group key distribution
//! - Cryptographic group ledger for membership management
//! - Invite token system
//! - Auto-delete timers

pub mod message;
pub mod double_ratchet;
pub mod x3dh;
pub mod group;
pub mod invite;
pub mod autodelete;

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Ratchet error: {0}")]
    RatchetError(String),

    #[error("Key agreement error: {0}")]
    KeyAgreementError(String),

    #[error("Group error: {0}")]
    GroupError(String),

    #[error("Invite error: {0}")]
    InviteError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
