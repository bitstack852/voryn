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
      navigation.replace('Contacts');
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

      <View style={styles.logoContainer}>
        <Text style={styles.title}>Voryn</Text>
        <Text style={styles.subtitle}>Private. Encrypted. Unreachable.</Text>
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
  logoContainer: { alignItems: 'center', marginBottom: 80 },
  title: { fontSize: 52, fontWeight: '700', color: '#FFFFFF', letterSpacing: 6 },
  subtitle: {
    fontSize: 13,
    color: '#666666',
    marginTop: 12,
    letterSpacing: 3,
    textTransform: 'uppercase',
  },
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
