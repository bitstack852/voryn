import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  TouchableOpacity,
} from 'react-native';

interface Props {
  attemptsRemaining?: number;
  onSubmit: (passcode: string) => Promise<boolean>;
}

export const PasscodeEntryScreen: React.FC<Props> = ({
  attemptsRemaining,
  onSubmit,
}) => {
  const [passcode, setPasscode] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);

  const handleSubmit = async () => {
    if (!passcode) return;
    setIsVerifying(true);
    setError(null);

    const success = await onSubmit(passcode);
    if (!success) {
      setError('Incorrect passcode');
      setPasscode('');
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
        onChangeText={(text) => {
          setPasscode(text);
          setError(null);
        }}
        onSubmitEditing={handleSubmit}
        placeholder="Passcode"
        placeholderTextColor="#555555"
        secureTextEntry
        autoFocus
        maxLength={32}
      />

      {error && <Text style={styles.errorText}>{error}</Text>}

      {attemptsRemaining !== undefined && attemptsRemaining <= 5 && (
        <Text style={styles.warningText}>
          {attemptsRemaining} attempts remaining before data wipe
        </Text>
      )}

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
  container: {
    flex: 1,
    backgroundColor: '#0D0D0D',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 24,
  },
  title: {
    fontSize: 36,
    fontWeight: '700',
    color: '#FFFFFF',
    letterSpacing: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#888888',
    marginTop: 8,
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
  inputError: {
    borderColor: '#FF3B30',
  },
  errorText: {
    color: '#FF3B30',
    fontSize: 13,
    marginTop: 8,
  },
  warningText: {
    color: '#FF9500',
    fontSize: 13,
    marginTop: 8,
    fontWeight: '600',
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
