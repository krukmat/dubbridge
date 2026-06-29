import { StyleSheet, Text } from "react-native";

import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { color, type } from "../theme";

export function ConfigErrorScreen({ message }: { message: string }) {
  return (
    <Screen
      testID="config-error-screen"
      contentContainerStyle={styles.centered}
    >
      <Panel>
        <Text style={styles.eyebrow}>Configuration required</Text>
        <Text style={styles.title}>Setup required</Text>
        <Text style={styles.message}>{message}</Text>
        <Text style={styles.hint}>
          Set EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL and DUBBRIDGE_ENV to continue.
        </Text>
      </Panel>
    </Screen>
  );
}

const styles = StyleSheet.create({
  centered: { justifyContent: "center" },
  eyebrow: { ...type.label, color: color.primaryStrong },
  title: { ...type.title, color: color.ink900 },
  message: { ...type.body, color: color.ink700 },
  hint: { ...type.meta, color: color.ink400 },
});
