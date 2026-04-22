/**
 * VorynBridge — Application logic layer.
 *
 * Uses the Rust native module (VorynCore) for real Ed25519 crypto when
 * available, falls back to JS implementation otherwise.
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { TurboModuleRegistry } from 'react-native';

const VorynCore = TurboModuleRegistry.get<any>('VorynCore');
const hasRustBridge = VorynCore != null;

// ── Types ─────────────────────────────────────────────────────────

export interface Identity {
  publicKey: Uint8Array;
  publicKeyHex: string;
  secretKeySeedHex: string;
  createdAt: string;
}

export type NetworkStatus = 'connecting' | 'connected' | 'disconnected';

export interface Contact {
  publicKeyHex: string;
  displayName: string | null;
  addedAt: string;
  lastSeen: string | null;
  isVerified: boolean;
  status: 'approved' | 'pending_sent' | 'pending_received' | 'denied';
  introMessage: string | null;
}

export interface StoredMessage {
  messageId: string;
  conversationId: string;
  senderPubkeyHex: string;
  plaintext: string;
  timestamp: number;
  status: 'pending' | 'sent' | 'delivered' | 'failed' | 'read';
  isMine: boolean;
}

export interface Conversation {
  contactPubkeyHex: string;
  displayName: string | null;
  conversationId: string;
  lastMessageText: string;
  lastMessageTimestamp: number;
  unreadCount: number;
}

// ── Storage Keys ──────────────────────────────────────────────────

const STORAGE_KEYS = {
  IDENTITY: '@voryn/identity',
  CONTACTS: '@voryn/contacts',
  MESSAGES: '@voryn/messages',
  MIGRATION_V2: '@voryn/migrated_v2',
  INVITE_TOKENS: '@voryn/invite_tokens',
};

export async function migrateWipePlaintextMessages(): Promise<void> {
  const already = await AsyncStorage.getItem(STORAGE_KEYS.MIGRATION_V2);
  if (already) return;
  await AsyncStorage.removeItem(STORAGE_KEYS.MESSAGES);
  await AsyncStorage.setItem(STORAGE_KEYS.MIGRATION_V2, '1');
}

// ── Crypto Helpers ────────────────────────────────────────────────

function generateRandomBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  if (typeof globalThis.crypto !== 'undefined' && globalThis.crypto.getRandomValues) {
    globalThis.crypto.getRandomValues(bytes);
  } else {
    for (let i = 0; i < length; i++) {
      bytes[i] = Math.floor(Math.random() * 256);
    }
  }
  return bytes;
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

function generateMessageId(): string {
  const bytes = generateRandomBytes(16);
  const hex = bytesToHex(bytes);
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

// ── Identity ──────────────────────────────────────────────────────

export async function helloFromRust(): Promise<string> {
  if (hasRustBridge) {
    try {
      return await VorynCore.hello();
    } catch {
      return 'Voryn Core v0.1.0 — Rust bridge error';
    }
  }
  return 'Voryn Core v0.1.0 — Private. Encrypted. Unreachable. (JS fallback)';
}

export async function generateIdentity(): Promise<Identity> {
  let publicKeyHex: string;
  let secretKeySeedHex: string;

  if (hasRustBridge) {
    try {
      const json = await VorynCore.generateIdentity();
      const data = JSON.parse(json);
      publicKeyHex = data.public_key_hex;
      secretKeySeedHex = data.secret_key_seed_hex;
    } catch {
      publicKeyHex = bytesToHex(generateRandomBytes(32));
      secretKeySeedHex = bytesToHex(generateRandomBytes(32));
    }
  } else {
    publicKeyHex = bytesToHex(generateRandomBytes(32));
    secretKeySeedHex = bytesToHex(generateRandomBytes(32));
  }

  const publicKey = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    publicKey[i] = parseInt(publicKeyHex.slice(i * 2, i * 2 + 2), 16);
  }

  const identity: Identity = {
    publicKey,
    publicKeyHex,
    secretKeySeedHex,
    createdAt: new Date().toISOString(),
  };

  await AsyncStorage.setItem(STORAGE_KEYS.IDENTITY, JSON.stringify({
    publicKeyHex: identity.publicKeyHex,
    secretKeySeedHex: identity.secretKeySeedHex,
    createdAt: identity.createdAt,
    rustGenerated: hasRustBridge,
  }));

  return identity;
}

export async function loadIdentity(): Promise<Identity | null> {
  try {
    const stored = await AsyncStorage.getItem(STORAGE_KEYS.IDENTITY);
    if (!stored) return null;

    const data = JSON.parse(stored);
    const publicKey = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      publicKey[i] = parseInt(data.publicKeyHex.slice(i * 2, i * 2 + 2), 16);
    }

    return {
      publicKey,
      publicKeyHex: data.publicKeyHex,
      secretKeySeedHex: data.secretKeySeedHex,
      createdAt: data.createdAt,
    };
  } catch {
    return null;
  }
}

export async function deleteIdentity(): Promise<void> {
  await AsyncStorage.multiRemove([
    STORAGE_KEYS.IDENTITY,
    STORAGE_KEYS.CONTACTS,
    STORAGE_KEYS.MESSAGES,
    STORAGE_KEYS.INVITE_TOKENS,
  ]);
}

// ── Network (legacy stubs — transport is WebSocket in NetworkService) ──

export async function startNetwork(bootstrapPeers: string[]): Promise<string> {
  const identity = await loadIdentity();
  const keypairSeedHex = identity?.secretKeySeedHex ?? '';

  const configJson = JSON.stringify({
    keypair_seed_hex: keypairSeedHex,
    bootstrap_peers: bootstrapPeers,
    listen_port: 47777,
    enable_mdns: true,
  });

  if (hasRustBridge) {
    try {
      const resultJson: string = await VorynCore.startNode(configJson);
      const result = JSON.parse(resultJson);
      if (!result.ok) throw new Error(result.error ?? 'Unknown error');
      return result.peer_id as string;
    } catch (e) {
      throw new Error(`Failed to start node: ${e}`);
    }
  }
  return identity?.publicKeyHex?.slice(0, 32) ?? 'js-fallback-peer';
}

export async function stopNetwork(): Promise<void> {
  if (hasRustBridge) {
    try { await VorynCore.stopNode(); } catch { /* ignore */ }
  }
}

