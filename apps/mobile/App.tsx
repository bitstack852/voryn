import React, { useEffect } from 'react';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { RootNavigator } from './src/navigation/RootNavigator';
import * as NetworkService from './src/services/NetworkService';
import * as VorynBridge from './src/services/VorynBridge';

const App: React.FC = () => {
  useEffect(() => {
    const unsubscribe = NetworkService.onMessage(
      async (from: string, dataHex: string, messageId: string) => {
        console.log('[App] Received message from', from.slice(0, 16));
        // Decode hex-encoded UTF-8 payload to plaintext
        const bytes = new Uint8Array(dataHex.length / 2);
        for (let i = 0; i < dataHex.length; i += 2) {
          bytes[i / 2] = parseInt(dataHex.slice(i, i + 2), 16);
        }
        const plaintext = new TextDecoder().decode(bytes);
        await VorynBridge.receiveMessage(from, plaintext, messageId);
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
