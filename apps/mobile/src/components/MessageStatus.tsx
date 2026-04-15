import React from 'react';
import { Text, StyleSheet } from 'react-native';

type Status = 'pending' | 'sent' | 'delivered' | 'failed';

interface Props {
  status: Status;
}

export const MessageStatus: React.FC<Props> = ({ status }) => {
  const getIndicator = () => {
    switch (status) {
      case 'pending':
        return { text: 'o', color: '#555555' };
      case 'sent':
        return { text: '\u2713', color: '#888888' };
      case 'delivered':
        return { text: '\u2713\u2713', color: '#4A9EFF' };
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
