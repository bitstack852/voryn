import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Alert,
  TextInput,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';

type Nav = NativeStackNavigationProp<RootStackParamList, 'ScanQR'>;

// Note: react-native-vision-camera requires camera permission and native setup.
// For now, we provide a paste-from-clipboard alternative alongside camera scanning.
// The camera scanner will be enabled once VisionCamera is configured.

export const ScanQRScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [pastedKey, setPastedKey] = useState('');
  const [isAdding, setIsAdding] = useState(false);

  const processVorynUri = async (uri: string) => {
    // Parse voryn:// URI or raw hex key
    let pubkeyHex = uri.trim();

    if (pubkeyHex.startsWith('voryn://')) {
      pubkeyHex = pubkeyHex.replace('voryn://', '');
    }

    // Validate hex key
    if (!/^[0-9a-fA-F]{64}$/.test(pubkeyHex)) {
      Alert.alert('Invalid Key', 'The scanned code is not a valid Voryn public key.');
      return;
    }

    // Check if it's our own key
    const identity = await VorynBridge.loadIdentity();
    if (identity && identity.publicKeyHex === pubkeyHex) {
      Alert.alert('That\'s You', 'You scanned your own public key.');
      return;
    }

    // Check if already added
    const contacts = await VorynBridge.getContacts();
    if (contacts.some((c) => c.publicKeyHex === pubkeyHex)) {
      Alert.alert('Already Added', 'This contact is already in your list.');
      navigation.goBack();
      return;
    }

    setIsAdding(true);

    Alert.prompt(
      'Add Contact',
      'Enter a name for this contact:',
      async (name) => {
        await VorynBridge.addContact(pubkeyHex, name || undefined);
        setIsAdding(false);
        Alert.alert('Contact Added', `${name || 'Contact'} has been added.`, [
          { text: 'OK', onPress: () => navigation.goBack() },
        ]);
      },
      'plain-text',
      '',
      'default',
    );
  };

  return (
    <View style={styles.container}>
      <View style={styles.cameraPlaceholder}>
        <Text style={styles.cameraIcon}>📷</Text>
        <Text style={styles.cameraText}>Camera QR scanning</Text>
        <Text style={styles.cameraSubtext}>Coming soon — use paste method below</Text>
      </View>

      <View style={styles.divider}>
        <View style={styles.dividerLine} />
        <Text style={styles.dividerText}>OR</Text>
        <View style={styles.dividerLine} />
      </View>

      <Text style={styles.pasteLabel}>Paste a Voryn key or voryn:// link</Text>
      <TextInput
        style={styles.pasteInput}
        value={pastedKey}
        onChangeText={setPastedKey}
        placeholder="Paste key here"
        placeholderTextColor="#555555"
        autoCapitalize="none"
        autoCorrect={false}
        multiline
      />

      <TouchableOpacity
        style={[styles.addButton, !pastedKey.trim() && styles.addButtonDisabled]}
        onPress={() => processVorynUri(pastedKey)}
        disabled={!pastedKey.trim() || isAdding}
      >
        <Text style={styles.addButtonText}>
          {isAdding ? 'Adding...' : 'Add Contact'}
        </Text>
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D', padding: 24 },
  cameraPlaceholder: {
    backgroundColor: '#1A1A1A',
    borderRadius: 16,
    padding: 40,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#333333',
    borderStyle: 'dashed',
  },
  cameraIcon: { fontSize: 48, marginBottom: 12 },
  cameraText: { fontSize: 16, color: '#888888' },
  cameraSubtext: { fontSize: 12, color: '#555555', marginTop: 4 },
  divider: { flexDirection: 'row', alignItems: 'center', marginVertical: 24 },
  dividerLine: { flex: 1, height: 1, backgroundColor: '#333333' },
  dividerText: { color: '#555555', marginHorizontal: 16, fontSize: 13 },
  pasteLabel: { fontSize: 14, color: '#888888', marginBottom: 8 },
  pasteInput: {
    backgroundColor: '#1A1A1A',
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 12,
    color: '#FFFFFF',
    fontSize: 13,
    fontFamily: 'monospace',
    borderWidth: 1,
    borderColor: '#333333',
    minHeight: 80,
    textAlignVertical: 'top',
  },
  addButton: {
    backgroundColor: '#4A9EFF',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginTop: 16,
  },
  addButtonDisabled: { opacity: 0.3 },
  addButtonText: { fontSize: 16, fontWeight: '600', color: '#FFFFFF' },
});
