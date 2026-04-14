import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  TouchableOpacity,
  Alert,
} from 'react-native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';

type Props = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'AddContact'>;
};

export const AddContactScreen: React.FC<Props> = ({ navigation }) => {
  const [publicKeyHex, setPublicKeyHex] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [isAdding, setIsAdding] = useState(false);

  const isValidHex = (hex: string): boolean => {
    return /^[0-9a-fA-F]{64}$/.test(hex);
  };

  const handleAdd = async () => {
    const trimmedKey = publicKeyHex.trim();
    if (!isValidHex(trimmedKey)) {
      Alert.alert('Invalid Key', 'Public key must be 64 hex characters (32 bytes).');
      return;
    }

    setIsAdding(true);
    try {
      await VorynBridge.addContact(trimmedKey, displayName.trim() || undefined);
      navigation.goBack();
    } catch (err) {
      Alert.alert('Error', 'Failed to add contact. Please try again.');
    }
    setIsAdding(false);
  };

  return (
    <View style={styles.container}>
      <Text style={styles.label}>Public Key (hex)</Text>
      <TextInput
        style={styles.input}
        value={publicKeyHex}
        onChangeText={setPublicKeyHex}
        placeholder="Paste 64-character hex public key"
        placeholderTextColor="#555555"
        autoCapitalize="none"
        autoCorrect={false}
        maxLength={64}
        fontFamily="monospace"
      />

      <Text style={styles.label}>Display Name (optional)</Text>
      <TextInput
        style={styles.input}
        value={displayName}
        onChangeText={setDisplayName}
        placeholder="e.g. Alice"
        placeholderTextColor="#555555"
        maxLength={50}
      />

      <TouchableOpacity
        style={[styles.button, !isValidHex(publicKeyHex.trim()) && styles.buttonDisabled]}
        onPress={handleAdd}
        disabled={isAdding || !isValidHex(publicKeyHex.trim())}
      >
        <Text style={styles.buttonText}>
          {isAdding ? 'Adding...' : 'Add Contact'}
        </Text>
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0D0D0D',
    padding: 20,
  },
  label: {
    fontSize: 14,
    color: '#888888',
    marginTop: 20,
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#1A1A1A',
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 12,
    color: '#FFFFFF',
    fontSize: 14,
    borderWidth: 1,
    borderColor: '#333333',
  },
  button: {
    backgroundColor: '#FFFFFF',
    paddingVertical: 16,
    borderRadius: 8,
    marginTop: 32,
    alignItems: 'center',
  },
  buttonDisabled: {
    opacity: 0.3,
  },
  buttonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#0D0D0D',
  },
});
