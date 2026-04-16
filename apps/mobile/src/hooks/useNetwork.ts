import { useCallback, useEffect, useRef, useState } from 'react';
import * as NetworkService from '../services/NetworkService';

type NetworkStatus = 'connecting' | 'connected' | 'disconnected';

export function useNetwork() {
  const [status, setStatus] = useState<NetworkStatus>(
    NetworkService.getStatus() as NetworkStatus,
  );
  const [peerCount, setPeerCount] = useState(NetworkService.getPeerCount());
  const [localPeerId, setLocalPeerId] = useState<string | null>(
    NetworkService.getLocalPeerId(),
  );

  // Keep local state in sync with the service on a short interval.
  const syncRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    syncRef.current = setInterval(() => {
      setStatus(NetworkService.getStatus() as NetworkStatus);
      setPeerCount(NetworkService.getPeerCount());
      setLocalPeerId(NetworkService.getLocalPeerId());
    }, 1000);

    return () => {
      if (syncRef.current !== null) clearInterval(syncRef.current);
    };
  }, []);

  const connect = useCallback(async () => {
    setStatus('connecting');
    try {
      await NetworkService.connect();
      setStatus('connected');
      setLocalPeerId(NetworkService.getLocalPeerId());
    } catch (e) {
      setStatus('disconnected');
      throw e;
    }
  }, []);

  const disconnect = useCallback(async () => {
    await NetworkService.disconnect();
    setStatus('disconnected');
    setPeerCount(0);
    setLocalPeerId(null);
  }, []);

  return {
    status,
    peerCount,
    localPeerId,
    connect,
    disconnect,
    setStatus,
    setPeerCount,
  };
}
