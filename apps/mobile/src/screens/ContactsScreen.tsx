import React, { useState, useCallback, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
} from 'react-native';
import { useNavigation, useFocusEffect } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';
import * as NetworkService from '../services/NetworkService';
import { useNetwork } from '../hooks/useNetwork';
import { colors } from '../theme/colors';

type Nav = NativeStackNavigationProp<RootStackParamList, 'Contacts'>;

export const ContactsScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [contacts, setContacts] = useState<VorynBridge.Contact[]>([]);
  const [pendingCount, setPendingCount] = useState(0);
  const { status, peerCount } = useNetwork();

  const load = useCallback(async () => {
    const approved = await VorynBridge.getApprovedContacts();
    setContacts(approved);
    const pending = await VorynBridge.getPendingReceivedContacts();
    setPendingCount(pending.length);
  }, []);

  useEffect(() => {
    NetworkService.connect().catch(() => {});
  }, []);

  useFocusEffect(useCallback(() => { load(); }, [load]));

  // Reload when a contact request arrives
  useEffect(() => {
    return NetworkService.onContactRequest(() => { load(); });
  }, [load]);

  const statusDotColor =
    status === 'connected' ? colors.success :
    status === 'connecting' ? colors.warning :
    colors.textMuted;

  const statusLabel =
    status === 'connected' ? `${peerCount} peer${peerCount !== 1 ? 's' : ''}` :
    status === 'connecting' ? 'Connecting…' :
    'Offline';

  return (
    <View style={styles.container}>
      <View style={styles.networkBar}>
        <View style={[styles.statusDot, { backgroundColor: statusDotColor }]} />
        <Text style={styles.statusText}>{statusLabel}</Text>
      </View>

      {pendingCount > 0 && (
        <TouchableOpacity
          style={styles.pendingBanner}
          onPress={() => navigation.navigate('PendingRequests')}
        >
          <Text style={styles.pendingText}>
            {pendingCount} pending contact request{pendingCount !== 1 ? 's' : ''}
          </Text>
          <Text style={styles.pendingArrow}>›</Text>
        </TouchableOpacity>
      )}

      {contacts.length === 0 ? (
        <View style={styles.emptyState}>
          <Text style={styles.emptyTitle}>No Contacts Yet</Text>
          <Text style={styles.emptySubtitle}>
            Share your public key with someone to get started
          </Text>
          <TouchableOpacity
            style={styles.primaryButton}
            onPress={() => navigation.navigate('ShareKey')}
          >
            <Text style={styles.primaryButtonText}>Share My Key</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={styles.secondaryButton}
            onPress={() => navigation.navigate('ScanQR')}
          >
            <Text style={styles.secondaryButtonText}>Scan QR / Add Contact</Text>
          </TouchableOpacity>
        </View>
      ) : (
        <>
          <FlatList
            data={contacts}
            keyExtractor={(item) => item.publicKeyHex}
            renderItem={({ item }) => (
              <TouchableOpacity
                style={styles.contactRow}
                onPress={() =>
                  navigation.navigate('Chat', {
                    contactPubkeyHex: item.publicKeyHex,
                    displayName: item.displayName ?? undefined,
                  })
                }
                onLongPress={() =>
                  navigation.navigate('ContactDetail', { contactPubkeyHex: item.publicKeyHex })
                }
              >
                <View style={styles.avatar}>
                  <Text style={styles.avatarText}>
                    {(item.displayName || '?')[0].toUpperCase()}
                  </Text>
                </View>
                <View style={styles.contactInfo}>
                  <Text style={styles.contactName}>
                    {item.displayName ?? 'Unknown'}
                  </Text>
                  <Text style={styles.contactKey}>
                    {item.publicKeyHex.slice(0, 16)}…
                  </Text>
                </View>
                {item.isVerified && <Text style={styles.verifiedBadge}>✓</Text>}
              </TouchableOpacity>
            )}
          />
          <View style={styles.bottomBar}>
            <TouchableOpacity style={styles.bottomButton} onPress={() => navigation.navigate('AddContact')}>
              <Text style={styles.bottomButtonText}>Add Contact</Text>
            </TouchableOpacity>
            <TouchableOpacity style={styles.bottomButton} onPress={() => navigation.navigate('ShareKey')}>
              <Text style={styles.bottomButtonText}>Share Key</Text>
            </TouchableOpacity>
            <TouchableOpacity style={styles.bottomButton} onPress={() => navigation.navigate('ScanQR')}>
              <Text style={styles.bottomButtonText}>Scan QR</Text>
            </TouchableOpacity>
            <TouchableOpacity style={styles.bottomButton} onPress={() => navigation.navigate('Settings')}>
              <Text style={styles.bottomButtonText}>Settings</Text>
            </TouchableOpacity>
          </View>
        </>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: colors.background },
  networkBar: {
    flexDirection: 'row', alignItems: 'center',
    paddingHorizontal: 16, paddingVertical: 6,
    borderBottomWidth: 1, borderBottomColor: colors.border,
  },
  statusDot: { width: 6, height: 6, borderRadius: 3, marginRight: 6 },
  statusText: { fontSize: 11, color: colors.textMuted, fontFamily: 'Menlo' },
  pendingBanner: {
    flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between',
    backgroundColor: colors.accentDark, paddingHorizontal: 16, paddingVertical: 12,
    borderBottomWidth: 1, borderBottomColor: colors.border,
  },
  pendingText: { fontSize: 14, color: colors.accent, fontWeight: '500' },
  pendingArrow: { fontSize: 20, color: colors.accent },
  emptyState: { flex: 1, alignItems: 'center', justifyContent: 'center', padding: 24 },
  emptyTitle: { fontSize: 22, fontWeight: '600', color: colors.textPrimary, marginTop: 20 },
  emptySubtitle: { fontSize: 14, color: colors.textSecondary, marginTop: 8, textAlign: 'center' },
  primaryButton: {
    backgroundColor: colors.textPrimary, paddingVertical: 14, paddingHorizontal: 40,
    borderRadius: 12, marginTop: 32, width: '100%', alignItems: 'center',
  },
  primaryButtonText: { fontSize: 16, fontWeight: '600', color: colors.background },
  secondaryButton: {
    paddingVertical: 14, paddingHorizontal: 40, borderRadius: 12, marginTop: 12,
    width: '100%', alignItems: 'center', borderWidth: 1, borderColor: colors.surfaceLight,
  },
  secondaryButtonText: { fontSize: 16, fontWeight: '600', color: colors.textSecondary },
  contactRow: {
    flexDirection: 'row', alignItems: 'center',
    paddingVertical: 14, paddingHorizontal: 20,
    borderBottomWidth: 1, borderBottomColor: colors.border,
  },
  avatar: {
    width: 44, height: 44, borderRadius: 22,
    backgroundColor: colors.accentDark,
    alignItems: 'center', justifyContent: 'center', marginRight: 14,
  },
  avatarText: { fontSize: 18, fontWeight: '600', color: colors.textPrimary },
  contactInfo: { flex: 1 },
  contactName: { fontSize: 16, fontWeight: '500', color: colors.textPrimary },
  contactKey: { fontSize: 11, color: colors.textMuted, fontFamily: 'Menlo', marginTop: 2 },
  verifiedBadge: { fontSize: 16, color: colors.success },
  bottomBar: {
    flexDirection: 'row', borderTopWidth: 1,
    borderTopColor: colors.border, paddingBottom: 8,
  },
  bottomButton: { flex: 1, paddingVertical: 14, alignItems: 'center' },
  bottomButtonText: { fontSize: 13, color: colors.textSecondary },
});
