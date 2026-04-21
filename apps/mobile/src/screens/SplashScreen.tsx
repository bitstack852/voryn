import React, { useEffect, useRef } from 'react';
import { View, Text, StyleSheet, Animated, Easing } from 'react-native';
import { colors } from '../theme/colors';

interface SplashScreenProps {
  onFinish: () => void;
}

/**
 * Voryn splash — "Redacted" direction.
 * Three VORYN wordmarks stacked; blue and red copies jitter on a
 * loop to read as RGB channel glitch. Tagline fades in, whole
 * screen fades out after ~2.2s into the normal flow.
 */
export const SplashScreen: React.FC<SplashScreenProps> = ({ onFinish }) => {
  const offsetBlue = useRef(new Animated.Value(0)).current;
  const offsetRed = useRef(new Animated.Value(0)).current;
  const fade = useRef(new Animated.Value(1)).current;
  const tagOpacity = useRef(new Animated.Value(0)).current;

  useEffect(() => {
    // Tagline fades in after the wordmark "settles"
    Animated.timing(tagOpacity, {
      toValue: 1,
      duration: 500,
      delay: 800,
      useNativeDriver: true,
    }).start();

    // Occasional glitch burst
    const glitch = Animated.loop(
      Animated.sequence([
        Animated.delay(1400),
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

    const timer = setTimeout(() => {
      Animated.timing(fade, {
        toValue: 0,
        duration: 400,
        easing: Easing.out(Easing.quad),
        useNativeDriver: true,
      }).start(() => {
        glitch.stop();
        onFinish();
      });
    }, 2200);

    return () => {
      clearTimeout(timer);
      glitch.stop();
    };
  }, []);

  const blueX = offsetBlue.interpolate({ inputRange: [0, 1], outputRange: [2, 6] });
  const redX = offsetRed.interpolate({ inputRange: [0, 1], outputRange: [-2, -6] });

  return (
    <Animated.View style={[styles.container, { opacity: fade }]}>
      <View style={styles.stack}>
        <Animated.Text
          style={[styles.glitch, styles.blue, { transform: [{ translateX: blueX }] }]}
        >
          VORYN
        </Animated.Text>
        <Animated.Text
          style={[styles.glitch, styles.red, { transform: [{ translateX: redX }] }]}
        >
          VORYN
        </Animated.Text>
        <Text style={styles.glitch}>VORYN</Text>
      </View>
      <Animated.Text style={[styles.tag, { opacity: tagOpacity }]}>
        // signal not traceable
      </Animated.Text>
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#050608',
    alignItems: 'center',
    justifyContent: 'center',
  },
  stack: {
    width: 260,
    height: 48,
    alignItems: 'center',
    justifyContent: 'center',
  },
  glitch: {
    position: 'absolute',
    fontSize: 36,
    fontWeight: '800',
    letterSpacing: 10,
    color: colors.textPrimary,
  },
  blue: { color: '#4A9EFF' },
  red: { color: '#FF3B30' },
  tag: {
    marginTop: 28,
    color: '#555',
    fontSize: 11,
    letterSpacing: 3,
    fontFamily: 'Menlo',
  },
});
