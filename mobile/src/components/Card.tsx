import {
  Pressable,
  StyleSheet,
  View,
  type StyleProp,
  type ViewStyle,
} from "react-native";

import { color, elevation, radius, space } from "../theme";

export type CardProps = {
  children: React.ReactNode;
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
  onPress,
  testID,
  style,
  accessibilityLabel,
}: CardProps) {
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

const styles = StyleSheet.create({
  card: {
    backgroundColor: color.raised,
    borderRadius: radius.lg,
    borderWidth: 1,
    borderColor: color.border,
    padding: space.lg,
    gap: space.sm,
  },
  pressed: { backgroundColor: color.sunken },
});
