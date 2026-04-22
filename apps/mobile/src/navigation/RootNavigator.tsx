import React, { useState, useEffect, useCallback } from 'react';
import { Linking, View } from 'react-native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { NavigationContainer, createNavigationContainerRef } from '@react-navigation/native';
import { OnboardingScreen } from '../screens/OnboardingScreen';
import { ChatsScreen } from '../screens/ChatsScreen';
import { ContactsScreen } from '../screens/ContactsScreen';
import { ChatScreen } from '../screens/ChatScreen';
import { SettingsScreen } from '../screens/SettingsScreen';
import { AddContactScreen } from '../screens/AddContactScreen';
import { ShareKeyScreen } from '../screens/ShareKeyScreen';
import { ContactDetailScreen } from '../screens/ContactDetailScreen';
import { DebugScreen } from '../screens/DebugScreen';
import { ScanQRScreen } from '../screens/ScanQRScreen';
import { PasscodeLockScreen } from '../screens/PasscodeLockScreen';
import { PasscodeSetupPrompt } from '../screens/PasscodeSetupPrompt';
import { PendingRequestsScreen } from '../screens/PendingRequestsScreen';
import { InviteAcceptScreen } from '../screens/InviteAcceptScreen';
import { SplashScreen } from '../screens/SplashScreen';
import * as PasscodeService from '../services/PasscodeService';
import * as VorynBridge from '../services/VorynBridge';
import { colors } from '../theme/colors';

export type RootStackParamList = {
  Onboarding: undefined;
  PasscodeSetup: undefined;
  PasscodeLock: undefined;
  MainTabs: undefined;
  Chat: { contactPubkeyHex: string; displayName?: string };
  Settings: undefined;
  AddContact: undefined;
  ShareKey: undefined;
  ContactDetail: { contactPubkeyHex: string };
  ScanQR: undefined;
  Debug: undefined;
  PendingRequests: undefined;
  InviteAccept: { fromPubkeyHex: string; token: string };
  // Tab screen names (for type safety in tab navigator)
  Chats: undefined;
  Contacts: undefined;
};

export const navigationRef = createNavigationContainerRef<RootStackParamList>();

const Stack = createNativeStackNavigator<RootStackParamList>();
const Tab = createBottomTabNavigator<RootStackParamList>();

const stackScreenOptions = {
  headerStyle: { backgroundColor: colors.background },
  headerTintColor: colors.textPrimary,
  headerTitleStyle: { fontWeight: '600' as const },
  contentStyle: { backgroundColor: colors.background },
};

// ── Tab bar icons (text-based, no icon library needed) ────────────

const ChatTabIcon: React.FC<{ focused: boolean; unread: number }> = ({ focused, unread }) => (
  <View style={{ alignItems: 'center' }}>
    <View style={{ position: 'relative' }}>
      <TabIcon label="💬" focused={focused} />
      {unread > 0 && (
        <View style={tabStyles.badge}>
          {/* badge dot only — count shown in label */}
        </View>
      )}
    </View>
  </View>
);

const TabIcon: React.FC<{ label: string; focused: boolean }> = ({ label, focused }) => (
  <View style={{ opacity: focused ? 1 : 0.5 }}>
    {/* placeholder — icons via emoji */}
  </View>
);

// ── Main tabs ─────────────────────────────────────────────────────

const MainTabs: React.FC = () => {
  const [pendingCount, setPendingCount] = useState(0);
  const [totalUnread, setTotalUnread] = useState(0);

  const refreshBadges = useCallback(async () => {
    const pending = await VorynBridge.getPendingReceivedContacts();
    setPendingCount(pending.length);
    const convs = await VorynBridge.getConversations();
    setTotalUnread(convs.reduce((sum, c) => sum + c.unreadCount, 0));
  }, []);

  useEffect(() => {
    refreshBadges();
    const interval = setInterval(refreshBadges, 3000);
    return () => clearInterval(interval);
  }, [refreshBadges]);

  return (
    <Tab.Navigator
      screenOptions={{
        headerStyle: { backgroundColor: colors.background },
        headerTintColor: colors.textPrimary,
        headerTitleStyle: { fontWeight: '600' },
        tabBarStyle: {
          backgroundColor: colors.background,
          borderTopColor: colors.border,
          borderTopWidth: 1,
        },
        tabBarActiveTintColor: colors.accent,
        tabBarInactiveTintColor: colors.textMuted,
      }}
    >
      <Tab.Screen
        name="Chats"
        component={ChatsScreen}
        options={{
          title: 'Chats',
          tabBarBadge: totalUnread > 0 ? totalUnread : undefined,
        }}
      />
      <Tab.Screen
        name="Contacts"
        component={ContactsScreen}
        options={{
          title: 'Contacts',
          tabBarBadge: pendingCount > 0 ? pendingCount : undefined,
          headerRight: () => null,
        }}
      />
    </Tab.Navigator>
  );
};

