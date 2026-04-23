import React, { useCallback, useRef, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  Animated,
  PanResponder,
} from 'react-native';
import { useNavigation, useFocusEffect } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../navigation/RootNavigator';
import * as VorynBridge from '../services/VorynBridge';
import { colors } from '../theme/colors';

type Nav = NativeStackNavigationProp<RootStackParamList, 'Chat'>;

const DELETE_WIDTH = 80;
const SWIPE_THRESHOLD = 40;

interface SwipeableRowProps {
  onDelete: () => void;
  onPress: () => void;
  children: React.ReactNode;
}

const SwipeableRow: React.FC<SwipeableRowProps> = ({ onDelete, onPress, children }) => {
  const translateX = useRef(new Animated.Value(0)).current;
  const isOpen = useRef(false);

  const close = () => {
    Animated.spring(translateX, { toValue: 0, useNativeDriver: true, bounciness: 0 }).start();
    isOpen.current = false;
  };

  const open = () => {
    Animated.spring(translateX, { toValue: -DELETE_WIDTH, useNativeDriver: true, bounciness: 0 }).start();
    isOpen.current = true;
  };

  const panResponder = useRef(
    PanResponder.create({
      onMoveShouldSetPanResponder: (_, g) => Math.abs(g.dx) > 5 && Math.abs(g.dx) > Math.abs(g.dy),
      onPanResponderMove: (_, g) => {
        const x = isOpen.current ? g.dx - DELETE_WIDTH : g.dx;
        if (x <= 0) translateX.setValue(Math.max(x, -DELETE_WIDTH));
      },
      onPanResponderRelease: (_, g) => {
        const x = isOpen.current ? g.dx - DELETE_WIDTH : g.dx;
        if (x < -SWIPE_THRESHOLD) open(); else close();
      },
    }),
  ).current;

  return (
    <View style={swipeStyles.container}>
      <View style={swipeStyles.deleteContainer}>
        <TouchableOpacity style={swipeStyles.deleteButton} onPress={() => { close(); onDelete(); }}>
          <Text style={swipeStyles.deleteText}>Delete</Text>
        </TouchableOpacity>
      </View>
      <Animated.View
        style={[swipeStyles.row, { transform: [{ translateX }] }]}
        {...panResponder.panHandlers}
      >
        <TouchableOpacity onPress={() => { if (isOpen.current) { close(); } else { onPress(); } }} activeOpacity={0.7}>
          {children}
        </TouchableOpacity>
      </Animated.View>
    </View>
  );
};

const swipeStyles = StyleSheet.create({
  container: { overflow: 'hidden' },
  deleteContainer: {
    position: 'absolute', right: 0, top: 0, bottom: 0,
    width: DELETE_WIDTH, backgroundColor: '#FF3B30',
    justifyContent: 'center', alignItems: 'center',
  },
  deleteButton: { flex: 1, width: '100%', justifyContent: 'center', alignItems: 'center' },
  deleteText: { color: '#FFFFFF', fontSize: 14, fontWeight: '600' },
  row: { backgroundColor: colors.background },
});

// ─────────────────────────────────────────────────────────────────────────────

export const ChatsScreen: React.FC = () => {
  const navigation = useNavigation<Nav>();
  const [conversations, setConversations] = useState<VorynBridge.Conversation[]>([]);

  const load = useCallback(async () => {
    const convs = await VorynBridge.getConversations();
    setConversations(convs);
  }, []);

  useFocusEffect(useCallback(() => { load(); }, [load]));

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

  if (conversations.length === 0) {
    return (
      <View style={styles.emptyContainer}>
        <Text style={styles.emptyTitle}>No messages yet</Text>
        <Text style={styles.emptySubtitle}>Add a contact and start a conversation</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <FlatList
        data={conversations}
        keyExtractor={(item) => item.conversationId}
        renderItem={({ item }) => (
          <SwipeableRow
            onDelete={async () => {
              await VorynBridge.deleteConversation(item.conversationId);
              await load();
            }}
            onPress={() =>
              navigation.navigate('Chat', {
                contactPubkeyHex: item.contactPubkeyHex,
                displayName: item.displayName ?? undefined,
              })
            }
          >
            <View style={styles.row}>
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
            </View>
          </SwipeableRow>
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
  row: { flexDirection: 'row', alignItems: 'center', paddingHorizontal: 16, paddingVertical: 12 },
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
});
