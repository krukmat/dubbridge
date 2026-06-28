import { StyleSheet, Text, View } from "react-native";

import { color, radius, space, type } from "../theme";

export type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

export type BadgeProps = {
  label: string;
  tone?: BadgeTone;
  testID?: string;
};

const TONES: Record<BadgeTone, { bg: string; fg: string }> = {
  neutral: { bg: color.sunken, fg: color.ink700 },
  success: { bg: color.successSubtle, fg: color.successStrong },
  warning: { bg: color.warningSubtle, fg: color.warningStrong },
  danger: { bg: color.dangerSubtle, fg: color.dangerPressed },
  info: { bg: color.infoSubtle, fg: color.infoStrong },
};
const SUCCESS_STATUSES = new Set([
  "grant",
  "granted",
  "active",
  "approved",
  "finalized",
  "published",
]);
const DANGER_STATUSES = new Set([
  "revoke",
  "revoked",
  "blocked",
  "rejected",
  "failed",
]);
const INFO_STATUSES = new Set(["pending", "processing", "in_review"]);

/** Resolve a tone, falling back to `neutral` for any unknown value (never throws). */
function resolveTone(tone: BadgeTone | undefined): { bg: string; fg: string } {
  return (tone && TONES[tone]) || TONES.neutral;
}

/**
 * Map a domain status string to a semantic tone. Defaults to `neutral` so an
 * unrecognized status renders calmly instead of crashing.
 */
export function statusTone(status: string | null | undefined): BadgeTone {
  if (status == null) return "neutral";
  if (SUCCESS_STATUSES.has(status)) return "success";
  if (DANGER_STATUSES.has(status)) return "danger";
  if (INFO_STATUSES.has(status)) return "info";
  return "neutral";
}

/** Small status pill: subtle background + toned text. */
export function Badge({ label, tone = "neutral", testID }: BadgeProps) {
  const palette = resolveTone(tone);
  return (
    <View
      testID={testID}
      style={[styles.badge, { backgroundColor: palette.bg }]}
      accessibilityRole="text"
    >
      <Text style={[styles.label, { color: palette.fg }]}>{label}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  badge: {
    alignSelf: "flex-start",
    borderRadius: radius.pill,
    paddingHorizontal: space.md,
    paddingVertical: space.xs,
  },
  label: { ...type.label },
});
