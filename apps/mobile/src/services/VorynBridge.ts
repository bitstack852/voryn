/**
 * VorynBridge — TypeScript wrapper around the Rust core.
 * Currently uses mock implementations. Will be replaced with UniFFI bindings.
 */

export interface Identity {
  publicKey: Uint8Array;
  publicKeyHex: string;
}

export type NetworkStatus = 'connecting' | 'connected' | 'disconnected';

export async function helloFromRust(): Promise<string> {
  return 'Voryn Core v0.1.0 — Private. Encrypted. Unreachable. (mock)';
}

export async function generateIdentity(): Promise<Identity> {
  const mockPk = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    mockPk[i] = Math.floor(Math.random() * 256);
  }
  const hex = Array.from(mockPk)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
  return { publicKey: mockPk, publicKeyHex: hex };
}

export async function loadIdentity(): Promise<Identity | null> {
  return null;
}

export async function startNetwork(_bootstrapPeers: string[]): Promise<void> {}

export async function stopNetwork(): Promise<void> {}

export async function getNetworkStatus(): Promise<{
  status: NetworkStatus;
  peerCount: number;
  peerId: string | null;
}> {
  return { status: 'disconnected', peerCount: 0, peerId: null };
}

export async function sendMessage(
  _recipientPubkeyHex: string,
  _plaintext: string,
): Promise<string> {
  return `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export async function getMessages(
  _conversationId: string,
  _limit: number = 50,
  _offset: number = 0,
): Promise<Array<{
  messageId: string;
  senderPubkey: string;
  plaintext: string;
  timestamp: number;
  status: string;
}>> {
  return [];
}

export async function addContact(
  _publicKeyHex: string,
  _displayName?: string,
): Promise<void> {}

export async function getContacts(): Promise<
  Array<{
    publicKeyHex: string;
    displayName: string | null;
    lastSeen: string | null;
    isVerified: boolean;
  }>
> {
  return [];
}
