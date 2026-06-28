import {
  ActivityIndicator,
  Pressable,
  StyleSheet,
  Text,
  type StyleProp,
  type ViewStyle,
} from "react-native";

import { color, radius, space, type } from "../theme";

export type ButtonVariant = "primary" | "secondary" | "danger";
export type ButtonSize = "md" | "sm";

export type ButtonProps = {
  label: string;
  onPress: () => void;
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  loading?: boolean;
  fullWidth?: boolean;
  selected?: boolean;
  testID?: string;
  accessibilityLabel?: string;
  style?: StyleProp<ViewStyle>;
};

const BACKGROUNDS: Record<ButtonVariant, { base: string; pressed: string }> = {
  primary: { base: color.primary, pressed: color.primaryPressed },
  secondary: { base: color.primarySubtle, pressed: color.sunken },
  danger: { base: color.danger, pressed: color.dangerPressed },
};

const FOREGROUNDS: Record<ButtonVariant, string> = {
  primary: color.onPrimary,
  secondary: color.primaryPressed,
  danger: color.onPrimary,
};

function createAccessibilityState(
  isInert: boolean,
  loading: boolean,
  selected: boolean | undefined,
) {
  return { disabled: isInert, busy: loading, ...(selected !== undefined ? { selected } : {}) };
}

function resolveButtonLayout(size: ButtonSize, fullWidth: boolean) {
  return [
    styles.base,
    size === "md" ? styles.md : styles.sm,
    fullWidth ? styles.fullWidth : styles.inline,
  ];
}

function resolveButtonBackground(
  pressed: boolean,
  isInert: boolean,
  palette: { base: string; pressed: string },
) {
  return { backgroundColor: pressed && !isInert ? palette.pressed : palette.base };
}

function ButtonContent({
  label,
  loading,
  foreground,
}: {
  label: string;
  loading: boolean;
  foreground: string;
}) {
  if (loading) {
    return <ActivityIndicator size="small" color={foreground} />;
  }

  return (
    <Text style={[styles.label, { color: foreground }]} numberOfLines={1}>
      {label}
    </Text>
  );
}

/**
 * Primary action primitive. Comfortable touch target (>=44pt), explicit pressed
 * and disabled visuals, and a loading state that blocks presses.
 */
export function Button({
  label,
  onPress,
  variant = "primary",
  size = "md",
  disabled = false,
  loading = false,
  fullWidth = false,
  selected,
  testID,
  accessibilityLabel,
  style,
}: ButtonProps) {
  const isInert = disabled || loading;
  const palette = BACKGROUNDS[variant];
  const foreground = FOREGROUNDS[variant];

  return (
    <Pressable
      testID={testID}
      onPress={isInert ? undefined : onPress}
      disabled={isInert}
      accessibilityRole="button"
      accessibilityLabel={accessibilityLabel ?? label}
      accessibilityState={createAccessibilityState(isInert, loading, selected)}
      style={({ pressed }) => [
        ...resolveButtonLayout(size, fullWidth),
        resolveButtonBackground(pressed, isInert, palette),
        disabled ? styles.disabled : null,
        style,
      ]}
    >
      <ButtonContent label={label} loading={loading} foreground={foreground} />
    </Pressable>
  );
}

const styles = StyleSheet.create({
  base: {
    borderRadius: radius.md,
    alignItems: "center",
    justifyContent: "center",
    paddingHorizontal: space.xl,
  },
  md: { minHeight: 48 },
  sm: { minHeight: 44, paddingHorizontal: space.lg },
  inline: { alignSelf: "flex-start" },
  fullWidth: { alignSelf: "stretch" },
  disabled: { opacity: 0.5 },
  label: { ...type.button },
});
