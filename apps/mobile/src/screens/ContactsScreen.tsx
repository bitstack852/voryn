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

type Nav = NativeStackNavigationProp<RootStackParamList, 'Contacts'>;

export const ContactsScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [contacts, setContacts] = useState<VorynBridge.Contact[]>([]);

  const loadContacts = useCallback(async () => {
    const result = await VorynBridge.getContacts();
    setContacts(result);
  }, []);

  // Connect to relay when Contacts screen loads (identity is guaranteed to exist)
  useEffect(() => {
    NetworkService.connect().catch((e) => {
      console.log('[Contacts] Relay connection failed:', e);
    });
  }, []);

  useFocusEffect(
    useCallback(() => {
      loadContacts();
    }, [loadContacts]),
  );

  return (
    <View style={styles.container}>
      {contacts.length === 0 ? (
        <View style={styles.emptyState}>
          <Text style={styles.emptyIcon}>🔐</Text>
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
            onPress={() => navigation.navigate('AddContact')}
          >
            <Text style={styles.secondaryButtonText}>Add Contact</Text>
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
                  navigation.navigate('ContactDetail', {
                    contactPubkeyHex: item.publicKeyHex,
                  })
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
                    {item.publicKeyHex.slice(0, 16)}...
                  </Text>
                </View>
                {item.isVerified && (
                  <Text style={styles.verifiedBadge}>✓</Text>
                )}
              </TouchableOpacity>
            )}
          />
          <View style={styles.bottomBar}>
            <TouchableOpacity
              style={styles.bottomButton}
              onPress={() => navigation.navigate('ShareKey')}
            >
              <Text style={styles.bottomButtonText}>Share Key</Text>
            </TouchableOpacity>
            <TouchableOpacity
              style={styles.bottomButton}
              onPress={() => navigation.navigate('AddContact')}
            >
              <Text style={styles.bottomButtonText}>Add Contact</Text>
            </TouchableOpacity>
            <TouchableOpacity
              style={styles.bottomButton}
              onPress={() => navigation.navigate('Settings')}
            >
              <Text style={styles.bottomButtonText}>Settings</Text>
            </TouchableOpacity>
          </View>
        </>
      )}

      {contacts.length === 0 && (
        <TouchableOpacity
          style={styles.settingsLink}
          onPress={() => navigation.navigate('Settings')}
        >
          <Text style={styles.settingsLinkText}>Settings</Text>
        </TouchableOpacity>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D' },
  emptyState: { flex: 1, alignItems: 'center', justifyContent: 'center', padding: 24 },
  emptyIcon: { fontSize: 48, marginBottom: 16 },
  emptyTitle: { fontSize: 22, fontWeight: '600', color: '#FFFFFF' },
  emptySubtitle: { fontSize: 14, color: '#888888', marginTop: 8, textAlign: 'center' },
  primaryButton: {
    backgroundColor: '#FFFFFF',
    paddingVertical: 14,
    paddingHorizontal: 40,
    borderRadius: 12,
    marginTop: 32,
    width: '100%',
    alignItems: 'center',
  },
  primaryButtonText: { fontSize: 16, fontWeight: '600', color: '#0D0D0D' },
  secondaryButton: {
    backgroundColor: 'transparent',
    paddingVertical: 14,
    paddingHorizontal: 40,
    borderRadius: 12,
    marginTop: 12,
    width: '100%',
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#2A2A2A',
  },
  secondaryButtonText: { fontSize: 16, fontWeight: '600', color: '#888888' },
  contactRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 14,
    paddingHorizontal: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#1A1A1A',
  },
  avatar: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#1A3A5C',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 14,
  },
  avatarText: { fontSize: 18, fontWeight: '600', color: '#FFFFFF' },
  contactInfo: { flex: 1 },
  contactName: { fontSize: 16, fontWeight: '500', color: '#FFFFFF' },
  contactKey: { fontSize: 11, color: '#555555', fontFamily: 'monospace', marginTop: 2 },
  verifiedBadge: { fontSize: 16, color: '#34C759' },
  bottomBar: {
    flexDirection: 'row',
    borderTopWidth: 1,
    borderTopColor: '#1A1A1A',
    paddingBottom: 34,
  },
  bottomButton: { flex: 1, paddingVertical: 14, alignItems: 'center' },
  bottomButtonText: { fontSize: 13, color: '#888888' },
  settingsLink: { padding: 16, alignItems: 'center' },
  settingsLinkText: { fontSize: 14, color: '#555555' },
});
