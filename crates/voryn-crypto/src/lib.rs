//! Voryn Crypto — Cryptographic primitives built on libsodium.
//!
//! Provides Ed25519 identity generation, X25519 key exchange,
//! XSalsa20-Poly1305 symmetric encryption, and secure key derivation.

pub mod identity;
pub mod keys;
pub mod signing;
pub mod encryption;
pub mod kdf;
pub mod secure_memory;

/// Initialize the sodiumoxide library. Must be called before any crypto operations.
pub fn init() -> Result<(), CryptoError> {
    sodiumoxide::init().map_err(|_| CryptoError::InitFailed)
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Failed to initialize sodiumoxide")]
    InitFailed,

    #[error("Key generation failed")]
    KeyGenFailed,

    #[error("Signing failed: {0}")]
    SigningError(String),

    #[error("Verification failed")]
    VerificationFailed,

    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Key derivation failed: {0}")]
    KdfError(String),
}
