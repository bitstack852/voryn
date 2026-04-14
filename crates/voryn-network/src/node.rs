//! libp2p node lifecycle — start, stop, peer management.
//!
//! The node runs as a long-lived background task on a Tokio runtime,
//! independent of the React Native JS thread.

use futures::StreamExt;
use libp2p::{
    identity,
    kad,
    mdns,
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, oneshot};
use tracing::{info, warn, error};

use crate::NetworkError;
use crate::config;

/// Handle to a running Voryn network node.
pub struct NodeHandle {
    /// Channel to send commands to the node event loop.
    command_tx: mpsc::Sender<NodeCommand>,
    /// Channel to receive events from the node.
    pub event_rx: Arc<Mutex<mpsc::Receiver<NodeEvent>>>,
    /// Shutdown signal.
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Our PeerId on the network.
    pub peer_id: PeerId,
}

impl NodeHandle {
    /// Send a message to a peer.
    pub async fn send_message(&self, peer_id: PeerId, data: Vec<u8>) -> Result<(), NetworkError> {
        self.command_tx
            .send(NodeCommand::SendMessage { peer_id, data })
            .await
            .map_err(|_| NetworkError::ConnectionFailed("Command channel closed".into()))
    }

    /// Discover a peer by their public key.
    pub async fn discover_peer(&self, peer_id: PeerId) -> Result<(), NetworkError> {
        self.command_tx
            .send(NodeCommand::DiscoverPeer { peer_id })
            .await
            .map_err(|_| NetworkError::ConnectionFailed("Command channel closed".into()))
    }

    /// Shut down the node gracefully.
    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Commands sent from the app to the network node.
#[derive(Debug)]
pub enum NodeCommand {
    SendMessage { peer_id: PeerId, data: Vec<u8> },
    DiscoverPeer { peer_id: PeerId },
}

/// Events emitted by the network node to the app.
#[derive(Debug, Clone)]
pub enum NodeEvent {
    /// We discovered a new peer.
    PeerDiscovered { peer_id: PeerId },
    /// A peer connected to us.
    PeerConnected { peer_id: PeerId },
    /// A peer disconnected.
    PeerDisconnected { peer_id: PeerId },
    /// We received a message from a peer.
    MessageReceived { peer_id: PeerId, data: Vec<u8> },
    /// Node started successfully.
    Started { peer_id: PeerId, listening_on: Vec<Multiaddr> },
}

/// Configuration for starting a network node.
#[derive(Clone)]
pub struct NodeConfig {
    /// Ed25519 secret key bytes (for deriving libp2p identity).
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

/// Combined network behaviour for Voryn.
#[derive(NetworkBehaviour)]
pub struct VorynBehaviour {
    /// Kademlia DHT for peer routing and discovery.
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// mDNS for local network peer discovery.
    pub mdns: mdns::tokio::Behaviour,
}

/// Start a Voryn network node on a background Tokio task.
///
/// Returns a NodeHandle for sending commands and receiving events.
pub async fn start_node(node_config: NodeConfig) -> Result<NodeHandle, NetworkError> {
    // Derive libp2p identity from Ed25519 secret key bytes
    let keypair = if node_config.keypair_bytes.is_empty() {
        identity::Keypair::generate_ed25519()
    } else {
        let sk_bytes = &node_config.keypair_bytes[..32]; // First 32 bytes = seed
        let mut seed = [0u8; 32];
        seed.copy_from_slice(sk_bytes);
        identity::Keypair::ed25519_from_bytes(seed)
            .map_err(|e| NetworkError::StartFailed(format!("Invalid keypair: {}", e)))?
    };

    let peer_id = PeerId::from(keypair.public());
    info!("Voryn node starting with PeerId: {}", peer_id);

    // Build the swarm
    let swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| NetworkError::StartFailed(format!("TCP transport error: {}", e)))?
        .with_behaviour(|key| {
            let peer_id = PeerId::from(key.public());

            // Kademlia DHT
            let store = kad::store::MemoryStore::new(peer_id);
            let mut kad_config = kad::Config::default();
            kad_config.set_replication_factor(
                std::num::NonZeroUsize::new(config::KAD_REPLICATION_FACTOR).unwrap(),
            );
            let kademlia = kad::Behaviour::with_config(peer_id, store, kad_config);

            // mDNS
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                peer_id,
            ).expect("mDNS behaviour creation failed");

            VorynBehaviour { kademlia, mdns }
        })
        .map_err(|e| NetworkError::StartFailed(format!("Behaviour error: {}", e)))?
        .build();

