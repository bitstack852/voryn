import React from 'react';
import { View, Text, StyleSheet, FlatList, TouchableOpacity } from 'react-native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';

type Props = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'Contacts'>;
};

export const ContactsScreen: React.FC<Props> = ({ navigation }) => {
  // TODO Phase 1: Load contacts from SQLCipher via Rust bridge
  const contacts: Array<{ pubkeyHex: string; displayName: string }> = [];

  return (
    <View style={styles.container}>
      {contacts.length === 0 ? (
        <View style={styles.emptyState}>
          <Text style={styles.emptyTitle}>No Contacts Yet</Text>
          <Text style={styles.emptySubtitle}>
            Add a contact by sharing public keys
          </Text>
        </View>
      ) : (
        <FlatList
          data={contacts}
          keyExtractor={(item) => item.pubkeyHex}
          renderItem={({ item }) => (
            <TouchableOpacity
              style={styles.contactRow}
              onPress={() =>
                navigation.navigate('Chat', {
                  contactPubkeyHex: item.pubkeyHex,
                  displayName: item.displayName,
                })
              }
            >
              <Text style={styles.contactName}>{item.displayName}</Text>
              <Text style={styles.contactKey}>
                {item.pubkeyHex.slice(0, 16)}...
              </Text>
            </TouchableOpacity>
          )}
        />
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
  container: {
    flex: 1,
    backgroundColor: '#0D0D0D',
  },
  emptyState: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  emptySubtitle: {
    fontSize: 14,
    color: '#888888',
    marginTop: 8,
  },
  contactRow: {
    paddingVertical: 16,
    paddingHorizontal: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#1A1A1A',
  },
  contactName: {
    fontSize: 16,
    fontWeight: '500',
    color: '#FFFFFF',
  },
  contactKey: {
    fontSize: 12,
    color: '#555555',
    fontFamily: 'monospace',
    marginTop: 4,
  },
  settingsButton: {
    padding: 16,
    alignItems: 'center',
    borderTopWidth: 1,
    borderTopColor: '#1A1A1A',
  },
  settingsButtonText: {
    fontSize: 14,
    color: '#888888',
  },
});
