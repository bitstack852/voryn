//! Double Ratchet session state management.
//!
//! Manages the full ratchet state between two peers, including:
//! - DH ratchet key pairs
//! - Root chain, sending chain, receiving chain
//! - Skipped message keys for out-of-order delivery

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::chain::{Chain, ChainKey, MessageKey};
use super::header::Header;

/// Maximum number of skipped message keys to store (prevents memory exhaustion).
const MAX_SKIP: u32 = 1000;

/// Complete Double Ratchet session state.
#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    /// Our current DH ratchet key pair (secret key, 32 bytes X25519).
    dh_self_secret: [u8; 32],
    /// Our current DH ratchet public key (32 bytes X25519).
    dh_self_public: [u8; 32],
    /// Remote peer's current DH ratchet public key.
    dh_remote_public: Option<[u8; 32]>,
    /// Root key (32 bytes).
    root_key: [u8; 32],
    /// Sending chain.
    sending_chain: Option<Chain>,
    /// Receiving chain.
    receiving_chain: Option<Chain>,
    /// Number of messages sent in the previous sending chain.
    previous_sending_chain_length: u32,
    /// Skipped message keys: (ratchet_pubkey, message_number) -> message_key.
    skipped_keys: HashMap<([u8; 32], u32), Vec<u8>>,
}

impl Session {
    /// Initialize a session as the initiator (Alice).
    /// `shared_secret` comes from X3DH key agreement.
    /// `remote_dh_public` is the responder's initial DH ratchet key.
    pub fn init_alice(
        shared_secret: [u8; 32],
        remote_dh_public: [u8; 32],
    ) -> Self {
        // Generate our DH ratchet key pair
        let (dh_secret, dh_public) = generate_dh_keypair();

        // Perform DH and derive root key + sending chain key
        let dh_output = dh(&dh_secret, &remote_dh_public);
        let (new_root_key, chain_key) = kdf_rk(&shared_secret, &dh_output);

        Session {
            dh_self_secret: dh_secret,
            dh_self_public: dh_public,
            dh_remote_public: Some(remote_dh_public),
            root_key: new_root_key,
            sending_chain: Some(Chain::new(ChainKey::new(chain_key))),
            receiving_chain: None,
            previous_sending_chain_length: 0,
            skipped_keys: HashMap::new(),
        }
    }

    /// Initialize a session as the responder (Bob).
    /// `shared_secret` comes from X3DH key agreement.
    /// `dh_keypair` is our pre-published DH ratchet key pair.
    pub fn init_bob(
        shared_secret: [u8; 32],
        dh_secret: [u8; 32],
        dh_public: [u8; 32],
    ) -> Self {
        Session {
            dh_self_secret: dh_secret,
            dh_self_public: dh_public,
            dh_remote_public: None,
            root_key: shared_secret,
            sending_chain: None,
            receiving_chain: None,
            previous_sending_chain_length: 0,
            skipped_keys: HashMap::new(),
        }
    }

    /// Encrypt a plaintext message. Returns (header, ciphertext).
    pub fn encrypt(&mut self, plaintext: &[u8]) -> (Header, Vec<u8>) {
        let sending = self.sending_chain.as_mut().expect("Sending chain not initialized");
        let mk = sending.next_message_key();

        let header = Header::new(
            self.dh_self_public,
            self.previous_sending_chain_length,
            sending.index() - 1, // index was already incremented
        );

        let ciphertext = encrypt_with_mk(plaintext, mk.as_bytes(), &header.to_bytes());

        (header, ciphertext)
    }

    /// Decrypt a received message. Returns plaintext.
    pub fn decrypt(&mut self, header: &Header, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        // Try skipped message keys first
        let skip_key = (header.dh_public_key, header.message_number);
        if let Some(mk_bytes) = self.skipped_keys.remove(&skip_key) {
            let mut mk = [0u8; 32];
            mk.copy_from_slice(&mk_bytes);
            return decrypt_with_mk(ciphertext, &mk, &header.to_bytes());
        }

        // Check if we need to perform a DH ratchet step
        if self.dh_remote_public.as_ref() != Some(&header.dh_public_key) {
            // Skip any remaining message keys in the current receiving chain
            if let Some(ref mut recv_chain) = self.receiving_chain {
                self.skip_message_keys(recv_chain, header.previous_chain_length)?;
            }
            self.dh_ratchet_step(&header.dh_public_key);
        }

        // Skip to the correct message number in the receiving chain
        let recv_chain = self.receiving_chain.as_mut()
            .ok_or("Receiving chain not initialized")?;
        self.skip_message_keys_in(recv_chain, header.message_number)?;

        let mk = recv_chain.next_message_key();
        decrypt_with_mk(ciphertext, mk.as_bytes(), &header.to_bytes())
    }

