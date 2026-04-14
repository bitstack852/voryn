/**
 * Voryn Shared Types — TypeScript type definitions shared between
 * the React Native app and any other TypeScript packages.
 */

// ── Identity ──────────────────────────────────────────────────────

export interface Identity {
  publicKey: Uint8Array;
  publicKeyHex: string;
}

// ── Contacts ──────────────────────────────────────────────────────

export interface Contact {
  publicKey: Uint8Array;
  publicKeyHex: string;
  displayName?: string;
  addedAt: string;
  lastSeen?: string;
  isBlocked: boolean;
  isVerified: boolean;
}

// ── Messages ──────────────────────────────────────────────────────

export type MessageStatus = 'pending' | 'sent' | 'delivered' | 'failed';

export interface Message {
  messageId: string;
  conversationId: string;
  senderPubkey: string;
  content: string; // Decrypted plaintext (only in memory)
  timestamp: number;
  status: MessageStatus;
  expiresAt?: string;
}

// ── Groups ────────────────────────────────────────────────────────

export type GroupRole = 'admin' | 'member';

export interface GroupMember {
  publicKeyHex: string;
  displayName?: string;
  role: GroupRole;
  joinedAt: string;
}

export interface Group {
  groupId: string;
  name: string;
  members: GroupMember[];
  createdAt: string;
  createdBy: string;
}

// ── Network ───────────────────────────────────────────────────────

export type NetworkStatus = 'connecting' | 'connected' | 'disconnected';

export interface PeerInfo {
  peerId: string;
  publicKeyHex: string;
  addresses: string[];
  lastSeen: number;
}

// ── Auth ──────────────────────────────────────────────────────────

export type AuthState = 'locked' | 'unlocked' | 'duress';

// ── Auto-Delete ───────────────────────────────────────────────────

export type AutoDeleteInterval =
  | '1h'
  | '24h'
  | '7d'
  | '30d'
  | { custom: number }; // seconds
