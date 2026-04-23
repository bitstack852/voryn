import React, { useCallback, useRef, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  Animated,
} from 'react-native';
import { Swipeable } from 'react-native-gesture-handler';
import { useNavigation, useFocusEffect } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';
import { colors } from '../theme/colors';

type Nav = NativeStackNavigationProp<RootStackParamList, 'Chat'>;

export const ChatsScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [conversations, setConversations] = useState<VorynBridge.Conversation[]>([]);
  const swipeableRefs = useRef<Map<string, Swipeable>>(new Map());

  const load = useCallback(async () => {
    const convs = await VorynBridge.getConversations();
    setConversations(convs);
  }, []);

  const handleDeleteConversation = async (conversationId: string) => {
    await VorynBridge.deleteConversation(conversationId);
    await load();
  };

  useFocusEffect(useCallback(() => {
    load();
    // Close any open swipeables when returning to screen
    swipeableRefs.current.forEach((ref) => ref?.close());
  }, [load]));

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    const now = new Date();
    const isToday = d.toDateString() === now.toDateString();
    if (isToday) return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
  };

  const initials = (name: string | null, pubkey: string) => {
    if (name) return name[0].toUpperCase();
    return pubkey.slice(0, 2).toUpperCase();
  };

  const renderRightActions = (
    progress: Animated.AnimatedInterpolation<number>,
    conversationId: string,
  ) => {
    const translateX = progress.interpolate({
      inputRange: [0, 1],
      outputRange: [80, 0],
    });
    return (
      <Animated.View style={[styles.deleteAction, { transform: [{ translateX }] }]}>
        <TouchableOpacity
          style={styles.deleteButton}
          onPress={() => handleDeleteConversation(conversationId)}
        >
          <Text style={styles.deleteText}>Delete</Text>
        </TouchableOpacity>
      </Animated.View>
    );
  };

  if (conversations.length === 0) {
    return (
      <View style={styles.emptyContainer}>
        <Text style={styles.emptyTitle}>No messages yet</Text>
        <Text style={styles.emptySubtitle}>
          Add a contact and start a conversation
        </Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <FlatList
        data={conversations}
        keyExtractor={(item) => item.conversationId}
        renderItem={({ item }) => (
          <Swipeable
            ref={(ref) => {
              if (ref) swipeableRefs.current.set(item.conversationId, ref);
              else swipeableRefs.current.delete(item.conversationId);
            }}
            renderRightActions={(progress) => renderRightActions(progress, item.conversationId)}
            rightThreshold={40}
            overshootRight={false}
          >
            <TouchableOpacity
              style={styles.row}
              onPress={() => {
                swipeableRefs.current.get(item.conversationId)?.close();
                navigation.navigate('Chat', {
                  contactPubkeyHex: item.contactPubkeyHex,
                  displayName: item.displayName ?? undefined,
                });
              }}
              activeOpacity={0.7}
            >
              <View style={styles.avatar}>
                <Text style={styles.avatarText}>
                  {initials(item.displayName, item.contactPubkeyHex)}
                </Text>
              </View>
              <View style={styles.info}>
                <View style={styles.topRow}>
                  <Text style={styles.name} numberOfLines={1}>
                    {item.displayName ?? item.contactPubkeyHex.slice(0, 12) + '…'}
                  </Text>
                  <Text style={styles.time}>{formatTime(item.lastMessageTimestamp)}</Text>
                </View>
                <View style={styles.bottomRow}>
                  <Text style={styles.preview} numberOfLines={1}>
                    {item.lastMessageText}
                  </Text>
                  {item.unreadCount > 0 && (
                    <View style={styles.badge}>
                      <Text style={styles.badgeText}>
                        {item.unreadCount > 99 ? '99+' : item.unreadCount}
                      </Text>
                    </View>
                  )}
                </View>
              </View>
            </TouchableOpacity>
          </Swipeable>
        )}
        ItemSeparatorComponent={() => <View style={styles.separator} />}
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: colors.background },
  emptyContainer: { flex: 1, backgroundColor: colors.background, alignItems: 'center', justifyContent: 'center', padding: 32 },
  emptyTitle: { fontSize: 18, fontWeight: '600', color: colors.textPrimary },
  emptySubtitle: { fontSize: 14, color: colors.textSecondary, marginTop: 8, textAlign: 'center' },
  row: { flexDirection: 'row', alignItems: 'center', paddingHorizontal: 16, paddingVertical: 12, backgroundColor: colors.background },
  avatar: {
    width: 48, height: 48, borderRadius: 24,
    backgroundColor: colors.accentDark,
    alignItems: 'center', justifyContent: 'center', marginRight: 12,
  },
  avatarText: { fontSize: 18, fontWeight: '600', color: colors.textPrimary },
  info: { flex: 1 },
  topRow: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'baseline' },
  name: { fontSize: 16, fontWeight: '500', color: colors.textPrimary, flex: 1, marginRight: 8 },
  time: { fontSize: 12, color: colors.textMuted },
  bottomRow: { flexDirection: 'row', alignItems: 'center', marginTop: 3 },
  preview: { fontSize: 14, color: colors.textSecondary, flex: 1, marginRight: 8 },
  badge: {
    backgroundColor: colors.accent, borderRadius: 10,
    minWidth: 20, height: 20, alignItems: 'center', justifyContent: 'center',
    paddingHorizontal: 5,
  },
  badgeText: { fontSize: 11, fontWeight: '700', color: colors.background },
  separator: { height: 1, backgroundColor: colors.border, marginLeft: 76 },
  deleteAction: { width: 80, justifyContent: 'center' },
  deleteButton: {
    flex: 1, backgroundColor: '#FF3B30',
    justifyContent: 'center', alignItems: 'center',
  },
  deleteText: { color: '#FFFFFF', fontSize: 14, fontWeight: '600' },
});