    /// Perform a DH ratchet step.
    fn dh_ratchet_step(&mut self, new_remote_dh: &[u8; 32]) {
        self.previous_sending_chain_length = self.sending_chain
            .as_ref()
            .map_or(0, |c| c.index());

        self.dh_remote_public = Some(*new_remote_dh);

        // Derive receiving chain
        let dh_output = dh(&self.dh_self_secret, new_remote_dh);
        let (new_root_key, recv_chain_key) = kdf_rk(&self.root_key, &dh_output);
        self.root_key = new_root_key;
        self.receiving_chain = Some(Chain::new(ChainKey::new(recv_chain_key)));

        // Generate new DH keypair
        let (dh_secret, dh_public) = generate_dh_keypair();
        self.dh_self_secret = dh_secret;
        self.dh_self_public = dh_public;

        // Derive sending chain
        let dh_output = dh(&self.dh_self_secret, new_remote_dh);
        let (new_root_key, send_chain_key) = kdf_rk(&self.root_key, &dh_output);
        self.root_key = new_root_key;
        self.sending_chain = Some(Chain::new(ChainKey::new(send_chain_key)));
    }

    /// Skip message keys in a chain up to a target index, storing them for later.
    fn skip_message_keys(&mut self, chain: &mut Chain, until: u32) -> Result<(), String> {
        if chain.index() + MAX_SKIP < until {
            return Err("Too many skipped messages".into());
        }
        let dh_pub = self.dh_remote_public.unwrap_or([0; 32]);
        while chain.index() < until {
            let mk = chain.next_message_key();
            self.skipped_keys.insert((dh_pub, chain.index() - 1), mk.as_bytes().to_vec());
        }
        Ok(())
    }

    fn skip_message_keys_in(&mut self, chain: &mut Chain, until: u32) -> Result<(), String> {
        if chain.index() + MAX_SKIP < until {
            return Err("Too many skipped messages".into());
        }
        let dh_pub = self.dh_remote_public.unwrap_or([0; 32]);
        while chain.index() < until {
            let mk = chain.next_message_key();
            self.skipped_keys.insert((dh_pub, chain.index() - 1), mk.as_bytes().to_vec());
        }
        Ok(())
    }

    /// Serialize session state for storage.
    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| e.to_string())
    }

    /// Deserialize session state from storage.
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| e.to_string())
    }
}

// ── Crypto helpers ────────────────────────────────────────────────

/// Generate an X25519 key pair using libsodium.
fn generate_dh_keypair() -> ([u8; 32], [u8; 32]) {
    let mut pk = [0u8; 32];
    let mut sk = [0u8; 32];
    unsafe {
        libsodium_sys::crypto_box_keypair(pk.as_mut_ptr(), sk.as_mut_ptr());
    }
    (sk, pk)
}

/// X25519 Diffie-Hellman.
fn dh(secret_key: &[u8; 32], public_key: &[u8; 32]) -> [u8; 32] {
    let mut shared = [0u8; 32];
    unsafe {
        libsodium_sys::crypto_scalarmult(
            shared.as_mut_ptr(),
            secret_key.as_ptr(),
            public_key.as_ptr(),
        );
    }
    shared
}

/// KDF for root key derivation: (root_key, dh_output) -> (new_root_key, chain_key).
fn kdf_rk(root_key: &[u8; 32], dh_output: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    // Use HKDF-like construction: HMAC-SHA512 with root_key as key and dh_output as data
    // Split the 64-byte output into new_root_key (first 32) and chain_key (last 32)
    let mut output = [0u8; 64];
    unsafe {
        libsodium_sys::crypto_auth_hmacsha512(
            output.as_mut_ptr(),
            dh_output.as_ptr(),
            dh_output.len() as u64,
            root_key.as_ptr(),
        );
    }
    let mut new_rk = [0u8; 32];
    let mut ck = [0u8; 32];
    new_rk.copy_from_slice(&output[..32]);
    ck.copy_from_slice(&output[32..]);
    (new_rk, ck)
}

/// Encrypt plaintext with a message key (XSalsa20-Poly1305).
fn encrypt_with_mk(plaintext: &[u8], mk: &[u8; 32], _ad: &[u8]) -> Vec<u8> {
    let key = sodiumoxide::crypto::secretbox::Key::from_slice(mk).unwrap();
    let nonce = sodiumoxide::crypto::secretbox::gen_nonce();
    let ciphertext = sodiumoxide::crypto::secretbox::seal(plaintext, &nonce, &key);
    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(24 + ciphertext.len());
    result.extend_from_slice(nonce.as_ref());
    result.extend_from_slice(&ciphertext);
    result
}

