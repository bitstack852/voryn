//! Symmetric encryption — XSalsa20-Poly1305 authenticated encryption.

use sodiumoxide::crypto::secretbox;
use crate::CryptoError;

/// Encrypt plaintext with a symmetric key using XSalsa20-Poly1305.
/// Returns (nonce, ciphertext).
pub fn encrypt(plaintext: &[u8], key: &secretbox::Key) -> (Vec<u8>, Vec<u8>) {
    let nonce = secretbox::gen_nonce();
    let ciphertext = secretbox::seal(plaintext, &nonce, key);
    (nonce.as_ref().to_vec(), ciphertext)
}

/// Decrypt ciphertext with a symmetric key using XSalsa20-Poly1305.
pub fn decrypt(
    ciphertext: &[u8],
    nonce_bytes: &[u8],
    key: &secretbox::Key,
) -> Result<Vec<u8>, CryptoError> {
    let nonce = secretbox::Nonce::from_slice(nonce_bytes)
        .ok_or(CryptoError::DecryptionFailed)?;
    secretbox::open(ciphertext, &nonce, key)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Generate a random symmetric encryption key.
pub fn generate_key() -> secretbox::Key {
    secretbox::gen_key()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        crate::init().unwrap();
        let key = generate_key();
        let plaintext = b"Private. Encrypted. Unreachable.";
        let (nonce, ciphertext) = encrypt(plaintext, &key);
        let decrypted = decrypt(&ciphertext, &nonce, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        crate::init().unwrap();
        let key1 = generate_key();
        let key2 = generate_key();
        let (nonce, ciphertext) = encrypt(b"test", &key1);
        assert!(decrypt(&ciphertext, &nonce, &key2).is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        crate::init().unwrap();
        let key = generate_key();
        let (nonce, mut ciphertext) = encrypt(b"test", &key);
        if let Some(byte) = ciphertext.last_mut() {
            *byte ^= 0xFF;
        }
        assert!(decrypt(&ciphertext, &nonce, &key).is_err());
    }
}
