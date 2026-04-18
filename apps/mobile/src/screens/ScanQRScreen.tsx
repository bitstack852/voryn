import React, { useState, useCallback, useRef } from 'react';
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
import { Camera, useCameraDevice, useCodeScanner, useCameraPermission } from 'react-native-vision-camera';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';

type Nav = NativeStackNavigationProp<RootStackParamList, 'ScanQR'>;

export const ScanQRScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [pastedKey, setPastedKey] = useState('');
  const [isAdding, setIsAdding] = useState(false);
  const [cameraOpen, setCameraOpen] = useState(false);
  const scannedRef = useRef(false);

  const { hasPermission, requestPermission } = useCameraPermission();
  const device = useCameraDevice('back');

  const processVorynUri = useCallback(async (uri: string) => {
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
  }, [navigation]);

  const codeScanner = useCodeScanner({
    codeTypes: ['qr'],
    onCodeScanned: (codes) => {
      if (scannedRef.current) return;
      const value = codes[0]?.value;
      if (value) {
        scannedRef.current = true;
        setCameraOpen(false);
        processVorynUri(value);
      }
    },
  });

  const handleOpenCamera = async () => {
    if (!hasPermission) {
      const granted = await requestPermission();
      if (!granted) {
        Alert.alert('Camera Access', 'Please allow camera access in Settings to scan QR codes.');
        return;
      }
    }
    scannedRef.current = false;
    setCameraOpen(true);
  };

  if (cameraOpen && device) {
    return (
      <View style={styles.cameraContainer}>
        <Camera
          style={StyleSheet.absoluteFill}
          device={device}
          isActive={cameraOpen}
          codeScanner={codeScanner}
        />
        <View style={styles.overlay}>
          <View style={styles.scanFrame} />
          <Text style={styles.scanHint}>Align the QR code within the frame</Text>
          <TouchableOpacity style={styles.cancelButton} onPress={() => setCameraOpen(false)}>
            <Text style={styles.cancelText}>Cancel</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <TouchableOpacity style={styles.scanButton} onPress={handleOpenCamera}>
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
  cameraContainer: { flex: 1, backgroundColor: '#000000' },
  overlay: {
    ...StyleSheet.absoluteFillObject,
    alignItems: 'center',
    justifyContent: 'center',
  },
  scanFrame: {
    width: 260,
    height: 260,
    borderWidth: 2,
    borderColor: '#4A9EFF',
    borderRadius: 12,
    backgroundColor: 'transparent',
  },
  scanHint: {
    color: '#FFFFFF',
    fontSize: 14,
    marginTop: 24,
    textAlign: 'center',
    backgroundColor: 'rgba(0,0,0,0.5)',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 8,
  },
  cancelButton: {
    marginTop: 32,
    backgroundColor: 'rgba(0,0,0,0.6)',
    paddingHorizontal: 32,
    paddingVertical: 12,
    borderRadius: 24,
    borderWidth: 1,
    borderColor: '#555555',
  },
  cancelText: { color: '#FFFFFF', fontSize: 16 },
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
