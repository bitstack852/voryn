import React, { useEffect } from 'react';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { RootNavigator } from './src/navigation/RootNavigator';
import * as NetworkService from './src/services/NetworkService';
import * as VorynBridge from './src/services/VorynBridge';

const App: React.FC = () => {
  useEffect(() => {
    // Wipe plaintext messages stored before encryption was wired (one-time migration)
    VorynBridge.migrateWipePlaintextMessages().catch(() => {});

    NetworkService.connect().catch((e) => {
      console.log('[App] Relay connection failed:', e);
    });

    // Retry any unsent contact requests when relay connects
    const unsubStatus = NetworkService.onStatusChange((status) => {
      if (status === 'connected') {
        VorynBridge.flushPendingContactRequests().catch(() => {});
      }
    });

    return () => {
      unsubStatus();
      NetworkService.disconnect();
    };
  }, []);

  return (
    <SafeAreaProvider>
      <RootNavigator />
    </SafeAreaProvider>
  );
};

export default App;
