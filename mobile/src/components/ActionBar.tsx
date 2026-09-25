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
 * Bottom action bar rendered as a normal flex sibling below the scrollable
 * Screen. This keeps it fixed in the viewport without overlaying the ScrollView
 * or competing with it for Android hit-testing.
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
    flexShrink: 0,
    backgroundColor: color.raised,
    borderTopWidth: 1,
    borderTopColor: color.border,
    paddingTop: space.md,
    paddingHorizontal: space.xxl,
    gap: space.sm,
    flexDirection: "row",
  },
});
