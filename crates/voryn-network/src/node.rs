//! libp2p node lifecycle — start, stop, peer management.
//!
//! The node runs as a long-lived background task on a Tokio runtime,
//! independent of the React Native JS thread.
//!
//! NOTE: libp2p integration is stubbed out until Cargo.lock is generated
//! from a dev environment with network access. The full implementation
//! exists but requires pinned dependency versions.

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, oneshot};
use tracing::info;

use crate::NetworkError;

/// Handle to a running Voryn network node.
pub struct NodeHandle {
    /// Channel to send commands to the node event loop.
    command_tx: mpsc::Sender<NodeCommand>,
    /// Channel to receive events from the node.
    pub event_rx: Arc<Mutex<mpsc::Receiver<NodeEvent>>>,
    /// Shutdown signal.
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Our PeerId on the network.
    pub peer_id: String,
}

impl NodeHandle {
    pub async fn send_message(&self, peer_id: &str, data: Vec<u8>) -> Result<(), NetworkError> {
        self.command_tx
            .send(NodeCommand::SendMessage { peer_id: peer_id.to_string(), data })
            .await
            .map_err(|_| NetworkError::ConnectionFailed("Command channel closed".into()))
    }

    pub async fn discover_peer(&self, peer_id: &str) -> Result<(), NetworkError> {
        self.command_tx
            .send(NodeCommand::DiscoverPeer { peer_id: peer_id.to_string() })
            .await
            .map_err(|_| NetworkError::ConnectionFailed("Command channel closed".into()))
    }

    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Commands sent from the app to the network node.
#[derive(Debug)]
pub enum NodeCommand {
    SendMessage { peer_id: String, data: Vec<u8> },
    DiscoverPeer { peer_id: String },
}

/// Events emitted by the network node to the app.
#[derive(Debug, Clone)]
pub enum NodeEvent {
    PeerDiscovered { peer_id: String },
    PeerConnected { peer_id: String },
    PeerDisconnected { peer_id: String },
    MessageReceived { peer_id: String, data: Vec<u8> },
    Started { peer_id: String, listening_on: Vec<String> },
}

/// Configuration for starting a network node.
#[derive(Clone)]
pub struct NodeConfig {
    pub keypair_bytes: Vec<u8>,
    pub bootstrap_peers: Vec<String>,
    pub listen_port: u16,
    pub enable_mdns: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self { keypair_bytes: Vec::new(), bootstrap_peers: Vec::new(), listen_port: 0, enable_mdns: true }
    }
}

/// Start a Voryn network node on a background Tokio task.
/// Stub implementation — full libp2p integration pending.
pub async fn start_node(config: NodeConfig) -> Result<NodeHandle, NetworkError> {
    let peer_id = format!("stub-peer-{}", hex::encode(config.keypair_bytes.get(..8).unwrap_or(&[0; 8])));
    info!("Voryn network node starting (stub) with PeerId: {}", peer_id);

    let (command_tx, _command_rx) = mpsc::channel::<NodeCommand>(256);
    let (_event_tx, event_rx) = mpsc::channel::<NodeEvent>(256);
    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();

    Ok(NodeHandle {
        command_tx,
        event_rx: Arc::new(Mutex::new(event_rx)),
        shutdown_tx: Some(shutdown_tx),
        peer_id,
    })
}

/// Stop a running network node.
pub fn stop_node(handle: NodeHandle) {
    handle.shutdown();
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_node_with_random_identity() {
        let config = NodeConfig::default();
        let handle = start_node(config).await.unwrap();
        assert!(!handle.peer_id.is_empty());
        handle.shutdown();
    }
}
