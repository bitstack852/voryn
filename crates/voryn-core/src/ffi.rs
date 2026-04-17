//! C FFI exports — callable from Swift/Objective-C via the static library.
//!
//! All functions return heap-allocated C strings; the caller must free them
//! with voryn_free_string(). Network functions use a global Tokio runtime
//! and a process-wide node handle stored in ACTIVE_NODE.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

use voryn_network::{NodeConfig, NodeEvent, NodeHandle};

use crate::bridge;

// ── Global runtime + node state ───────────────────────────────────

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("voryn-rt")
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

struct ActiveNode {
    handle: NodeHandle,
}

static ACTIVE_NODE: OnceLock<Mutex<Option<ActiveNode>>> = OnceLock::new();

fn node_store() -> &'static Mutex<Option<ActiveNode>> {
    ACTIVE_NODE.get_or_init(|| Mutex::new(None))
}

// ── Crypto functions ──────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn voryn_hello() -> *const c_char {
    let s = bridge::hello_from_rust();
    to_c_string(s)
}

/// Generate a new identity.
/// Returns JSON: `{"public_key_hex":"...","secret_key_seed_hex":"..."}`
#[no_mangle]
pub extern "C" fn voryn_generate_identity() -> *const c_char {
    let identity = bridge::generate_identity();
    let json = format!(
        r#"{{"public_key_hex":"{}","secret_key_seed_hex":"{}"}}"#,
        identity.public_key_hex,
        hex_encode(&identity.secret_key_seed),
    );
    to_c_string(json)
}

/// Compute a safety number from two 32-byte public keys.
#[no_mangle]
pub unsafe extern "C" fn voryn_compute_safety_number(
    our_pk: *const u8,
    their_pk: *const u8,
) -> *const c_char {
    if our_pk.is_null() || their_pk.is_null() {
        return to_c_string(String::new());
    }
    let our_bytes = unsafe { std::slice::from_raw_parts(our_pk, 32) };
    let their_bytes = unsafe { std::slice::from_raw_parts(their_pk, 32) };
    let sn = bridge::compute_safety_number(our_bytes.to_vec(), their_bytes.to_vec());
    to_c_string(sn)
}

// ── Network functions ─────────────────────────────────────────────

