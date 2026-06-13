import { StyleSheet, Text, View } from "react-native";

import { color, space, type } from "../theme";

export type ScreenHeaderProps = {
  /** Uppercase eyebrow/kicker above the title. */
  kicker?: string;
  title: string;
  /** Optional supporting copy below the title. */
  copy?: string;
};

/** Consistent screen header: kicker -> display title -> optional copy. */
export function ScreenHeader({ kicker, title, copy }: ScreenHeaderProps) {
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
  kicker: { ...type.label, color: color.primary },
  title: { ...type.display, color: color.ink900 },
  copy: { ...type.body, color: color.ink500 },
});
