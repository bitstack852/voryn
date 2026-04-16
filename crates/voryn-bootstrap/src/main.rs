//! Voryn Bootstrap Node — DHT peer discovery server.
//!
//! Runs a full libp2p node in server mode. New Voryn devices dial this node
//! to join the Kademlia DHT. It does NOT store messages or relay traffic.
//!
//! Usage:
//!   voryn-bootstrap --listen /ip4/0.0.0.0/tcp/4001
//!   voryn-bootstrap --listen /ip4/0.0.0.0/tcp/4001 --identity-file /opt/voryn/data/node.key

use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use futures::StreamExt;
use libp2p::{
    identify, kad,
    multiaddr::Protocol,
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr,
};
use tracing::{debug, info, warn};

#[derive(Parser, Debug)]
#[command(name = "voryn-bootstrap")]
#[command(about = "Voryn DHT Bootstrap Node — peer discovery server")]
struct Args {
    /// Listen address (multiaddr or host:port)
    #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/4001")]
    listen: String,

    /// Path to store the node identity key (Ed25519 seed, hex-encoded JSON)
    #[arg(short, long, default_value = "node-identity.key")]
    identity_file: PathBuf,

    #[arg(long, default_value = "info")]
    log_level: String,
}

// ── Swarm behaviour ───────────────────────────────────────────────

#[derive(NetworkBehaviour)]
struct BootstrapBehaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    identify: identify::Behaviour,
}

// ── Identity persistence ──────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedKey {
    seed_hex: String,
}

fn load_or_create_keypair(path: &PathBuf) -> anyhow::Result<libp2p::identity::Keypair> {
    if path.exists() {
        let json = std::fs::read_to_string(path)?;
        let pk: PersistedKey = serde_json::from_str(&json)?;
        let seed_bytes = hex_decode(&pk.seed_hex)?;
        if seed_bytes.len() < 32 {
            anyhow::bail!("Seed too short");
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&seed_bytes[..32]);
        let secret = libp2p::identity::ed25519::SecretKey::try_from_bytes(&mut seed)?;
        let kp = libp2p::identity::ed25519::Keypair::from(secret);
        let keypair = libp2p::identity::Keypair::from(kp);
        info!("Loaded identity: {}", keypair.public().to_peer_id());
        Ok(keypair)
    } else {
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let ed25519_kp = keypair
            .clone()
            .try_into_ed25519()
            .expect("just generated ed25519");
        let seed_hex = hex_encode(ed25519_kp.secret().as_ref());
        let persisted = PersistedKey { seed_hex };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(&persisted)?)?;
        info!("Generated new identity: {}", keypair.public().to_peer_id());
        Ok(keypair)
    }
}

// ── Main ──────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let filter = args
        .log_level
        .parse::<tracing_subscriber::filter::LevelFilter>()
        .unwrap_or(tracing_subscriber::filter::LevelFilter::INFO);
    tracing_subscriber::fmt().with_max_level(filter).init();

    info!("=== Voryn Bootstrap Node v{} ===", env!("CARGO_PKG_VERSION"));

    let keypair = load_or_create_keypair(&args.identity_file)?;
    let local_peer_id = keypair.public().to_peer_id();
    info!("PeerId: {}", local_peer_id);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.clone())
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();

            let mut kademlia = kad::Behaviour::new(
                peer_id,
                kad::store::MemoryStore::new(peer_id),
            );
            // Bootstrap nodes run in Server mode — they answer queries.
            kademlia.set_mode(Some(kad::Mode::Server));

            let identify = identify::Behaviour::new(identify::Config::new(
                "/voryn/1.0.0".to_string(),
                key.public(),
            ));

            Ok(BootstrapBehaviour { kademlia, identify })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(300)))
        .build();

    let listen_addr: Multiaddr = parse_listen_addr(&args.listen);
    swarm.listen_on(listen_addr)?;

    info!("Bootstrap node running. Ctrl+C to stop.");

    let mut peer_count: u64 = 0;

    loop {
        tokio::select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {}/p2p/{}", address, local_peer_id);
                    info!("Bootstrap multiaddr: {}/p2p/{}", address, local_peer_id);
                }
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    peer_count += 1;
                    let addr = match &endpoint {
                        libp2p::core::ConnectedPoint::Dialer { address, .. } => address.to_string(),
                        libp2p::core::ConnectedPoint::Listener { send_back_addr, .. } => send_back_addr.to_string(),
                    };
                    info!("Peer connected: {} from {} (total: {})", peer_id, addr, peer_count);
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    debug!("Peer disconnected: {}", peer_id);
                }
                SwarmEvent::Behaviour(BootstrapBehaviourEvent::Identify(
                    identify::Event::Received { peer_id, info, .. },
                )) => {
                    for addr in &info.listen_addrs {
                        swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                    }
                    debug!("Identified peer {}: protocols={:?}", peer_id, info.protocols);
                }
                SwarmEvent::Behaviour(BootstrapBehaviourEvent::Kademlia(
                    kad::Event::RoutingUpdated { peer, is_new_peer: true, .. },
                )) => {
                    info!("New DHT peer: {}", peer);
                }
                SwarmEvent::Behaviour(BootstrapBehaviourEvent::Kademlia(
                    kad::Event::InboundRequest { request },
                )) => {
                    debug!("DHT inbound: {:?}", request);
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    warn!("Outgoing connection error {:?}: {}", peer_id, error);
                }
                _ => {}
            },
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down bootstrap node...");
                break;
            }
        }
    }

    info!("Bootstrap node stopped. Total peers served: {}", peer_count);
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────

fn parse_listen_addr(addr: &str) -> Multiaddr {
    if let Ok(ma) = addr.parse::<Multiaddr>() {
        return ma;
    }
    // Try host:port fallback
    if let Ok(sa) = addr.parse::<std::net::SocketAddr>() {
        let ip = sa.ip();
        let port = sa.port();
        let proto = match ip {
            std::net::IpAddr::V4(v4) => Protocol::Ip4(v4),
            std::net::IpAddr::V6(v6) => Protocol::Ip6(v6),
        };
        let mut ma = Multiaddr::empty();
        ma.push(proto);
        ma.push(Protocol::Tcp(port));
        return ma;
    }
    warn!("Could not parse listen addr '{}', falling back to 0.0.0.0:4001", addr);
    "/ip4/0.0.0.0/tcp/4001".parse().unwrap()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| anyhow::anyhow!(e)))
        .collect()
}
