import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Share,
  ScrollView,
  Alert,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import QRCode from 'react-native-qrcode-svg';
import * as VorynBridge from '../services/VorynBridge';
import { colors } from '../theme/colors';

type Nav = NativeStackNavigationProp<RootStackParamList, 'ShareKey'>;

export const ShareKeyScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [publicKeyHex, setPublicKeyHex] = useState<string>('');

  useEffect(() => {
    VorynBridge.loadIdentity().then((identity) => {
      if (identity) setPublicKeyHex(identity.publicKeyHex);
    });
  }, []);

  const handleShareKey = async () => {
    try {
      await Share.share({
        message: `voryn://${publicKeyHex}`,
        title: 'My Voryn Public Key',
      });
    } catch { /* cancelled */ }
  };

  const handleGenerateInviteLink = async () => {
    try {
      const link = await VorynBridge.generateInviteLink();
      await Share.share({
        message: link,
        title: 'Voryn Invite Link',
      });
    } catch {
      Alert.alert('Error', 'Failed to generate invite link.');
    }
  };

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <Text style={styles.title}>My Voryn Key</Text>
      <Text style={styles.subtitle}>Scan this QR code to add me as a contact</Text>

      {publicKeyHex ? (
        <View style={styles.qrContainer}>
          <View style={styles.qrBox}>
            <QRCode
              value={`voryn://${publicKeyHex}`}
              size={220}
              backgroundColor="#FFFFFF"
              color="#000000"
            />
          </View>
        </View>
      ) : null}

      <Text style={styles.keyLabel}>Public Key</Text>
      <Text style={styles.keyText} selectable>{publicKeyHex}</Text>

      <TouchableOpacity style={styles.shareButton} onPress={handleShareKey} activeOpacity={0.8}>
        <Text style={styles.shareButtonText}>Share Key</Text>
      </TouchableOpacity>

      <TouchableOpacity style={styles.inviteButton} onPress={handleGenerateInviteLink} activeOpacity={0.8}>
        <Text style={styles.inviteButtonText}>Generate Invite Link</Text>
      </TouchableOpacity>

      <TouchableOpacity
        style={styles.scanButton}
        onPress={() => navigation.navigate('ScanQR')}
        activeOpacity={0.8}
      >
        <Text style={styles.scanButtonText}>Scan Contact's QR Code</Text>
      </TouchableOpacity>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: colors.background },
  content: { alignItems: 'center', padding: 24 },
  title: { fontSize: 24, fontWeight: '600', color: colors.textPrimary, marginTop: 16 },
  subtitle: { fontSize: 14, color: colors.textSecondary, marginTop: 8, textAlign: 'center' },
  qrContainer: { marginTop: 32, marginBottom: 24 },
  qrBox: { backgroundColor: '#FFFFFF', padding: 20, borderRadius: 16 },
  keyLabel: {
    fontSize: 13, color: colors.textSecondary, textTransform: 'uppercase',
    letterSpacing: 1, alignSelf: 'flex-start', marginTop: 16,
  },
  keyText: {
    fontSize: 12, color: colors.textMuted, fontFamily: 'Menlo',
    marginTop: 8, alignSelf: 'flex-start',
  },
  shareButton: {
    backgroundColor: colors.textPrimary, paddingVertical: 16,
    borderRadius: 12, alignItems: 'center', marginTop: 24, width: '100%',
  },
  shareButtonText: { fontSize: 16, fontWeight: '600', color: colors.background },
  inviteButton: {
    backgroundColor: colors.accentDark, paddingVertical: 16,
    borderRadius: 12, alignItems: 'center', marginTop: 12, width: '100%',
    borderWidth: 1, borderColor: colors.accent,
  },
  inviteButtonText: { fontSize: 16, fontWeight: '600', color: colors.accent },
  scanButton: {
    backgroundColor: 'transparent', paddingVertical: 16,
    borderRadius: 12, alignItems: 'center', marginTop: 12, width: '100%',
    borderWidth: 1, borderColor: colors.surfaceLight,
  },
  scanButtonText: { fontSize: 16, fontWeight: '600', color: colors.textSecondary },
});
