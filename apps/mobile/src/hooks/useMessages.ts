import { useState } from 'react';
import type { Message } from '@voryn/shared-types';

/**
 * Hook for managing messages in a conversation.
 * Full implementation in Phase 1 when encrypted messaging is integrated.
 */
export function useMessages(_conversationId: string) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const loadMessages = async () => {
    setIsLoading(true);
    // TODO Phase 1: Load from SQLCipher via Rust bridge
    setIsLoading(false);
  };

  const sendMessage = async (_plaintext: string) => {
    // TODO Phase 1: Encrypt and send via Rust bridge
  };

  return { messages, isLoading, loadMessages, sendMessage, setMessages };
}
