/**
 * NetworkService — WebSocket connection to the Voryn relay node.
 *
 * Handles:
 * - WebSocket connection to boot1.voryn.bitstack.website
 * - Peer registration (sends our public key)
 * - Message relay (send/receive encrypted messages)
 * - Protocol message routing (contact_request, contact_accepted, contact_denied)
 * - Online peer tracking
 * - Auto-reconnect on disconnect
 */

import * as VorynBridge from './VorynBridge';

const RELAY_WS_URL = 'ws://boot1.voryn.bitstack.website:4001/ws';

type NetworkStatus = 'disconnected' | 'connecting' | 'connected';
type MessageHandler = (fromPeerId: string, text: string, messageId: string) => void;
type PeerHandler = (peerId: string, online?: boolean) => void;
type ErrorHandler = (message: string) => void;
type StatusHandler = (status: NetworkStatus) => void;
type AckHandler = (messageId: string) => void;
type ContactRequestHandler = (fromPubkey: string, data: Record<string, any>) => void;

let ws: WebSocket | null = null;
let networkStatus: NetworkStatus = 'disconnected';
let registeredPeerId: string | null = null;
let onlinePeers: Set<string> = new Set();
let messageHandlers: MessageHandler[] = [];
let peerHandlers: PeerHandler[] = [];
let errorHandlers: ErrorHandler[] = [];
let statusHandlers: StatusHandler[] = [];
let ackHandlers: AckHandler[] = [];
let contactRequestHandlers: ContactRequestHandler[] = [];
let recentErrors: string[] = [];
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let pingTimer: ReturnType<typeof setInterval> | null = null;

function setStatus(status: NetworkStatus) {
  networkStatus = status;
  for (const h of statusHandlers) h(status);
}

function recordError(msg: string) {
  recentErrors = [...recentErrors.slice(-19), msg];
  for (const h of errorHandlers) h(msg);
}

// ── Public API ────────────────────────────────────────────────────

export function getStatus(): NetworkStatus { return networkStatus; }
export function getPeerCount(): number { return onlinePeers.size; }
export function getLocalPeerId(): string | null { return registeredPeerId; }
export function getOnlinePeers(): string[] { return Array.from(onlinePeers); }
export function isPeerOnline(peerId: string): boolean { return onlinePeers.has(peerId); }
export function getRecentErrors(): string[] { return [...recentErrors]; }
export function getBootstrapInfo(): { peers: string[]; connected: boolean } {
  return { peers: [RELAY_WS_URL], connected: networkStatus === 'connected' };
}

export function onMessage(handler: MessageHandler): () => void {
  messageHandlers.push(handler);
  return () => { messageHandlers = messageHandlers.filter((h) => h !== handler); };
}

export function onPeerConnected(handler: PeerHandler): () => void {
  peerHandlers.push(handler);
  return () => { peerHandlers = peerHandlers.filter((h) => h !== handler); };
}

export function onPeerChange(handler: PeerHandler): () => void {
  return onPeerConnected(handler);
}

export function onError(handler: ErrorHandler): () => void {
  errorHandlers.push(handler);
  return () => { errorHandlers = errorHandlers.filter((h) => h !== handler); };
}

export function onStatusChange(handler: StatusHandler): () => void {
  statusHandlers.push(handler);
  return () => { statusHandlers = statusHandlers.filter((h) => h !== handler); };
}

export function onAck(handler: AckHandler): () => void {
  ackHandlers.push(handler);
  return () => { ackHandlers = ackHandlers.filter((h) => h !== handler); };
}

export function onContactRequest(handler: ContactRequestHandler): () => void {
  contactRequestHandlers.push(handler);
  return () => { contactRequestHandlers = contactRequestHandlers.filter((h) => h !== handler); };
}