/// Start the P2P node.
///
/// `config_json` must be a UTF-8 JSON string:
/// ```json
/// {
///   "keypair_seed_hex": "abcd...",   // 32-byte Ed25519 seed (hex). Empty = random.
///   "bootstrap_peers": ["/dns4/boot1.voryn.bitstack.website/tcp/4001/p2p/<PeerId>"],
///   "listen_port": 0,                // 0 = OS-assigned
///   "enable_mdns": true
/// }
/// ```
///
/// Returns JSON: `{"ok":true,"peer_id":"..."}` or `{"ok":false,"error":"..."}`.
#[no_mangle]
pub unsafe extern "C" fn voryn_start_node(config_json: *const c_char) -> *const c_char {
    let json_str = match unsafe { CStr::from_ptr(config_json) }.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => return error_json(format!("Invalid UTF-8 in config: {}", e)),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => return error_json(format!("Invalid JSON: {}", e)),
    };

    let keypair_seed_hex = parsed["keypair_seed_hex"].as_str().unwrap_or("").to_string();
    let keypair_bytes = hex_decode(&keypair_seed_hex).unwrap_or_default();

    let bootstrap_peers: Vec<String> = parsed["bootstrap_peers"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let listen_port = parsed["listen_port"].as_u64().unwrap_or(0) as u16;
    let enable_mdns = parsed["enable_mdns"].as_bool().unwrap_or(true);

    let config = NodeConfig { keypair_bytes, bootstrap_peers, listen_port, enable_mdns };

    let result = runtime().block_on(voryn_network::start_node(config));

    match result {
        Ok(handle) => {
            let peer_id = handle.peer_id.clone();
            let mut store = node_store().lock().unwrap();
            *store = Some(ActiveNode { handle });
            to_c_string(format!(r#"{{"ok":true,"peer_id":"{}"}}"#, peer_id))
        }
        Err(e) => error_json(e.to_string()),
    }
}

/// Stop the running P2P node.
/// Returns JSON: `{"ok":true}` or `{"ok":false,"error":"..."}`.
#[no_mangle]
pub extern "C" fn voryn_stop_node() -> *const c_char {
    let mut store = node_store().lock().unwrap();
    if let Some(node) = store.take() {
        node.handle.shutdown();
        to_c_string(r#"{"ok":true}"#.into())
    } else {
        error_json("No node running".into())
    }
}

/// Send an encrypted message to a peer.
///
/// `peer_id_cstr` — libp2p PeerId string (base58/base58btc multihash).
/// `data` — raw bytes pointer.
/// `data_len` — length of the data buffer.
///
/// Returns JSON: `{"ok":true}` or `{"ok":false,"error":"..."}`.
#[no_mangle]
pub unsafe extern "C" fn voryn_send_message(
    peer_id_cstr: *const c_char,
    data: *const u8,
    data_len: usize,
) -> *const c_char {
    let peer_id = match unsafe { CStr::from_ptr(peer_id_cstr) }.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => return error_json(format!("Invalid peer_id UTF-8: {}", e)),
    };

    if data.is_null() || data_len == 0 {
        return error_json("Empty data".into());
    }

    let bytes = unsafe { std::slice::from_raw_parts(data, data_len) }.to_vec();

    let store = node_store().lock().unwrap();
    match store.as_ref() {
        Some(node) => match node.handle.send_message_cmd(peer_id, bytes) {
            Ok(()) => to_c_string(r#"{"ok":true}"#.into()),
            Err(e) => error_json(e.to_string()),
        },
        None => error_json("Node not started".into()),
    }
}

/// Poll for the next queued network event (non-blocking).
///
/// Returns a JSON string describing the event, or a null pointer if the
/// queue is empty.  The caller must free the returned string.
///
/// Event shapes:
/// ```json
/// {"type":"started",      "peer_id":"...", "addrs":["..."]}
/// {"type":"discovered",   "peer_id":"..."}
/// {"type":"connected",    "peer_id":"..."}
/// {"type":"disconnected", "peer_id":"..."}
/// {"type":"message",      "peer_id":"...", "data_hex":"..."}
/// ```
#[no_mangle]
pub extern "C" fn voryn_poll_event() -> *const c_char {
    let store = node_store().lock().unwrap();
    let event = store.as_ref().and_then(|n| n.handle.poll_event());
    drop(store);

    match event {
        None => std::ptr::null(),
        Some(e) => to_c_string(serialize_event(e)),
    }
}

/// Returns a JSON string with the current node status.
/// `{"running":true,"peer_id":"..."}` or `{"running":false}`.
#[no_mangle]
pub extern "C" fn voryn_node_status() -> *const c_char {
    let store = node_store().lock().unwrap();
    match store.as_ref() {
        Some(node) => to_c_string(format!(r#"{{"running":true,"peer_id":"{}"}}"#, node.handle.peer_id)),
        None => to_c_string(r#"{"running":false}"#.into()),
    }
}

/// Free a string returned by any voryn_ function.
#[no_mangle]
pub extern "C" fn voryn_free_string(s: *const c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s as *mut c_char)) };
    }
}

// ── Helpers ───────────────────────────────────────────────────────

fn to_c_string(s: String) -> *const c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

fn error_json(msg: String) -> *const c_char {
    to_c_string(format!(r#"{{"ok":false,"error":"{}"}}"#, msg.replace('"', "\\\"")))
}

fn serialize_event(event: NodeEvent) -> String {
    match event {
        NodeEvent::Started { peer_id, listening_on } => {
            let addrs = listening_on
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(",");
            format!(r#"{{"type":"started","peer_id":"{}","addrs":[{}]}}"#, peer_id, addrs)
        }
        NodeEvent::PeerDiscovered { peer_id } => {
            format!(r#"{{"type":"discovered","peer_id":"{}"}}"#, peer_id)
        }
        NodeEvent::PeerConnected { peer_id } => {
            format!(r#"{{"type":"connected","peer_id":"{}"}}"#, peer_id)
        }
        NodeEvent::PeerDisconnected { peer_id } => {
            format!(r#"{{"type":"disconnected","peer_id":"{}"}}"#, peer_id)
        }
        NodeEvent::MessageReceived { peer_id, data } => {
            format!(
                r#"{{"type":"message","peer_id":"{}","data_hex":"{}"}}"#,
                peer_id,
                hex_encode(&data)
            )
        }
        NodeEvent::Error { message } => {
            format!(r#"{{"type":"error","peer_id":"","message":"{}"}}"#, message.replace('"', "\\\""))
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}
