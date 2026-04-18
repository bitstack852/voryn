//! libp2p node lifecycle — start, stop, peer management.
//!
//! The node runs as a long-lived Tokio task. The swarm drives Kademlia DHT,
//! mDNS local discovery, Identify, and our custom /voryn/message/1.0.0 protocol.
//! A shared event queue lets the FFI layer poll for inbound events without
//! needing async context.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use libp2p::{
    identify, kad, relay,
    mdns,
    multiaddr::Protocol,
    noise,
    request_response::{self, json, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm,
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use hickory_resolver::config::{ResolverConfig, ResolverOpts};

use crate::protocols::{VorynRequest, VorynResponse, MESSAGE_PROTOCOL};
use crate::NetworkError;

// ── Public types ──────────────────────────────────────────────────

/// Handle to a running Voryn network node.
pub struct NodeHandle {
    pub peer_id: String,
    command_tx: mpsc::Sender<NodeCommand>,
    pub event_queue: Arc<Mutex<VecDeque<NodeEvent>>>,
}

impl NodeHandle {
    pub fn send_message_cmd(&self, peer_id: String, data: Vec<u8>) -> Result<(), NetworkError> {
        self.command_tx
            .try_send(NodeCommand::SendMessage { peer_id, data })
            .map_err(|_| NetworkError::ConnectionFailed("Command channel full or closed".into()))
    }

    pub fn discover_peer_cmd(&self, peer_id: String) -> Result<(), NetworkError> {
        self.command_tx
            .try_send(NodeCommand::DiscoverPeer { peer_id })
            .map_err(|_| NetworkError::ConnectionFailed("Command channel full or closed".into()))
    }

    pub fn poll_event(&self) -> Option<NodeEvent> {
        self.event_queue.lock().ok()?.pop_front()
    }

    pub fn shutdown(&self) {
        let _ = self.command_tx.try_send(NodeCommand::Shutdown);
    }
}

/// Commands the app sends to the swarm task.
#[derive(Debug)]
pub enum NodeCommand {
    SendMessage { peer_id: String, data: Vec<u8> },
    DiscoverPeer { peer_id: String },
    Shutdown,
}

/// Events emitted by the swarm task.
#[derive(Debug, Clone)]
pub enum NodeEvent {
    PeerDiscovered { peer_id: String },
    PeerConnected { peer_id: String },
    PeerDisconnected { peer_id: String },
    MessageReceived { peer_id: String, data: Vec<u8> },
    Started { peer_id: String, listening_on: Vec<String> },
    Error { message: String },
}

/// Configuration for starting a network node.
#[derive(Clone)]
pub struct NodeConfig {
    /// Raw 32-byte Ed25519 seed used as the libp2p identity.
    /// Falls back to a randomly generated keypair if empty.
    pub keypair_bytes: Vec<u8>,
    /// Bootstrap peer multiaddrs: `/dns4/host/tcp/4001/p2p/<PeerId>`.
    pub bootstrap_peers: Vec<String>,
    /// TCP listen port. 0 = OS-assigned.
    pub listen_port: u16,
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

// ── Combined swarm behaviour ──────────────────────────────────────

#[derive(NetworkBehaviour)]
struct VorynBehaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    mdns: mdns::tokio::Behaviour,
    identify: identify::Behaviour,
    messaging: json::Behaviour<VorynRequest, VorynResponse>,
    relay: relay::client::Behaviour,
}

// ── Public API ────────────────────────────────────────────────────

/// Build a libp2p Swarm and drive it on a background Tokio task.
pub async fn start_node(config: NodeConfig) -> Result<NodeHandle, NetworkError> {
    let keypair = build_keypair(&config.keypair_bytes)?;
    let local_peer_id = keypair.public().to_peer_id();
    info!("Starting Voryn node with PeerId: {}", local_peer_id);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)
        .map_err(|e| NetworkError::StartFailed(e.to_string()))?
        .with_relay_client(noise::Config::new, yamux::Config::default)
        .map_err(|e| NetworkError::StartFailed(e.to_string()))?
        .with_dns_config(ResolverConfig::cloudflare(), ResolverOpts::default())
        .with_behaviour(|key, relay_client| {
            let peer_id = key.public().to_peer_id();

            let mut kademlia = kad::Behaviour::new(
                peer_id,
                kad::store::MemoryStore::new(peer_id),
            );
            kademlia.set_mode(Some(kad::Mode::Server));

            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)
                .map_err(|e| NetworkError::StartFailed(e.to_string()))?;

            let identify = identify::Behaviour::new(identify::Config::new(
                "/voryn/1.0.0".to_string(),
                key.public(),
            ));

            let messaging = json::Behaviour::<VorynRequest, VorynResponse>::new(
                [(
                    StreamProtocol::new(MESSAGE_PROTOCOL),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            );

            Ok(VorynBehaviour { kademlia, mdns, identify, messaging, relay: relay_client })
        })
        .map_err(|e| NetworkError::StartFailed(e.to_string()))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // Add bootstrap peers to kademlia and dial them.
    let mut bootstrapped = false;
    let mut relay_bootstrap_addrs: HashMap<PeerId, Multiaddr> = HashMap::new();
    for addr_str in &config.bootstrap_peers {
        match addr_str.parse::<Multiaddr>() {
            Ok(addr) => {
                if let Some(peer_id) = extract_peer_id(&addr) {
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                    let _ = swarm.dial(addr.clone());
                    relay_bootstrap_addrs.insert(peer_id, addr);
                    bootstrapped = true;
                    info!("Added bootstrap peer: {}", peer_id);
                } else {
                    warn!("Bootstrap addr missing /p2p component: {}", addr_str);
                }
            }
            Err(e) => warn!("Invalid bootstrap addr '{}': {}", addr_str, e),
        }
    }

    let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", config.listen_port)
        .parse()
        .map_err(|e: libp2p::multiaddr::Error| NetworkError::StartFailed(e.to_string()))?;
    swarm.listen_on(listen_addr).map_err(|e| NetworkError::StartFailed(e.to_string()))?;

    if bootstrapped {
        if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
            warn!("DHT bootstrap (no known peers yet): {}", e);
        }
    }

    let (command_tx, command_rx) = mpsc::channel::<NodeCommand>(256);
    let event_queue: Arc<Mutex<VecDeque<NodeEvent>>> = Arc::new(Mutex::new(VecDeque::new()));
    let eq_clone = event_queue.clone();

    tokio::spawn(run_swarm(swarm, command_rx, eq_clone, relay_bootstrap_addrs));

    Ok(NodeHandle {
        peer_id: local_peer_id.to_string(),
        command_tx,
        event_queue,
    })
}

