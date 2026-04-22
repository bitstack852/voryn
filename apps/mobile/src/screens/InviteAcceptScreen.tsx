import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import { useRoute, useNavigation } from '@react-navigation/native';
import type { RouteProp } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';
import { colors } from '../theme/colors';

type InviteRoute = RouteProp<RootStackParamList, 'InviteAccept'>;
type Nav = NativeStackNavigationProp<RootStackParamList, 'InviteAccept'>;

export const InviteAcceptScreen: React.FC = () => {
  const route = useRoute<InviteRoute>();
  const navigation = useNavigation<Nav>();
  const { fromPubkeyHex } = route.params;
  const [loading, setLoading] = useState(false);
  const [done, setDone] = useState(false);

  const shortKey = fromPubkeyHex.slice(0, 12) + '…';

  const handleAccept = async () => {
    setLoading(true);
    try {
      await VorynBridge.addContact(fromPubkeyHex);
      setDone(true);
    } finally {
      setLoading(false);
    }
  };

  const handleDecline = () => {
    navigation.goBack();
  };

  if (done) {
    return (
      <View style={styles.container}>
        <Text style={styles.icon}>✓</Text>
        <Text style={styles.title}>Request Sent</Text>
        <Text style={styles.subtitle}>
          Your contact request has been sent. You'll be able to message them once they approve.
        </Text>
        <TouchableOpacity style={styles.primaryButton} onPress={() => navigation.navigate('MainTabs')}>
          <Text style={styles.primaryButtonText}>Done</Text>
        </TouchableOpacity>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.avatar}>
        <Text style={styles.avatarText}>{fromPubkeyHex.slice(0, 2).toUpperCase()}</Text>
      </View>
      <Text style={styles.title}>Invitation</Text>
      <Text style={styles.subtitle}>
        Someone wants to connect with you on Voryn.
      </Text>
      <View style={styles.keyBox}>
        <Text style={styles.keyLabel}>Their key</Text>
        <Text style={styles.keyText}>{shortKey}</Text>
      </View>
      <Text style={styles.note}>
        Accepting sends them a contact request. They'll need to approve before you can message each other.
      </Text>
      <TouchableOpacity
        style={[styles.primaryButton, loading && styles.buttonDisabled]}
        onPress={handleAccept}
        disabled={loading}
      >
        {loading ? (
          <ActivityIndicator color={colors.background} />
        ) : (
          <Text style={styles.primaryButtonText}>Accept Invitation</Text>
        )}
      </TouchableOpacity>
      <TouchableOpacity style={styles.secondaryButton} onPress={handleDecline}>
        <Text style={styles.secondaryButtonText}>Decline</Text>
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1, backgroundColor: colors.background,
    alignItems: 'center', justifyContent: 'center', padding: 32,
  },
  avatar: {
    width: 72, height: 72, borderRadius: 36,
    backgroundColor: colors.accentDark,
    alignItems: 'center', justifyContent: 'center', marginBottom: 20,
  },
  avatarText: { fontSize: 26, fontWeight: '700', color: colors.textPrimary },
  icon: { fontSize: 48, color: colors.success, marginBottom: 16 },
  title: { fontSize: 24, fontWeight: '700', color: colors.textPrimary, marginBottom: 8 },
  subtitle: { fontSize: 15, color: colors.textSecondary, textAlign: 'center', marginBottom: 24 },
  keyBox: {
    backgroundColor: colors.surface, borderRadius: 10, padding: 16,
    width: '100%', marginBottom: 16,
  },
  keyLabel: { fontSize: 11, color: colors.textMuted, textTransform: 'uppercase', letterSpacing: 1, marginBottom: 4 },
  keyText: { fontSize: 14, color: colors.textPrimary, fontFamily: 'Menlo' },
  note: { fontSize: 13, color: colors.textMuted, textAlign: 'center', marginBottom: 32 },
  primaryButton: {
    backgroundColor: colors.textPrimary, paddingVertical: 16,
    borderRadius: 12, alignItems: 'center', width: '100%', marginBottom: 12,
  },
  buttonDisabled: { opacity: 0.5 },
  primaryButtonText: { fontSize: 16, fontWeight: '600', color: colors.background },
  secondaryButton: { paddingVertical: 12 },
  secondaryButtonText: { fontSize: 15, color: colors.textMuted },
});
