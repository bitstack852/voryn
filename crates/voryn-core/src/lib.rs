//! Voryn Core — UniFFI entry point and orchestration layer.
//!
//! This crate ties together all Voryn subsystems (crypto, network, storage, protocol)
//! and exposes them to React Native via UniFFI-generated bindings.

pub mod auth;
pub mod keystore;
pub mod messaging;
pub mod wipe;

use voryn_crypto::identity::{Identity, IdentityWithSecret};

/// Temporary bridge test function — verifies Rust ↔ React Native FFI works.
pub fn hello_from_rust() -> String {
    "Voryn Core v0.1.0 — Private. Encrypted. Unreachable.".to_string()
}

/// Generate a new cryptographic identity (Ed25519 keypair).
/// Returns the public-facing identity; the secret key must be stored separately.
pub fn generate_identity() -> Result<Identity, VorynError> {
    voryn_crypto::identity::generate_identity()
        .map_err(|e| VorynError::CryptoError(e.to_string()))
}

/// Generate a full identity with secret key for persistence.
pub fn generate_full_identity() -> Result<IdentityWithSecret, VorynError> {
    voryn_crypto::identity::generate_full_identity()
        .map_err(|e| VorynError::CryptoError(e.to_string()))
}

#[derive(Debug, thiserror::Error)]
pub enum VorynError {
    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

impl From<voryn_crypto::CryptoError> for VorynError {
    fn from(e: voryn_crypto::CryptoError) -> Self {
        VorynError::CryptoError(e.to_string())
    }
}

impl From<voryn_network::NetworkError> for VorynError {
    fn from(e: voryn_network::NetworkError) -> Self {
        VorynError::NetworkError(e.to_string())
    }
}

impl From<voryn_storage::StorageError> for VorynError {
    fn from(e: voryn_storage::StorageError) -> Self {
        VorynError::StorageError(e.to_string())
    }
}

impl From<voryn_protocol::ProtocolError> for VorynError {
    fn from(e: voryn_protocol::ProtocolError) -> Self {
        VorynError::ProtocolError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_from_rust() {
        let greeting = hello_from_rust();
        assert!(greeting.contains("Voryn Core"));
        assert!(greeting.contains("Unreachable"));
    }

    #[test]
    fn test_generate_identity() {
        let id = generate_identity().unwrap();
        assert_eq!(id.public_key.len(), 32);
    }

    #[test]
    fn test_generate_full_identity() {
        let full = generate_full_identity().unwrap();
        assert_eq!(full.identity.public_key.len(), 32);
        assert_eq!(full.keypair.secret_key.as_bytes().len(), 64);
    }
}
