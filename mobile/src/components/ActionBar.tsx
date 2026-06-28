import { useContext, type ReactNode } from "react";
import { StyleSheet, View } from "react-native";
import { SafeAreaInsetsContext } from "react-native-safe-area-context";

import { color, space } from "../theme";

const ZERO_INSETS = { top: 0, bottom: 0, left: 0, right: 0 };

/** Height of the button row inside the bar (excluding safe-area padding). */
export const ACTION_BAR_CONTENT_HEIGHT = 52;

export type ActionBarProps = {
  children: ReactNode;
  testID?: string;
};

/**
 * Sticky action bar anchored to the bottom of the screen.
 * Place outside the scrollable Screen so it stays visible without scrolling.
 * The parent container must use `flex: 1` with `position: relative`.
 *
 * Pair with `Screen` prop `extraBottomPadding={ACTION_BAR_CONTENT_HEIGHT + space.xl}`
 * so the last scrollable row is never occluded by the bar.
 */
export function ActionBar({ children, testID }: ActionBarProps) {
  const insets = useContext(SafeAreaInsetsContext) ?? ZERO_INSETS;

  return (
    <View
      testID={testID}
      style={[styles.bar, { paddingBottom: insets.bottom + space.md }]}
    >
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  bar: {
    position: "absolute",
    bottom: 0,
    left: 0,
    right: 0,
    backgroundColor: color.raised,
    borderTopWidth: 1,
    borderTopColor: color.border,
    paddingTop: space.md,
    paddingHorizontal: space.xxl,
    gap: space.sm,
    flexDirection: "row",
  },
});
