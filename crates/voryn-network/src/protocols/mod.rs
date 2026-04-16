//! Custom Voryn network protocols.
//!
//! - `/voryn/message/1.0.0` — Direct encrypted messaging
//! - `/voryn/ack/1.0.0` — Delivery confirmation
//! - `/voryn/wipe/1.0.0` — Remote wipe command
//! - `/voryn/group-sync/1.0.0` — Group message synchronization
//! - `/voryn/revocation/1.0.0` — Identity revocation broadcast

use serde::{Deserialize, Serialize};

pub const MESSAGE_PROTOCOL: &str = "/voryn/message/1.0.0";
pub const ACK_PROTOCOL: &str = "/voryn/ack/1.0.0";
pub const WIPE_PROTOCOL: &str = "/voryn/wipe/1.0.0";
pub const GROUP_SYNC_PROTOCOL: &str = "/voryn/group-sync/1.0.0";
pub const REVOCATION_PROTOCOL: &str = "/voryn/revocation/1.0.0";

/// Outbound message request carrying opaque encrypted bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VorynRequest {
    /// Encrypted message payload (Double Ratchet ciphertext).
    pub data: Vec<u8>,
}

/// Acknowledgment response from the receiving peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VorynResponse {
    pub accepted: bool,
}
