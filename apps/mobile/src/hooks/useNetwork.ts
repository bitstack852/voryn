import { useState } from 'react';
import type { NetworkStatus } from '@voryn/shared-types';

/**
 * Hook for managing the P2P network connection state.
 * Full implementation in Phase 1 when libp2p node is integrated.
 */
export function useNetwork() {
  const [status, setStatus] = useState<NetworkStatus>('disconnected');
  const [peerCount, setPeerCount] = useState(0);

  const connect = async () => {
    setStatus('connecting');
    // TODO Phase 1: Call VorynBridge.startNetwork()
    setStatus('connected');
  };

  const disconnect = async () => {
    // TODO Phase 1: Call VorynBridge.stopNetwork()
    setStatus('disconnected');
    setPeerCount(0);
  };

  return { status, peerCount, connect, disconnect, setStatus, setPeerCount };
}
