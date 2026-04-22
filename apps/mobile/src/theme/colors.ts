/**
 * Voryn color palette — dark theme, ice-cyan accent.
 *
 * Ice accent selected to reinforce "cold / secure / unreachable"
 * aesthetic and separate Voryn from the default Signal-blue that
 * every privacy app uses.
 */
export const colors = {
  // Backgrounds
  background: '#0D0D0D',
  surface: '#1A1A1A',
  surfaceLight: '#2A2A2A',

  // Text
  textPrimary: '#FFFFFF',
  textSecondary: '#888888',
  textMuted: '#555555',

  // Accent — Ice Cyan
  accent: '#8CE8FF',
  accentDark: '#1F4D5C',

  // Status
  success: '#34C759',
  warning: '#FF9500',
  error: '#FF3B30',

  // Borders
  border: '#1A1A1A',
  borderLight: '#333333',
} as const;
