import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Share,
  ScrollView,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import QRCode from 'react-native-qrcode-svg';
import * as VorynBridge from '../services/VorynBridge';

type Nav = NativeStackNavigationProp<RootStackParamList, 'ShareKey'>;

export const ShareKeyScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [publicKeyHex, setPublicKeyHex] = useState<string>('');

  useEffect(() => {
    const load = async () => {
      const identity = await VorynBridge.loadIdentity();
      if (identity) {
        setPublicKeyHex(identity.publicKeyHex);
      }
    };
    load();
  }, []);

  const handleShare = async () => {
    try {
      await Share.share({
        message: `voryn://${publicKeyHex}`,
        title: 'My Voryn Public Key',
      });
    } catch {
      // User cancelled
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
      <Text style={styles.keyText} selectable>
        {publicKeyHex}
      </Text>

      <TouchableOpacity style={styles.shareButton} onPress={handleShare} activeOpacity={0.8}>
        <Text style={styles.shareButtonText}>Share Key</Text>
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
  container: { flex: 1, backgroundColor: '#0D0D0D' },
  content: { alignItems: 'center', padding: 24 },
  title: { fontSize: 24, fontWeight: '600', color: '#FFFFFF', marginTop: 16 },
  subtitle: { fontSize: 14, color: '#888888', marginTop: 8, textAlign: 'center' },
  qrContainer: { marginTop: 32, marginBottom: 24 },
  qrBox: {
    backgroundColor: '#FFFFFF',
    padding: 20,
    borderRadius: 16,
  },
  keyLabel: {
    fontSize: 13,
    color: '#888888',
    textTransform: 'uppercase',
    letterSpacing: 1,
    alignSelf: 'flex-start',
    marginTop: 16,
  },
  keyText: {
    fontSize: 12,
    color: '#666666',
    fontFamily: 'monospace',
    marginTop: 8,
    alignSelf: 'flex-start',
  },
  shareButton: {
    backgroundColor: '#FFFFFF',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginTop: 24,
    width: '100%',
  },
  shareButtonText: { fontSize: 16, fontWeight: '600', color: '#0D0D0D' },
  scanButton: {
    backgroundColor: 'transparent',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginTop: 12,
    width: '100%',
    borderWidth: 1,
    borderColor: '#4A9EFF',
  },
  scanButtonText: { fontSize: 16, fontWeight: '600', color: '#4A9EFF' },
});
