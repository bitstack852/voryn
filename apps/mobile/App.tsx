import React, { useEffect } from 'react';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { RootNavigator } from './src/navigation/RootNavigator';
import * as NetworkService from './src/services/NetworkService';

const App: React.FC = () => {
  useEffect(() => {
    // Auto-connect to relay when app starts
    NetworkService.connect().catch((e) => {
      console.log('[App] Initial relay connection failed:', e);
    });

    return () => {
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
