import { useState } from 'react';

interface Message {
  messageId: string;
  conversationId: string;
  senderPubkey: string;
  content: string;
  timestamp: number;
  status: 'pending' | 'sent' | 'delivered' | 'failed';
}

export function useMessages(_conversationId: string) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const loadMessages = async () => {
    setIsLoading(true);
    setIsLoading(false);
  };

  const sendMessage = async (_plaintext: string) => {};

  return { messages, isLoading, loadMessages, sendMessage, setMessages };
}
