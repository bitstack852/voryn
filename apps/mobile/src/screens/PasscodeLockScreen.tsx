import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as PasscodeService from '../services/PasscodeService';
import * as VorynBridge from '../services/VorynBridge';

type Nav = NativeStackNavigationProp<RootStackParamList, 'PasscodeLock'>;

export const PasscodeLockScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [passcode, setPasscode] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);

  const handleSubmit = async () => {
    if (!passcode) return;
    setIsVerifying(true);
    setError(null);

    // Check if we should wipe
    if (await PasscodeService.shouldWipe()) {
      await VorynBridge.deleteIdentity();
      await PasscodeService.removePasscode();
      Alert.alert('Data Wiped', 'Too many failed attempts. All data has been deleted.');
      navigation.reset({ index: 0, routes: [{ name: 'Onboarding' }] });
      return;
    }

    const valid = await PasscodeService.verifyPasscode(passcode);
    if (valid) {
      navigation.reset({ index: 0, routes: [{ name: 'Contacts' }] });
    } else {
      const remaining = await PasscodeService.getRemainingAttempts();
      setError(`Incorrect passcode. ${remaining} attempts remaining.`);
      setPasscode('');

      if (remaining <= 0) {
        await VorynBridge.deleteIdentity();
        await PasscodeService.removePasscode();
        Alert.alert('Data Wiped', 'Too many failed attempts. All data has been deleted.');
        navigation.reset({ index: 0, routes: [{ name: 'Onboarding' }] });
      }
    }
    setIsVerifying(false);
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Voryn</Text>
      <Text style={styles.subtitle}>Enter passcode to unlock</Text>

      <TextInput
        style={[styles.input, error && styles.inputError]}
        value={passcode}
        onChangeText={(text) => { setPasscode(text); setError(null); }}
        onSubmitEditing={handleSubmit}
        placeholder="Passcode"
        placeholderTextColor="#555555"
        secureTextEntry
        autoFocus
        maxLength={32}
      />

      {error && <Text style={styles.errorText}>{error}</Text>}

      <TouchableOpacity
        style={styles.button}
        onPress={handleSubmit}
        disabled={isVerifying || !passcode}
      >
        <Text style={styles.buttonText}>
          {isVerifying ? 'Verifying...' : 'Unlock'}
        </Text>
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D', alignItems: 'center', justifyContent: 'center', padding: 24 },
  title: { fontSize: 36, fontWeight: '700', color: '#FFFFFF', letterSpacing: 4 },
  subtitle: { fontSize: 14, color: '#888888', marginTop: 8 },
  input: {
    backgroundColor: '#1A1A1A', borderRadius: 8, paddingHorizontal: 20, paddingVertical: 16,
    color: '#FFFFFF', fontSize: 24, textAlign: 'center', letterSpacing: 8, width: '100%',
    marginTop: 40, borderWidth: 1, borderColor: '#333333',
  },
  inputError: { borderColor: '#FF3B30' },
  errorText: { color: '#FF3B30', fontSize: 13, marginTop: 8 },
  button: { backgroundColor: '#FFFFFF', paddingVertical: 16, paddingHorizontal: 48, borderRadius: 12, marginTop: 24, width: '100%', alignItems: 'center' },
  buttonText: { fontSize: 16, fontWeight: '600', color: '#0D0D0D' },
});
