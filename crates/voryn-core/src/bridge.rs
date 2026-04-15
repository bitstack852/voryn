//! UniFFI bridge — functions exposed to React Native.
//!
//! These functions are defined in voryn-core.udl and called from TypeScript
//! via the generated UniFFI bindings.

use voryn_crypto::identity;
use voryn_crypto::signing;
use voryn_crypto::encryption;
use voryn_crypto::dh;

/// Identity data returned to the JS layer.
pub struct VorynIdentity {
    pub public_key: Vec<u8>,
    pub public_key_hex: String,
    pub secret_key_seed: Vec<u8>,
}

/// Encrypted data bundle.
pub struct EncryptedData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Bridge test function.
pub fn hello_from_rust() -> String {
    "Voryn Core v0.1.0 — Private. Encrypted. Unreachable.".to_string()
}

/// Generate a new Ed25519 identity.
pub fn generate_identity() -> VorynIdentity {
    voryn_crypto::init().ok();
    let full = identity::generate_full_identity().expect("Identity generation failed");

    VorynIdentity {
        public_key: full.identity.public_key.clone(),
        public_key_hex: full.identity.public_key_hex.clone(),
        secret_key_seed: full.keypair.secret_key.as_bytes().to_vec(),
    }
}

/// Get public key bytes from an identity.
pub fn get_public_key(identity: VorynIdentity) -> Vec<u8> {
    identity.public_key
}

/// Get public key hex string from an identity.
pub fn get_public_key_hex(identity: VorynIdentity) -> String {
    identity.public_key_hex
}

/// Sign a message with a secret key.
pub fn sign_message(message: Vec<u8>, secret_key: Vec<u8>) -> Vec<u8> {
    voryn_crypto::init().ok();
    let sk = sodiumoxide::crypto::sign::SecretKey::from_slice(&secret_key)
        .expect("Invalid secret key");
    signing::sign_message(&message, &sk)
}

/// Verify a signature.
pub fn verify_signature(message: Vec<u8>, signature: Vec<u8>, public_key: Vec<u8>) -> bool {
    voryn_crypto::init().ok();
    let pk = sodiumoxide::crypto::sign::PublicKey::from_slice(&public_key);
    match pk {
        Some(pk) => signing::verify_signature(&message, &signature, &pk).is_ok(),
        None => false,
    }
}

/// Encrypt a message for a peer using DH key exchange + XSalsa20-Poly1305.
pub fn encrypt_for_peer(
    plaintext: Vec<u8>,
    our_secret_key: Vec<u8>,
    their_public_key: Vec<u8>,
) -> EncryptedData {
    voryn_crypto::init().ok();

    let our_sk = sodiumoxide::crypto::sign::SecretKey::from_slice(&our_secret_key)
        .expect("Invalid secret key");
    let their_pk = sodiumoxide::crypto::sign::PublicKey::from_slice(&their_public_key)
        .expect("Invalid public key");

    // Compute shared secret via DH
    let shared = dh::compute_shared_secret(&our_sk, &their_pk)
        .expect("DH computation failed");

    // Derive symmetric key
    let sym_key_bytes = voryn_crypto::kdf::derive_key(shared.as_bytes(), b"vorynmsg", 1)
        .expect("KDF failed");
    let sym_key = sodiumoxide::crypto::secretbox::Key::from_slice(&sym_key_bytes)
        .expect("Invalid symmetric key");

    // Encrypt
    let (nonce, ciphertext) = encryption::encrypt(&plaintext, &sym_key);

    // Sign the ciphertext
    let mut signed_data = Vec::with_capacity(nonce.len() + ciphertext.len());
    signed_data.extend_from_slice(&nonce);
    signed_data.extend_from_slice(&ciphertext);
    let signature = signing::sign_message(&signed_data, &our_sk);

    EncryptedData {
        nonce,
        ciphertext,
        signature,
    }
}

