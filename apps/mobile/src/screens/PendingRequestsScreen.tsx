import React, { useCallback, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useFocusEffect } from '@react-navigation/native';
import * as VorynBridge from '../services/VorynBridge';
import * as NetworkService from '../services/NetworkService';
import { colors } from '../theme/colors';

function generateId(): string {
  const bytes = new Uint8Array(8);
  if (typeof globalThis.crypto !== 'undefined' && globalThis.crypto.getRandomValues) {
    globalThis.crypto.getRandomValues(bytes);
  } else {
    for (let i = 0; i < 8; i++) bytes[i] = Math.floor(Math.random() * 256);
  }
  return Array.from(bytes).map((b) => b.toString(16).padStart(2, '0')).join('');
}

export const PendingRequestsScreen: React.FC = () => {
  const [pending, setPending] = useState<VorynBridge.Contact[]>([]);

  const load = useCallback(async () => {
    const contacts = await VorynBridge.getPendingReceivedContacts();
    setPending(contacts);
  }, []);

  useFocusEffect(useCallback(() => { load(); }, [load]));

  const sendResponse = async (recipientPubkey: string, type: 'cacc' | 'cden') => {
    const identity = await VorynBridge.loadIdentity();
    if (!identity) return;

    const payload = JSON.stringify({ t: type });
    const encrypted = await VorynBridge.encryptMessage(
      payload,
      identity.secretKeySeedHex,
      identity.publicKeyHex,
      recipientPubkey,
    );
    if (encrypted) {
      NetworkService.sendToPeer(recipientPubkey, encrypted.envelopeHex, generateId());
    }
  };

  const handleApprove = async (contact: VorynBridge.Contact) => {
    await VorynBridge.approveContact(contact.publicKeyHex);
    await sendResponse(contact.publicKeyHex, 'cacc');
    load();
  };

  const handleDeny = (contact: VorynBridge.Contact) => {
    Alert.alert(
      'Deny Request',
      `Deny contact request from ${contact.displayName ?? contact.publicKeyHex.slice(0, 16) + '…'}?`,
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Deny',
          style: 'destructive',
          onPress: async () => {
            await VorynBridge.denyContact(contact.publicKeyHex);
            await sendResponse(contact.publicKeyHex, 'cden');
            load();
          },
        },
      ],
    );
  };

  if (pending.length === 0) {
    return (
      <View style={styles.empty}>
        <Text style={styles.emptyText}>No pending requests</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <FlatList
        data={pending}
        keyExtractor={(item) => item.publicKeyHex}
        renderItem={({ item }) => (
          <View style={styles.card}>
            <View style={styles.avatar}>
              <Text style={styles.avatarText}>
                {(item.displayName ?? '?')[0].toUpperCase()}
              </Text>
            </View>
            <View style={styles.info}>
              {item.displayName && (
                <Text style={styles.name}>{item.displayName}</Text>
              )}
              <Text style={styles.key}>{item.publicKeyHex.slice(0, 20)}…</Text>
              {item.introMessage ? (
                <Text style={styles.intro}>{item.introMessage}</Text>
              ) : null}
            </View>
            <View style={styles.actions}>
              <TouchableOpacity
                style={styles.approveButton}
                onPress={() => handleApprove(item)}
              >
                <Text style={styles.approveText}>Approve</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={styles.denyButton}
                onPress={() => handleDeny(item)}
              >
                <Text style={styles.denyText}>Deny</Text>
              </TouchableOpacity>
            </View>
          </View>
        )}
        ItemSeparatorComponent={() => <View style={styles.separator} />}
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: colors.background },
  empty: { flex: 1, backgroundColor: colors.background, alignItems: 'center', justifyContent: 'center' },
  emptyText: { fontSize: 16, color: colors.textSecondary },
  card: { flexDirection: 'row', alignItems: 'flex-start', padding: 16 },
  avatar: {
    width: 44, height: 44, borderRadius: 22,
    backgroundColor: colors.accentDark,
    alignItems: 'center', justifyContent: 'center', marginRight: 12,
  },
  avatarText: { fontSize: 16, fontWeight: '600', color: colors.textPrimary },
  info: { flex: 1 },
  name: { fontSize: 15, fontWeight: '600', color: colors.textPrimary, marginBottom: 2 },
  key: { fontSize: 11, color: colors.textMuted, fontFamily: 'Menlo' },
  intro: { fontSize: 13, color: colors.textSecondary, marginTop: 6, fontStyle: 'italic' },
  actions: { flexDirection: 'column', gap: 8, marginLeft: 8 },
  approveButton: {
    backgroundColor: colors.accent, paddingVertical: 8, paddingHorizontal: 14,
    borderRadius: 8, alignItems: 'center',
  },
  approveText: { fontSize: 13, fontWeight: '600', color: colors.background },
  denyButton: {
    backgroundColor: 'transparent', paddingVertical: 8, paddingHorizontal: 14,
    borderRadius: 8, alignItems: 'center', borderWidth: 1, borderColor: colors.borderLight,
  },
  denyText: { fontSize: 13, fontWeight: '600', color: colors.textSecondary },
  separator: { height: 1, backgroundColor: colors.border },
});
