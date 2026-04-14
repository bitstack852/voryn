import React from 'react';
import { View, Text, StyleSheet, ScrollView } from 'react-native';

export const SettingsScreen: React.FC = () => {
  // TODO Phase 1: Display identity public key, network status
  // TODO Phase 2: Passcode settings, biometric toggle
  // TODO Phase 3: Security settings (attempt limit, duress, time lock)

  return (
    <ScrollView style={styles.container}>
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Identity</Text>
        <Text style={styles.label}>Public Key</Text>
        <Text style={styles.value}>Not yet generated</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Network</Text>
        <Text style={styles.label}>Status</Text>
        <Text style={styles.value}>Disconnected</Text>
        <Text style={styles.label}>Peers</Text>
        <Text style={styles.value}>0</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Security</Text>
        <Text style={styles.label}>Passcode</Text>
        <Text style={styles.value}>Not configured</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>About</Text>
        <Text style={styles.label}>Version</Text>
        <Text style={styles.value}>0.1.0 (Phase 0)</Text>
        <Text style={styles.label}>Build</Text>
        <Text style={styles.value}>Development</Text>
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
});
