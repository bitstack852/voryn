//! Ed25519 digital signatures — sign and verify messages.

use sodiumoxide::crypto::sign;
use crate::CryptoError;

/// Sign a message with an Ed25519 secret key.
/// Returns the detached signature (64 bytes).
pub fn sign_message(message: &[u8], secret_key: &sign::SecretKey) -> Vec<u8> {
    let signature = sign::sign_detached(message, secret_key);
    signature.as_ref().to_vec()
}

/// Verify a detached Ed25519 signature against a message and public key.
pub fn verify_signature(
    message: &[u8],
    signature: &[u8],
    public_key: &sign::PublicKey,
) -> Result<(), CryptoError> {
    let sig = sign::Signature::from_bytes(signature)
        .map_err(|_| CryptoError::VerificationFailed)?;
    if sign::verify_detached(&sig, message, public_key) {
        Ok(())
    } else {
        Err(CryptoError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sodiumoxide::crypto::sign;

    #[test]
    fn test_sign_and_verify() {
        crate::init().unwrap();
        let (pk, sk) = sign::gen_keypair();
        let message = b"Private. Encrypted. Unreachable.";
        let sig = sign_message(message, &sk);
        assert!(verify_signature(message, &sig, &pk).is_ok());
    }

    #[test]
    fn test_tampered_message_fails() {
        crate::init().unwrap();
        let (pk, sk) = sign::gen_keypair();
        let message = b"original message";
        let sig = sign_message(message, &sk);
        assert!(verify_signature(b"tampered message", &sig, &pk).is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        crate::init().unwrap();
        let (_pk1, sk1) = sign::gen_keypair();
        let (pk2, _sk2) = sign::gen_keypair();
        let message = b"test message";
        let sig = sign_message(message, &sk1);
        assert!(verify_signature(message, &sig, &pk2).is_err());
    }
}