export async function getNetworkStatus(): Promise<{
  status: NetworkStatus;
  peerCount: number;
  peerId: string | null;
}> {
  if (hasRustBridge) {
    try {
      const json: string = await VorynCore.nodeStatus();
      const s = JSON.parse(json);
      return { status: s.running ? 'connected' : 'disconnected', peerCount: 0, peerId: s.peer_id ?? null };
    } catch { /* fall through */ }
  }
  const identity = await loadIdentity();
  return { status: 'disconnected', peerCount: 0, peerId: identity?.publicKeyHex?.slice(0, 16) ?? null };
}

export async function pollNetworkEvent(): Promise<NetworkEvent | null> {
  if (!hasRustBridge) return null;
  try {
    const result = await VorynCore.pollEvent();
    if (result == null) return null;
    return JSON.parse(result) as NetworkEvent;
  } catch {
    return null;
  }
}

// ── Encryption ────────────────────────────────────────────────────

export async function encryptMessage(
  plaintext: string,
  ourSecretKeyHex: string,
  ourPublicKeyHex: string,
  theirPublicKeyHex: string,
): Promise<{ envelopeHex: string } | null> {
  if (!hasRustBridge) return null;
  try {
    const json = await VorynCore.encryptMessage(plaintext, ourSecretKeyHex, ourPublicKeyHex, theirPublicKeyHex);
    const result = JSON.parse(json);
    if (!result.ok) return null;
    return { envelopeHex: result.envelope_hex };
  } catch {
    return null;
  }
}

export async function decryptMessage(
  envelopeHex: string,
  ourSecretKeyHex: string,
): Promise<{ plaintext: string; senderPk: string } | null> {
  if (!hasRustBridge) return null;
  try {
    const json = await VorynCore.decryptMessage(envelopeHex, ourSecretKeyHex);
    const result = JSON.parse(json);
    if (!result.ok) return null;
    return { plaintext: result.plaintext, senderPk: result.sender_pk };
  } catch {
    return null;
  }
}

