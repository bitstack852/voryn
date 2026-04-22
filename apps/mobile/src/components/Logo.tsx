import React from 'react';
import Svg, { Circle, Line } from 'react-native-svg';
import { colors } from '../theme/colors';

interface LogoProps {
  size?: number;
  color?: string;
}

/**
 * Voryn logo mark — "Mesh Node".
 * Four cardinal peer nodes around a central self-node,
 * bound by two concentric range rings. Pure stroke, no fills
 * except the center dot, so it reads cleanly at any size.
 */
export const Logo: React.FC<LogoProps> = ({
  size = 86,
  color = colors.accent,
}) => (
  <Svg width={size} height={size} viewBox="0 0 36 36" fill="none">
    {/* Range rings */}
    <Circle cx="18" cy="18" r="14" stroke={color} strokeWidth={0.6} opacity={0.4} />
    <Circle cx="18" cy="18" r="9" stroke={color} strokeWidth={0.6} opacity={0.6} />

    {/* Connectors */}
    <Line x1="18" y1="6" x2="18" y2="30" stroke={color} strokeWidth={0.4} opacity={0.4} />
    <Line x1="6" y1="18" x2="30" y2="18" stroke={color} strokeWidth={0.4} opacity={0.4} />

    {/* Peer nodes */}
    <Circle cx="18" cy="6" r="1.4" fill={color} />
    <Circle cx="30" cy="18" r="1.4" fill={color} />
    <Circle cx="18" cy="30" r="1.4" fill={color} />
    <Circle cx="6" cy="18" r="1.4" fill={color} />

    {/* Self */}
    <Circle cx="18" cy="18" r="3" fill={colors.textPrimary} />
  </Svg>
);
