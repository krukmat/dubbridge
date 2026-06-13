import { StyleSheet, View, type StyleProp, type ViewStyle } from "react-native";

import { color, radius, space } from "../theme";

export type PanelProps = {
  children: React.ReactNode;
  testID?: string;
  style?: StyleProp<ViewStyle>;
};

/** Static raised surface (border, no elevation) for grouped, non-tappable content. */
export function Panel({ children, testID, style }: PanelProps) {
  return (
    <View testID={testID} style={[styles.panel, style]}>
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  panel: {
    backgroundColor: color.raised,
    borderRadius: radius.lg,
    borderWidth: 1,
    borderColor: color.border,
    padding: space.xl,
    gap: space.md,
  },
});
