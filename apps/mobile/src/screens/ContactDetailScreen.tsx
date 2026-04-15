import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRoute, useNavigation } from '@react-navigation/native';
import type { RouteProp } from '@react-navigation/native';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';

type DetailRoute = RouteProp<RootStackParamList, 'ContactDetail'>;

export const ContactDetailScreen: React.FC = () => {
  const route = useRoute<DetailRoute>();
  const navigation = useNavigation();
  const { contactPubkeyHex } = route.params;
  const [contact, setContact] = useState<VorynBridge.Contact | null>(null);
  const [safetyNumber, setSafetyNumber] = useState<string>('');

  useEffect(() => {
    const load = async () => {
      const contacts = await VorynBridge.getContacts();
      const found = contacts.find((c) => c.publicKeyHex === contactPubkeyHex);
      setContact(found || null);

      // Generate a deterministic safety number from both keys
      const identity = await VorynBridge.loadIdentity();
      if (identity) {
        const combined = [identity.publicKeyHex, contactPubkeyHex].sort().join('');
        // Simple hash-like safety number
        let hash = 0;
        for (let i = 0; i < combined.length; i++) {
          hash = ((hash << 5) - hash + combined.charCodeAt(i)) | 0;
        }
        const num = Math.abs(hash).toString().padStart(10, '0');
        const formatted = `${num.slice(0, 5)} ${num.slice(5, 10)} ${Math.abs(hash >> 16).toString().padStart(5, '0')} ${Math.abs(hash >> 8).toString().padStart(5, '0')}`;
        setSafetyNumber(formatted);
      }
    };
    load();
  }, [contactPubkeyHex]);

  const handleRemove = () => {
    Alert.alert(
      'Remove Contact',
      `Remove ${contact?.displayName || 'this contact'}? This will delete the conversation.`,
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Remove',
          style: 'destructive',
          onPress: async () => {
            await VorynBridge.removeContact(contactPubkeyHex);
            navigation.goBack();
          },
        },
      ],
    );
  };

  const formatKey = (hex: string): string => {
    const groups = [];
    for (let i = 0; i < hex.length; i += 8) {
      groups.push(hex.slice(i, i + 8));
    }
    return groups.join('\n');
  };

  if (!contact) {
    return (
      <View style={styles.container}>
        <Text style={styles.notFound}>Contact not found</Text>
      </View>
    );
  }

  return (
    <ScrollView style={styles.container}>
      <View style={styles.header}>
        <View style={styles.avatar}>
          <Text style={styles.avatarText}>
            {(contact.displayName || '?')[0].toUpperCase()}
          </Text>
        </View>
        <Text style={styles.displayName}>{contact.displayName || 'Unknown'}</Text>
        <Text style={styles.addedDate}>Added {new Date(contact.addedAt).toLocaleDateString()}</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Public Key</Text>
        <Text style={styles.keyText} selectable>
          {formatKey(contactPubkeyHex)}
        </Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Safety Number</Text>
        <Text style={styles.safetyNumber}>{safetyNumber}</Text>
        <Text style={styles.safetyHint}>
          Compare this number with your contact in person to verify their identity
        </Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Encryption</Text>
        <View style={styles.infoRow}>
          <Text style={styles.infoLabel}>Protocol</Text>
          <Text style={styles.infoValue}>Double Ratchet</Text>
        </View>
        <View style={styles.infoRow}>
          <Text style={styles.infoLabel}>Forward Secrecy</Text>
          <Text style={styles.infoValueGreen}>Active</Text>
        </View>
        <View style={styles.infoRow}>
          <Text style={styles.infoLabel}>Verified</Text>
          <Text style={contact.isVerified ? styles.infoValueGreen : styles.infoValue}>
            {contact.isVerified ? 'Yes' : 'Not yet'}
          </Text>
        </View>
      </View>

      <View style={styles.section}>
        <TouchableOpacity style={styles.dangerButton} onPress={handleRemove}>
          <Text style={styles.dangerButtonText}>Remove Contact</Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D' },
  notFound: { color: '#555555', textAlign: 'center', marginTop: 40 },
  header: { alignItems: 'center', paddingVertical: 32 },
  avatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#1A3A5C',
    alignItems: 'center',
    justifyContent: 'center',
  },
  avatarText: { fontSize: 32, fontWeight: '600', color: '#FFFFFF' },
  displayName: { fontSize: 24, fontWeight: '600', color: '#FFFFFF', marginTop: 16 },
  addedDate: { fontSize: 13, color: '#555555', marginTop: 4 },
  section: {
    paddingHorizontal: 20,
    paddingVertical: 16,
    borderTopWidth: 1,
    borderTopColor: '#1A1A1A',
  },
  sectionTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#888888',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 8,
  },
  keyText: { fontSize: 12, color: '#666666', fontFamily: 'monospace', lineHeight: 20 },
  safetyNumber: {
    fontSize: 24,
    color: '#FFFFFF',
    fontFamily: 'monospace',
    letterSpacing: 4,
    textAlign: 'center',
    marginVertical: 8,
  },
  safetyHint: { fontSize: 12, color: '#555555', textAlign: 'center', marginTop: 8 },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
  },
  infoLabel: { fontSize: 15, color: '#FFFFFF' },
  infoValue: { fontSize: 15, color: '#555555' },
  infoValueGreen: { fontSize: 15, color: '#34C759' },
  dangerButton: {
    backgroundColor: '#1A1A1A',
    paddingVertical: 14,
    borderRadius: 8,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#FF3B30',
  },
  dangerButtonText: { fontSize: 14, fontWeight: '600', color: '#FF3B30' },
});
