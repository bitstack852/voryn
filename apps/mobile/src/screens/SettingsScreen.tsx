import React, { useState, useEffect } from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity, Alert } from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';
import * as PasscodeService from '../services/PasscodeService';

type Nav = NativeStackNavigationProp<RootStackParamList, 'Settings'>;

export const SettingsScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [publicKeyHex, setPublicKeyHex] = useState<string>('Loading...');
  const [createdAt, setCreatedAt] = useState<string>('');
  const [contactCount, setContactCount] = useState<number>(0);
  const [hasPasscode, setHasPasscode] = useState(false);

  useEffect(() => {
    const load = async () => {
      const identity = await VorynBridge.loadIdentity();
      if (identity) {
        setPublicKeyHex(identity.publicKeyHex);
        setCreatedAt(identity.createdAt);
      } else {
        setPublicKeyHex('Not generated');
      }
      const contacts = await VorynBridge.getContacts();
      setContactCount(contacts.length);
      const pc = await PasscodeService.hasPasscode();
      setHasPasscode(pc);
    };
    load();
  }, []);

  const handleDeleteIdentity = () => {
    Alert.alert(
      'Delete Identity',
      'This will permanently delete your identity, contacts, and all messages.',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Delete Everything',
          style: 'destructive',
          onPress: async () => {
            await VorynBridge.deleteIdentity();
            navigation.reset({ index: 0, routes: [{ name: 'Onboarding' }] });
          },
        },
      ],
    );
  };

  return (
    <ScrollView style={styles.container}>
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Identity</Text>
        <Text style={styles.label}>Public Key</Text>
        <Text style={styles.value} selectable numberOfLines={4}>{publicKeyHex}</Text>
        {createdAt ? (
          <>
            <Text style={styles.label}>Created</Text>
            <Text style={styles.value}>{new Date(createdAt).toLocaleDateString()}</Text>
          </>
        ) : null}
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Network</Text>
        <Text style={styles.label}>Status</Text>
        <Text style={styles.value}>Offline (P2P not connected)</Text>
        <Text style={styles.label}>Contacts</Text>
        <Text style={styles.value}>{contactCount}</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Security</Text>
        <Text style={styles.label}>Encryption</Text>
        <Text style={styles.value}>XSalsa20-Poly1305</Text>
        <Text style={styles.label}>Key Exchange</Text>
        <Text style={styles.value}>X25519 Diffie-Hellman</Text>
        <Text style={styles.label}>Protocol</Text>
        <Text style={styles.value}>Double Ratchet</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>About</Text>
        <Text style={styles.label}>Version</Text>
        <Text style={styles.value}>0.1.0</Text>
      </View>

      <View style={styles.section}>
        {!hasPasscode ? (
          <TouchableOpacity style={styles.linkButton} onPress={() => navigation.navigate('PasscodeSetup')}>
            <Text style={styles.linkButtonText}>Set Passcode Lock</Text>
          </TouchableOpacity>
        ) : (
          <TouchableOpacity style={styles.linkButton} onPress={async () => {
            await PasscodeService.removePasscode();
            setHasPasscode(false);
            Alert.alert('Passcode Removed', 'Passcode lock has been disabled.');
          }}>
            <Text style={styles.linkButtonText}>Remove Passcode Lock</Text>
          </TouchableOpacity>
        )}
        <TouchableOpacity style={styles.linkButton} onPress={() => navigation.navigate('ShareKey')}>
          <Text style={styles.linkButtonText}>Share My Public Key</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.linkButton} onPress={() => navigation.navigate('Debug')}>
          <Text style={styles.linkButtonText}>Debug Console</Text>
        </TouchableOpacity>
      </View>

      <View style={styles.section}>
        <TouchableOpacity style={styles.dangerButton} onPress={handleDeleteIdentity}>
          <Text style={styles.dangerButtonText}>Delete Identity & All Data</Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D' },
  section: { paddingHorizontal: 20, paddingVertical: 16, borderBottomWidth: 1, borderBottomColor: '#1A1A1A' },
  sectionTitle: { fontSize: 13, fontWeight: '600', color: '#888888', textTransform: 'uppercase', letterSpacing: 1, marginBottom: 12 },
  label: { fontSize: 15, color: '#FFFFFF', marginTop: 8 },
  value: { fontSize: 13, color: '#555555', fontFamily: 'monospace', marginTop: 2 },
  dangerButton: { backgroundColor: '#1A1A1A', paddingVertical: 14, borderRadius: 8, alignItems: 'center', borderWidth: 1, borderColor: '#FF3B30' },
  linkButton: { backgroundColor: '#1A1A1A', paddingVertical: 14, borderRadius: 8, alignItems: 'center', marginTop: 8, borderWidth: 1, borderColor: '#333333' },
  linkButtonText: { fontSize: 14, color: '#FFFFFF' },
  dangerButtonText: { fontSize: 14, fontWeight: '600', color: '#FF3B30' },
});
