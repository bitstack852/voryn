//! Cryptographic identity — Ed25519 keypair generation and management.

use sodiumoxide::crypto::sign;
use zeroize::Zeroize;

use crate::CryptoError;

/// A Voryn identity consisting of a public key and display-friendly ID.
#[derive(Debug, Clone)]
pub struct Identity {
    /// Ed25519 public key (32 bytes).
    pub public_key: Vec<u8>,
    /// Hex-encoded public key for display.
    pub public_key_hex: String,
}

/// Ed25519 keypair — the secret key implements Zeroize for secure cleanup.
pub struct Keypair {
    pub public_key: sign::PublicKey,
    pub secret_key: SecretKeyWrapper,
}

/// Wrapper around Ed25519 secret key that zeroes memory on drop.
pub struct SecretKeyWrapper {
    bytes: Vec<u8>,
}

impl SecretKeyWrapper {
    pub fn new(sk: &sign::SecretKey) -> Self {
        Self {
            bytes: sk.as_ref().to_vec(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn to_sign_secret_key(&self) -> Option<sign::SecretKey> {
        sign::SecretKey::from_slice(&self.bytes)
    }
}

impl Drop for SecretKeyWrapper {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

/// Generate a new Ed25519 identity.
pub fn generate_identity() -> Result<Identity, CryptoError> {
    crate::init()?;

    let (pk, _sk) = sign::gen_keypair();
    let public_key = pk.as_ref().to_vec();
    let public_key_hex = hex::encode(&public_key);

    Ok(Identity {
        public_key,
        public_key_hex,
    })
}

/// Generate a full keypair (public + secret).
/// The caller is responsible for securely storing the secret key.
pub fn generate_keypair() -> Result<Keypair, CryptoError> {
    crate::init()?;

    let (pk, sk) = sign::gen_keypair();
    Ok(Keypair {
        public_key: pk,
        secret_key: SecretKeyWrapper::new(&sk),
    })
}

// hex encoding helper — we keep it minimal to avoid extra dependencies
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_identity() {
        let id = generate_identity().unwrap();
        assert_eq!(id.public_key.len(), 32);
        assert_eq!(id.public_key_hex.len(), 64);
    }

    #[test]
    fn test_keypairs_are_unique() {
        let kp1 = generate_keypair().unwrap();
        let kp2 = generate_keypair().unwrap();
        assert_ne!(kp1.public_key, kp2.public_key);
    }

    #[test]
    fn test_secret_key_roundtrip() {
        let kp = generate_keypair().unwrap();
        let recovered = kp.secret_key.to_sign_secret_key().unwrap();
        assert_eq!(kp.secret_key.as_bytes(), recovered.as_ref());
    }
}
