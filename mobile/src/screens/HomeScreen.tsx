import { StyleSheet, Text, View } from "react-native";

import { useAuth } from "../auth/AuthProvider";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { color, space, type } from "../theme";

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

  return (
    <Screen testID="home-screen">
      <ScreenHeader
        kicker="DubBridge"
        title="Your workspace"
        copy="Browse and manage your media assets."
      />

      <View style={styles.actions}>
        <Button
          testID="home-open-assets"
          label="Browse assets"
          variant="secondary"
          onPress={onOpenAssets}
          fullWidth
        />
        <Button
          testID="home-open-upload"
          label="Upload asset"
          variant="secondary"
          onPress={onOpenUpload}
          fullWidth
        />
        <Button
          testID="home-open-review"
          label="Review inbox"
          variant="secondary"
          onPress={onOpenReview}
          fullWidth
        />
        <Button
          testID="home-open-organizations"
          label="Organizations and projects"
          variant="secondary"
          onPress={onOpenOrganizations}
          fullWidth
        />
        <Button
          testID="home-sign-out"
          label="Sign out"
          variant="secondary"
          onPress={() => void auth.logout()}
          fullWidth
        />
      </View>

      <Panel>
        <Text style={styles.metaLabel}>Environment</Text>
        <Text style={styles.metaValue}>{dubbridgeEnv}</Text>
        <Text style={styles.metaLabel}>Gateway</Text>
        <Text style={styles.metaValue} numberOfLines={2}>{gatewayBaseUrl}</Text>
      </Panel>
    </Screen>
  );
}

const styles = StyleSheet.create({
  actions: { gap: space.md },
  metaLabel: { ...type.label, color: color.ink400 },
  metaValue: { ...type.meta, color: color.ink700 },
});