pub fn stop_node(handle: NodeHandle) {
    handle.shutdown();
}

// ── Swarm event loop ──────────────────────────────────────────────

async fn run_swarm(
    mut swarm: Swarm<VorynBehaviour>,
    mut command_rx: mpsc::Receiver<NodeCommand>,
    event_queue: Arc<Mutex<VecDeque<NodeEvent>>>,
    relay_bootstrap_addrs: HashMap<PeerId, Multiaddr>,
) {
    let mut pending: HashMap<PeerId, Vec<Vec<u8>>> = HashMap::new();
    let mut listen_addrs: Vec<String> = Vec::new();

    loop {
        tokio::select! {
            Some(cmd) = command_rx.recv() => {
                if handle_command(cmd, &mut swarm, &mut pending, &event_queue) {
                    break;
                }
            }
            event = swarm.select_next_some() => {
                handle_swarm_event(
                    event, &mut swarm, &mut pending, &event_queue,
                    &mut listen_addrs, &relay_bootstrap_addrs,
                );
            }
        }
    }
    info!("Voryn swarm task exited");
}

/// Returns `true` if the task should exit.
fn handle_command(
    cmd: NodeCommand,
    swarm: &mut Swarm<VorynBehaviour>,
    pending: &mut HashMap<PeerId, Vec<Vec<u8>>>,
    _event_queue: &Arc<Mutex<VecDeque<NodeEvent>>>,
) -> bool {
    match cmd {
        NodeCommand::SendMessage { peer_id, data } => {
            match peer_id.parse::<PeerId>() {
                Ok(target) => {
                    if swarm.is_connected(&target) {
                        swarm.behaviour_mut().messaging.send_request(&target, VorynRequest { data });
                    } else {
                        pending.entry(target).or_default().push(data);
                        swarm.behaviour_mut().kademlia.get_closest_peers(target);
                    }
                }
                Err(e) => error!("SendMessage: invalid PeerId '{}': {}", peer_id, e),
            }
            false
        }
        NodeCommand::DiscoverPeer { peer_id } => {
            match peer_id.parse::<PeerId>() {
                Ok(target) => { swarm.behaviour_mut().kademlia.get_closest_peers(target); }
                Err(e) => error!("DiscoverPeer: invalid PeerId '{}': {}", peer_id, e),
            }
            false
        }
        NodeCommand::Shutdown => true,
    }
}