export async function connect(): Promise<void> {
  if (networkStatus === 'connected' || networkStatus === 'connecting') return;

  const identity = await VorynBridge.loadIdentity();
  if (!identity) throw new Error('No identity — create one first');

  registeredPeerId = identity.publicKeyHex;
  setStatus('connecting');

  return new Promise((resolve, reject) => {
    try {
      ws = new WebSocket(RELAY_WS_URL);

      ws.onopen = () => {
        ws?.send(JSON.stringify({ type: 'register', peer_id: registeredPeerId }));
        setStatus('connected');

        if (pingTimer) clearInterval(pingTimer);
        pingTimer = setInterval(() => {
          if (ws?.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: 'ping' }));
          }
        }, 30000);

        resolve();
      };

      ws.onmessage = (event) => {
        try {
          handleServerMessage(JSON.parse(event.data as string));
        } catch (e) {
          console.warn('[Network] Failed to parse message:', e);
        }
      };

      ws.onclose = () => {
        setStatus('disconnected');
        onlinePeers.clear();
        if (pingTimer) { clearInterval(pingTimer); pingTimer = null; }

        if (reconnectTimer) clearTimeout(reconnectTimer);
        reconnectTimer = setTimeout(() => {
          connect().catch(() => {});
        }, 5000);
      };

      ws.onerror = () => {
        const msg = 'WebSocket connection failed';
        recordError(msg);
        if (networkStatus === 'connecting') {
          setStatus('disconnected');
          reject(new Error(msg));
        }
      };
    } catch (e) {
      setStatus('disconnected');
      reject(e);
    }
  });
}

export function disconnect(): void {
  if (reconnectTimer) { clearTimeout(reconnectTimer); reconnectTimer = null; }
  if (pingTimer) { clearInterval(pingTimer); pingTimer = null; }
  if (ws) { ws.close(); ws = null; }
  setStatus('disconnected');
  onlinePeers.clear();
}

export function sendToPeer(recipientPubkeyHex: string, payload: string, messageId: string): boolean {
  if (!ws || ws.readyState !== WebSocket.OPEN) return false;
  ws.send(JSON.stringify({ type: 'send', to: recipientPubkeyHex, payload, message_id: messageId }));
  return true;
}

// ── Internal ──────────────────────────────────────────────────────

function handleServerMessage(data: any) {
  switch (data.type) {
    case 'server_hello':
      break;

    case 'message':
      if (data.payload) {
        const msgId = data.message_id || generateMessageId();
        processIncoming(data.from, data.payload, msgId).catch(console.warn);
      }
      break;

    case 'ack':
      if (data.message_id) {
        for (const h of ackHandlers) h(data.message_id);
      }
      break;

    case 'peer_online':
      if (data.peer_id && data.peer_id !== registeredPeerId) {
        onlinePeers.add(data.peer_id);
        for (const h of peerHandlers) h(data.peer_id, true);
      }
      break;

    case 'peer_offline':
      if (data.peer_id) {
        onlinePeers.delete(data.peer_id);
        for (const h of peerHandlers) h(data.peer_id, false);
      }
      break;

    case 'pong':
      break;

    case 'error': {
      const msg = data.message ?? 'Unknown relay error';
      recordError(msg);
      break;
    }
  }
}

async function processIncoming(fromPubkey: string, envelopeHex: string, msgId: string): Promise<void> {
  const identity = await VorynBridge.loadIdentity();
  if (!identity) return;

  const decrypted = await VorynBridge.decryptMessage(envelopeHex, identity.secretKeySeedHex);
  if (!decrypted) return;

  let parsed: any;
  try {
    parsed = JSON.parse(decrypted.plaintext);
  } catch {
    // Plain string — treat as chat message (pre-envelope legacy)
    await VorynBridge.storeIncomingMessage(fromPubkey, decrypted.plaintext, msgId);
    for (const h of messageHandlers) h(fromPubkey, decrypted.plaintext, msgId);
    return;
  }

  switch (parsed.t) {
    case 'msg':
      await VorynBridge.storeIncomingMessage(fromPubkey, parsed.text ?? '', msgId);
      for (const h of messageHandlers) h(fromPubkey, parsed.text ?? '', msgId);
      break;

    case 'creq':
      await VorynBridge.receiveContactRequest(fromPubkey, parsed.name ?? null, parsed.intro ?? null);
      for (const h of contactRequestHandlers) h(fromPubkey, parsed);
      break;

    case 'cacc':
      await VorynBridge.approveContact(fromPubkey);
      for (const h of contactRequestHandlers) h(fromPubkey, parsed);
      break;

    case 'cden':
      await VorynBridge.denyContact(fromPubkey);
      for (const h of contactRequestHandlers) h(fromPubkey, parsed);
      break;
  }
}

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
