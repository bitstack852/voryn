import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Share,
  Alert,
} from 'react-native';
import * as VorynBridge from '../services/VorynBridge';

export const ShareKeyScreen: React.FC = () => {
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
        message: publicKeyHex,
        title: 'My Voryn Public Key',
      });
    } catch {
      // User cancelled
    }
  };

  const handleCopy = () => {
    // React Native doesn't have Clipboard in core anymore,
    // but Share serves the same purpose
    Alert.alert('Public Key', publicKeyHex, [
      { text: 'Share', onPress: handleShare },
      { text: 'OK' },
    ]);
  };

  // Format key in groups of 8 for readability
  const formatKey = (hex: string): string => {
    const groups = [];
    for (let i = 0; i < hex.length; i += 8) {
      groups.push(hex.slice(i, i + 8));
    }
    return groups.join(' ');
  };

  return (
    <View style={styles.container}>
      <View style={styles.keyCard}>
        <Text style={styles.label}>Your Public Key</Text>
        <Text style={styles.keyText} selectable>
          {formatKey(publicKeyHex)}
        </Text>
        <Text style={styles.hint}>
          Share this key with contacts so they can message you
        </Text>
      </View>

      <TouchableOpacity style={styles.shareButton} onPress={handleShare} activeOpacity={0.8}>
        <Text style={styles.shareButtonText}>Share Key</Text>
      </TouchableOpacity>

      <TouchableOpacity style={styles.copyButton} onPress={handleCopy} activeOpacity={0.8}>
        <Text style={styles.copyButtonText}>View Full Key</Text>
      </TouchableOpacity>

      <View style={styles.infoBox}>
        <Text style={styles.infoTitle}>How it works</Text>
        <Text style={styles.infoText}>
          1. Share your public key with someone you want to message
        </Text>
        <Text style={styles.infoText}>
          2. They add your key as a contact in their Voryn app
        </Text>
        <Text style={styles.infoText}>
          3. You add their key as a contact in your app
        </Text>
        <Text style={styles.infoText}>
          4. You can now exchange end-to-end encrypted messages
        </Text>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D', padding: 20 },
  keyCard: {
    backgroundColor: '#1A1A1A',
    borderRadius: 16,
    padding: 24,
    marginTop: 20,
    borderWidth: 1,
    borderColor: '#2A2A2A',
  },
  label: {
    fontSize: 13,
    color: '#888888',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 12,
  },
  keyText: {
    fontSize: 14,
    color: '#FFFFFF',
    fontFamily: 'monospace',
    lineHeight: 24,
  },
  hint: { fontSize: 12, color: '#555555', marginTop: 16 },
  shareButton: {
    backgroundColor: '#FFFFFF',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginTop: 24,
  },
  shareButtonText: { fontSize: 16, fontWeight: '600', color: '#0D0D0D' },
  copyButton: {
    backgroundColor: 'transparent',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginTop: 8,
    borderWidth: 1,
    borderColor: '#2A2A2A',
  },
  copyButtonText: { fontSize: 16, fontWeight: '600', color: '#888888' },
  infoBox: {
    marginTop: 32,
    paddingHorizontal: 4,
  },
  infoTitle: { fontSize: 14, fontWeight: '600', color: '#888888', marginBottom: 12 },
  infoText: { fontSize: 13, color: '#555555', marginTop: 6, lineHeight: 20 },
});