fn handle_swarm_event(
    event: SwarmEvent<VorynBehaviourEvent>,
    swarm: &mut Swarm<VorynBehaviour>,
    pending: &mut HashMap<PeerId, Vec<Vec<u8>>>,
    event_queue: &Arc<Mutex<VecDeque<NodeEvent>>>,
    listen_addrs: &mut Vec<String>,
    relay_bootstrap_addrs: &HashMap<PeerId, Multiaddr>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            let local_peer_id = *swarm.local_peer_id();
            // Register relay address as external so identify advertises it
            if address.iter().any(|p| matches!(p, Protocol::P2pCircuit)) {
                let relay_addr = address.clone().with(Protocol::P2p(local_peer_id));
                swarm.add_external_address(relay_addr.clone());
                info!("Relay address registered: {}", relay_addr);
            }
            let addr_str = format!("{}/p2p/{}", address, local_peer_id);
            info!("Listening on {}", addr_str);
            listen_addrs.push(addr_str);
            push_event(
                event_queue,
                NodeEvent::Started {
                    peer_id: local_peer_id.to_string(),
                    listening_on: listen_addrs.clone(),
                },
            );
        }

        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            info!("Connected: {}", peer_id);
            push_event(event_queue, NodeEvent::PeerConnected { peer_id: peer_id.to_string() });
            if let Some(msgs) = pending.remove(&peer_id) {
                for data in msgs {
                    swarm.behaviour_mut().messaging.send_request(&peer_id, VorynRequest { data });
                }
            }
            // Reserve a relay slot on this peer if it's a bootstrap relay
            if let Some(bootstrap_addr) = relay_bootstrap_addrs.get(&peer_id) {
                let circuit_addr = bootstrap_addr.clone().with(Protocol::P2pCircuit);
                if let Err(e) = swarm.listen_on(circuit_addr) {
                    warn!("Relay reservation request failed: {}", e);
                } else {
                    info!("Relay reservation requested on {}", peer_id);
                }
            }
        }
        SwarmEvent::ConnectionClosed { peer_id, .. } => {
            debug!("Disconnected: {}", peer_id);
            push_event(event_queue, NodeEvent::PeerDisconnected { peer_id: peer_id.to_string() });
        }

        SwarmEvent::Behaviour(VorynBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, addr) in list {
                let peer_id_str = peer_id.to_string();
                info!("mDNS: {} @ {}", peer_id_str, addr);
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                if !swarm.is_connected(&peer_id) {
                    let dial_addr = addr.with(Protocol::P2p(peer_id));
                    if let Err(e) = swarm.dial(dial_addr) {
                        warn!("mDNS dial failed: {}", e);
                    }
                }
                push_event(event_queue, NodeEvent::PeerDiscovered { peer_id: peer_id_str });
            }
        }
        SwarmEvent::Behaviour(VorynBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
            for (peer_id, _) in list {
                debug!("mDNS expired: {}", peer_id);
            }
        }

        SwarmEvent::Behaviour(VorynBehaviourEvent::Identify(identify::Event::Received {
            peer_id,
            info,
            ..
        })) => {
            for addr in info.listen_addrs {
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
            }
        }

        SwarmEvent::Behaviour(VorynBehaviourEvent::Kademlia(
            kad::Event::RoutingUpdated { peer, .. },
        )) => {
            debug!("DHT routing updated: {}", peer);
        }
        SwarmEvent::Behaviour(VorynBehaviourEvent::Kademlia(
            kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::GetClosestPeers(Ok(ok)),
                ..
            },
        )) => {
            for peer_info in ok.peers {
                let peer_id_str = peer_info.peer_id.to_string();
                debug!("DHT found peer: {} addrs={}", peer_id_str, peer_info.addrs.len());
                if pending.contains_key(&peer_info.peer_id) && !swarm.is_connected(&peer_info.peer_id) {
                    for addr in &peer_info.addrs {
                        let dial_addr = addr.clone().with(Protocol::P2p(peer_info.peer_id.clone()));
                        if swarm.dial(dial_addr).is_ok() {
                            info!("DHT: dialing pending-message peer {}", peer_id_str);
                            break;
                        }
                    }
                }
                push_event(event_queue, NodeEvent::PeerDiscovered { peer_id: peer_id_str });
            }
        }

        SwarmEvent::Behaviour(VorynBehaviourEvent::Messaging(
            request_response::Event::Message { peer, message },
        )) => match message {
            request_response::Message::Request { request, channel, .. } => {
                debug!("Inbound message from {}: {} bytes", peer, request.data.len());
                let _ = swarm
                    .behaviour_mut()
                    .messaging
                    .send_response(channel, VorynResponse { accepted: true });
                push_event(
                    event_queue,
                    NodeEvent::MessageReceived {
                        peer_id: peer.to_string(),
                        data: request.data,
                    },
                );
            }
            request_response::Message::Response { response, .. } => {
                debug!("Delivery ack from {}: accepted={}", peer, response.accepted);
            }
        },
        SwarmEvent::Behaviour(VorynBehaviourEvent::Messaging(
            request_response::Event::OutboundFailure { peer, error, .. },
        )) => {
            warn!("Send failure to {}: {}", peer, error);
        }

        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
            let msg = format!("Dial failed {:?}: {}", peer_id, error);
            warn!("{}", msg);
            push_event(event_queue, NodeEvent::Error { message: msg });
        }

        SwarmEvent::ListenerError { error, .. } => {
            let msg = format!("Listener error: {}", error);
            warn!("{}", msg);
            push_event(event_queue, NodeEvent::Error { message: msg });
        }

        SwarmEvent::ListenerClosed { reason, .. } => {
            let msg = format!("Listener closed: {:?}", reason);
            warn!("{}", msg);
            push_event(event_queue, NodeEvent::Error { message: msg });
        }

        SwarmEvent::Behaviour(VorynBehaviourEvent::Relay(event)) => {
            debug!("Relay event: {:?}", event);
        }

        _ => {}
    }
}

