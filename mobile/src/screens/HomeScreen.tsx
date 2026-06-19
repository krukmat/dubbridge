import { StyleSheet, Text, TouchableOpacity, View } from "react-native";

import { useAuth } from "../auth/AuthProvider";
import { Card } from "../components/Card";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { color, space, type } from "../theme";

const NAV_CARDS = [
  {
    testID: "home-open-assets" as const,
    title: "Browse assets",
    subtitle: "View and manage uploaded media",
    key: "assets",
  },
  {
    testID: "home-open-upload" as const,
    title: "Upload asset",
    subtitle: "Add new media to your workspace",
    key: "upload",
  },
  {
    testID: "home-open-review" as const,
    title: "Review inbox",
    subtitle: "Approve or reject pending review tasks",
    key: "review",
  },
  {
    testID: "home-open-organizations" as const,
    title: "Organizations and projects",
    subtitle: "Manage teams and project workspaces",
    key: "organizations",
  },
] as const;

export function HomeScreen({
  dubbridgeEnv,
  gatewayBaseUrl,
  onOpenAssets,
  onOpenUpload,
  onOpenReview,
  onOpenOrganizations,
}: {
  dubbridgeEnv: string;
  gatewayBaseUrl: string;
  onOpenAssets: () => void;
  onOpenUpload: () => void;
  onOpenReview: () => void;
  onOpenOrganizations: () => void;
}) {
  const auth = useAuth();

  const callbacks: Record<string, () => void> = {
    assets: onOpenAssets,
    upload: onOpenUpload,
    review: onOpenReview,
    organizations: onOpenOrganizations,
  };

  return (
    <Screen testID="home-screen">
      <ScreenHeader
        kicker="DubBridge"
        title="Your workspace"
        copy="Browse and manage your media assets."
      />

      <View style={styles.navCards}>
        {NAV_CARDS.map((card) => (
          <Card
            key={card.key}
            testID={card.testID}
            title={card.title}
            subtitle={card.subtitle}
            trailing="chevron"
            onPress={callbacks[card.key]}
            accessibilityLabel={card.title}
          />
        ))}
      </View>

      <View style={styles.signOutRow}>
        <TouchableOpacity
          testID="home-sign-out"
          onPress={() => void auth.logout()}
          accessibilityRole="button"
          accessibilityLabel="Sign out"
        >
          <Text style={styles.signOutText}>Sign out</Text>
        </TouchableOpacity>
      </View>

      {__DEV__ && (
        <Panel>
          <Text style={styles.metaLabel}>Environment</Text>
          <Text style={styles.metaValue}>{dubbridgeEnv}</Text>
          <Text style={styles.metaLabel}>Gateway</Text>
          <Text style={styles.metaValue} numberOfLines={2}>{gatewayBaseUrl}</Text>
        </Panel>
      )}
    </Screen>
  );
}

const styles = StyleSheet.create({
  navCards: { gap: space.md },
  signOutRow: {
    marginTop: space.lg,
    alignItems: "center",
  },
  signOutText: { ...type.meta, color: color.ink400 },
  metaLabel: { ...type.label, color: color.ink400 },
  metaValue: { ...type.meta, color: color.ink700 },
});
