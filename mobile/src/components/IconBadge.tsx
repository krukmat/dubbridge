import { StyleSheet, Text, View } from "react-native";

import { color, radius, space, type } from "../theme";

export type IconBadgeTone = "primary" | "success" | "info" | "neutral";

export type IconBadgeProps = {
  symbol: string;
  tone?: IconBadgeTone;
  testID?: string;
};

const TONES: Record<IconBadgeTone, { bg: string; fg: string }> = {
  primary: { bg: color.primarySubtle, fg: color.primaryPressed },
  success: { bg: color.successSubtle, fg: color.successStrong },
  info: { bg: color.infoSubtle, fg: color.infoStrong },
  neutral: { bg: color.sunken, fg: color.ink700 },
};

export function IconBadge({
  symbol,
  tone = "neutral",
  testID,
}: IconBadgeProps) {
  const palette = TONES[tone] ?? TONES.neutral;
  return (
    <View
      testID={testID}
      accessible={false}
      style={[styles.badge, { backgroundColor: palette.bg }]}
    >
      <Text style={[styles.symbol, { color: palette.fg }]}>{symbol}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  badge: {
    width: 36,
    height: 36,
    borderRadius: radius.pill,
    alignItems: "center",
    justifyContent: "center",
    flexShrink: 0,
  },
  symbol: {
    ...type.label,
    fontSize: 11,
    lineHeight: 14,
    letterSpacing: 0.3,
  },
});
