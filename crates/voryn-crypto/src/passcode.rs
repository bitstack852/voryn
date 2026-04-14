//! Passcode key derivation — Argon2id for brute-force resistant key wrapping.
//!
//! The user's passcode is used to derive a key that wraps the hardware-bound
//! database encryption key. This provides an additional authentication layer
//! beyond device possession.

use zeroize::Zeroize;
use crate::CryptoError;

/// Parameters for Argon2id key derivation.
/// Tuned for mobile devices (~500ms per derivation on modern hardware).
#[derive(Debug, Clone)]
pub struct PasscodeParams {
    /// Memory cost in KiB (default: 65536 = 64MB).
    pub memory_kib: u32,
    /// Number of iterations (default: 3).
    pub iterations: u32,
    /// Salt (16 bytes, randomly generated on passcode creation).
    pub salt: [u8; 16],
}

impl Default for PasscodeParams {
    fn default() -> Self {
        let mut salt = [0u8; 16];
        sodiumoxide::randombytes::randombytes_into(&mut salt);
        Self {
            memory_kib: 65536,
            iterations: 3,
            salt,
        }
    }
}

/// Derive a 32-byte key from a passcode using Argon2id.
pub fn derive_key_from_passcode(
    passcode: &str,
    params: &PasscodeParams,
) -> Result<[u8; 32], CryptoError> {
    crate::init()?;

    let mut key = [0u8; 32];

    let ret = unsafe {
        libsodium_sys::crypto_pwhash(
            key.as_mut_ptr(),
            key.len() as u64,
            passcode.as_ptr() as *const i8,
            passcode.len() as u64,
            params.salt.as_ptr(),
            params.iterations as u64,
            params.memory_kib as usize * 1024,
            libsodium_sys::crypto_pwhash_ALG_ARGON2ID13 as i32,
        )
    };

    if ret != 0 {
        return Err(CryptoError::KdfError("Argon2id derivation failed".into()));
    }

    Ok(key)
}

/// Wrap a key with a passcode-derived key (encrypt the key).
pub fn wrap_key(
    key_to_wrap: &[u8; 32],
    passcode: &str,
    params: &PasscodeParams,
) -> Result<WrappedKey, CryptoError> {
    let wrapping_key = derive_key_from_passcode(passcode, params)?;

    let sym_key = sodiumoxide::crypto::secretbox::Key::from_slice(&wrapping_key)
        .ok_or(CryptoError::KdfError("Failed to create wrapping key".into()))?;
    let nonce = sodiumoxide::crypto::secretbox::gen_nonce();
    let ciphertext = sodiumoxide::crypto::secretbox::seal(key_to_wrap, &nonce, &sym_key);

    // Zero wrapping key
    let mut wk = wrapping_key;
    wk.zeroize();

    Ok(WrappedKey {
        ciphertext,
        nonce: {
            let mut arr = [0u8; 24];
            arr.copy_from_slice(nonce.as_ref());
            arr
        },
        params: params.clone(),
    })
}

/// Unwrap a key using a passcode.
pub fn unwrap_key(
    wrapped: &WrappedKey,
    passcode: &str,
) -> Result<[u8; 32], CryptoError> {
    let wrapping_key = derive_key_from_passcode(passcode, &wrapped.params)?;

    let sym_key = sodiumoxide::crypto::secretbox::Key::from_slice(&wrapping_key)
        .ok_or(CryptoError::KdfError("Failed to create wrapping key".into()))?;
    let nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&wrapped.nonce)
        .ok_or(CryptoError::DecryptionFailed)?;

    let plaintext = sodiumoxide::crypto::secretbox::open(&wrapped.ciphertext, &nonce, &sym_key)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    // Zero wrapping key
    let mut wk = wrapping_key;
    wk.zeroize();

    if plaintext.len() != 32 {
        return Err(CryptoError::DecryptionFailed);
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&plaintext);
    Ok(key)
}

/// A key wrapped (encrypted) with a passcode-derived key.
#[derive(Debug, Clone)]
pub struct WrappedKey {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 24],
    pub params: PasscodeParams,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        crate::init().unwrap();
        let params = PasscodeParams {
            memory_kib: 1024, // Low for testing
            iterations: 1,
            salt: [0xAA; 16],
        };

        let key1 = derive_key_from_passcode("mypasscode", &params).unwrap();
        let key2 = derive_key_from_passcode("mypasscode", &params).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_passcodes_different_keys() {
        crate::init().unwrap();
        let params = PasscodeParams {
            memory_kib: 1024,
            iterations: 1,
            salt: [0xBB; 16],
        };

        let key1 = derive_key_from_passcode("passcode1", &params).unwrap();
        let key2 = derive_key_from_passcode("passcode2", &params).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_wrap_unwrap_roundtrip() {
        crate::init().unwrap();
        let params = PasscodeParams {
            memory_kib: 1024,
            iterations: 1,
            salt: [0xCC; 16],
        };

        let original_key = [0x42u8; 32];
        let wrapped = wrap_key(&original_key, "test_passcode", &params).unwrap();
        let unwrapped = unwrap_key(&wrapped, "test_passcode").unwrap();
        assert_eq!(original_key, unwrapped);
    }

    #[test]
    fn test_wrong_passcode_fails() {
        crate::init().unwrap();
        let params = PasscodeParams {
            memory_kib: 1024,
            iterations: 1,
            salt: [0xDD; 16],
        };

        let wrapped = wrap_key(&[0x42; 32], "correct", &params).unwrap();
        let result = unwrap_key(&wrapped, "wrong");
        assert!(result.is_err());
    }
}
