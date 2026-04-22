import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  TouchableOpacity,
  Alert,
  Animated,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as PasscodeService from '../services/PasscodeService';
import * as VorynBridge from '../services/VorynBridge';
import { colors } from '../theme/colors';

type Nav = NativeStackNavigationProp<RootStackParamList, 'PasscodeLock'>;

export const PasscodeLockScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [passcode, setPasscode] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);

  const offsetBlue = useRef(new Animated.Value(0)).current;
  const offsetRed = useRef(new Animated.Value(0)).current;

  useEffect(() => {
    const glitch = Animated.loop(
      Animated.sequence([
        Animated.delay(2000),
        Animated.parallel([
          Animated.timing(offsetBlue, { toValue: 1, duration: 80, useNativeDriver: true }),
          Animated.timing(offsetRed, { toValue: 1, duration: 80, useNativeDriver: true }),
        ]),
        Animated.parallel([
          Animated.timing(offsetBlue, { toValue: 0, duration: 80, useNativeDriver: true }),
          Animated.timing(offsetRed, { toValue: 0, duration: 80, useNativeDriver: true }),
        ]),
      ]),
    );
    glitch.start();
    return () => glitch.stop();
  }, []);

  const blueX = offsetBlue.interpolate({ inputRange: [0, 1], outputRange: [2, 6] });
  const redX = offsetRed.interpolate({ inputRange: [0, 1], outputRange: [-2, -6] });

  const handleSubmit = async () => {
    if (!passcode) return;
    setIsVerifying(true);
    setError(null);

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
      <View style={styles.stack}>
        <Animated.Text style={[styles.glitch, styles.blue, { transform: [{ translateX: blueX }] }]}>
          VORYN
        </Animated.Text>
        <Animated.Text style={[styles.glitch, styles.red, { transform: [{ translateX: redX }] }]}>
          VORYN
        </Animated.Text>
        <Text style={styles.glitch}>VORYN</Text>
      </View>
      <Text style={styles.subtitle}>// enter passcode</Text>

      <TextInput
        style={[styles.input, error && styles.inputError]}
        value={passcode}
        onChangeText={(text) => { setPasscode(text); setError(null); }}
        onSubmitEditing={handleSubmit}
        placeholder="Passcode"
        placeholderTextColor={colors.textMuted}
        secureTextEntry
        autoFocus
        maxLength={32}
        keyboardType="number-pad"
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
  container: { flex: 1, backgroundColor: '#050608', alignItems: 'center', justifyContent: 'center', padding: 24 },
  stack: { width: 260, height: 48, alignItems: 'center', justifyContent: 'center' },
  glitch: { position: 'absolute', fontSize: 36, fontWeight: '800', letterSpacing: 10, color: colors.textPrimary },
  blue: { color: '#4A9EFF' },
  red: { color: '#FF3B30' },
  subtitle: { marginTop: 20, color: colors.textMuted, fontSize: 11, letterSpacing: 3, fontFamily: 'Menlo' },
  input: {
    backgroundColor: colors.surface, borderRadius: 8, paddingHorizontal: 20, paddingVertical: 16,
    color: colors.textPrimary, fontSize: 24, textAlign: 'center', letterSpacing: 8, width: '100%',
    marginTop: 48, borderWidth: 1, borderColor: colors.borderLight,
  },
  inputError: { borderColor: colors.error },
  errorText: { color: colors.error, fontSize: 13, marginTop: 8, fontFamily: 'Menlo' },
  button: { backgroundColor: colors.textPrimary, paddingVertical: 16, paddingHorizontal: 48, borderRadius: 12, marginTop: 24, width: '100%', alignItems: 'center' },
  buttonText: { fontSize: 16, fontWeight: '600', color: colors.background },
});
