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
  Alert,
} from 'react-native';
import { useRoute } from '@react-navigation/native';
import type { RouteProp } from '@react-navigation/native';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as NetworkService from '../services/NetworkService';
import * as VorynBridge from '../services/VorynBridge';
import { colors } from '../theme/colors';

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
      await VorynBridge.markConversationRead(convId);
    };
    init();
  }, [contactPubkeyHex]);

  useEffect(() => {
    loadMessages();
    const interval = setInterval(loadMessages, 2000);
    return () => clearInterval(interval);
  }, [loadMessages]);

  // Reload when a message arrives from this contact via relay
  useEffect(() => {
    return NetworkService.onMessage((from) => {
      if (from === contactPubkeyHex) loadMessages();
    });
  }, [contactPubkeyHex, loadMessages]);

  // Update tick to delivered when relay ACKs our outbound message
  useEffect(() => {
    return NetworkService.onAck(async (messageId) => {
      await VorynBridge.updateMessageStatus(messageId, 'delivered');
      loadMessages();
    });
  }, [loadMessages]);

  const handleSend = async () => {
    if (!messageText.trim()) return;
    const text = messageText.trim();
    setMessageText('');
    await VorynBridge.sendMessage(contactPubkeyHex, text);
    await loadMessages();
  };

  const handleDeleteMessage = (messageId: string) => {
    Alert.alert('Delete Message', 'Remove this message from your device?', [
      { text: 'Cancel', style: 'cancel' },
      {
        text: 'Delete',
        style: 'destructive',
        onPress: async () => {
          await VorynBridge.deleteMessage(messageId);
          await loadMessages();
        },
      },
    ]);
  };

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const StatusTick: React.FC<{ status: string }> = ({ status }) => {
    if (status === 'pending') {
      return <Text style={[styles.tick, styles.tickMuted]}>✓</Text>;
    }
    if (status === 'sent') {
      return <Text style={[styles.tick, styles.tickGrey]}>✓</Text>;
    }
    if (status === 'delivered') {
      return <Text style={[styles.tick, styles.tickGrey]}>✓✓</Text>;
    }
    if (status === 'failed') {
      return <Text style={[styles.tick, styles.tickFailed]}>✗</Text>;
    }
    return null;
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
          <TouchableOpacity
            activeOpacity={0.8}
            onLongPress={() => handleDeleteMessage(item.messageId)}
            delayLongPress={400}
          >
            <View style={[styles.messageBubble, item.isMine ? styles.myMessage : styles.theirMessage]}>
              <Text style={styles.messageText}>{item.plaintext}</Text>
              <View style={styles.messageFooter}>
                <Text style={styles.messageTime}>{formatTime(item.timestamp)}</Text>
                {item.isMine && <StatusTick status={item.status} />}
              </View>
            </View>
          </TouchableOpacity>
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
          placeholderTextColor={colors.textMuted}
          multiline
          maxLength={4096}
        />
        <TouchableOpacity
          style={[styles.sendButton, !messageText.trim() && styles.sendButtonDisabled]}
          onPress={handleSend}
          disabled={!messageText.trim()}
        >
          <Text style={[styles.sendButtonText, !messageText.trim() && { color: colors.textMuted }]}>
            Send
          </Text>
        </TouchableOpacity>
      </View>
    </KeyboardAvoidingView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: colors.background },
  messageList: { padding: 16, flexGrow: 1 },
  messageBubble: {
    maxWidth: '80%',
    paddingVertical: 10,
    paddingHorizontal: 14,
    borderRadius: 18,
    marginVertical: 3,
  },
  myMessage: { backgroundColor: colors.accentDark, alignSelf: 'flex-end', borderBottomRightRadius: 4 },
  theirMessage: { backgroundColor: colors.surface, alignSelf: 'flex-start', borderBottomLeftRadius: 4 },
  messageText: { fontSize: 15, color: colors.textPrimary, lineHeight: 20 },
  messageFooter: { flexDirection: 'row', justifyContent: 'flex-end', alignItems: 'center', marginTop: 4 },
  messageTime: { fontSize: 11, color: colors.textSecondary },
  tick: { fontSize: 11, marginLeft: 4 },
  tickMuted: { color: colors.border },
  tickGrey: { color: colors.textSecondary },
  tickFailed: { color: '#FF3B30' },
  emptyState: { flex: 1, alignItems: 'center', justifyContent: 'center', paddingVertical: 60 },
  emptyText: { fontSize: 14, color: colors.textMuted },
  emptySubtext: { fontSize: 12, color: colors.surfaceLight, marginTop: 4 },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  textInput: {
    flex: 1,
    backgroundColor: colors.surface,
    borderRadius: 22,
    paddingHorizontal: 18,
    paddingVertical: 10,
    color: colors.textPrimary,
    fontSize: 15,
    maxHeight: 120,
  },
  sendButton: { marginLeft: 8, paddingVertical: 10, paddingHorizontal: 16 },
  sendButtonDisabled: { opacity: 0.3 },
  sendButtonText: { fontSize: 15, fontWeight: '600', color: colors.accent },
});
