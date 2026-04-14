//! Key conversion utilities — Ed25519 ↔ X25519 for Diffie-Hellman key exchange.

use sodiumoxide::crypto::sign;
use crate::CryptoError;

/// Convert an Ed25519 public key to an X25519 public key for DH exchange.
pub fn ed25519_pk_to_x25519(ed_pk: &sign::PublicKey) -> Result<[u8; 32], CryptoError> {
    let mut x25519_pk = [0u8; 32];
    // libsodium: crypto_sign_ed25519_pk_to_curve25519
    let ret = unsafe {
        libsodium_sys::crypto_sign_ed25519_pk_to_curve25519(
            x25519_pk.as_mut_ptr(),
            ed_pk.as_ref().as_ptr(),
        )
    };
    if ret != 0 {
        return Err(CryptoError::KeyGenFailed);
    }
    Ok(x25519_pk)
}

/// Convert an Ed25519 secret key to an X25519 secret key for DH exchange.
pub fn ed25519_sk_to_x25519(ed_sk: &sign::SecretKey) -> Result<[u8; 32], CryptoError> {
    let mut x25519_sk = [0u8; 32];
    let ret = unsafe {
        libsodium_sys::crypto_sign_ed25519_sk_to_curve25519(
            x25519_sk.as_mut_ptr(),
            ed_sk.as_ref().as_ptr(),
        )
    };
    if ret != 0 {
        return Err(CryptoError::KeyGenFailed);
    }
    Ok(x25519_sk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sodiumoxide::crypto::sign;

    #[test]
    fn test_ed25519_to_x25519_conversion() {
        crate::init().unwrap();
        let (pk, sk) = sign::gen_keypair();
        let x_pk = ed25519_pk_to_x25519(&pk).unwrap();
        let x_sk = ed25519_sk_to_x25519(&sk).unwrap();
        assert_eq!(x_pk.len(), 32);
        assert_eq!(x_sk.len(), 32);
        // Verify keys are non-zero
        assert!(x_pk.iter().any(|&b| b != 0));
        assert!(x_sk.iter().any(|&b| b != 0));
    }
}
