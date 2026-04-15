import { useState } from 'react';

type NetworkStatus = 'connecting' | 'connected' | 'disconnected';

export function useNetwork() {
  const [status, setStatus] = useState<NetworkStatus>('disconnected');
  const [peerCount, setPeerCount] = useState(0);

  const connect = async () => {
    setStatus('connecting');
    setStatus('connected');
  };

  const disconnect = async () => {
    setStatus('disconnected');
    setPeerCount(0);
  };

  return { status, peerCount, connect, disconnect, setStatus, setPeerCount };
}
