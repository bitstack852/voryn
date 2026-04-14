//! Messaging orchestration — ties together crypto, protocol, storage, and network
//! to provide a high-level API for sending and receiving encrypted messages.

use voryn_crypto::dh::SharedSecret;
use voryn_crypto::encryption;
use voryn_crypto::signing;
use voryn_protocol::exchange;
use voryn_protocol::message::{EncryptedMessage, DeliveryAck};

/// Encrypt and sign a plaintext message for a recipient.
///
/// Phase 1: Uses DH shared secret → HKDF → XSalsa20-Poly1305.
/// Phase 2 will replace this with Double Ratchet encryption.
pub fn prepare_message(
    plaintext: &[u8],
    shared_secret: &SharedSecret,
    sender_sk: &sodiumoxide::crypto::sign::SecretKey,
    sender_pk: &[u8],
) -> Result<EncryptedMessage, String> {
    // Derive symmetric key from shared secret
    let sym_key_bytes = voryn_crypto::kdf::derive_key(
        shared_secret.as_bytes(),
        b"vorynmsg",
        1,
    ).map_err(|e| e.to_string())?;

    let sym_key = sodiumoxide::crypto::secretbox::Key::from_slice(&sym_key_bytes)
        .ok_or("Failed to create symmetric key")?;

    // Encrypt
    let (nonce, ciphertext) = encryption::encrypt(plaintext, &sym_key);

    // Sign (nonce || ciphertext)
    let mut signed_data = Vec::with_capacity(nonce.len() + ciphertext.len());
    signed_data.extend_from_slice(&nonce);
    signed_data.extend_from_slice(&ciphertext);
    let signature = signing::sign_message(&signed_data, sender_sk);

    Ok(EncryptedMessage {
        sender_pubkey: sender_pk.to_vec(),
        nonce,
        ciphertext,
        signature,
        timestamp: exchange::current_timestamp_ms(),
        message_id: exchange::generate_message_id(),
    })
}

/// Verify and decrypt a received message.
pub fn receive_message(
    msg: &EncryptedMessage,
    shared_secret: &SharedSecret,
    sender_pk: &sodiumoxide::crypto::sign::PublicKey,
) -> Result<Vec<u8>, String> {
    // Verify signature
    let signed_data = msg.signed_payload();
    signing::verify_signature(&signed_data, &msg.signature, sender_pk)
        .map_err(|_| "Signature verification failed".to_string())?;

    // Derive the same symmetric key
    let sym_key_bytes = voryn_crypto::kdf::derive_key(
        shared_secret.as_bytes(),
        b"vorynmsg",
        1,
    ).map_err(|e| e.to_string())?;

    let sym_key = sodiumoxide::crypto::secretbox::Key::from_slice(&sym_key_bytes)
        .ok_or("Failed to create symmetric key")?;

    // Decrypt
    encryption::decrypt(&msg.ciphertext, &msg.nonce, &sym_key)
        .map_err(|_| "Decryption failed".to_string())
}

/// Create a delivery acknowledgment for a received message.
pub fn create_ack(
    message_id: &str,
    our_pk: &[u8],
    our_sk: &sodiumoxide::crypto::sign::SecretKey,
) -> DeliveryAck {
    let received_at = exchange::current_timestamp_ms();

    // Sign (message_id_bytes || received_at as big-endian)
    let mut ack_data = Vec::new();
    ack_data.extend_from_slice(message_id.as_bytes());
    ack_data.extend_from_slice(&received_at.to_be_bytes());
    let signature = signing::sign_message(&ack_data, our_sk);

    DeliveryAck {
        message_id: message_id.to_string(),
        ack_pubkey: our_pk.to_vec(),
        received_at,
        signature,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voryn_crypto::dh::compute_shared_secret;
    use sodiumoxide::crypto::sign;

    #[test]
    fn test_encrypt_decrypt_message_roundtrip() {
        voryn_crypto::init().unwrap();
        let (pk_a, sk_a) = sign::gen_keypair();
        let (pk_b, sk_b) = sign::gen_keypair();

        // Alice sends to Bob
        let shared_ab = compute_shared_secret(&sk_a, &pk_b).unwrap();
        let plaintext = b"Hello Bob, this is a secret message!";
        let encrypted = prepare_message(
            plaintext,
            &shared_ab,
            &sk_a,
            pk_a.as_ref(),
        ).unwrap();

        // Bob receives from Alice
        let shared_ba = compute_shared_secret(&sk_b, &pk_a).unwrap();
        let decrypted = receive_message(&encrypted, &shared_ba, &pk_a).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_tampered_message_rejected() {
        voryn_crypto::init().unwrap();
        let (pk_a, sk_a) = sign::gen_keypair();
        let (pk_b, sk_b) = sign::gen_keypair();

        let shared_ab = compute_shared_secret(&sk_a, &pk_b).unwrap();
        let mut encrypted = prepare_message(
            b"original",
            &shared_ab,
            &sk_a,
            pk_a.as_ref(),
        ).unwrap();

        // Tamper with ciphertext
        if let Some(byte) = encrypted.ciphertext.last_mut() {
            *byte ^= 0xFF;
        }

        let shared_ba = compute_shared_secret(&sk_b, &pk_a).unwrap();
        let result = receive_message(&encrypted, &shared_ba, &pk_a);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_sender_key_rejected() {
        voryn_crypto::init().unwrap();
        let (pk_a, sk_a) = sign::gen_keypair();
        let (pk_b, sk_b) = sign::gen_keypair();
        let (pk_c, _sk_c) = sign::gen_keypair(); // Impersonator

        let shared_ab = compute_shared_secret(&sk_a, &pk_b).unwrap();
        let encrypted = prepare_message(
            b"test",
            &shared_ab,
            &sk_a,
            pk_a.as_ref(),
        ).unwrap();

        // Bob tries to verify with wrong sender key (pk_c instead of pk_a)
        let shared_ba = compute_shared_secret(&sk_b, &pk_a).unwrap();
        let result = receive_message(&encrypted, &shared_ba, &pk_c);
        assert!(result.is_err());
    }

    #[test]
    fn test_ack_creation() {
        voryn_crypto::init().unwrap();
        let (pk, sk) = sign::gen_keypair();
        let ack = create_ack("msg-001", pk.as_ref(), &sk);

        assert_eq!(ack.message_id, "msg-001");
        assert_eq!(ack.ack_pubkey, pk.as_ref());
        assert!(!ack.signature.is_empty());
    }
}
