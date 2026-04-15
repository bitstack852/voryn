export type MessageStatus = 'pending' | 'sent' | 'delivered' | 'failed';
export type NetworkStatus = 'connecting' | 'connected' | 'disconnected';
export type AuthState = 'locked' | 'unlocked' | 'duress';
export type GroupRole = 'admin' | 'member';
export type AutoDeleteInterval = '1h' | '24h' | '7d' | '30d' | { custom: number };
