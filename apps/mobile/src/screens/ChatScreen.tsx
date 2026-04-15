import React, { useState } from 'react';
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

type ChatRoute = RouteProp<RootStackParamList, 'Chat'>;

interface ChatMessage {
  id: string;
  text: string;
  isMine: boolean;
  timestamp: number;
  status: 'pending' | 'sent' | 'delivered';
}

export const ChatScreen: React.FC = () => {
  const route = useRoute<ChatRoute>();
  const [messageText, setMessageText] = useState('');
  const [messages] = useState<ChatMessage[]>([]);
  const _contactPubkeyHex = route.params.contactPubkeyHex;

  const handleSend = () => {
    if (!messageText.trim()) return;
    setMessageText('');
  };

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={90}
    >
      <FlatList
        data={messages}
        keyExtractor={(item) => item.id}
        inverted
        contentContainerStyle={styles.messageList}
        renderItem={({ item }) => (
          <View
            style={[
              styles.messageBubble,
              item.isMine ? styles.myMessage : styles.theirMessage,
            ]}
          >
            <Text style={styles.messageText}>{item.text}</Text>
          </View>
        )}
        ListEmptyComponent={
          <View style={styles.emptyState}>
            <Text style={styles.emptyText}>
              Messages are end-to-end encrypted
            </Text>
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
        <TouchableOpacity style={styles.sendButton} onPress={handleSend}>
          <Text style={styles.sendButtonText}>Send</Text>
        </TouchableOpacity>
      </View>
    </KeyboardAvoidingView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D0D0D' },
  messageList: { padding: 16, flexGrow: 1 },
  messageBubble: {
    maxWidth: '75%',
    paddingVertical: 10,
    paddingHorizontal: 14,
    borderRadius: 16,
    marginVertical: 4,
  },
  myMessage: {
    backgroundColor: '#1A3A5C',
    alignSelf: 'flex-end',
    borderBottomRightRadius: 4,
  },
  theirMessage: {
    backgroundColor: '#1A1A1A',
    alignSelf: 'flex-start',
    borderBottomLeftRadius: 4,
  },
  messageText: { fontSize: 15, color: '#FFFFFF', lineHeight: 20 },
  emptyState: { flex: 1, alignItems: 'center', justifyContent: 'center', paddingVertical: 40 },
  emptyText: { fontSize: 13, color: '#555555' },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderTopWidth: 1,
    borderTopColor: '#1A1A1A',
  },
  textInput: {
    flex: 1,
    backgroundColor: '#1A1A1A',
    borderRadius: 20,
    paddingHorizontal: 16,
    paddingVertical: 10,
    color: '#FFFFFF',
    fontSize: 15,
    maxHeight: 120,
  },
  sendButton: { marginLeft: 8, paddingVertical: 10, paddingHorizontal: 16 },
  sendButtonText: { fontSize: 15, fontWeight: '600', color: '#4A9EFF' },
});
