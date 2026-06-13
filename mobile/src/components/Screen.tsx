import { useContext, type ReactNode } from "react";
import {
  ScrollView,
  StyleSheet,
  View,
  type StyleProp,
  type ViewStyle,
} from "react-native";

type RefreshControlElement = React.ComponentProps<
  typeof ScrollView
>["refreshControl"];
import { SafeAreaInsetsContext } from "react-native-safe-area-context";

import { color, space } from "../theme";

export type ScreenEdge = "top" | "bottom";

export type ScreenProps = {
  children: ReactNode;
  /** Render content inside a ScrollView instead of a static View. */
  scroll?: boolean;
  /** RefreshControl element, only applied when `scroll` is true. */
  refreshControl?: RefreshControlElement;
  contentContainerStyle?: StyleProp<ViewStyle>;
  /** Which safe-area edges to pad. Defaults to top + bottom. */
  edges?: ScreenEdge[];
  testID?: string;
};

const ZERO_INSETS = { top: 0, bottom: 0, left: 0, right: 0 };

/**
 * Canvas-backgrounded screen wrapper that applies real safe-area insets when a
 * SafeAreaProvider is present (App root) and degrades to zero insets otherwise
 * (e.g. unit tests that render a screen in isolation). Replaces the per-screen
 * `marginTop` status-bar hacks.
 */
export function Screen({
  children,
  scroll = false,
  refreshControl,
  contentContainerStyle,
  edges = ["top", "bottom"],
  testID,
}: ScreenProps) {
  const insets = useContext(SafeAreaInsetsContext) ?? ZERO_INSETS;
  const padTop = edges.includes("top") ? insets.top : 0;
  const padBottom = edges.includes("bottom") ? insets.bottom : 0;

  if (scroll) {
    return (
      <ScrollView
        testID={testID}
        style={styles.canvas}
        contentContainerStyle={[
          styles.content,
          { paddingTop: space.xxl + padTop, paddingBottom: space.xxl + padBottom },
          contentContainerStyle,
        ]}
        refreshControl={refreshControl}
      >
        {children}
      </ScrollView>
    );
  }

  return (
    <View
      testID={testID}
      style={[
        styles.canvas,
        styles.content,
        { paddingTop: space.xxl + padTop, paddingBottom: space.xxl + padBottom },
        contentContainerStyle,
      ]}
    >
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  canvas: { flex: 1, backgroundColor: color.canvas },
  content: {
    paddingHorizontal: space.xxl,
    gap: space.xl,
  },
});
