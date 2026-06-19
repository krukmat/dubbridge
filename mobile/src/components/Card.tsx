import {
  Pressable,
  StyleSheet,
  Text,
  View,
  type StyleProp,
  type ViewStyle,
} from "react-native";

import { color, elevation, radius, space, type } from "../theme";

export type CardProps = {
  children?: React.ReactNode;
  /** Navigation title rendered as the card heading. */
  title?: string;
  /** One-line descriptor rendered below the title. */
  subtitle?: string;
  /** When "chevron", a trailing › affordance is rendered (decorative, not a separate a11y target). */
  trailing?: "chevron";
  /** When provided, the card becomes a tappable, accessible button. */
  onPress?: () => void;
  testID?: string;
  style?: StyleProp<ViewStyle>;
  accessibilityLabel?: string;
};

/**
 * Raised, optionally tappable container. Tappable cards float on one soft
 * elevation level and expose pressed feedback + button role; static use renders
 * a plain raised surface.
 */
export function Card({
  children,
  title,
  subtitle,
  trailing,
  onPress,
  testID,
  style,
  accessibilityLabel,
}: CardProps) {
  const chevronEl = trailing === "chevron" ? (
    <Text style={styles.chevron} accessibilityElementsHidden importantForAccessibility="no">›</Text>
  ) : null;

  // Title-mode: title (+ optional subtitle) and chevron share a flex row.
  // Children-mode: children render normally; chevron is an absolute corner decoration.
  const inner = title != null ? (
    <>
      <View style={styles.row}>
        <View style={styles.textBlock}>
          <Text style={styles.cardTitle} numberOfLines={1}>{title}</Text>
          {subtitle != null && (
            <Text style={styles.cardSubtitle} numberOfLines={1}>{subtitle}</Text>
          )}
        </View>
        {chevronEl}
      </View>
      {children}
    </>
  ) : (
    <>
      {children}
      {chevronEl != null && (
        <Text style={[styles.chevron, styles.chevronAbsolute]} accessibilityElementsHidden importantForAccessibility="no">›</Text>
      )}
    </>
  );

  if (onPress) {
    return (
      <Pressable
        testID={testID}
        onPress={onPress}
        accessibilityRole="button"
        accessibilityLabel={accessibilityLabel}
        style={({ pressed }) => [
          styles.card,
          elevation.card,
          pressed ? styles.pressed : null,
          style,
        ]}
      >
        {inner}
      </Pressable>
    );
  }

  return (
    <View testID={testID} style={[styles.card, elevation.card, style]}>
      {inner}
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: color.raised,
    borderRadius: radius.lg,
    borderWidth: 1,
    borderColor: color.border,
    padding: space.lg,
    gap: space.sm,
    position: "relative",
  },
  pressed: { backgroundColor: color.sunken },
  row: {
    flexDirection: "row",
    alignItems: "center",
    gap: space.sm,
  },
  textBlock: { flex: 1 },
  cardTitle: { ...type.heading, color: color.ink900 },
  cardSubtitle: { ...type.meta, color: color.ink400 },
  chevron: { fontSize: 22, color: color.ink400, lineHeight: 26 },
  chevronAbsolute: { position: "absolute", top: space.lg, right: space.lg },
});