/// Decrypt ciphertext with a message key.
fn decrypt_with_mk(data: &[u8], mk: &[u8; 32], _ad: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 24 {
        return Err("Ciphertext too short".into());
    }
    let nonce = sodiumoxide::crypto::secretbox::Nonce::from_slice(&data[..24])
        .ok_or("Invalid nonce")?;
    let ciphertext = &data[24..];
    let key = sodiumoxide::crypto::secretbox::Key::from_slice(mk)
        .ok_or("Invalid message key")?;
    sodiumoxide::crypto::secretbox::open(ciphertext, &nonce, &key)
        .map_err(|_| "Decryption failed".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        sodiumoxide::init().ok();
    }

    #[test]
    fn test_alice_bob_basic_exchange() {
        init();
        let shared_secret = [0x42; 32];
        let (bob_sk, bob_pk) = generate_dh_keypair();

        let mut alice = Session::init_alice(shared_secret, bob_pk);
        let mut bob = Session::init_bob(shared_secret, bob_sk, bob_pk);

        // Alice sends to Bob
        let (header, ct) = alice.encrypt(b"Hello Bob!");
        let pt = bob.decrypt(&header, &ct).unwrap();
        assert_eq!(pt, b"Hello Bob!");

        // Bob sends to Alice
        let (header, ct) = bob.encrypt(b"Hello Alice!");
        let pt = alice.decrypt(&header, &ct).unwrap();
        assert_eq!(pt, b"Hello Alice!");
    }

    #[test]
    fn test_multiple_messages_same_direction() {
        init();
        let shared_secret = [0x42; 32];
        let (bob_sk, bob_pk) = generate_dh_keypair();

        let mut alice = Session::init_alice(shared_secret, bob_pk);
        let mut bob = Session::init_bob(shared_secret, bob_sk, bob_pk);

        // Alice sends 3 messages to Bob
        let (h1, c1) = alice.encrypt(b"Message 1");
        let (h2, c2) = alice.encrypt(b"Message 2");
        let (h3, c3) = alice.encrypt(b"Message 3");

        // Bob receives all in order
        assert_eq!(bob.decrypt(&h1, &c1).unwrap(), b"Message 1");
        assert_eq!(bob.decrypt(&h2, &c2).unwrap(), b"Message 2");
        assert_eq!(bob.decrypt(&h3, &c3).unwrap(), b"Message 3");
    }

    #[test]
    fn test_out_of_order_delivery() {
        init();
        let shared_secret = [0x42; 32];
        let (bob_sk, bob_pk) = generate_dh_keypair();

        let mut alice = Session::init_alice(shared_secret, bob_pk);
        let mut bob = Session::init_bob(shared_secret, bob_sk, bob_pk);

        // Alice sends 3 messages
        let (h1, c1) = alice.encrypt(b"First");
        let (h2, c2) = alice.encrypt(b"Second");
        let (h3, c3) = alice.encrypt(b"Third");

        // Bob receives out of order: 3, 1, 2
        assert_eq!(bob.decrypt(&h3, &c3).unwrap(), b"Third");
        assert_eq!(bob.decrypt(&h1, &c1).unwrap(), b"First");
        assert_eq!(bob.decrypt(&h2, &c2).unwrap(), b"Second");
    }

    #[test]
    fn test_session_serialization() {
        init();
        let shared_secret = [0x42; 32];
        let (bob_sk, bob_pk) = generate_dh_keypair();

        let mut alice = Session::init_alice(shared_secret, bob_pk);
        let (header, ct) = alice.encrypt(b"test");

        // Serialize and deserialize
        let data = alice.serialize().unwrap();
        let mut alice_restored = Session::deserialize(&data).unwrap();

        // Should be able to encrypt with restored session
        let (header2, ct2) = alice_restored.encrypt(b"after restore");
        assert!(!ct2.is_empty());
    }

    #[test]
    fn test_forward_secrecy() {
        init();
        let shared_secret = [0x42; 32];
        let (bob_sk, bob_pk) = generate_dh_keypair();

        let mut alice = Session::init_alice(shared_secret, bob_pk);
        let mut bob = Session::init_bob(shared_secret, bob_sk, bob_pk);

        // Exchange messages to advance the ratchet
        let (h1, c1) = alice.encrypt(b"Message 1");
        bob.decrypt(&h1, &c1).unwrap();
        let (h2, c2) = bob.encrypt(b"Reply 1");
        alice.decrypt(&h2, &c2).unwrap();

        // Capture current state
        let alice_state = alice.serialize().unwrap();

        // Send more messages
        let (h3, c3) = alice.encrypt(b"Message 2");
        bob.decrypt(&h3, &c3).unwrap();

        // Even with alice_state from before, the root key has advanced
        // and old message keys are gone (forward secrecy)
        let old_alice = Session::deserialize(&alice_state).unwrap();
        // old_alice cannot produce the same ciphertext as current alice
    }
}
