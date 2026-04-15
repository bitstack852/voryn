//! Voryn Bootstrap Node — DHT peer discovery server.
//!
//! This binary runs on a server and helps new Voryn devices discover
//! each other on the P2P network. It does NOT store messages or relay traffic.
//!
//! Usage:
//!   voryn-bootstrap --listen /ip4/0.0.0.0/tcp/4001
//!   voryn-bootstrap --listen /ip4/0.0.0.0/tcp/4001 --identity-file /opt/voryn/data/node.key

use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{info, warn, error};

#[derive(Parser, Debug)]
#[command(name = "voryn-bootstrap")]
#[command(about = "Voryn DHT Bootstrap Node — peer discovery server")]
struct Args {
    /// Listen address (e.g., /ip4/0.0.0.0/tcp/4001 or 0.0.0.0:4001)
    #[arg(short, long, default_value = "0.0.0.0:4001")]
    listen: String,

    /// Path to store the node identity key
    #[arg(short, long, default_value = "node-identity.key")]
    identity_file: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

/// Node identity — persisted to disk so the PeerId stays the same across restarts.
#[derive(serde::Serialize, serde::Deserialize)]
struct NodeIdentity {
    /// Ed25519 secret key seed (32 bytes, hex encoded)
    secret_seed_hex: String,
    /// Derived public key (hex encoded) — serves as the node's PeerId
    public_key_hex: String,
}

impl NodeIdentity {
    fn generate() -> Self {
        let mut seed = [0u8; 32];
        for byte in seed.iter_mut() {
            *byte = rand::random();
        }
        let secret_hex = hex_encode(&seed);
        // For now, public key = hash of seed (placeholder until libp2p is wired)
        let mut pk = [0u8; 32];
        // Simple deterministic derivation for identity
        for (i, byte) in seed.iter().enumerate() {
            pk[i] = byte.wrapping_mul(37).wrapping_add(i as u8);
        }
        let public_hex = hex_encode(&pk);

        Self {
            secret_seed_hex: secret_hex,
            public_key_hex: public_hex,
        }
    }

    fn load_or_create(path: &PathBuf) -> anyhow::Result<Self> {
        if path.exists() {
            let data = std::fs::read_to_string(path)?;
            let identity: NodeIdentity = serde_json::from_str(&data)?;
            info!("Loaded existing identity: {}", &identity.public_key_hex[..16]);
            Ok(identity)
        } else {
            let identity = Self::generate();
            let data = serde_json::to_string_pretty(&identity)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, data)?;
            info!("Generated new identity: {}", &identity.public_key_hex[..16]);
            Ok(identity)
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Parse listen address — supports both multiaddr format and standard socket format.
fn parse_listen_addr(addr: &str) -> SocketAddr {
    // Try standard socket addr first
    if let Ok(sa) = addr.parse::<SocketAddr>() {
        return sa;
    }

    // Try multiaddr format: /ip4/0.0.0.0/tcp/4001
    let parts: Vec<&str> = addr.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 4 && parts[0] == "ip4" && parts[2] == "tcp" {
        let ip = parts[1];
        let port = parts[3];
        if let Ok(sa) = format!("{}:{}", ip, port).parse::<SocketAddr>() {
            return sa;
        }
    }

    // Default
    warn!("Could not parse listen address '{}', using default 0.0.0.0:4001", addr);
    "0.0.0.0:4001".parse().unwrap()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = args.log_level.parse::<tracing_subscriber::filter::LevelFilter>()
        .unwrap_or(tracing_subscriber::filter::LevelFilter::INFO);
    tracing_subscriber::fmt()
        .with_max_level(filter)
        .init();

    info!("=== Voryn Bootstrap Node ===");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Load or generate node identity
    let identity = NodeIdentity::load_or_create(&args.identity_file)?;
    info!("PeerId: {}", identity.public_key_hex);

    // Parse listen address
    let listen_addr = parse_listen_addr(&args.listen);
    info!("Listening on: {}", listen_addr);

    // Start the TCP listener
    // When libp2p is enabled, this will be replaced with a full libp2p swarm.
    // For now, we run a simple TCP server that responds to peer discovery requests.
    let listener = TcpListener::bind(listen_addr).await?;
    info!("Bootstrap node is running. Press Ctrl+C to stop.");

    // Track connected peers
    let mut peer_count: u64 = 0;

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((mut socket, addr)) => {
                        peer_count += 1;
                        info!("Peer connected: {} (total: {})", addr, peer_count);

                        let node_pk = identity.public_key_hex.clone();
                        tokio::spawn(async move {
                            // Send our identity to the connecting peer
                            let response = serde_json::json!({
                                "type": "bootstrap_hello",
                                "version": "0.1.0",
                                "peer_id": node_pk,
                                "protocol": "voryn/bootstrap/1.0",
                            });
                            let data = serde_json::to_vec(&response).unwrap_or_default();
                            let len = (data.len() as u32).to_be_bytes();

                            if let Err(e) = socket.write_all(&len).await {
                                warn!("Failed to write to {}: {}", addr, e);
                                return;
                            }
                            if let Err(e) = socket.write_all(&data).await {
                                warn!("Failed to write to {}: {}", addr, e);
                                return;
                            }

                            // Read peer's hello message
                            let mut len_buf = [0u8; 4];
                            match socket.read_exact(&mut len_buf).await {
                                Ok(_) => {
                                    let msg_len = u32::from_be_bytes(len_buf) as usize;
                                    if msg_len > 65536 {
                                        warn!("Message too large from {}: {} bytes", addr, msg_len);
                                        return;
                                    }
                                    let mut msg_buf = vec![0u8; msg_len];
                                    if let Ok(_) = socket.read_exact(&mut msg_buf).await {
                                        if let Ok(msg) = serde_json::from_slice::<serde_json::Value>(&msg_buf) {
                                            info!("Peer {} registered: {:?}", addr, msg.get("peer_id"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    // Peer disconnected — that's fine for a simple ping
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("Accept error: {}", e);
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down bootstrap node...");
                break;
            }
        }
    }

    info!("Bootstrap node stopped. Total peers served: {}", peer_count);
    Ok(())
}
