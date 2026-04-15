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

type Nav = NativeStackNavigationProp<RootStackParamList, 'PasscodeSetup'>;

export const PasscodeSetupPrompt: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [step, setStep] = useState<'create' | 'confirm'>('create');
  const [passcode, setPasscode] = useState('');
  const [confirmPasscode, setConfirmPasscode] = useState('');

  const handleCreate = () => {
    if (passcode.length < 4) {
      Alert.alert('Too Short', 'Passcode must be at least 4 characters.');
      return;
    }
    setStep('confirm');
  };

  const handleConfirm = async () => {
    if (confirmPasscode !== passcode) {
      Alert.alert('Mismatch', 'Passcodes do not match.');
      setConfirmPasscode('');
      return;
    }

    await PasscodeService.setPasscode(passcode);
    navigation.reset({ index: 0, routes: [{ name: 'Contacts' }] });
  };

  const handleSkip = () => {
    navigation.reset({ index: 0, routes: [{ name: 'Contacts' }] });
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>
        {step === 'create' ? 'Set a Passcode' : 'Confirm Passcode'}
      </Text>
      <Text style={styles.subtitle}>
        {step === 'create'
          ? 'Protect your messages with a passcode'
          : 'Enter your passcode again'}
      </Text>

      <TextInput
        style={styles.input}
        value={step === 'create' ? passcode : confirmPasscode}
        onChangeText={step === 'create' ? setPasscode : setConfirmPasscode}
        placeholder="Passcode"
        placeholderTextColor="#555555"
        secureTextEntry
        autoFocus
        maxLength={32}
        onSubmitEditing={step === 'create' ? handleCreate : handleConfirm}
      />

      <TouchableOpacity
        style={styles.button}
        onPress={step === 'create' ? handleCreate : handleConfirm}
      >
        <Text style={styles.buttonText}>
          {step === 'create' ? 'Continue' : 'Set Passcode'}
        </Text>
      </TouchableOpacity>

      {step === 'create' && (
        <TouchableOpacity style={styles.skipButton} onPress={handleSkip}>
          <Text style={styles.skipText}>Skip for now</Text>
        </TouchableOpacity>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D', alignItems: 'center', justifyContent: 'center', padding: 24 },
  title: { fontSize: 24, fontWeight: '600', color: '#FFFFFF' },
  subtitle: { fontSize: 14, color: '#888888', marginTop: 8, textAlign: 'center' },
  input: {
    backgroundColor: '#1A1A1A', borderRadius: 8, paddingHorizontal: 20, paddingVertical: 16,
    color: '#FFFFFF', fontSize: 24, textAlign: 'center', letterSpacing: 8, width: '100%',
    marginTop: 40, borderWidth: 1, borderColor: '#333333',
  },
  button: { backgroundColor: '#FFFFFF', paddingVertical: 16, paddingHorizontal: 48, borderRadius: 12, marginTop: 24, width: '100%', alignItems: 'center' },
  buttonText: { fontSize: 16, fontWeight: '600', color: '#0D0D0D' },
  skipButton: { marginTop: 16, paddingVertical: 12 },
  skipText: { fontSize: 14, color: '#555555' },
});
