import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';

type Props = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'Onboarding'>;
};

export const OnboardingScreen: React.FC<Props> = ({ navigation }) => {
  const handleCreateIdentity = () => {
    // TODO Phase 1: Call voryn-core generate_identity() via UniFFI bridge
    navigation.replace('Contacts');
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Voryn</Text>
      <Text style={styles.subtitle}>Private. Encrypted. Unreachable.</Text>

      <View style={styles.spacer} />

      <TouchableOpacity style={styles.button} onPress={handleCreateIdentity}>
        <Text style={styles.buttonText}>Create Identity</Text>
      </TouchableOpacity>

      <TouchableOpacity
        style={[styles.button, styles.secondaryButton]}
        onPress={() => {
          // TODO Phase 5: Invite token redemption
        }}
      >
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
