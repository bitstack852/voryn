import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  ActivityIndicator,
  StatusBar,
  Animated,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';
import { colors } from '../theme/colors';

type Nav = NativeStackNavigationProp<RootStackParamList, 'Onboarding'>;

export const OnboardingScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [isLoading, setIsLoading] = useState(true);
  const [isCreating, setIsCreating] = useState(false);
  const offsetBlue = useRef(new Animated.Value(0)).current;
  const offsetRed = useRef(new Animated.Value(0)).current;

  useEffect(() => {
    const glitch = Animated.loop(
      Animated.sequence([
        Animated.delay(2200),
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

  useEffect(() => {
    const checkExisting = async () => {
      const existing = await VorynBridge.loadIdentity();
      if (existing) {
        navigation.replace('MainTabs');
        return;
      }
      setIsLoading(false);
    };
    checkExisting();
  }, [navigation]);

  const handleCreateIdentity = async () => {
    setIsCreating(true);
    try {
      await VorynBridge.generateIdentity();
      navigation.replace('PasscodeSetup');
    } catch (err) {
      console.error('Failed to create identity:', err);
      setIsCreating(false);
    }
  };

  if (isLoading) {
    return (
      <View style={styles.container}>
        <StatusBar barStyle="light-content" backgroundColor="#0D0D0D" />
        <ActivityIndicator size="large" color="#FFFFFF" />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor="#0D0D0D" />

      <View style={styles.header}>
        <View style={styles.glitchStack}>
          <Animated.Text style={[styles.glitchText, styles.glitchBlue, { transform: [{ translateX: blueX }] }]}>
            VORYN
          </Animated.Text>
          <Animated.Text style={[styles.glitchText, styles.glitchRed, { transform: [{ translateX: redX }] }]}>
            VORYN
          </Animated.Text>
          <Text style={styles.glitchText}>VORYN</Text>
        </View>
        <Text style={styles.tagline}>PEER · MESH · UNCENSORABLE</Text>
      </View>

      <View style={styles.buttonContainer}>
        <TouchableOpacity
          style={styles.button}
          onPress={handleCreateIdentity}
          disabled={isCreating}
          activeOpacity={0.8}
        >
          {isCreating ? (
            <ActivityIndicator color="#0D0D0D" />
          ) : (
            <Text style={styles.buttonText}>Create Identity</Text>
          )}
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.button, styles.secondaryButton]}
          activeOpacity={0.8}
        >
          <Text style={[styles.buttonText, styles.secondaryButtonText]}>
            I Have an Invite
          </Text>
        </TouchableOpacity>

        <Text style={styles.versionText}>v0.1.0</Text>
      </View>
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
  header: { alignItems: 'center', marginTop: 80 },
  glitchStack: { width: 260, height: 52, alignItems: 'center', justifyContent: 'center' },
  glitchText: { position: 'absolute', fontSize: 36, fontWeight: '800', letterSpacing: 10, color: colors.textPrimary },
  glitchBlue: { color: '#4A9EFF' },
  glitchRed: { color: '#FF3B30' },
  tagline: { fontSize: 11, letterSpacing: 3, color: colors.accent, marginTop: 18, fontFamily: 'Menlo' },
  buttonContainer: { width: '100%', alignItems: 'center' },
  button: {
    backgroundColor: '#FFFFFF',
    paddingVertical: 16,
    paddingHorizontal: 48,
    borderRadius: 12,
    marginTop: 16,
    width: '100%',
    alignItems: 'center',
  },
  buttonText: { fontSize: 16, fontWeight: '600', color: '#0D0D0D' },
  secondaryButton: {
    backgroundColor: 'transparent',
    borderWidth: 1,
    borderColor: '#2A2A2A',
  },
  secondaryButtonText: { color: '#666666' },
  versionText: { fontSize: 11, color: '#333333', marginTop: 32 },
});