export async function peerIdFromPublicKey(publicKeyHex: string): Promise<string | null> {
  if (!hasRustBridge) return null;
  try {
    const result = await VorynCore.peerIdFromPublicKey(publicKeyHex);
    return result || null;
  } catch {
    return null;
  }
}

export async function sendRawToPeer(peerId: string, dataHex: string): Promise<void> {
  if (!hasRustBridge) throw new Error('Native bridge not available');
  const resultJson: string = await VorynCore.sendMessage(peerId, dataHex);
  const result = JSON.parse(resultJson);
  if (!result.ok) throw new Error(result.error ?? 'Send failed');
}

// ── Network event type ────────────────────────────────────────────

export type NetworkEventType = 'started' | 'discovered' | 'connected' | 'disconnected' | 'message' | 'error';

export interface NetworkEvent {
  type: NetworkEventType;
  peer_id: string;
  addrs?: string[];
  data_hex?: string;
  message?: string;
}

// ── Contacts ──────────────────────────────────────────────────────

async function loadContactsFromStorage(): Promise<Contact[]> {
  try {
    const stored = await AsyncStorage.getItem(STORAGE_KEYS.CONTACTS);
    if (!stored) return [];
    const contacts = JSON.parse(stored);
    // Migrate legacy contacts that don't have status field
    return contacts.map((c: any) => ({
      ...c,
      status: c.status ?? 'approved',
      introMessage: c.introMessage ?? null,
    }));
  } catch {
    return [];
  }
}

async function saveContactsToStorage(contacts: Contact[]): Promise<void> {
  await AsyncStorage.setItem(STORAGE_KEYS.CONTACTS, JSON.stringify(contacts));
}

export async function addContact(
  publicKeyHex: string,
  displayName?: string,
  introMessage?: string,
): Promise<void> {
  const contacts = await loadContactsFromStorage();
  if (contacts.some((c) => c.publicKeyHex === publicKeyHex)) return;

  contacts.push({
    publicKeyHex,
    displayName: displayName ?? null,
    addedAt: new Date().toISOString(),
    lastSeen: null,
    isVerified: false,
    status: 'pending_sent',
    introMessage: introMessage ?? null,
  });
  await saveContactsToStorage(contacts);

  await sendContactRequest(publicKeyHex, displayName ?? null, introMessage ?? null);
}

async function sendContactRequest(
  recipientPubkeyHex: string,
  displayName: string | null,
  introMessage: string | null,
): Promise<void> {
  const identity = await loadIdentity();
  if (!identity || !hasRustBridge) return;

  const payload = JSON.stringify({ t: 'creq', name: displayName, intro: introMessage });

  try {
    const encrypted = await encryptMessage(
      payload,
      identity.secretKeySeedHex,
      identity.publicKeyHex,
      recipientPubkeyHex,
    );
    if (!encrypted) return;

    const NetworkService = require('./NetworkService');
    NetworkService.sendToPeer(recipientPubkeyHex, encrypted.envelopeHex, generateMessageId());
  } catch {
    // Will retry on reconnect via flushPendingContactRequests
  }
}

export async function flushPendingContactRequests(): Promise<void> {
  const contacts = await loadContactsFromStorage();
  for (const contact of contacts.filter((c) => c.status === 'pending_sent')) {
    await sendContactRequest(contact.publicKeyHex, contact.displayName, contact.introMessage);
  }
}

export async function receiveContactRequest(
  senderPubkeyHex: string,
  displayName: string | null,
  introMessage: string | null,
): Promise<void> {
  const contacts = await loadContactsFromStorage();
  const existing = contacts.find((c) => c.publicKeyHex === senderPubkeyHex);

  if (existing) {
    if (existing.status === 'approved') return;
    if (existing.status === 'pending_sent') {
      // Mutual request — auto-approve
      existing.status = 'approved';
      await saveContactsToStorage(contacts);
      return;
    }
    if (existing.status === 'pending_received') return;
    // Was denied — allow re-request
    existing.status = 'pending_received';
    existing.introMessage = introMessage;
    if (displayName) existing.displayName = displayName;
    await saveContactsToStorage(contacts);
    return;
  }

  contacts.push({
    publicKeyHex: senderPubkeyHex,
    displayName: displayName ?? null,
    addedAt: new Date().toISOString(),
    lastSeen: null,
    isVerified: false,
    status: 'pending_received',
    introMessage: introMessage ?? null,
  });
  await saveContactsToStorage(contacts);
}

