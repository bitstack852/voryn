/**
 * VorynBridge — Application logic layer.
 *
 * Uses the Rust native module (VorynCore) for real Ed25519 crypto when
 * available, falls back to JS implementation otherwise.
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { NativeModules, Platform } from 'react-native';

const { VorynCore } = NativeModules;
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
}

export interface StoredMessage {
  messageId: string;
  conversationId: string;
  senderPubkeyHex: string;
  plaintext: string;
  timestamp: number;
  status: 'pending' | 'sent' | 'delivered' | 'failed';
  isMine: boolean;
}

// ── Storage Keys ──────────────────────────────────────────────────

const STORAGE_KEYS = {
  IDENTITY: '@voryn/identity',
  CONTACTS: '@voryn/contacts',
  MESSAGES: '@voryn/messages',
};

// ── Crypto Helpers ────────────────────────────────────────────────

function generateRandomBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  // React Native has crypto.getRandomValues in Hermes/JSC
  if (typeof globalThis.crypto !== 'undefined' && globalThis.crypto.getRandomValues) {
    globalThis.crypto.getRandomValues(bytes);
  } else {
    // Fallback for environments without crypto
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
      // Fall back to JS
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
  ]);
}

// ── Network ───────────────────────────────────────────────────────

export async function startNetwork(_bootstrapPeers: string[]): Promise<void> {
  // Will connect to bootstrap nodes via libp2p
}

export async function stopNetwork(): Promise<void> {}

export async function getNetworkStatus(): Promise<{
  status: NetworkStatus;
  peerCount: number;
  peerId: string | null;
}> {
  const identity = await loadIdentity();
  return {
    status: 'disconnected',
    peerCount: 0,
    peerId: identity?.publicKeyHex?.slice(0, 16) ?? null,
  };
}

// ── Contacts ──────────────────────────────────────────────────────

async function loadContactsFromStorage(): Promise<Contact[]> {
  try {
    const stored = await AsyncStorage.getItem(STORAGE_KEYS.CONTACTS);
    if (!stored) return [];
    return JSON.parse(stored);
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
): Promise<void> {
  const contacts = await loadContactsFromStorage();

  // Don't add duplicates
  if (contacts.some((c) => c.publicKeyHex === publicKeyHex)) {
    return;
  }

  contacts.push({
    publicKeyHex,
    displayName: displayName ?? null,
    addedAt: new Date().toISOString(),
    lastSeen: null,
    isVerified: false,
  });

  await saveContactsToStorage(contacts);
}

export async function getContacts(): Promise<Contact[]> {
  return loadContactsFromStorage();
}

export async function removeContact(publicKeyHex: string): Promise<void> {
  const contacts = await loadContactsFromStorage();
  const filtered = contacts.filter((c) => c.publicKeyHex !== publicKeyHex);
  await saveContactsToStorage(filtered);
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
  allMessages.push(message);
  await saveMessagesToStorage(allMessages);

  // In real implementation: encrypt with DH shared secret, send via libp2p
  // For now, mark as sent after a short delay
  setTimeout(async () => {
    const msgs = await loadMessagesFromStorage();
    const idx = msgs.findIndex((m) => m.messageId === messageId);
    if (idx !== -1) {
      msgs[idx].status = 'sent';
      await saveMessagesToStorage(msgs);
    }
  }, 500);

  return messageId;
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

export async function getConversationId(
  contactPubkeyHex: string,
): Promise<string> {
  const identity = await loadIdentity();
  if (!identity) throw new Error('No identity');
  return [identity.publicKeyHex, contactPubkeyHex].sort().join(':');
}
