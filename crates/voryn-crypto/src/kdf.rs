//! Key derivation — HKDF and Argon2id for deriving encryption keys.

use crate::CryptoError;

/// Derive a symmetric key from a shared secret using HKDF (HMAC-SHA-512-256).
/// Uses libsodium's crypto_kdf functions.
pub fn derive_key(
    master_key: &[u8],
    context: &[u8; 8],
    subkey_id: u64,
) -> Result<[u8; 32], CryptoError> {
    if master_key.len() < 32 {
        return Err(CryptoError::KdfError("Master key too short".into()));
    }

    let mut subkey = [0u8; 32];
    let ret = unsafe {
        libsodium_sys::crypto_kdf_derive_from_key(
            subkey.as_mut_ptr(),
            subkey.len(),
            subkey_id,
            context.as_ptr() as *const i8,
            master_key.as_ptr(),
        )
    };

    if ret != 0 {
        return Err(CryptoError::KdfError("Key derivation failed".into()));
    }

    Ok(subkey)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        crate::init().unwrap();
        let master = [0xABu8; 32];
        let ctx = b"vorynkdf";
        let key1 = derive_key(&master, ctx, 1).unwrap();
        let key2 = derive_key(&master, ctx, 2).unwrap();
        // Different subkey IDs produce different keys
        assert_ne!(key1, key2);
        // Same inputs produce same output (deterministic)
        let key1_again = derive_key(&master, ctx, 1).unwrap();
        assert_eq!(key1, key1_again);
    }

    #[test]
    fn test_short_master_key_fails() {
        crate::init().unwrap();
        let short = [0u8; 16];
        assert!(derive_key(&short, b"vorynkdf", 1).is_err());
    }
}