// ── Root navigator ────────────────────────────────────────────────

export const RootNavigator: React.FC = () => {
  const [showSplash, setShowSplash] = useState(true);
  const [isReady, setIsReady] = useState(false);
  const [hasPasscode, setHasPasscode] = useState(false);
  const [isLocked, setIsLocked] = useState(true);
  const [pendingDeepLink, setPendingDeepLink] = useState<string | null>(null);

  useEffect(() => {
    const init = async () => {
      const passcodeSet = await PasscodeService.hasPasscode();
      setHasPasscode(passcodeSet);
      setIsLocked(passcodeSet);
      setIsReady(true);
    };
    init();

    // Capture deep links before navigator is ready
    Linking.getInitialURL().then((url) => {
      if (url) setPendingDeepLink(url);
    });

    const sub = Linking.addEventListener('url', (event) => {
      handleDeepLink(event.url);
    });

    return () => sub.remove();
  }, []);

  const handleDeepLink = (url: string) => {
    if (!url) return;
    if (url.startsWith('voryn://invite')) {
      try {
        const parsed = new URL(url.replace('voryn://', 'https://voryn.app/'));
        const fromPubkey = parsed.searchParams.get('from') ?? '';
        const token = parsed.searchParams.get('t') ?? '';
        if (/^[0-9a-fA-F]{64}$/.test(fromPubkey) && navigationRef.isReady()) {
          navigationRef.navigate('InviteAccept', { fromPubkeyHex: fromPubkey, token });
        }
      } catch { /* malformed URL */ }
    }
  };

  const onNavigatorReady = () => {
    if (pendingDeepLink) {
      handleDeepLink(pendingDeepLink);
      setPendingDeepLink(null);
    }
  };

  if (showSplash) {
    return <SplashScreen onFinish={() => setShowSplash(false)} />;
  }

  if (!isReady) return null;

  return (
    <NavigationContainer ref={navigationRef} onReady={onNavigatorReady}>
      <Stack.Navigator
        initialRouteName={hasPasscode && isLocked ? 'PasscodeLock' : 'Onboarding'}
        screenOptions={stackScreenOptions}
      >
        <Stack.Screen name="PasscodeLock" component={PasscodeLockScreen} options={{ headerShown: false }} />
        <Stack.Screen name="PasscodeSetup" component={PasscodeSetupPrompt} options={{ headerShown: false }} />
        <Stack.Screen name="Onboarding" component={OnboardingScreen} options={{ headerShown: false }} />
        <Stack.Screen name="MainTabs" component={MainTabs} options={{ headerShown: false }} />
        <Stack.Screen
          name="Chat"
          component={ChatScreen}
          options={({ route }) => ({ title: route.params.displayName ?? 'Chat' })}
        />
        <Stack.Screen name="Settings" component={SettingsScreen} options={{ title: 'Settings' }} />
        <Stack.Screen name="AddContact" component={AddContactScreen} options={{ title: 'Add Contact' }} />
        <Stack.Screen name="ShareKey" component={ShareKeyScreen} options={{ title: 'My Key' }} />
        <Stack.Screen name="ContactDetail" component={ContactDetailScreen} options={{ title: 'Contact' }} />
        <Stack.Screen name="ScanQR" component={ScanQRScreen} options={{ title: 'Scan QR Code' }} />
        <Stack.Screen name="Debug" component={DebugScreen} options={{ title: 'Debug' }} />
        <Stack.Screen name="PendingRequests" component={PendingRequestsScreen} options={{ title: 'Contact Requests' }} />
        <Stack.Screen name="InviteAccept" component={InviteAcceptScreen} options={{ title: 'Invitation', headerShown: false }} />
      </Stack.Navigator>
    </NavigationContainer>
  );
};

const tabStyles = {
  badge: {
    position: 'absolute' as const,
    top: -2, right: -6,
    width: 8, height: 8,
    borderRadius: 4,
    backgroundColor: colors.accent,
  },
};
