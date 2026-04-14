import React, { useState, useEffect } from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity, Clipboard } from 'react-native';
import * as VorynBridge from '../services/VorynBridge';

export const SettingsScreen: React.FC = () => {
  const [publicKeyHex, setPublicKeyHex] = useState<string>('Loading...');
  const [networkStatus, setNetworkStatus] = useState<string>('disconnected');
  const [peerCount, setPeerCount] = useState<number>(0);
  const [peerId, setPeerId] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      const identity = await VorynBridge.loadIdentity();
      if (identity) {
        setPublicKeyHex(identity.publicKeyHex);
      } else {
        setPublicKeyHex('Not generated');
      }

      const network = await VorynBridge.getNetworkStatus();
      setNetworkStatus(network.status);
      setPeerCount(network.peerCount);
      setPeerId(network.peerId);
    };
    load();
  }, []);

  const copyPublicKey = () => {
    if (publicKeyHex && publicKeyHex !== 'Not generated') {
      Clipboard.setString(publicKeyHex);
    }
  };

  return (
    <ScrollView style={styles.container}>
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Identity</Text>

        <Text style={styles.label}>Public Key</Text>
        <TouchableOpacity onPress={copyPublicKey}>
          <Text style={styles.value} numberOfLines={2}>
            {publicKeyHex}
          </Text>
          {publicKeyHex !== 'Not generated' && (
            <Text style={styles.hint}>Tap to copy</Text>
          )}
        </TouchableOpacity>

        {peerId && (
          <>
            <Text style={styles.label}>Peer ID</Text>
            <Text style={styles.value} numberOfLines={1}>
              {peerId}
            </Text>
          </>
        )}
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Network</Text>
        <Text style={styles.label}>Status</Text>
        <Text
          style={[
            styles.value,
            networkStatus === 'connected' && styles.valueGreen,
          ]}
        >
          {networkStatus}
        </Text>
        <Text style={styles.label}>Connected Peers</Text>
        <Text style={styles.value}>{peerCount}</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Security</Text>
        <Text style={styles.label}>Encryption</Text>
        <Text style={styles.value}>XSalsa20-Poly1305 (Phase 1)</Text>
        <Text style={styles.label}>Key Exchange</Text>
        <Text style={styles.value}>X25519 Diffie-Hellman</Text>
        <Text style={styles.label}>Signing</Text>
        <Text style={styles.value}>Ed25519</Text>
        <Text style={styles.label}>Key Storage</Text>
        <Text style={styles.value}>Software (Phase 2: Hardware)</Text>
        <Text style={styles.label}>Passcode</Text>
        <Text style={styles.value}>Not configured (Phase 2)</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>About</Text>
        <Text style={styles.label}>Version</Text>
        <Text style={styles.value}>0.1.0</Text>
        <Text style={styles.label}>Phase</Text>
        <Text style={styles.value}>1 — Foundation</Text>
        <Text style={styles.label}>Protocol</Text>
        <Text style={styles.value}>Basic DH (Phase 2: Double Ratchet)</Text>
      </View>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0D0D0D',
  },
  section: {
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#1A1A1A',
  },
  sectionTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#888888',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 12,
  },
  label: {
    fontSize: 15,
    color: '#FFFFFF',
    marginTop: 8,
  },
  value: {
    fontSize: 13,
    color: '#555555',
    fontFamily: 'monospace',
    marginTop: 2,
  },
  valueGreen: {
    color: '#34C759',
  },
  hint: {
    fontSize: 11,
    color: '#333333',
    marginTop: 2,
  },
});
