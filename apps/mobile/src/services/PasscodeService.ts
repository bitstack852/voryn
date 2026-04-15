/**
 * PasscodeService — manages app-level passcode lock.
 *
 * Stores a hashed passcode in AsyncStorage. When the Rust bridge is connected,
 * this will use Argon2id for key derivation instead of simple hashing.
 */

import AsyncStorage from '@react-native-async-storage/async-storage';

const PASSCODE_KEY = '@voryn/passcode_hash';
const ATTEMPT_COUNT_KEY = '@voryn/passcode_attempts';
const MAX_ATTEMPTS = 10;

// Simple hash for now — will be replaced with Argon2id via Rust bridge
function simpleHash(input: string): string {
  let hash = 0;
  for (let i = 0; i < input.length; i++) {
    const char = input.charCodeAt(i);
    hash = ((hash << 5) - hash + char) | 0;
  }
  // Mix it more
  for (let i = 0; i < 1000; i++) {
    hash = ((hash << 5) - hash + (hash >> 3)) | 0;
  }
  return Math.abs(hash).toString(16).padStart(8, '0');
}

export async function hasPasscode(): Promise<boolean> {
  const hash = await AsyncStorage.getItem(PASSCODE_KEY);
  return hash !== null;
}

export async function setPasscode(passcode: string): Promise<void> {
  const hash = simpleHash(passcode);
  await AsyncStorage.setItem(PASSCODE_KEY, hash);
  await AsyncStorage.setItem(ATTEMPT_COUNT_KEY, '0');
}

export async function verifyPasscode(passcode: string): Promise<boolean> {
  const storedHash = await AsyncStorage.getItem(PASSCODE_KEY);
  if (!storedHash) return true; // No passcode set

  const inputHash = simpleHash(passcode);
  if (inputHash === storedHash) {
    await AsyncStorage.setItem(ATTEMPT_COUNT_KEY, '0');
    return true;
  }

  // Track failed attempts
  const attempts = parseInt(await AsyncStorage.getItem(ATTEMPT_COUNT_KEY) || '0', 10);
  await AsyncStorage.setItem(ATTEMPT_COUNT_KEY, (attempts + 1).toString());

  return false;
}

export async function getFailedAttempts(): Promise<number> {
  const attempts = await AsyncStorage.getItem(ATTEMPT_COUNT_KEY);
  return parseInt(attempts || '0', 10);
}

export async function getRemainingAttempts(): Promise<number> {
  const attempts = await getFailedAttempts();
  return MAX_ATTEMPTS - attempts;
}

export async function shouldWipe(): Promise<boolean> {
  const attempts = await getFailedAttempts();
  return attempts >= MAX_ATTEMPTS;
}

export async function removePasscode(): Promise<void> {
  await AsyncStorage.removeItem(PASSCODE_KEY);
  await AsyncStorage.removeItem(ATTEMPT_COUNT_KEY);
}

export async function changePasscode(oldPasscode: string, newPasscode: string): Promise<boolean> {
  const valid = await verifyPasscode(oldPasscode);
  if (!valid) return false;
  await setPasscode(newPasscode);
  return true;
}
