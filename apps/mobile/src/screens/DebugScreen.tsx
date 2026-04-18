import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
  Share,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import * as VorynBridge from '../services/VorynBridge';
import * as NetworkService from '../services/NetworkService';
import * as PasscodeService from '../services/PasscodeService';

export const DebugScreen: React.FC = () => {
  const navigation = useNavigation();
  const [identity, setIdentity] = useState<VorynBridge.Identity | null>(null);
  const [contacts, setContacts] = useState<VorynBridge.Contact[]>([]);
  const [messageCount, setMessageCount] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);

  const addLog = (msg: string) => {
    const ts = new Date().toLocaleTimeString();
    setLogs((prev) => [`[${ts}] ${msg}`, ...prev].slice(0, 50));
  };

  useEffect(() => {
    const load = async () => {
      const id = await VorynBridge.loadIdentity();
      setIdentity(id);
      const c = await VorynBridge.getContacts();
      setContacts(c);
      addLog('Loaded identity and contacts');
    };
    load();

    // Show any errors that fired before this screen was open
    NetworkService.getRecentErrors().forEach((msg) => addLog(`NET ERROR: ${msg}`));
    const unsubError = NetworkService.onError((msg) => addLog(`NET ERROR: ${msg}`));
    return () => { unsubError(); };
  }, []);

  const handleShareKey = async () => {
    if (!identity) return;
    try {
      await Share.share({
        message: `My Voryn public key:\n${identity.publicKeyHex}`,
        title: 'Voryn Public Key',
      });
      addLog('Shared public key');
    } catch {
      addLog('Share cancelled');
    }
  };

  const handleTestRustBridge = async () => {
    addLog('Calling Rust hello...');
    try {
      const msg = await VorynBridge.helloFromRust();
      addLog(`Rust bridge: ${msg}`);
    } catch (e) {
      addLog(`Rust bridge error: ${e}`);
    }
  };

  const handleTestBootstrap = async () => {
    addLog('Testing bootstrap connection...');
    try {
      const info = NetworkService.getBootstrapInfo();
      addLog(`Bootstrap peers: ${info.peers.length}, connected: ${info.connected}`);
      info.peers.forEach((p) => addLog(`  ${p}`));
      await NetworkService.connect();
      const status = NetworkService.getStatus();
      const peers = NetworkService.getPeerCount();
      const peerId = NetworkService.getLocalPeerId();
      addLog(`Network status: ${status}, peers: ${peers}`);
      addLog(`Local peer ID: ${peerId ?? 'null'}`);
    } catch (e) {
      addLog(`Bootstrap error: ${e}`);
    }
  };

  const handleNetworkStatus = async () => {
    addLog('Checking network status via Rust...');
    try {
      const s = await VorynBridge.getNetworkStatus();
      addLog(`Status: ${s.status}, peers: ${s.peerCount}`);
      addLog(`Peer ID: ${s.peerId ?? 'null'}`);
    } catch (e) {
      addLog(`Status error: ${e}`);
    }
  };

  const handleTestPasscode = async () => {
    const has = await PasscodeService.hasPasscode();
    addLog(`Passcode set: ${has}`);
    if (has) {
      const remaining = await PasscodeService.getRemainingAttempts();
      addLog(`Remaining attempts: ${remaining}`);
    }
  };

  const handleAddTestContact = async () => {
    const testKey = Array.from({ length: 64 }, () =>
      Math.floor(Math.random() * 16).toString(16),
    ).join('');
    await VorynBridge.addContact(testKey, `Test User ${contacts.length + 1}`);
    const c = await VorynBridge.getContacts();
    setContacts(c);
    addLog(`Added test contact: ${testKey.slice(0, 16)}...`);
  };

  const handleSendTestMessage = async () => {
    if (contacts.length === 0) {
      Alert.alert('No Contacts', 'Add a test contact first.');
      return;
    }
    const contact = contacts[0];
    await VorynBridge.sendMessage(contact.publicKeyHex, 'Hello from Voryn debug!');
    addLog(`Sent test message to ${contact.displayName || contact.publicKeyHex.slice(0, 16)}`);
  };

  const handleClearAll = () => {
    Alert.alert('Clear All Data', 'Delete everything?', [
      { text: 'Cancel', style: 'cancel' },
      {
        text: 'Delete',
        style: 'destructive',
        onPress: async () => {
          await VorynBridge.deleteIdentity();
          addLog('All data cleared');
          navigation.reset({ index: 0, routes: [{ name: 'Onboarding' as never }] });
        },
      },
    ]);
  };

  return (
    <ScrollView style={styles.container}>
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Identity</Text>
        {identity ? (
          <>
            <Text style={styles.monoText} selectable>
              {identity.publicKeyHex}
            </Text>
            <Text style={styles.subText}>Created: {identity.createdAt}</Text>
            <TouchableOpacity style={styles.actionButton} onPress={handleShareKey}>
              <Text style={styles.actionButtonText}>Share Public Key</Text>
            </TouchableOpacity>
          </>
        ) : (
          <Text style={styles.subText}>No identity</Text>
        )}
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Contacts ({contacts.length})</Text>
        {contacts.map((c) => (
          <Text key={c.publicKeyHex} style={styles.monoText}>
            {c.displayName || 'Unknown'}: {c.publicKeyHex.slice(0, 16)}...
          </Text>
        ))}
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Actions</Text>
        <TouchableOpacity style={styles.actionButton} onPress={handleTestRustBridge}>
          <Text style={styles.actionButtonText}>Test Rust Bridge (hello)</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.actionButton} onPress={handleNetworkStatus}>
          <Text style={styles.actionButtonText}>Check Network Status</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.actionButton} onPress={handleTestBootstrap}>
          <Text style={styles.actionButtonText}>Connect to Bootstrap</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.actionButton} onPress={handleAddTestContact}>
          <Text style={styles.actionButtonText}>Add Test Contact</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.actionButton} onPress={handleSendTestMessage}>
          <Text style={styles.actionButtonText}>Send Test Message</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.actionButton} onPress={handleTestPasscode}>
          <Text style={styles.actionButtonText}>Check Passcode Status</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.dangerButton} onPress={handleClearAll}>
          <Text style={styles.dangerButtonText}>Clear All Data</Text>
        </TouchableOpacity>
      </View>

      <View style={styles.section}>
        <View style={styles.logHeader}>
          <Text style={styles.sectionTitle}>Log</Text>
          {logs.length > 0 && (
            <TouchableOpacity onPress={() => setLogs([])}>
              <Text style={styles.clearLogText}>Clear</Text>
            </TouchableOpacity>
          )}
        </View>
        {logs.map((log, i) => (
          <Text key={i} style={styles.logText}>{log}</Text>
        ))}
        {logs.length === 0 && <Text style={styles.subText}>No activity yet</Text>}
      </View>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D' },
  section: {
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#1A1A1A',
  },
  sectionTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#4A9EFF',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 8,
  },
  monoText: { fontSize: 11, color: '#888888', fontFamily: 'monospace', marginTop: 4 },
  subText: { fontSize: 12, color: '#555555', marginTop: 4 },
  actionButton: {
    backgroundColor: '#1A1A1A',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
    marginTop: 8,
    borderWidth: 1,
    borderColor: '#333333',
  },
  actionButtonText: { fontSize: 14, color: '#FFFFFF' },
  dangerButton: {
    backgroundColor: '#1A1A1A',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
    marginTop: 8,
    borderWidth: 1,
    borderColor: '#FF3B30',
  },
  dangerButtonText: { fontSize: 14, color: '#FF3B30' },
  logText: { fontSize: 10, color: '#666666', fontFamily: 'monospace', marginTop: 2 },
  logHeader: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 },
  clearLogText: { fontSize: 13, color: '#4A9EFF' },
});
