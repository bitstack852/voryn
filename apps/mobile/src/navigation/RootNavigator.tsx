import React, { useState, useEffect } from 'react';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { NavigationContainer } from '@react-navigation/native';
import { OnboardingScreen } from '../screens/OnboardingScreen';
import { ContactsScreen } from '../screens/ContactsScreen';
import { ChatScreen } from '../screens/ChatScreen';
import { SettingsScreen } from '../screens/SettingsScreen';
import { AddContactScreen } from '../screens/AddContactScreen';
import { ShareKeyScreen } from '../screens/ShareKeyScreen';
import { ContactDetailScreen } from '../screens/ContactDetailScreen';
import { DebugScreen } from '../screens/DebugScreen';
import { PasscodeLockScreen } from '../screens/PasscodeLockScreen';
import { PasscodeSetupPrompt } from '../screens/PasscodeSetupPrompt';
import * as PasscodeService from '../services/PasscodeService';

export type RootStackParamList = {
  Onboarding: undefined;
  PasscodeSetup: undefined;
  PasscodeLock: undefined;
  Contacts: undefined;
  Chat: { contactPubkeyHex: string; displayName?: string };
  Settings: undefined;
  AddContact: undefined;
  ShareKey: undefined;
  ContactDetail: { contactPubkeyHex: string };
  Debug: undefined;
};

const Stack = createNativeStackNavigator<RootStackParamList>();

export const RootNavigator: React.FC = () => {
  const [isReady, setIsReady] = useState(false);
  const [hasPasscode, setHasPasscode] = useState(false);
  const [isLocked, setIsLocked] = useState(true);

  useEffect(() => {
    const init = async () => {
      const passcodeSet = await PasscodeService.hasPasscode();
      setHasPasscode(passcodeSet);
      setIsLocked(passcodeSet);
      setIsReady(true);
    };
    init();
  }, []);

  if (!isReady) return null;

  return (
    <NavigationContainer>
      <Stack.Navigator
        initialRouteName={hasPasscode && isLocked ? 'PasscodeLock' : 'Onboarding'}
        screenOptions={{
          headerStyle: { backgroundColor: '#0D0D0D' },
          headerTintColor: '#FFFFFF',
          headerTitleStyle: { fontWeight: '600' },
          contentStyle: { backgroundColor: '#0D0D0D' },
        }}
      >
        <Stack.Screen
          name="PasscodeLock"
          component={PasscodeLockScreen}
          options={{ headerShown: false }}
        />
        <Stack.Screen
          name="PasscodeSetup"
          component={PasscodeSetupPrompt}
          options={{ headerShown: false }}
        />
        <Stack.Screen
          name="Onboarding"
          component={OnboardingScreen}
          options={{ headerShown: false }}
        />
        <Stack.Screen
          name="Contacts"
          component={ContactsScreen}
          options={{ title: 'Voryn' }}
        />
        <Stack.Screen
          name="Chat"
          component={ChatScreen}
          options={({ route }) => ({
            title: route.params.displayName ?? 'Chat',
          })}
        />
        <Stack.Screen name="Settings" component={SettingsScreen} options={{ title: 'Settings' }} />
        <Stack.Screen name="AddContact" component={AddContactScreen} options={{ title: 'Add Contact' }} />
        <Stack.Screen name="ShareKey" component={ShareKeyScreen} options={{ title: 'My Key' }} />
        <Stack.Screen name="ContactDetail" component={ContactDetailScreen} options={{ title: 'Contact' }} />
        <Stack.Screen name="Debug" component={DebugScreen} options={{ title: 'Debug' }} />
      </Stack.Navigator>
    </NavigationContainer>
  );
};