    // Create communication channels
    let (command_tx, command_rx) = mpsc::channel::<NodeCommand>(256);
    let (event_tx, event_rx) = mpsc::channel::<NodeEvent>(256);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let listen_port = node_config.listen_port;
    let bootstrap_peers = node_config.bootstrap_peers.clone();

    // Spawn the node event loop
    tokio::spawn(async move {
        run_node_event_loop(swarm, command_rx, event_tx, shutdown_rx, listen_port, bootstrap_peers).await;
    });

    Ok(NodeHandle {
        command_tx,
        event_rx: Arc::new(Mutex::new(event_rx)),
        shutdown_tx: Some(shutdown_tx),
        peer_id,
    })
}

/// Main event loop for the libp2p node.
async fn run_node_event_loop(
    mut swarm: Swarm<VorynBehaviour>,
    mut command_rx: mpsc::Receiver<NodeCommand>,
    event_tx: mpsc::Sender<NodeEvent>,
    mut shutdown_rx: oneshot::Receiver<()>,
    listen_port: u16,
    bootstrap_peers: Vec<String>,
) {
    // Start listening
    let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", listen_port)
        .parse()
        .expect("Valid multiaddr");

    if let Err(e) = swarm.listen_on(listen_addr) {
        error!("Failed to start listening: {}", e);
        return;
    }

    // Connect to bootstrap peers
    for addr_str in &bootstrap_peers {
        if let Ok(addr) = addr_str.parse::<Multiaddr>() {
            info!("Dialing bootstrap peer: {}", addr);
            if let Err(e) = swarm.dial(addr.clone()) {
                warn!("Failed to dial bootstrap peer {}: {}", addr, e);
            }
        }
    }

    let mut discovered_peers: HashMap<PeerId, Vec<Multiaddr>> = HashMap::new();

    loop {
        tokio::select! {
            // Handle shutdown
            _ = &mut shutdown_rx => {
                info!("Voryn node shutting down");
                break;
            }

            // Handle incoming commands from the app
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    NodeCommand::SendMessage { peer_id, data: _data } => {
                        // In Phase 1, we use request-response for messaging.
                        // For now, log the intent.
                        info!("Send message to peer {} ({} bytes)", peer_id, _data.len());
                        // TODO: Implement request-response protocol
                    }
                    NodeCommand::DiscoverPeer { peer_id } => {
                        info!("Discovering peer: {}", peer_id);
                        swarm.behaviour_mut().kademlia.get_closest_peers(peer_id);
                    }
                }
            }

            // Handle swarm events
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {}", address);
                    }

                    SwarmEvent::Behaviour(VorynBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                        for (peer_id, addr) in peers {
                            info!("mDNS discovered: {} at {}", peer_id, addr);
                            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                            discovered_peers.entry(peer_id).or_default().push(addr);
                            let _ = event_tx.send(NodeEvent::PeerDiscovered { peer_id }).await;
                        }
                    }

                    SwarmEvent::Behaviour(VorynBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                        for (peer_id, _addr) in peers {
                            info!("mDNS peer expired: {}", peer_id);
                            discovered_peers.remove(&peer_id);
                        }
                    }

                    SwarmEvent::Behaviour(VorynBehaviourEvent::Kademlia(kad::Event::RoutingUpdated { peer, .. })) => {
                        info!("Kademlia routing updated for: {}", peer);
                    }

                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("Connected to: {}", peer_id);
                        let _ = event_tx.send(NodeEvent::PeerConnected { peer_id }).await;
                    }

                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        info!("Disconnected from: {}", peer_id);
                        let _ = event_tx.send(NodeEvent::PeerDisconnected { peer_id }).await;
                    }

                    _ => {}
                }
            }
        }
    }
}

/// Stop a running network node.
pub fn stop_node(handle: NodeHandle) {
    handle.shutdown();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_node_with_random_identity() {
        let config = NodeConfig::default();
        let handle = start_node(config).await.unwrap();
        assert!(!handle.peer_id.to_string().is_empty());
        handle.shutdown();
    }
}
