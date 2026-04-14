//! libp2p node lifecycle — start, stop, peer management.
//!
//! The node runs as a long-lived background task on a Tokio runtime,
//! independent of the React Native JS thread.

use crate::NetworkError;

/// Handle to a running Voryn network node.
pub struct NodeHandle {
    // Will hold the Tokio JoinHandle and shutdown signal in Phase 1
    _placeholder: (),
}

/// Configuration for starting a network node.
pub struct NodeConfig {
    /// Ed25519 secret key bytes (for deriving libp2p PeerId).
    pub keypair_bytes: Vec<u8>,
    /// Bootstrap peer multiaddresses.
    pub bootstrap_peers: Vec<String>,
    /// Port to listen on (0 = random).
    pub listen_port: u16,
    /// Enable mDNS for local network discovery.
    pub enable_mdns: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            keypair_bytes: Vec::new(),
            bootstrap_peers: Vec::new(),
            listen_port: 0,
            enable_mdns: true,
        }
    }
}

/// Start a Voryn network node. Returns a handle for lifecycle management.
/// Full implementation in Phase 1, Milestone 1.3.
pub fn start_node(_config: NodeConfig) -> Result<NodeHandle, NetworkError> {
    tracing::info!("Voryn network node starting (stub)");
    Ok(NodeHandle { _placeholder: () })
}

/// Stop a running network node.
pub fn stop_node(_handle: NodeHandle) -> Result<(), NetworkError> {
    tracing::info!("Voryn network node stopping (stub)");
    Ok(())
}
