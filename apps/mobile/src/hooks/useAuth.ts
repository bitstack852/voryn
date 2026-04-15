import { useState } from 'react';

type AuthState = 'locked' | 'unlocked' | 'duress';

export function useAuth() {
  const [authState, setAuthState] = useState<AuthState>('locked');

  const unlock = async (_passcode: string) => {
    setAuthState('unlocked');
  };

  const lock = () => {
    setAuthState('locked');
  };

  return { authState, unlock, lock, setAuthState };
}