// ── Helpers ───────────────────────────────────────────────────────

fn build_keypair(bytes: &[u8]) -> Result<libp2p::identity::Keypair, NetworkError> {
    if bytes.len() >= 32 {
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes[..32]);
        let secret = libp2p::identity::ed25519::SecretKey::try_from_bytes(&mut seed)
            .map_err(|e| NetworkError::StartFailed(format!("Bad Ed25519 seed: {}", e)))?;
        let kp = libp2p::identity::ed25519::Keypair::from(secret);
        Ok(libp2p::identity::Keypair::from(kp))
    } else {
        Ok(libp2p::identity::Keypair::generate_ed25519())
    }
}

fn extract_peer_id(addr: &Multiaddr) -> Option<PeerId> {
    addr.iter().find_map(|p| {
        if let Protocol::P2p(peer_id) = p {
            Some(peer_id)
        } else {
            None
        }
    })
}

fn push_event(queue: &Arc<Mutex<VecDeque<NodeEvent>>>, event: NodeEvent) {
    if let Ok(mut q) = queue.lock() {
        if q.len() < 256 {
            q.push_back(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_node_random_identity() {
        let config = NodeConfig::default();
        let handle = start_node(config).await.unwrap();
        assert!(!handle.peer_id.is_empty());
        handle.shutdown();
    }

    #[tokio::test]
    async fn test_start_node_same_seed_same_peer_id() {
        let seed = vec![42u8; 32];
        let config = NodeConfig { keypair_bytes: seed.clone(), ..Default::default() };
        let h1 = start_node(config.clone()).await.unwrap();
        let h2 = start_node(config).await.unwrap();
        assert_eq!(h1.peer_id, h2.peer_id);
        h1.shutdown();
        h2.shutdown();
    }
}
