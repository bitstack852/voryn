import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';
import { colors } from '../theme/colors';

type Nav = NativeStackNavigationProp<RootStackParamList, 'AddContact'>;

export const AddContactScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [publicKeyHex, setPublicKeyHex] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [introMessage, setIntroMessage] = useState('');
  const [isSending, setIsSending] = useState(false);

  const isValidHex = (hex: string): boolean => /^[0-9a-fA-F]{64}$/.test(hex);

  const handleSend = async () => {
    const trimmedKey = publicKeyHex.trim();
    if (!isValidHex(trimmedKey)) {
      Alert.alert('Invalid Key', 'Public key must be 64 hex characters (32 bytes).');
      return;
    }

    const identity = await VorynBridge.loadIdentity();
    if (identity?.publicKeyHex === trimmedKey) {
      Alert.alert("That's You", "You can't add yourself as a contact.");
      return;
    }

    const existing = await VorynBridge.getContacts();
    if (existing.some((c) => c.publicKeyHex === trimmedKey && c.status === 'approved')) {
      Alert.alert('Already a Contact', 'This person is already in your contacts.');
      return;
    }

    setIsSending(true);
    try {
      await VorynBridge.addContact(trimmedKey, displayName.trim() || undefined, introMessage.trim() || undefined);
      Alert.alert(
        'Request Sent',
        'Your contact request has been sent. You\'ll be notified when they respond.',
        [{ text: 'OK', onPress: () => navigation.goBack() }],
      );
    } catch {
      Alert.alert('Error', 'Failed to send request. Please try again.');
    }
    setIsSending(false);
  };

  return (
    <View style={styles.container}>
      <Text style={styles.label}>Their Public Key</Text>
      <TextInput
        style={styles.input}
        value={publicKeyHex}
        onChangeText={setPublicKeyHex}
        placeholder="Paste 64-character hex key"
        placeholderTextColor={colors.textMuted}
        autoCapitalize="none"
        autoCorrect={false}
        maxLength={64}
      />

      <Text style={styles.label}>Your Name (shown to them)</Text>
      <TextInput
        style={styles.input}
        value={displayName}
        onChangeText={setDisplayName}
        placeholder="Optional"
        placeholderTextColor={colors.textMuted}
        maxLength={50}
      />

      <Text style={styles.label}>Intro Message</Text>
      <TextInput
        style={[styles.input, styles.textArea]}
        value={introMessage}
        onChangeText={setIntroMessage}
        placeholder="Optional — shown with your request"
        placeholderTextColor={colors.textMuted}
        maxLength={200}
        multiline
      />
      <Text style={styles.charCount}>{introMessage.length}/200</Text>

      <TouchableOpacity
        style={[styles.button, !isValidHex(publicKeyHex.trim()) && styles.buttonDisabled]}
        onPress={handleSend}
        disabled={isSending || !isValidHex(publicKeyHex.trim())}
      >
        <Text style={styles.buttonText}>
          {isSending ? 'Sending…' : 'Send Request'}
        </Text>
      </TouchableOpacity>

      <TouchableOpacity style={styles.scanButton} onPress={() => navigation.navigate('ScanQR')}>
        <Text style={styles.scanButtonText}>Scan QR Code Instead</Text>
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: colors.background, padding: 20 },
  label: { fontSize: 13, color: colors.textSecondary, textTransform: 'uppercase', letterSpacing: 0.8, marginTop: 24, marginBottom: 8 },
  input: {
    backgroundColor: colors.surface, borderRadius: 8,
    paddingHorizontal: 16, paddingVertical: 12,
    color: colors.textPrimary, fontSize: 14,
    borderWidth: 1, borderColor: colors.borderLight,
  },
  textArea: { minHeight: 80, textAlignVertical: 'top' },
  charCount: { fontSize: 11, color: colors.textMuted, textAlign: 'right', marginTop: 4 },
  button: {
    backgroundColor: colors.textPrimary, paddingVertical: 16,
    borderRadius: 8, marginTop: 32, alignItems: 'center',
  },
  buttonDisabled: { opacity: 0.3 },
  buttonText: { fontSize: 16, fontWeight: '600', color: colors.background },
  scanButton: { marginTop: 16, alignItems: 'center', paddingVertical: 12 },
  scanButtonText: { fontSize: 15, color: colors.accent },
});
