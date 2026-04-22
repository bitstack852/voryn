import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  ActivityIndicator,
  StatusBar,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';
import { Logo } from '../components/Logo';
import { colors } from '../theme/colors';

type Nav = NativeStackNavigationProp<RootStackParamList, 'Onboarding'>;

export const OnboardingScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [isLoading, setIsLoading] = useState(true);
  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    const checkExisting = async () => {
      const existing = await VorynBridge.loadIdentity();
      if (existing) {
        navigation.replace('Contacts');
        return;
      }
      setIsLoading(false);
    };
    checkExisting();
  }, [navigation]);

  const handleCreateIdentity = async () => {
    setIsCreating(true);
    try {
      await VorynBridge.generateIdentity();
      navigation.replace('PasscodeSetup');
    } catch (err) {
      console.error('Failed to create identity:', err);
      setIsCreating(false);
    }
  };

  if (isLoading) {
    return (
      <View style={styles.container}>
        <StatusBar barStyle="light-content" backgroundColor="#0D0D0D" />
        <ActivityIndicator size="large" color="#FFFFFF" />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor="#0D0D0D" />

      <View style={styles.header}>
        <Logo size={86} />
        <Text style={styles.wordmark}>VORYN</Text>
        <Text style={styles.tagline}>PEER · MESH · UNCENSORABLE</Text>
      </View>

      <View style={styles.buttonContainer}>
        <TouchableOpacity
          style={styles.button}
          onPress={handleCreateIdentity}
          disabled={isCreating}
          activeOpacity={0.8}
        >
          {isCreating ? (
            <ActivityIndicator color="#0D0D0D" />
          ) : (
            <Text style={styles.buttonText}>Create Identity</Text>
          )}
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.button, styles.secondaryButton]}
          activeOpacity={0.8}
        >
          <Text style={[styles.buttonText, styles.secondaryButtonText]}>
            I Have an Invite
          </Text>
        </TouchableOpacity>

        <Text style={styles.versionText}>v0.1.0</Text>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0D0D0D',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 24,
  },
  header: { alignItems: 'center', marginTop: 80 },
  wordmark: { fontSize: 36, fontWeight: '700', letterSpacing: 6, color: colors.textPrimary, marginTop: 18 },
  tagline: { fontSize: 11, letterSpacing: 3, color: colors.accent, marginTop: 10, fontFamily: 'Menlo' },
  buttonContainer: { width: '100%', alignItems: 'center' },
  button: {
    backgroundColor: '#FFFFFF',
    paddingVertical: 16,
    paddingHorizontal: 48,
    borderRadius: 12,
    marginTop: 16,
    width: '100%',
    alignItems: 'center',
  },
  buttonText: { fontSize: 16, fontWeight: '600', color: '#0D0D0D' },
  secondaryButton: {
    backgroundColor: 'transparent',
    borderWidth: 1,
    borderColor: '#2A2A2A',
  },
  secondaryButtonText: { color: '#666666' },
  versionText: { fontSize: 11, color: '#333333', marginTop: 32 },
});
