import React, { useState, useCallback } from 'react';
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

type Nav = NativeStackNavigationProp<RootStackParamList, 'Contacts'>;

interface ContactItem {
  publicKeyHex: string;
  displayName: string | null;
  lastSeen: string | null;
  isVerified: boolean;
}

export const ContactsScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [contacts, setContacts] = useState<ContactItem[]>([]);

  const loadContacts = useCallback(async () => {
    const result = await VorynBridge.getContacts();
    setContacts(result);
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
          <Text style={styles.emptyTitle}>No Contacts Yet</Text>
          <Text style={styles.emptySubtitle}>
            Add a contact by sharing public keys
          </Text>
          <TouchableOpacity
            style={styles.addButton}
            onPress={() => navigation.navigate('AddContact')}
          >
            <Text style={styles.addButtonText}>Add Contact</Text>
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
              >
                <View style={styles.contactInfo}>
                  <Text style={styles.contactName}>
                    {item.displayName ?? 'Unknown'}
                  </Text>
                  <Text style={styles.contactKey}>
                    {item.publicKeyHex.slice(0, 16)}...
                  </Text>
                </View>
                {item.isVerified && (
                  <Text style={styles.verifiedBadge}>Verified</Text>
                )}
              </TouchableOpacity>
            )}
          />
          <TouchableOpacity
            style={styles.floatingAdd}
            onPress={() => navigation.navigate('AddContact')}
          >
            <Text style={styles.floatingAddText}>+</Text>
          </TouchableOpacity>
        </>
      )}

      <TouchableOpacity
        style={styles.settingsButton}
        onPress={() => navigation.navigate('Settings')}
      >
        <Text style={styles.settingsButtonText}>Settings</Text>
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D' },
  emptyState: { flex: 1, alignItems: 'center', justifyContent: 'center' },
  emptyTitle: { fontSize: 20, fontWeight: '600', color: '#FFFFFF' },
  emptySubtitle: { fontSize: 14, color: '#888888', marginTop: 8 },
  addButton: {
    marginTop: 24,
    backgroundColor: '#FFFFFF',
    paddingVertical: 12,
    paddingHorizontal: 32,
    borderRadius: 8,
  },
  addButtonText: { fontSize: 15, fontWeight: '600', color: '#0D0D0D' },
  contactRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 16,
    paddingHorizontal: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#1A1A1A',
  },
  contactInfo: { flex: 1 },
  contactName: { fontSize: 16, fontWeight: '500', color: '#FFFFFF' },
  contactKey: { fontSize: 12, color: '#555555', fontFamily: 'monospace', marginTop: 4 },
  verifiedBadge: { fontSize: 11, color: '#34C759', fontWeight: '600' },
  floatingAdd: {
    position: 'absolute',
    right: 20,
    bottom: 80,
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#FFFFFF',
    alignItems: 'center',
    justifyContent: 'center',
  },
  floatingAddText: { fontSize: 28, fontWeight: '300', color: '#0D0D0D', marginTop: -2 },
  settingsButton: {
    padding: 16,
    alignItems: 'center',
    borderTopWidth: 1,
    borderTopColor: '#1A1A1A',
  },
  settingsButtonText: { fontSize: 14, color: '#888888' },
});