export async function approveContact(pubkeyHex: string): Promise<void> {
  const contacts = await loadContactsFromStorage();
  const idx = contacts.findIndex((c) => c.publicKeyHex === pubkeyHex);
  if (idx !== -1) {
    contacts[idx].status = 'approved';
    await saveContactsToStorage(contacts);
  }
}

export async function denyContact(pubkeyHex: string): Promise<void> {
  const contacts = await loadContactsFromStorage();
  await saveContactsToStorage(contacts.filter((c) => c.publicKeyHex !== pubkeyHex));
}

export async function getContacts(): Promise<Contact[]> {
  return loadContactsFromStorage();
}

export async function getApprovedContacts(): Promise<Contact[]> {
  const all = await loadContactsFromStorage();
  return all.filter((c) => c.status === 'approved');
}

export async function getPendingReceivedContacts(): Promise<Contact[]> {
  const all = await loadContactsFromStorage();
  return all.filter((c) => c.status === 'pending_received');
}

export async function removeContact(publicKeyHex: string): Promise<void> {
  const contacts = await loadContactsFromStorage();
  await saveContactsToStorage(contacts.filter((c) => c.publicKeyHex !== publicKeyHex));
}

// ── Messages ──────────────────────────────────────────────────────

async function loadMessagesFromStorage(): Promise<StoredMessage[]> {
  try {
    const stored = await AsyncStorage.getItem(STORAGE_KEYS.MESSAGES);
    if (!stored) return [];
    return JSON.parse(stored);
  } catch {
    return [];
  }
}

async function saveMessagesToStorage(messages: StoredMessage[]): Promise<void> {
  await AsyncStorage.setItem(STORAGE_KEYS.MESSAGES, JSON.stringify(messages));
}

export async function sendMessage(
  recipientPubkeyHex: string,
  plaintext: string,
): Promise<string> {
  const identity = await loadIdentity();
  if (!identity) throw new Error('No identity — create one first');

  const messageId = generateMessageId();
  const conversationId = [identity.publicKeyHex, recipientPubkeyHex].sort().join(':');

  const message: StoredMessage = {
    messageId,
    conversationId,
    senderPubkeyHex: identity.publicKeyHex,
    plaintext,
    timestamp: Date.now(),
    status: 'pending',
    isMine: true,
  };

  const allMessages = await loadMessagesFromStorage();
  const deduped = allMessages.filter(
    (m) => !(m.conversationId === conversationId && m.plaintext === plaintext && m.status === 'failed'),
  );
  deduped.push(message);
  await saveMessagesToStorage(deduped);

  if (!hasRustBridge) {
    const msgs = await loadMessagesFromStorage();
    const idx = msgs.findIndex((m) => m.messageId === messageId);
    if (idx !== -1) { msgs[idx].status = 'failed'; await saveMessagesToStorage(msgs); }
    return messageId;
  }

  try {
    // Wrap in protocol envelope so receiver can distinguish msg types
    const envelope = JSON.stringify({ t: 'msg', text: plaintext });
    const encrypted = await encryptMessage(
      envelope,
      identity.secretKeySeedHex,
      identity.publicKeyHex,
      recipientPubkeyHex,
    );
    if (!encrypted) throw new Error('Encryption failed');

    const NetworkService = require('./NetworkService');
    const sent = NetworkService.sendToPeer(recipientPubkeyHex, encrypted.envelopeHex, messageId);

    const msgs = await loadMessagesFromStorage();
    const idx = msgs.findIndex((m) => m.messageId === messageId);
    if (idx !== -1) {
      msgs[idx].status = sent ? 'sent' : 'pending';
      await saveMessagesToStorage(msgs);
    }
  } catch {
    const msgs = await loadMessagesFromStorage();
    const idx = msgs.findIndex((m) => m.messageId === messageId);
    if (idx !== -1) { msgs[idx].status = 'failed'; await saveMessagesToStorage(msgs); }
  }

  return messageId;
}

