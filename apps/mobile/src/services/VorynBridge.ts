/**
 * VorynBridge — TypeScript wrapper around the UniFFI-generated Rust bindings.
 *
 * This module provides a clean API for the React Native layer to call into
 * the Rust core. The actual UniFFI bindings will replace the mock implementations
 * once the bridge is fully wired.
 */

import type { Identity, NetworkStatus } from '@voryn/shared-types';

// Placeholder: will be replaced with actual UniFFI import
// import { VorynCore } from 'voryn-core';

// ── Identity ──────────────────────────────────────────────────────

/**
 * Test the FFI bridge by calling hello_from_rust().
 */
export async function helloFromRust(): Promise<string> {
  // TODO: Replace with actual UniFFI call
  return 'Voryn Core v0.1.0 — Private. Encrypted. Unreachable. (mock)';
}

/**
 * Generate a new cryptographic identity.
 * Creates an Ed25519 keypair; secret key is stored in hardware keystore.
 */
export async function generateIdentity(): Promise<Identity> {
  // TODO: Replace with UniFFI call to voryn_core::generate_full_identity()
  // Mock: generate random bytes for development
  const mockPk = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    mockPk[i] = Math.floor(Math.random() * 256);
  }
  const hex = Array.from(mockPk)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
  return { publicKey: mockPk, publicKeyHex: hex };
}

/**
 * Load the existing identity from local storage, if it exists.
 */
export async function loadIdentity(): Promise<Identity | null> {
  // TODO: Replace with UniFFI call to load from SQLCipher
  return null;
}

// ── Network ───────────────────────────────────────────────────────

/**
 * Start the P2P network node.
 */
export async function startNetwork(
  _bootstrapPeers: string[],
): Promise<void> {
  // TODO: Replace with UniFFI call to voryn_network::start_node()
}

/**
 * Stop the P2P network node.
 */
export async function stopNetwork(): Promise<void> {
  // TODO: Replace with UniFFI call
}

/**
 * Get current network status.
 */
export async function getNetworkStatus(): Promise<{
  status: NetworkStatus;
  peerCount: number;
  peerId: string | null;
}> {
  // TODO: Replace with UniFFI call
  return { status: 'disconnected', peerCount: 0, peerId: null };
}

// ── Messaging ─────────────────────────────────────────────────────

/**
 * Send an encrypted message to a peer.
 * Returns the message ID for delivery tracking.
 */
export async function sendMessage(
  recipientPubkeyHex: string,
  plaintext: string,
): Promise<string> {
  // TODO: Replace with UniFFI call to voryn_core::messaging::prepare_message()
  const msgId = `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  console.log(`[mock] Sending message to ${recipientPubkeyHex.slice(0, 16)}...`);
  return msgId;
}

/**
 * Get messages for a conversation.
 */
export async function getMessages(
  conversationId: string,
  limit: number = 50,
  offset: number = 0,
): Promise<
  Array<{
    messageId: string;
    senderPubkey: string;
    plaintext: string;
    timestamp: number;
    status: string;
  }>
> {
  // TODO: Replace with UniFFI call to load from SQLCipher + decrypt
  return [];
}

// ── Contacts ──────────────────────────────────────────────────────

/**
 * Add a contact by their public key hex string.
 */
export async function addContact(
  publicKeyHex: string,
  displayName?: string,
): Promise<void> {
  // TODO: Replace with UniFFI call to store in SQLCipher
  console.log(`[mock] Adding contact ${publicKeyHex.slice(0, 16)}...`);
}

/**
 * Get all contacts.
 */
export async function getContacts(): Promise<
  Array<{
    publicKeyHex: string;
    displayName: string | null;
    lastSeen: string | null;
    isVerified: boolean;
  }>
> {
  // TODO: Replace with UniFFI call
  return [];
}
