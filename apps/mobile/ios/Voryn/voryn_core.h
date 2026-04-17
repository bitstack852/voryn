#ifndef VORYN_CORE_H
#define VORYN_CORE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ── Crypto ─────────────────────────────────────────────────────────

const char* voryn_hello(void);
const char* voryn_generate_identity(void);
const char* voryn_compute_safety_number(const uint8_t* our_pk, const uint8_t* their_pk);

// ── Network ────────────────────────────────────────────────────────

// Start the P2P node. config_json: {"keypair_seed_hex":"...","bootstrap_peers":[...],"listen_port":0,"enable_mdns":true}
// Returns: {"ok":true,"peer_id":"..."} or {"ok":false,"error":"..."}
const char* voryn_start_node(const char* config_json);

// Stop the running node.
// Returns: {"ok":true} or {"ok":false,"error":"..."}
const char* voryn_stop_node(void);

// Send encrypted bytes to a peer by PeerId string.
// Returns: {"ok":true} or {"ok":false,"error":"..."}
const char* voryn_send_message(const char* peer_id, const uint8_t* data, size_t data_len);

// Poll for the next network event (non-blocking).
// Returns a JSON event string, or NULL if the queue is empty.
// Event types: "started", "discovered", "connected", "disconnected", "message"
const char* voryn_poll_event(void);

// Returns: {"running":true,"peer_id":"..."} or {"running":false}
const char* voryn_node_status(void);

// ── Encryption ─────────────────────────────────────────────────────

// Encrypt plaintext (UTF-8) for a peer.
// Returns: {"ok":true,"envelope_hex":"..."} or {"ok":false,"error":"..."}
const char* voryn_encrypt_message(const char* plaintext, const char* our_secret_key_hex, const char* our_public_key_hex, const char* their_public_key_hex);

// Decrypt an envelope received from a peer.
// Returns: {"ok":true,"plaintext":"...","sender_pk":"<hex>"} or {"ok":false,"error":"..."}
const char* voryn_decrypt_message(const char* envelope_hex, const char* our_secret_key_hex);

// Convert a 32-byte Ed25519 public key (hex) to a libp2p PeerId string.
// Returns the PeerId string, or empty string on error.
const char* voryn_peer_id_from_public_key(const char* public_key_hex);

// ── Memory ─────────────────────────────────────────────────────────

// Free any string returned by a voryn_ function.
void voryn_free_string(const char* s);

#ifdef __cplusplus
}
#endif

#endif
