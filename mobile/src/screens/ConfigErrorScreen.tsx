import { StyleSheet, Text, View } from "react-native";

export function ConfigErrorScreen({ message }: { message: string }) {
  return (
    <View testID="config-error-screen" style={styles.container}>
      <View style={styles.panel}>
        <Text style={styles.eyebrow}>Configuration required</Text>
        <Text style={styles.title}>Mobile app cannot start yet.</Text>
        <Text style={styles.message}>{message}</Text>
        <Text style={styles.hint}>
          The app expects Expo config values from `DUBBRIDGE_ENV` and
          `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL`.
        </Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "#f4efe5",
    padding: 24,
  },
  panel: {
    width: "100%",
    maxWidth: 420,
    borderRadius: 8,
    backgroundColor: "#fffaf0",
    padding: 24,
    gap: 12,
    borderWidth: 1,
    borderColor: "#d7c8aa",
  },
  eyebrow: {
    fontSize: 13,
    fontWeight: "600",
    letterSpacing: 0,
    textTransform: "uppercase",
    color: "#7d5d1f",
  },
  title: {
    fontSize: 28,
    fontWeight: "700",
    color: "#23180c",
  },
  message: {
    fontSize: 16,
    lineHeight: 24,
    color: "#3f301a",
  },
  hint: {
    fontSize: 14,
    lineHeight: 21,
    color: "#6b5b44",
  },
});
