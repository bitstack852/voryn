//! Pre-key bundles for X3DH key agreement.
//!
//! Each device publishes a pre-key bundle containing:
//! - Identity key (long-term, Ed25519 → X25519)
//! - Signed pre-key (medium-term, rotated periodically)
//! - One-time pre-keys (consumed on first contact)

use serde::{Deserialize, Serialize};

/// A pre-key bundle published by a device for initial key agreement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreKeyBundle {
    /// Long-term identity public key (X25519, 32 bytes).
    pub identity_key: [u8; 32],
    /// Signed pre-key public key (X25519, 32 bytes).
    pub signed_prekey: [u8; 32],
    /// Signature over signed_prekey by the identity key.
    pub signed_prekey_signature: Vec<u8>,
    /// One-time pre-key public key (X25519, 32 bytes). Optional.
    pub one_time_prekey: Option<[u8; 32]>,
    /// One-time pre-key ID (for tracking consumption).
    pub one_time_prekey_id: Option<u32>,
}

/// Result of an X3DH key agreement.
#[derive(Debug)]
pub struct X3DHResult {
    /// Shared secret (32 bytes) for initializing the Double Ratchet.
    pub shared_secret: [u8; 32],
    /// The ephemeral public key sent to the responder.
    pub ephemeral_public_key: [u8; 32],
    /// Whether a one-time pre-key was used.
    pub used_one_time_prekey: bool,
}

/// Perform X3DH as the initiator (Alice).
///
/// Computes: shared_secret = KDF(DH1 || DH2 || DH3 [|| DH4])
/// where:
///   DH1 = DH(IK_A, SPK_B) — our identity key with their signed prekey
///   DH2 = DH(EK_A, IK_B)  — our ephemeral key with their identity key
///   DH3 = DH(EK_A, SPK_B) — our ephemeral key with their signed prekey
///   DH4 = DH(EK_A, OPK_B) — our ephemeral key with their one-time prekey (optional)
pub fn initiate_x3dh(
    our_identity_sk: &[u8; 32],
    their_bundle: &PreKeyBundle,
) -> Result<X3DHResult, String> {
    // Generate ephemeral keypair
    let mut ek_pk = [0u8; 32];
    let mut ek_sk = [0u8; 32];
    unsafe {
        libsodium_sys::crypto_box_keypair(ek_pk.as_mut_ptr(), ek_sk.as_mut_ptr());
    }

    // DH1: DH(our_identity, their_signed_prekey)
    let dh1 = scalarmult(our_identity_sk, &their_bundle.signed_prekey);

    // DH2: DH(ephemeral, their_identity)
    let dh2 = scalarmult(&ek_sk, &their_bundle.identity_key);

    // DH3: DH(ephemeral, their_signed_prekey)
    let dh3 = scalarmult(&ek_sk, &their_bundle.signed_prekey);

    // DH4 (optional): DH(ephemeral, their_one_time_prekey)
    let mut dh_concat = Vec::with_capacity(128);
    dh_concat.extend_from_slice(&dh1);
    dh_concat.extend_from_slice(&dh2);
    dh_concat.extend_from_slice(&dh3);

    let used_otp = if let Some(otpk) = &their_bundle.one_time_prekey {
        let dh4 = scalarmult(&ek_sk, otpk);
        dh_concat.extend_from_slice(&dh4);
        true
    } else {
        false
    };

    // Derive shared secret via HKDF-SHA256
    let shared_secret = hkdf_sha256(&dh_concat, b"VorynX3DH");

    // Zero ephemeral secret key
    let mut ek_sk_copy = ek_sk;
    zeroize::Zeroize::zeroize(&mut ek_sk_copy);

    Ok(X3DHResult {
        shared_secret,
        ephemeral_public_key: ek_pk,
        used_one_time_prekey: used_otp,
    })
}

/// X25519 scalar multiplication.
fn scalarmult(secret_key: &[u8; 32], public_key: &[u8; 32]) -> [u8; 32] {
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

/// Simplified HKDF-SHA256 for deriving the shared secret.
fn hkdf_sha256(input: &[u8], info: &[u8]) -> [u8; 32] {
    // Extract phase: HMAC-SHA256(salt=zeros, input)
    let salt = [0u8; 32];
    let mut prk = [0u8; 32];
    unsafe {
        libsodium_sys::crypto_auth_hmacsha256(
            prk.as_mut_ptr(),
            input.as_ptr(),
            input.len() as u64,
            salt.as_ptr(),
        );
    }

    // Expand phase: HMAC-SHA256(prk, info || 0x01)
    let mut expand_input = Vec::with_capacity(info.len() + 1);
    expand_input.extend_from_slice(info);
    expand_input.push(0x01);

    let mut okm = [0u8; 32];
    unsafe {
        libsodium_sys::crypto_auth_hmacsha256(
            okm.as_mut_ptr(),
            expand_input.as_ptr(),
            expand_input.len() as u64,
            prk.as_ptr(),
        );
    }
    okm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prekey_bundle_serialization() {
        let bundle = PreKeyBundle {
            identity_key: [1u8; 32],
            signed_prekey: [2u8; 32],
            signed_prekey_signature: vec![3u8; 64],
            one_time_prekey: Some([4u8; 32]),
            one_time_prekey_id: Some(42),
        };

        let bytes = bincode::serialize(&bundle).unwrap();
        let recovered: PreKeyBundle = bincode::deserialize(&bytes).unwrap();
        assert_eq!(recovered.identity_key, [1u8; 32]);
        assert_eq!(recovered.one_time_prekey_id, Some(42));
    }
}
