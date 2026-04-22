//! Voryn Relay Node — WebSocket-based message relay
//!
//! Devices connect via WebSocket, register their public key, and exchange
//! encrypted messages through the relay. The relay never sees plaintext —
//! it only routes opaque encrypted payloads between peers.
//!
//! Protocol (JSON over WebSocket):
//!   Client → Server: { "type": "register", "peer_id": "<hex pubkey>" }
//!   Client → Server: { "type": "send", "to": "<hex pubkey>", "payload": "...", "message_id": "..." }
//!   Server → Client: { "type": "message", "from": "<hex pubkey>", "payload": "...", "message_id": "..." }
//!   Server → Client: { "type": "ack", "message_id": "...", "status": "delivered|offline" }
//!   Server → Client: { "type": "peer_online", "peer_id": "<hex pubkey>" }
//!   Server → Client: { "type": "peer_offline", "peer_id": "<hex pubkey>" }
//!   Client → Server: { "type": "ping" }
//!   Server → Client: { "type": "pong" }

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, error};
use warp::Filter;
use warp::ws::{Message, WebSocket};

#[derive(Parser, Debug)]
#[command(name = "voryn-bootstrap")]
#[command(about = "Voryn Relay Node — WebSocket message relay")]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0:4001")]
    listen: String,

    #[arg(short, long, default_value = "node-identity.key")]
    identity_file: PathBuf,

    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeIdentity {
    secret_seed_hex: String,
    public_key_hex: String,
}

impl NodeIdentity {
    fn generate() -> Self {
        let mut seed = [0u8; 32];
        for byte in seed.iter_mut() {
            *byte = rand::random();
        }
        let mut pk = [0u8; 32];
        for (i, byte) in seed.iter().enumerate() {
            pk[i] = byte.wrapping_mul(37).wrapping_add(i as u8);
        }
        Self {
            secret_seed_hex: hex_encode(&seed),
            public_key_hex: hex_encode(&pk),
        }
    }

    fn load_or_create(path: &PathBuf) -> anyhow::Result<Self> {
        if path.exists() {
            let data = std::fs::read_to_string(path)?;
            match serde_json::from_str::<NodeIdentity>(&data) {
                Ok(identity) => {
                    info!("Loaded identity: {}", &identity.public_key_hex[..16]);
                    return Ok(identity);
                }
                Err(e) => {
                    warn!("Identity file invalid ({}), regenerating", e);
                    let _ = std::fs::remove_file(path);
                }
            }
        }
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

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// A connected peer with a channel to send messages to them.
struct ConnectedPeer {
    tx: mpsc::UnboundedSender<String>,
}

type PeerMap = Arc<RwLock<HashMap<String, ConnectedPeer>>>;

/// Handle a single WebSocket connection.
async fn handle_ws(ws: WebSocket, peers: PeerMap, node_pk: String) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let mut peer_id: Option<String> = None;

    // Send server hello
    let hello = serde_json::json!({
        "type": "server_hello",
        "version": "0.2.0",
        "server_id": node_pk,
    });
    let _ = ws_tx.send(Message::text(hello.to_string())).await;

    // Spawn task to forward channel messages to WebSocket
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Read messages from WebSocket
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(m) => m,
            Err(_) => break,
        };

        if msg.is_close() {
            break;
        }

        let text = match msg.to_str() {
            Ok(t) => t.to_string(),
            Err(_) => continue,
        };

        let json: serde_json::Value = match serde_json::from_str(&text) {
            Ok(j) => j,
            Err(_) => continue,
        };