/// Decrypt a message from a peer.
pub fn decrypt_from_peer(
    data: EncryptedData,
    our_secret_key: Vec<u8>,
    their_public_key: Vec<u8>,
) -> Vec<u8> {
    voryn_crypto::init().ok();

    let our_sk = sodiumoxide::crypto::sign::SecretKey::from_slice(&our_secret_key)
        .expect("Invalid secret key");
    let their_pk = sodiumoxide::crypto::sign::PublicKey::from_slice(&their_public_key)
        .expect("Invalid public key");

    // Verify signature
    let mut signed_data = Vec::with_capacity(data.nonce.len() + data.ciphertext.len());
    signed_data.extend_from_slice(&data.nonce);
    signed_data.extend_from_slice(&data.ciphertext);
    signing::verify_signature(&signed_data, &data.signature, &their_pk)
        .expect("Signature verification failed");

    // Compute shared secret
    let shared = dh::compute_shared_secret(&our_sk, &their_pk)
        .expect("DH computation failed");

    // Derive symmetric key
    let sym_key_bytes = voryn_crypto::kdf::derive_key(shared.as_bytes(), b"vorynmsg", 1)
        .expect("KDF failed");
    let sym_key = sodiumoxide::crypto::secretbox::Key::from_slice(&sym_key_bytes)
        .expect("Invalid symmetric key");

    // Decrypt
    encryption::decrypt(&data.ciphertext, &data.nonce, &sym_key)
        .expect("Decryption failed")
}

/// Derive a key from a passcode using Argon2id.
pub fn derive_passcode_key(
    passcode: String,
    salt: Vec<u8>,
    memory_kib: u32,
    iterations: u32,
) -> Vec<u8> {
    voryn_crypto::init().ok();
    let mut salt_arr = [0u8; 16];
    salt_arr.copy_from_slice(&salt[..16.min(salt.len())]);

    let params = voryn_crypto::passcode::PasscodeParams {
        memory_kib,
        iterations,
        salt: salt_arr,
    };

    let key = voryn_crypto::passcode::derive_key_from_passcode(&passcode, &params)
        .expect("Argon2id derivation failed");
    key.to_vec()
}

/// Compute a safety number from two public keys.
pub fn compute_safety_number(our_public_key: Vec<u8>, their_public_key: Vec<u8>) -> String {
    let mut our_pk = [0u8; 32];
    let mut their_pk = [0u8; 32];
    our_pk.copy_from_slice(&our_public_key[..32]);
    their_pk.copy_from_slice(&their_public_key[..32]);

    let sn = voryn_protocol::safety_number::compute_safety_number(&our_pk, &their_pk);
    sn.display
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let identity = generate_identity();
        assert_eq!(identity.public_key.len(), 32);
        assert_eq!(identity.public_key_hex.len(), 64);
        assert_eq!(identity.secret_key_seed.len(), 64);
    }

    #[test]
    fn test_sign_and_verify() {
        let identity = generate_identity();
        let message = b"test message".to_vec();
        let sig = sign_message(message.clone(), identity.secret_key_seed.clone());
        assert!(verify_signature(message, sig, identity.public_key));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let alice = generate_identity();
        let bob = generate_identity();

        let plaintext = b"Hello Bob from Alice!".to_vec();
        let encrypted = encrypt_for_peer(
            plaintext.clone(),
            alice.secret_key_seed.clone(),
            bob.public_key.clone(),
        );

        let decrypted = decrypt_from_peer(
            encrypted,
            bob.secret_key_seed.clone(),
            alice.public_key.clone(),
        );

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_safety_number_symmetric() {
        let alice = generate_identity();
        let bob = generate_identity();

        let sn1 = compute_safety_number(alice.public_key.clone(), bob.public_key.clone());
        let sn2 = compute_safety_number(bob.public_key, alice.public_key);
        assert_eq!(sn1, sn2);
    }
}
