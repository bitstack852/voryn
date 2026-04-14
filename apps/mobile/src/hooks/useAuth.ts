import { useState } from 'react';
import type { AuthState } from '@voryn/shared-types';

/**
 * Hook for managing app authentication state.
 * Full implementation in Phase 2 when passcode layer is integrated.
 */
export function useAuth() {
  const [authState, setAuthState] = useState<AuthState>('locked');

  const unlock = async (_passcode: string) => {
    // TODO Phase 2: Verify passcode via Rust bridge
    setAuthState('unlocked');
  };

  const lock = () => {
    setAuthState('locked');
  };

  return { authState, unlock, lock, setAuthState };
}