        let msg_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match msg_type {
            "register" => {
                let id = json.get("peer_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if id.is_empty() {
                    continue;
                }

                info!("Peer registered: {}...", &id[..16.min(id.len())]);
                peer_id = Some(id.clone());

                // Notify existing peers about new peer
                {
                    let map = peers.read().await;
                    let online_msg = serde_json::json!({
                        "type": "peer_online",
                        "peer_id": &id,
                    }).to_string();

                    for (_, p) in map.iter() {
                        let _ = p.tx.send(online_msg.clone());
                    }

                    // Send list of online peers to new peer
                    for existing_id in map.keys() {
                        let msg = serde_json::json!({
                            "type": "peer_online",
                            "peer_id": existing_id,
                        }).to_string();
                        let _ = tx.send(msg);
                    }
                }

                // Register
                peers.write().await.insert(id, ConnectedPeer { tx: tx.clone() });
            }

            "send" => {
                let to = json.get("to").and_then(|v| v.as_str()).unwrap_or("");
                let payload = json.get("payload").and_then(|v| v.as_str()).unwrap_or("");
                let message_id = json.get("message_id").and_then(|v| v.as_str()).unwrap_or("");
                let from = peer_id.as_deref().unwrap_or("unknown");

                if to.is_empty() || payload.is_empty() {
                    continue;
                }

                info!("Relay: {}... → {}... ({} bytes)",
                    &from[..16.min(from.len())],
                    &to[..16.min(to.len())],
                    payload.len()
                );

                let forward = serde_json::json!({
                    "type": "message",
                    "from": from,
                    "payload": payload,
                    "message_id": message_id,
                }).to_string();

                let map = peers.read().await;
                if let Some(recipient) = map.get(to) {
                    if recipient.tx.send(forward).is_ok() {
                        // ACK delivered
                        let ack = serde_json::json!({
                            "type": "ack",
                            "message_id": message_id,
                            "status": "delivered",
                        }).to_string();
                        let _ = tx.send(ack);
                    }
                } else {
                    // Recipient offline
                    let ack = serde_json::json!({
                        "type": "ack",
                        "message_id": message_id,
                        "status": "offline",
                    }).to_string();
                    let _ = tx.send(ack);
                }
            }

            "ping" => {
                let _ = tx.send(serde_json::json!({"type": "pong"}).to_string());
            }

            _ => {
                warn!("Unknown message type: {}", msg_type);
            }
        }
    }

    // Cleanup: remove peer and notify others
    if let Some(id) = &peer_id {
        info!("Peer disconnected: {}...", &id[..16.min(id.len())]);
        peers.write().await.remove(id);

        let offline_msg = serde_json::json!({
            "type": "peer_offline",
            "peer_id": id,
        }).to_string();

        let map = peers.read().await;
        for (_, p) in map.iter() {
            let _ = p.tx.send(offline_msg.clone());
        }
    }

    forward_task.abort();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let filter = args.log_level.parse::<tracing_subscriber::filter::LevelFilter>()
        .unwrap_or(tracing_subscriber::filter::LevelFilter::INFO);
    tracing_subscriber::fmt()
        .with_max_level(filter)
        .init();

    info!("=== Voryn Relay Node v0.2.0 ===");

    let identity = NodeIdentity::load_or_create(&args.identity_file)?;
    let node_pk = identity.public_key_hex.clone();
    info!("PeerId: {}", node_pk);

    let peers: PeerMap = Arc::new(RwLock::new(HashMap::new()));

    // Health check endpoint
    let health = warp::path("health")
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok", "version": "0.2.0"})));

    // Peer count endpoint
    let peers_for_status = peers.clone();
    let status = warp::path("status")
        .and(warp::any().map(move || peers_for_status.clone()))
        .and_then(|peers: PeerMap| async move {
            let count = peers.read().await.len();
            let peer_ids: Vec<String> = peers.read().await.keys()
                .map(|k| format!("{}...", &k[..16.min(k.len())]))
                .collect();
            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                "peers": count,
                "peer_ids": peer_ids,
            })))
        });

    // WebSocket endpoint
    let peers_for_ws = peers.clone();
    let node_pk_for_ws = node_pk.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || peers_for_ws.clone()))
        .and(warp::any().map(move || node_pk_for_ws.clone()))
        .map(|ws: warp::ws::Ws, peers: PeerMap, node_pk: String| {
            ws.on_upgrade(move |socket| handle_ws(socket, peers, node_pk))
        });

    let routes = ws_route.or(health).or(status);

    let addr: std::net::SocketAddr = args.listen.parse()
        .unwrap_or_else(|_| "0.0.0.0:4001".parse().unwrap());

    info!("Listening on: {}", addr);
    info!("WebSocket: ws://{}:{}/ws", addr.ip(), addr.port());
    info!("Health: http://{}:{}/health", addr.ip(), addr.port());
    info!("Status: http://{}:{}/status", addr.ip(), addr.port());
    info!("Relay node is running.");

    warp::serve(routes).run(addr).await;

    Ok(())
}
