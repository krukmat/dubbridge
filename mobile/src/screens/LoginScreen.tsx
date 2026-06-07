import { Pressable, StyleSheet, Text, View } from "react-native";

import { useAuth } from "../auth/AuthProvider";

export function LoginScreen() {
  const auth = useAuth();

  return (
    <View testID="login-screen" style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.kicker}>Session gateway shell</Text>
        <Text style={styles.title}>DubBridge mobile</Text>
        <Text style={styles.copy}>
          Sign in with the system browser to establish the gateway-backed mobile
          session for this device.
        </Text>
      </View>

      <Pressable onPress={() => void auth.login()} style={styles.primaryButton}>
        <Text style={styles.primaryButtonText}>Sign in with session gateway</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#f4efe5",
    justifyContent: "space-between",
    padding: 24,
  },
  header: {
    marginTop: 48,
    gap: 16,
  },
  kicker: {
    fontSize: 12,
    fontWeight: "700",
    textTransform: "uppercase",
    color: "#855f19",
  },
  title: {
    fontSize: 40,
    fontWeight: "700",
    color: "#1f1305",
  },
  copy: {
    fontSize: 17,
    lineHeight: 26,
    color: "#4e412e",
    maxWidth: 420,
  },
  primaryButton: {
    borderRadius: 8,
    backgroundColor: "#1f1305",
    paddingHorizontal: 18,
    paddingVertical: 16,
    alignItems: "center",
    marginBottom: 12,
  },
  primaryButtonText: {
    fontSize: 15,
    fontWeight: "600",
    color: "#fff8ee",
  },
});
