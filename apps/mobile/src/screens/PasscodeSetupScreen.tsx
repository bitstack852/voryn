import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  TouchableOpacity,
  Alert,
} from 'react-native';

interface Props {
  onComplete: (passcode: string) => void;
}

export const PasscodeSetupScreen: React.FC<Props> = ({ onComplete }) => {
  const [step, setStep] = useState<'create' | 'confirm'>('create');
  const [passcode, setPasscode] = useState('');
  const [confirm, setConfirm] = useState('');

  const handleCreate = () => {
    if (passcode.length < 6) {
      Alert.alert('Too Short', 'Passcode must be at least 6 characters.');
      return;
    }
    setStep('confirm');
  };

  const handleConfirm = () => {
    if (confirm !== passcode) {
      Alert.alert('Mismatch', 'Passcodes do not match. Please try again.');
      setConfirm('');
      return;
    }
    onComplete(passcode);
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>
        {step === 'create' ? 'Set Passcode' : 'Confirm Passcode'}
      </Text>
      <Text style={styles.subtitle}>
        {step === 'create'
          ? 'This passcode protects your encryption keys'
          : 'Enter your passcode again to confirm'}
      </Text>

      <TextInput
        style={styles.input}
        value={step === 'create' ? passcode : confirm}
        onChangeText={step === 'create' ? setPasscode : setConfirm}
        placeholder="Enter passcode"
        placeholderTextColor="#555555"
        secureTextEntry
        autoFocus
        maxLength={32}
      />

      <TouchableOpacity
        style={styles.button}
        onPress={step === 'create' ? handleCreate : handleConfirm}
      >
        <Text style={styles.buttonText}>
          {step === 'create' ? 'Continue' : 'Set Passcode'}
        </Text>
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0D0D0D',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 24,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  subtitle: {
    fontSize: 14,
    color: '#888888',
    marginTop: 8,
    textAlign: 'center',
  },
  input: {
    backgroundColor: '#1A1A1A',
    borderRadius: 8,
    paddingHorizontal: 20,
    paddingVertical: 16,
    color: '#FFFFFF',
    fontSize: 24,
    textAlign: 'center',
    letterSpacing: 8,
    width: '100%',
    marginTop: 40,
    borderWidth: 1,
    borderColor: '#333333',
  },
  button: {
    backgroundColor: '#FFFFFF',
    paddingVertical: 16,
    paddingHorizontal: 48,
    borderRadius: 8,
    marginTop: 24,
    width: '100%',
    alignItems: 'center',
  },
  buttonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#0D0D0D',
  },
});
