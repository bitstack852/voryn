//! Diffie-Hellman key exchange — X25519 shared secret derivation.
//!
//! Converts Ed25519 keys to X25519 and performs scalar multiplication
//! to derive a shared secret for symmetric encryption.

use sodiumoxide::crypto::sign;
use zeroize::Zeroize;

use crate::keys::{ed25519_pk_to_x25519, ed25519_sk_to_x25519};
use crate::CryptoError;

/// Shared secret derived from X25519 DH exchange (32 bytes).
/// Auto-zeroed on drop.
pub struct SharedSecret {
    bytes: [u8; 32],
}

impl SharedSecret {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl Drop for SharedSecret {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

/// Perform X25519 Diffie-Hellman key exchange.
///
/// Given our Ed25519 secret key and the peer's Ed25519 public key,
/// derive a shared secret that both parties can independently compute.
pub fn compute_shared_secret(
    our_sk: &sign::SecretKey,
    their_pk: &sign::PublicKey,
) -> Result<SharedSecret, CryptoError> {
    let our_x25519_sk = ed25519_sk_to_x25519(our_sk)?;
    let their_x25519_pk = ed25519_pk_to_x25519(their_pk)?;

    // crypto_scalarmult: X25519(our_sk, their_pk) -> shared_secret
    let mut shared = [0u8; 32];
    let ret = unsafe {
        libsodium_sys::crypto_scalarmult(
            shared.as_mut_ptr(),
            our_x25519_sk.as_ptr(),
            their_x25519_pk.as_ptr(),
        )
    };

    // Zero the temporary X25519 secret key
    let mut sk_copy = our_x25519_sk;
    sk_copy.zeroize();

    if ret != 0 {
        return Err(CryptoError::EncryptionError("DH exchange failed".into()));
    }

    Ok(SharedSecret { bytes: shared })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sodiumoxide::crypto::sign;

    #[test]
    fn test_shared_secret_symmetric() {
        crate::init().unwrap();
        let (pk_a, sk_a) = sign::gen_keypair();
        let (pk_b, sk_b) = sign::gen_keypair();

        // Alice computes shared secret with Bob's public key
        let shared_ab = compute_shared_secret(&sk_a, &pk_b).unwrap();
        // Bob computes shared secret with Alice's public key
        let shared_ba = compute_shared_secret(&sk_b, &pk_a).unwrap();

        // Both should be identical
        assert_eq!(shared_ab.as_bytes(), shared_ba.as_bytes());
    }

    #[test]
    fn test_different_peers_different_secrets() {
        crate::init().unwrap();
        let (_pk_a, sk_a) = sign::gen_keypair();
        let (pk_b, _sk_b) = sign::gen_keypair();
        let (pk_c, _sk_c) = sign::gen_keypair();

        let shared_ab = compute_shared_secret(&sk_a, &pk_b).unwrap();
        let shared_ac = compute_shared_secret(&sk_a, &pk_c).unwrap();

        assert_ne!(shared_ab.as_bytes(), shared_ac.as_bytes());
    }
}
