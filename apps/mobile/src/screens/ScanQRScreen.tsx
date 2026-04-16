import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Alert,
  TextInput,
  NativeModules,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';

const { QRScanner } = NativeModules;

type Nav = NativeStackNavigationProp<RootStackParamList, 'ScanQR'>;

export const ScanQRScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [pastedKey, setPastedKey] = useState('');
  const [isAdding, setIsAdding] = useState(false);

  const processVorynUri = async (uri: string) => {
    let pubkeyHex = uri.trim();

    if (pubkeyHex.startsWith('voryn://')) {
      pubkeyHex = pubkeyHex.replace('voryn://', '');
    }

    if (!/^[0-9a-fA-F]{64}$/.test(pubkeyHex)) {
      Alert.alert('Invalid Key', 'Not a valid Voryn public key.');
      return;
    }

    const identity = await VorynBridge.loadIdentity();
    if (identity && identity.publicKeyHex === pubkeyHex) {
      Alert.alert("That's You", 'You scanned your own key.');
      return;
    }

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

  const handleScanCamera = async () => {
    if (!QRScanner) {
      Alert.alert('Not Available', 'QR scanner is not available on this device.');
      return;
    }

    try {
      const result = await QRScanner.scan();
      if (result) {
        await processVorynUri(result);
      }
    } catch (e: any) {
      if (e.code === 'CANCELLED') {
        // User cancelled — do nothing
      } else if (e.code === 'CAMERA_DENIED') {
        Alert.alert('Camera Access', 'Please allow camera access in Settings to scan QR codes.');
      } else {
        Alert.alert('Error', e.message || 'Failed to scan QR code.');
      }
    }
  };

  return (
    <View style={styles.container}>
      <TouchableOpacity style={styles.scanButton} onPress={handleScanCamera}>
        <Text style={styles.scanIcon}>📷</Text>
        <Text style={styles.scanText}>Open Camera Scanner</Text>
        <Text style={styles.scanSubtext}>Point at a Voryn QR code</Text>
      </TouchableOpacity>

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
  scanButton: {
    backgroundColor: '#1A3A5C',
    borderRadius: 16,
    padding: 32,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#4A9EFF',
  },
  scanIcon: { fontSize: 48, marginBottom: 12 },
  scanText: { fontSize: 18, fontWeight: '600', color: '#FFFFFF' },
  scanSubtext: { fontSize: 13, color: '#888888', marginTop: 4 },
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
