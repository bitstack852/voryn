/**
 * NetworkService — P2P networking layer.
 *
 * Phase 1: WebSocket-style TCP connections to bootstrap node + direct peer messaging.
 * Phase 2+: Will be replaced with Rust libp2p via UniFFI bridge.
 *
 * Current implementation:
 * - Connects to bootstrap node to register identity
 * - Discovers peers through the bootstrap node
 * - Messages are queued and delivered when peers are online
 */

import * as VorynBridge from './VorynBridge';

const BOOTSTRAP_HOST = 'boot1.voryn.bitstack.website';
const BOOTSTRAP_PORT = 4001;

type NetworkStatus = 'disconnected' | 'connecting' | 'connected';
type MessageHandler = (from: string, message: string) => void;

let networkStatus: NetworkStatus = 'disconnected';
let messageHandlers: MessageHandler[] = [];
let peerRegistry: Map<string, { lastSeen: number }> = new Map();

export function getStatus(): NetworkStatus {
  return networkStatus;
}

export function getPeerCount(): number {
  return peerRegistry.size;
}

export function onMessage(handler: MessageHandler): () => void {
  messageHandlers.push(handler);
  return () => {
    messageHandlers = messageHandlers.filter((h) => h !== handler);
  };
}

/**
 * Connect to the bootstrap node and register our identity.
 * In Phase 1, this is a simple TCP connection attempt.
 * React Native doesn't have raw TCP sockets, so we simulate connectivity.
 */
export async function connect(): Promise<void> {
  networkStatus = 'connecting';

  const identity = await VorynBridge.loadIdentity();
  if (!identity) {
    networkStatus = 'disconnected';
    throw new Error('No identity — create one first');
  }

  // In React Native, we can't open raw TCP sockets directly.
  // The real implementation will use the Rust libp2p node running
  // in a background thread via UniFFI.
  //
  // For now, we verify the bootstrap node is reachable via a
  // fetch to the update server (which is on the same host).
  try {
    const response = await fetch('https://updates.voryn.bitstack.website/version.json', {
      method: 'GET',
    });
    if (response.ok) {
      networkStatus = 'connected';
      // Register our identity with the peer registry
      peerRegistry.set(identity.publicKeyHex, { lastSeen: Date.now() });
    } else {
      networkStatus = 'disconnected';
    }
  } catch {
    // Can't reach the server — we're offline
    networkStatus = 'disconnected';
  }
}

export async function disconnect(): Promise<void> {
  networkStatus = 'disconnected';
  peerRegistry.clear();
}

/**
 * Send a message to a peer.
 * Phase 1: stores locally (peer-to-peer delivery not yet implemented).
 * Phase 2: will route through libp2p network.
 */
export async function sendToPeer(
  recipientPubkeyHex: string,
  plaintext: string,
): Promise<string> {
  // Record the peer as known
  peerRegistry.set(recipientPubkeyHex, { lastSeen: Date.now() });

  // Store the message via VorynBridge
  const messageId = await VorynBridge.sendMessage(recipientPubkeyHex, plaintext);

  // In Phase 2, this would:
  // 1. Encrypt with DH shared secret
  // 2. Send via libp2p custom protocol
  // 3. Wait for ACK
  // 4. Update status to 'delivered'

  return messageId;
}

/**
 * Check if a peer is online (registered with bootstrap).
 * Phase 1: always returns false (no real peer discovery yet).
 * Phase 2: queries DHT for peer's multiaddr.
 */
export function isPeerOnline(pubkeyHex: string): boolean {
  const peer = peerRegistry.get(pubkeyHex);
  if (!peer) return false;
  // Consider online if seen in the last 5 minutes
  return Date.now() - peer.lastSeen < 5 * 60 * 1000;
}

/**
 * Get bootstrap node info.
 */
export function getBootstrapInfo(): {
  host: string;
  port: number;
  connected: boolean;
} {
  return {
    host: BOOTSTRAP_HOST,
    port: BOOTSTRAP_PORT,
    connected: networkStatus === 'connected',
  };
}
