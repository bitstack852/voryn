import React from 'react';
import { Text, StyleSheet } from 'react-native';
import type { MessageStatus as Status } from '@voryn/shared-types';

interface Props {
  status: Status;
}

/**
 * Delivery status indicator for messages.
 * - pending: clock icon (waiting to send)
 * - sent: single checkmark
 * - delivered: double checkmark
 * - failed: exclamation mark
 */
export const MessageStatus: React.FC<Props> = ({ status }) => {
  const getIndicator = () => {
    switch (status) {
      case 'pending':
        return { text: 'o', color: '#555555' };
      case 'sent':
        return { text: '✓', color: '#888888' };
      case 'delivered':
        return { text: '✓✓', color: '#4A9EFF' };
      case 'failed':
        return { text: '!', color: '#FF3B30' };
      default:
        return { text: '', color: '#555555' };
    }
  };

  const { text, color } = getIndicator();

  return <Text style={[styles.indicator, { color }]}>{text}</Text>;
};

const styles = StyleSheet.create({
  indicator: {
    fontSize: 12,
    marginLeft: 4,
  },
});
