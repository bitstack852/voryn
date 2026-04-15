import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';

type Nav = NativeStackNavigationProp<RootStackParamList, 'Onboarding'>;

export const OnboardingScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [isLoading, setIsLoading] = useState(true);
  const [isCreating, setIsCreating] = useState(false);
  const [bridgeStatus, setBridgeStatus] = useState<string>('');

  useEffect(() => {
    const checkExisting = async () => {
      try {
        const existing = await VorynBridge.loadIdentity();
        if (existing) {
          navigation.replace('Contacts');
          return;
        }
        const hello = await VorynBridge.helloFromRust();
        setBridgeStatus(hello);
      } catch {
        setBridgeStatus('Bridge not connected');
      }
      setIsLoading(false);
    };
    checkExisting();
  }, [navigation]);

  const handleCreateIdentity = async () => {
    setIsCreating(true);
    try {
      await VorynBridge.generateIdentity();
      navigation.replace('Contacts');
    } catch (err) {
      console.error('Failed to create identity:', err);
    }
    setIsCreating(false);
  };

  if (isLoading) {
    return (
      <View style={styles.container}>
        <ActivityIndicator size="large" color="#FFFFFF" />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Voryn</Text>
      <Text style={styles.subtitle}>Private. Encrypted. Unreachable.</Text>

      {bridgeStatus ? (
        <Text style={styles.bridgeStatus}>{bridgeStatus}</Text>
      ) : null}

      <View style={styles.spacer} />

      <TouchableOpacity
        style={styles.button}
        onPress={handleCreateIdentity}
        disabled={isCreating}
      >
        {isCreating ? (
          <ActivityIndicator color="#0D0D0D" />
        ) : (
          <Text style={styles.buttonText}>Create Identity</Text>
        )}
      </TouchableOpacity>

      <TouchableOpacity style={[styles.button, styles.secondaryButton]}>
        <Text style={[styles.buttonText, styles.secondaryButtonText]}>
          I Have an Invite
        </Text>
      </TouchableOpacity>
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
  title: {
    fontSize: 48,
    fontWeight: '700',
    color: '#FFFFFF',
    letterSpacing: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#888888',
    marginTop: 8,
    letterSpacing: 2,
  },
  bridgeStatus: {
    fontSize: 11,
    color: '#555555',
    marginTop: 16,
    fontFamily: 'monospace',
  },
  spacer: {
    height: 80,
  },
  button: {
    backgroundColor: '#FFFFFF',
    paddingVertical: 16,
    paddingHorizontal: 48,
    borderRadius: 8,
    marginTop: 16,
    width: '100%',
    alignItems: 'center',
  },
  buttonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#0D0D0D',
  },
  secondaryButton: {
    backgroundColor: 'transparent',
    borderWidth: 1,
    borderColor: '#333333',
  },
  secondaryButtonText: {
    color: '#888888',
  },
});
