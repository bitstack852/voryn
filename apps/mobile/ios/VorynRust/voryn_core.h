#ifndef VORYN_CORE_H
#define VORYN_CORE_H

#include <stdint.h>
#include <stddef.h>

// ── Crypto ─────────────────────────────────────────────────────────

const char* voryn_hello(void);
const char* voryn_generate_identity(void);
const char* voryn_compute_safety_number(const uint8_t* our_pk, const uint8_t* their_pk);

// ── Network ────────────────────────────────────────────────────────

const char* voryn_start_node(const char* config_json);
const char* voryn_stop_node(void);
const char* voryn_send_message(const char* peer_id, const uint8_t* data, size_t data_len);
const char* voryn_poll_event(void);
const char* voryn_node_status(void);

// ── Memory ─────────────────────────────────────────────────────────

void voryn_free_string(const char* s);

#endif
