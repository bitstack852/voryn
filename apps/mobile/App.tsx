import React, { useEffect } from 'react';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { RootNavigator } from './src/navigation/RootNavigator';
import * as NetworkService from './src/services/NetworkService';
import * as VorynBridge from './src/services/VorynBridge';

const App: React.FC = () => {
  useEffect(() => {
    const unsubscribe = NetworkService.onMessage(
      async (from: string, payload: string, messageId: string) => {
        console.log('[App] Received message from', from.slice(0, 16));
        await VorynBridge.receiveMessage(from, payload, messageId);
      },
    );

    NetworkService.connect().catch((e) =>
      console.warn('[App] P2P connect failed:', e),
    );

    return () => {
      unsubscribe();
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
