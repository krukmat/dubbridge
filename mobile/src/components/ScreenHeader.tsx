import { StyleSheet, Text, View } from "react-native";

import { color, space, type } from "../theme";

export type ScreenHeaderProps = {
  /** Uppercase eyebrow/kicker above the title. */
  kicker?: string;
  title: string;
  /** Optional supporting copy below the title. */
  copy?: string;
  /**
   * Compact variant for stack-pushed screens that already have a native header.
   * Renders a small kicker only — no large display title or copy — so the screen
   * gains back the ~120 px the full-size header would have consumed.
   */
  compact?: boolean;
};

/** Consistent screen header: kicker -> display title -> optional copy. */
export function ScreenHeader({ kicker, title, copy, compact = false }: ScreenHeaderProps) {
  if (compact) {
    return kicker ? (
      <View style={styles.compactHeader}>
        <Text style={styles.kicker}>{kicker}</Text>
      </View>
    ) : null;
  }

  return (
    <View style={styles.header}>
      {kicker ? <Text style={styles.kicker}>{kicker}</Text> : null}
      <Text style={styles.title}>{title}</Text>
      {copy ? <Text style={styles.copy}>{copy}</Text> : null}
    </View>
  );
}

const styles = StyleSheet.create({
  header: { gap: space.sm },
  compactHeader: { paddingBottom: space.xs },
  kicker: { ...type.label, color: color.primaryStrong },
  title: { ...type.display, color: color.ink900 },
  copy: { ...type.body, color: color.ink500 },
});
