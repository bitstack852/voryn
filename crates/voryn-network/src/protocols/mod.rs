//! Custom Voryn network protocols.
//!
//! - `/voryn/message/1.0.0` — Direct encrypted messaging
//! - `/voryn/ack/1.0.0` — Delivery confirmation
//! - `/voryn/wipe/1.0.0` — Remote wipe command
//! - `/voryn/group-sync/1.0.0` — Group message synchronization
//! - `/voryn/revocation/1.0.0` — Identity revocation broadcast

/// Protocol ID for direct messaging.
pub const MESSAGE_PROTOCOL: &str = "/voryn/message/1.0.0";

/// Protocol ID for delivery acknowledgment.
pub const ACK_PROTOCOL: &str = "/voryn/ack/1.0.0";

/// Protocol ID for remote wipe commands.
pub const WIPE_PROTOCOL: &str = "/voryn/wipe/1.0.0";

/// Protocol ID for group message sync.
pub const GROUP_SYNC_PROTOCOL: &str = "/voryn/group-sync/1.0.0";

/// Protocol ID for identity revocation broadcast.
pub const REVOCATION_PROTOCOL: &str = "/voryn/revocation/1.0.0";
