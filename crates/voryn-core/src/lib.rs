//! Voryn Core — UniFFI entry point and orchestration layer.
//!
//! This crate ties together all Voryn subsystems (crypto, network, storage, protocol)
//! and exposes them to React Native via UniFFI-generated bindings.

pub mod auth;
pub mod keystore;

use voryn_crypto::identity::Identity;

/// Temporary bridge test function — verifies Rust ↔ React Native FFI works.
pub fn hello_from_rust() -> String {
    "Voryn Core v0.1.0 — Private. Encrypted. Unreachable.".to_string()
}

/// Generate a new cryptographic identity (Ed25519 keypair).
/// Returns the public-facing identity; the secret key is stored internally.
pub fn generate_identity() -> Result<Identity, VorynError> {
    voryn_crypto::identity::generate_identity()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_from_rust() {
        let greeting = hello_from_rust();
        assert!(greeting.contains("Voryn Core"));
        assert!(greeting.contains("Unreachable"));
    }
}
