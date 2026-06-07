import { Pressable, StyleSheet, Text, View } from "react-native";

import { useAuth } from "../auth/AuthProvider";

export function HomeScreen({
  dubbridgeEnv,
  gatewayBaseUrl,
  onOpenAssets,
}: {
  dubbridgeEnv: string;
  gatewayBaseUrl: string;
  onOpenAssets: () => void;
}) {
  const auth = useAuth();

  return (
    <View testID="home-screen" style={styles.container}>
      <View style={styles.hero}>
        <Text style={styles.kicker}>Authenticated shell</Text>
        <Text style={styles.title}>Mobile home</Text>
        <Text style={styles.copy}>
          Your mobile session is active through the gateway. Browse the current
          asset surface or sign out of this device session.
        </Text>
      </View>

      <View style={styles.metaPanel}>
        <Text style={styles.metaLabel}>Environment</Text>
        <Text style={styles.metaValue}>{dubbridgeEnv}</Text>
        <Text style={styles.metaLabel}>Gateway base URL</Text>
        <Text style={styles.metaValue}>{gatewayBaseUrl}</Text>
      </View>

      <View style={styles.actions}>
        <Pressable onPress={onOpenAssets} style={styles.secondaryButton}>
          <Text style={styles.secondaryButtonText}>Browse assets</Text>
        </Pressable>

        <Pressable onPress={() => void auth.logout()} style={styles.button}>
          <Text style={styles.buttonText}>Sign out</Text>
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#f3f6f4",
    padding: 24,
    gap: 24,
  },
  hero: {
    marginTop: 24,
    gap: 12,
  },
  kicker: {
    fontSize: 12,
    fontWeight: "700",
    textTransform: "uppercase",
    color: "#15715f",
  },
  title: {
    fontSize: 34,
    fontWeight: "700",
    color: "#0f1720",
  },
  copy: {
    fontSize: 17,
    lineHeight: 25,
    color: "#3c4954",
  },
  metaPanel: {
    borderRadius: 8,
    backgroundColor: "#ffffff",
    borderWidth: 1,
    borderColor: "#cfdbd6",
    padding: 20,
    gap: 8,
  },
  metaLabel: {
    fontSize: 12,
    fontWeight: "700",
    textTransform: "uppercase",
    color: "#58726d",
  },
  metaValue: {
    fontSize: 16,
    lineHeight: 22,
    color: "#10212a",
  },
  button: {
    alignSelf: "flex-start",
    borderRadius: 8,
    backgroundColor: "#10212a",
    paddingHorizontal: 18,
    paddingVertical: 14,
  },
  buttonText: {
    fontSize: 15,
    fontWeight: "600",
    color: "#f8fbf9",
  },
  actions: {
    gap: 12,
  },
  secondaryButton: {
    alignSelf: "flex-start",
    borderRadius: 8,
    backgroundColor: "#dfe8e5",
    paddingHorizontal: 18,
    paddingVertical: 14,
  },
  secondaryButtonText: {
    fontSize: 15,
    fontWeight: "600",
    color: "#14312d",
  },
});
