//! Voryn Network — Decentralized P2P networking built on rust-libp2p.
//!
//! Every Voryn device runs a full DHT node. This crate handles:
//! - Transport (TCP/Noise)
//! - Peer discovery (Kademlia DHT + mDNS)
//! - Custom messaging protocol (/voryn/message/1.0.0)
//! - Traffic obfuscation helpers

pub mod config;
pub mod discovery;
pub mod node;
pub mod obfuscation;
pub mod protocols;
pub mod transport;

pub use node::{start_node, stop_node, NodeCommand, NodeConfig, NodeEvent, NodeHandle};

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Failed to start node: {0}")]
    StartFailed(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Transport error: {0}")]
    TransportError(String),
}
