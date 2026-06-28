import {
  Pressable,
  StyleSheet,
  Text,
  View,
  type StyleProp,
  type ViewStyle,
} from "react-native";

import { color, elevation, radius, space, type } from "../theme";
import type { BadgeTone } from "./Badge";

export type CardProps = {
  children?: React.ReactNode;
  leadingAdornment?: React.ReactNode;
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
  /**
   * When set, renders a leading media placeholder tile toned by the given BadgeTone.
   * Use until a real poster_url is available (X-S-210-1).
   */
  mediaTone?: BadgeTone;
};

/**
 * Raised, optionally tappable container. Tappable cards float on one soft
 * elevation level and expose pressed feedback + button role; static use renders
 * a plain raised surface.
 */
const MEDIA_TONE_BG: Record<BadgeTone, string> = {
  neutral: color.sunken,
  success: color.successSubtle,
  warning: color.warningSubtle,
  danger: color.dangerSubtle,
  info: color.infoSubtle,
};

function Chevron() {
  return <Text style={styles.chevron} accessibilityElementsHidden importantForAccessibility="no">›</Text>;
}

function MediaPlaceholder({ mediaTone }: { mediaTone?: BadgeTone }) {
  if (mediaTone == null) return null;
  return (
    <View
      style={[styles.mediaPlaceholder, { backgroundColor: MEDIA_TONE_BG[mediaTone] }]}
      accessibilityElementsHidden
      importantForAccessibility="no"
    />
  );
}

function TitleContent({
  mediaTone,
  leadingAdornment,
  title,
  subtitle,
  trailing,
  children,
}: {
  mediaTone?: BadgeTone;
  leadingAdornment?: React.ReactNode;
  title: string;
  subtitle?: string;
  trailing?: "chevron";
  children?: React.ReactNode;
}) {
  return (
    <>
      <View style={styles.row}>
        <MediaPlaceholder mediaTone={mediaTone} />
        {leadingAdornment}
        <View style={styles.textBlock}>
          <Text style={styles.cardTitle} numberOfLines={1}>{title}</Text>
          {subtitle != null && (
            <Text style={styles.cardSubtitle} numberOfLines={1}>{subtitle}</Text>
          )}
        </View>
        {trailing === "chevron" ? <Chevron /> : null}
      </View>
      {children}
    </>
  );
}

function ChildrenContent({
  mediaTone,
  leadingAdornment,
  trailing,
  children,
}: {
  mediaTone?: BadgeTone;
  leadingAdornment?: React.ReactNode;
  trailing?: "chevron";
  children?: React.ReactNode;
}) {
  if (mediaTone == null && leadingAdornment == null) {
    return (
      <>
        {children}
        {trailing === "chevron" ? (
          <Text style={[styles.chevron, styles.chevronAbsolute]} accessibilityElementsHidden importantForAccessibility="no">›</Text>
        ) : null}
      </>
    );
  }

  return (
    <View style={styles.row}>
      <MediaPlaceholder mediaTone={mediaTone} />
      {leadingAdornment}
      <View style={styles.childrenBlock}>
        {children}
      </View>
      {trailing === "chevron" ? (
        <Text style={[styles.chevron, styles.chevronAbsolute]} accessibilityElementsHidden importantForAccessibility="no">›</Text>
      ) : null}
    </View>
  );
}

function CardContainer({
  onPress,
  testID,
  style,
  accessibilityLabel,
  children,
}: {
  onPress?: () => void;
  testID?: string;
  style?: StyleProp<ViewStyle>;
  accessibilityLabel?: string;
  children: React.ReactNode;
}) {
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
        {children}
      </Pressable>
    );
  }

  return (
    <View testID={testID} style={[styles.card, elevation.card, style]}>
      {children}
    </View>
  );
}

export function Card({
  children,
  leadingAdornment,
  title,
  subtitle,
  trailing,
  onPress,
  testID,
  style,
  accessibilityLabel,
  mediaTone,
}: CardProps) {
  const inner = title != null ? (
    <TitleContent
      mediaTone={mediaTone}
      leadingAdornment={leadingAdornment}
      title={title}
      subtitle={subtitle}
      trailing={trailing}
      children={children}
    />
  ) : (
    <ChildrenContent
      mediaTone={mediaTone}
      leadingAdornment={leadingAdornment}
      trailing={trailing}
      children={children}
    />
  );

  return (
    <CardContainer
      onPress={onPress}
      testID={testID}
      style={style}
      accessibilityLabel={accessibilityLabel}
    >
      {inner}
    </CardContainer>
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
    gap: space.md,
  },
  textBlock: { flex: 1 },
  childrenBlock: { flex: 1, gap: space.sm },
  cardTitle: { ...type.heading, color: color.ink900 },
  cardSubtitle: { ...type.meta, color: color.ink400 },
  chevron: { fontSize: 22, color: color.ink400, lineHeight: 26 },
  chevronAbsolute: { position: "absolute", top: space.lg, right: space.lg },
  mediaPlaceholder: {
    width: 48,
    height: 48,
    borderRadius: radius.md,
    flexShrink: 0,
  },
});
