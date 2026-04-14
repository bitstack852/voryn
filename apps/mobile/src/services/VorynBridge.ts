/**
 * VorynBridge — TypeScript wrapper around the UniFFI-generated Rust bindings.
 *
 * This module provides a clean API for the React Native layer to call into
 * the Rust core. The actual UniFFI bindings will be generated in Phase 0.3.
 */

import type { Identity } from '@voryn/shared-types';

// Placeholder: will be replaced with actual UniFFI import
// import { VorynCore } from 'voryn-core';

/**
 * Test the FFI bridge by calling hello_from_rust().
 */
export async function helloFromRust(): Promise<string> {
  // TODO: Replace with actual UniFFI call
  return 'Voryn Core v0.1.0 — Private. Encrypted. Unreachable. (mock)';
}

/**
 * Generate a new cryptographic identity.
 */
export async function generateIdentity(): Promise<Identity> {
  // TODO Phase 1: Call voryn-core generate_identity() via UniFFI
  throw new Error('Not implemented — requires UniFFI bridge');
}

/**
 * Start the P2P network node.
 */
export async function startNetwork(
  _bootstrapPeers: string[],
): Promise<void> {
  // TODO Phase 1: Call voryn-core start_network() via UniFFI
  throw new Error('Not implemented — requires UniFFI bridge');
}

/**
 * Stop the P2P network node.
 */
export async function stopNetwork(): Promise<void> {
  // TODO Phase 1: Call voryn-core stop_network() via UniFFI
  throw new Error('Not implemented — requires UniFFI bridge');
}

/**
 * Send an encrypted message to a peer.
 */
export async function sendMessage(
  _recipientPubkeyHex: string,
  _plaintext: string,
): Promise<string> {
  // TODO Phase 1: Encrypt + send via Rust bridge, return message_id
  throw new Error('Not implemented — requires UniFFI bridge');
}
