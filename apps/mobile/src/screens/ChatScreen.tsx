import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TextInput,
  TouchableOpacity,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { useRoute } from '@react-navigation/native';
import type { RouteProp } from '@react-navigation/native';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';
import * as NetworkService from '../services/NetworkService';
// NetworkService used for sendToPeer

type ChatRoute = RouteProp<RootStackParamList, 'Chat'>;

export const ChatScreen: React.FC = () => {
  const route = useRoute<ChatRoute>();
  const [messageText, setMessageText] = useState('');
  const [messages, setMessages] = useState<VorynBridge.StoredMessage[]>([]);
  const [conversationId, setConversationId] = useState<string>('');
  const contactPubkeyHex = route.params.contactPubkeyHex;

  const loadMessages = useCallback(async () => {
    if (!conversationId) return;
    const msgs = await VorynBridge.getMessages(conversationId);
    setMessages(msgs);
  }, [conversationId]);

  useEffect(() => {
    const init = async () => {
      const convId = await VorynBridge.getConversationId(contactPubkeyHex);
      setConversationId(convId);
    };
    init();
  }, [contactPubkeyHex]);

  useEffect(() => {
    loadMessages();
    const interval = setInterval(loadMessages, 2000);
    return () => clearInterval(interval);
  }, [loadMessages]);

  const handleSend = async () => {
    if (!messageText.trim()) return;
    const text = messageText.trim();
    setMessageText('');

    // Store locally first
    const messageId = await VorynBridge.sendMessage(contactPubkeyHex, text);

    // Send via relay if connected
    if (NetworkService.getStatus() === 'connected') {
      NetworkService.sendToPeer(contactPubkeyHex, text, messageId);
    }

    await loadMessages();
  };

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const statusIcon = (status: string) => {
    switch (status) {
      case 'pending': return '\u23F3';
      case 'sent': return '\u2713';
      case 'delivered': return '\u2713\u2713';
      default: return '';
    }
  };

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={90}
    >
      <FlatList
        data={messages}
        keyExtractor={(item) => item.messageId}
        inverted
        contentContainerStyle={styles.messageList}
        renderItem={({ item }) => (
          <View style={[styles.messageBubble, item.isMine ? styles.myMessage : styles.theirMessage]}>
            <Text style={styles.messageText}>{item.plaintext}</Text>
            <View style={styles.messageFooter}>
              <Text style={styles.messageTime}>{formatTime(item.timestamp)}</Text>
              {item.isMine && <Text style={styles.messageStatus}>{statusIcon(item.status)}</Text>}
            </View>
          </View>
        )}
        ListEmptyComponent={
          <View style={styles.emptyState}>
            <Text style={styles.emptyText}>Messages are end-to-end encrypted</Text>
            <Text style={styles.emptySubtext}>Send a message to start the conversation</Text>
          </View>
        }
      />

      <View style={styles.inputContainer}>
        <TextInput
          style={styles.textInput}
          value={messageText}
          onChangeText={setMessageText}
          placeholder="Message"
          placeholderTextColor="#555555"
          multiline
          maxLength={4096}
        />
        <TouchableOpacity
          style={[styles.sendButton, !messageText.trim() && styles.sendButtonDisabled]}
          onPress={handleSend}
          disabled={!messageText.trim()}
        >
          <Text style={[styles.sendButtonText, !messageText.trim() && { color: '#555555' }]}>
            Send
          </Text>
        </TouchableOpacity>
      </View>
    </KeyboardAvoidingView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D' },
  messageList: { padding: 16, flexGrow: 1 },
  messageBubble: { maxWidth: '80%', paddingVertical: 10, paddingHorizontal: 14, borderRadius: 18, marginVertical: 3 },
  myMessage: { backgroundColor: '#1A3A5C', alignSelf: 'flex-end', borderBottomRightRadius: 4 },
  theirMessage: { backgroundColor: '#1A1A1A', alignSelf: 'flex-start', borderBottomLeftRadius: 4 },
  messageText: { fontSize: 15, color: '#FFFFFF', lineHeight: 20 },
  messageFooter: { flexDirection: 'row', justifyContent: 'flex-end', marginTop: 4 },
  messageTime: { fontSize: 11, color: '#888888' },
  messageStatus: { fontSize: 11, color: '#888888', marginLeft: 4 },
  emptyState: { flex: 1, alignItems: 'center', justifyContent: 'center', paddingVertical: 60 },
  emptyText: { fontSize: 14, color: '#555555' },
  emptySubtext: { fontSize: 12, color: '#333333', marginTop: 4 },
  inputContainer: {
    flexDirection: 'row', alignItems: 'flex-end', paddingHorizontal: 12,
    paddingVertical: 8, borderTopWidth: 1, borderTopColor: '#1A1A1A',
  },
  textInput: {
    flex: 1, backgroundColor: '#1A1A1A', borderRadius: 22, paddingHorizontal: 18,
    paddingVertical: 10, color: '#FFFFFF', fontSize: 15, maxHeight: 120,
  },
  sendButton: { marginLeft: 8, paddingVertical: 10, paddingHorizontal: 16 },
  sendButtonDisabled: { opacity: 0.3 },
  sendButtonText: { fontSize: 15, fontWeight: '600', color: '#4A9EFF' },
});
