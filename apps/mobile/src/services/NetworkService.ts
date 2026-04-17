/**
 * NetworkService — P2P networking layer.
 *
 * Drives the Rust libp2p node running on a background thread inside the
 * native library. Consumers call connect/disconnect and register message
 * handlers; the service polls the Rust event queue every 500 ms.
 */

import * as VorynBridge from './VorynBridge';

// Bootstrap peer multiaddrs.
// Update the /p2p/<PeerId> component after re-deploying the bootstrap binary.
const BOOTSTRAP_PEERS: string[] = [
  '/dns4/boot1.voryn.bitstack.website/tcp/4001/p2p/12D3KooWMnagsbtuh6ytx5VWUPDhq9BePidVwmpEU7GG9ZHTnv3X',
];

const POLL_INTERVAL_MS = 500;

type NetworkStatus = 'disconnected' | 'connecting' | 'connected';
type MessageHandler = (fromPeerId: string, dataHex: string) => void;
type PeerHandler = (peerId: string) => void;

let networkStatus: NetworkStatus = 'disconnected';
let localPeerId: string | null = null;
let connectedPeers: Set<string> = new Set();
let messageHandlers: MessageHandler[] = [];
let peerConnectHandlers: PeerHandler[] = [];
let pollTimer: ReturnType<typeof setInterval> | null = null;

// ── Public API ────────────────────────────────────────────────────

export function getStatus(): NetworkStatus {
  return networkStatus;
}

export function getPeerCount(): number {
  return connectedPeers.size;
}

export function getLocalPeerId(): string | null {
  return localPeerId;
}

export function getBootstrapInfo(): { peers: string[]; connected: boolean } {
  return { peers: BOOTSTRAP_PEERS, connected: networkStatus === 'connected' };
}

/** Register a handler for inbound messages. Returns an unsubscribe function. */
export function onMessage(handler: MessageHandler): () => void {
  messageHandlers.push(handler);
  return () => { messageHandlers = messageHandlers.filter((h) => h !== handler); };
}

/** Register a handler for peer connection events. Returns an unsubscribe function. */
export function onPeerConnected(handler: PeerHandler): () => void {
  peerConnectHandlers.push(handler);
  return () => { peerConnectHandlers = peerConnectHandlers.filter((h) => h !== handler); };
}

/**
 * Start the Rust libp2p node and begin polling for events.
 * Safe to call multiple times — no-ops if already connected.
 */
export async function connect(): Promise<void> {
  if (networkStatus !== 'disconnected') return;
  networkStatus = 'connecting';

  try {
    const peerId = await VorynBridge.startNetwork(BOOTSTRAP_PEERS);
    localPeerId = peerId;
    networkStatus = 'connected';
    startPolling();
  } catch (e) {
    networkStatus = 'disconnected';
    throw e;
  }
}

/** Stop the node and cancel the event poll timer. */
export async function disconnect(): Promise<void> {
  stopPolling();
  await VorynBridge.stopNetwork();
  networkStatus = 'disconnected';
  localPeerId = null;
  connectedPeers.clear();
}

/**
 * Send an encrypted message to a peer.
 * `plaintext` is UTF-8; it will be hex-encoded and passed to the Rust node.
 * In a later session the plaintext will be encrypted with the Double Ratchet
 * before being handed off here.
 */
export async function sendToPeer(
  recipientPeerId: string,
  plaintext: string,
): Promise<string> {
  const messageId = generateMessageId();

  // Encode plaintext as hex (TODO: replace with Double Ratchet encryption)
  const encoder = new TextEncoder();
  const bytes = encoder.encode(plaintext);
  const dataHex = Array.from(bytes).map((b) => b.toString(16).padStart(2, '0')).join('');

  await VorynBridge.sendRawToPeer(recipientPeerId, dataHex);

  // Also persist locally via VorynBridge message storage
  await VorynBridge.sendMessage(recipientPeerId, plaintext);

  return messageId;
}

/**
 * Check if a peer is currently connected.
 */
export function isPeerOnline(peerId: string): boolean {
  return connectedPeers.has(peerId);
}

// ── Event polling ─────────────────────────────────────────────────

function startPolling(): void {
  if (pollTimer !== null) return;
  pollTimer = setInterval(drainEventQueue, POLL_INTERVAL_MS);
}

function stopPolling(): void {
  if (pollTimer !== null) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

async function drainEventQueue(): Promise<void> {
  // Drain all queued events in one poll cycle.
  let event = await VorynBridge.pollNetworkEvent();
  while (event !== null) {
    handleEvent(event);
    event = await VorynBridge.pollNetworkEvent();
  }
}

function handleEvent(event: VorynBridge.NetworkEvent): void {
  switch (event.type) {
    case 'started':
      // Node is now listening; update local peer ID if provided
      if (event.peer_id) localPeerId = event.peer_id;
      break;

    case 'discovered':
      // Peer found via mDNS or DHT — may or may not be connected yet
      break;

    case 'connected':
      connectedPeers.add(event.peer_id);
      for (const h of peerConnectHandlers) h(event.peer_id);
      break;

    case 'disconnected':
      connectedPeers.delete(event.peer_id);
      break;

    case 'message':
      if (event.data_hex) {
        for (const h of messageHandlers) h(event.peer_id, event.data_hex);
      }
      break;
  }
}

// ── Helpers ───────────────────────────────────────────────────────

function generateMessageId(): string {
  const bytes = new Uint8Array(16);
  if (typeof globalThis.crypto !== 'undefined' && globalThis.crypto.getRandomValues) {
    globalThis.crypto.getRandomValues(bytes);
  } else {
    for (let i = 0; i < 16; i++) bytes[i] = Math.floor(Math.random() * 256);
  }
  const hex = Array.from(bytes).map((b) => b.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}