export async function storeIncomingMessage(
  senderPubkeyHex: string,
  text: string,
  messageId: string,
): Promise<void> {
  const identity = await loadIdentity();
  if (!identity) return;

  const allMessages = await loadMessagesFromStorage();
  if (allMessages.some((m) => m.messageId === messageId)) return;

  const conversationId = [identity.publicKeyHex, senderPubkeyHex].sort().join(':');

  allMessages.push({
    messageId,
    conversationId,
    senderPubkeyHex,
    plaintext: text,
    timestamp: Date.now(),
    status: 'delivered',
    isMine: false,
  });
  await saveMessagesToStorage(allMessages);
}

export async function updateMessageStatus(
  messageId: string,
  status: StoredMessage['status'],
): Promise<void> {
  const msgs = await loadMessagesFromStorage();
  const idx = msgs.findIndex((m) => m.messageId === messageId);
  if (idx !== -1) {
    msgs[idx].status = status;
    await saveMessagesToStorage(msgs);
  }
}

export async function deleteMessage(messageId: string): Promise<void> {
  const msgs = await loadMessagesFromStorage();
  await saveMessagesToStorage(msgs.filter((m) => m.messageId !== messageId));
}

export async function getMessages(
  conversationId: string,
  _limit: number = 50,
  _offset: number = 0,
): Promise<StoredMessage[]> {
  const allMessages = await loadMessagesFromStorage();
  return allMessages
    .filter((m) => m.conversationId === conversationId)
    .sort((a, b) => b.timestamp - a.timestamp);
}

export async function getConversationId(contactPubkeyHex: string): Promise<string> {
  const identity = await loadIdentity();
  if (!identity) throw new Error('No identity');
  return [identity.publicKeyHex, contactPubkeyHex].sort().join(':');
}

export async function getConversations(): Promise<Conversation[]> {
  const identity = await loadIdentity();
  if (!identity) return [];

  const allMessages = await loadMessagesFromStorage();
  const contacts = await loadContactsFromStorage();

  const convMap = new Map<string, StoredMessage[]>();
  for (const msg of allMessages) {
    const arr = convMap.get(msg.conversationId) ?? [];
    arr.push(msg);
    convMap.set(msg.conversationId, arr);
  }

  const conversations: Conversation[] = [];
  for (const [convId, msgs] of convMap.entries()) {
    const parts = convId.split(':');
    const contactPubkey = parts.find((p) => p !== identity.publicKeyHex) ?? '';
    const contact = contacts.find((c) => c.publicKeyHex === contactPubkey);

    const sorted = [...msgs].sort((a, b) => b.timestamp - a.timestamp);
    const last = sorted[0];
    const unread = msgs.filter((m) => !m.isMine && m.status !== 'read').length;

    conversations.push({
      contactPubkeyHex: contactPubkey,
      displayName: contact?.displayName ?? null,
      conversationId: convId,
      lastMessageText: last.plaintext,
      lastMessageTimestamp: last.timestamp,
      unreadCount: unread,
    });
  }

  return conversations.sort((a, b) => b.lastMessageTimestamp - a.lastMessageTimestamp);
}

export async function markConversationRead(conversationId: string): Promise<void> {
  const msgs = await loadMessagesFromStorage();
  let changed = false;
  for (const msg of msgs) {
    if (msg.conversationId === conversationId && !msg.isMine && msg.status !== 'read') {
      msg.status = 'read';
      changed = true;
    }
  }
  if (changed) await saveMessagesToStorage(msgs);
}

// ── Invite Links ──────────────────────────────────────────────────

export async function generateInviteLink(): Promise<string> {
  const identity = await loadIdentity();
  if (!identity) throw new Error('No identity');

  const token = bytesToHex(generateRandomBytes(8));

  const stored = await AsyncStorage.getItem(STORAGE_KEYS.INVITE_TOKENS);
  const tokens: string[] = stored ? JSON.parse(stored) : [];
  tokens.push(token);
  await AsyncStorage.setItem(STORAGE_KEYS.INVITE_TOKENS, JSON.stringify(tokens));

  return `voryn://invite?from=${identity.publicKeyHex}&t=${token}`;
}
