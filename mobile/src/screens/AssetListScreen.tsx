import { useEffect, useState } from "react";
import {
  ActivityIndicator,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";

export type AssetSummary = {
  id: string;
  title: string;
  uploader_id: string;
  status: string;
  created_at: string;
  updated_at: string;
};

type AssetListScreenProps = {
  gatewayBaseUrl: string;
  onOpenAsset: (asset: AssetSummary) => void;
};

type AssetListViewState =
  | { kind: "loading" }
  | { kind: "ready"; assets: AssetSummary[] }
  | { kind: "empty" }
  | { kind: "error"; message: string }
  | { kind: "not_available" };

function formatStatus(status: string): string {
  return status
    .split("_")
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");
}

export function AssetListScreen({
  gatewayBaseUrl,
  onOpenAsset,
}: AssetListScreenProps) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<AssetListViewState>({
    kind: "loading",
  });

  useEffect(() => {
    let isActive = true;

    async function loadAssets(): Promise<void> {
      setViewState({ kind: "loading" });

      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await client.get<AssetSummary[]>(
        "/api/assets?view=mobile",
        auth.sessionRef,
      );

      if (!isActive) {
        return;
      }

      if (!result.ok) {
        if (result.error.kind === "session_expired") {
          await auth.logout();
          return;
        }

        if (result.error.kind === "http" && result.error.status === 404) {
          setViewState({ kind: "not_available" });
          return;
        }

        const message =
          result.error.kind === "network"
            ? result.error.message
            : result.error.kind === "forbidden"
              ? "You do not have access to the mobile asset list."
              : `Gateway request failed with status ${result.error.status}.`;
        setViewState({ kind: "error", message });
        return;
      }

      await auth.onSessionRotation(result.value.sessionRotation);

      if (!isActive) {
        return;
      }

      if (result.value.data.length === 0) {
        setViewState({ kind: "empty" });
        return;
      }

      setViewState({ kind: "ready", assets: result.value.data });
    }

    void loadAssets();

    return () => {
      isActive = false;
    };
  }, [auth, gatewayBaseUrl]);

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.kicker}>Gateway assets</Text>
        <Text style={styles.title}>Asset list</Text>
        <Text style={styles.copy}>
          This view loads the authenticated mobile asset surface through the
          session gateway.
        </Text>
      </View>

      {viewState.kind === "loading" ? (
        <View style={styles.centerPanel}>
          <ActivityIndicator size="small" color="#1a5d50" />
          <Text style={styles.panelTitle}>Loading assets…</Text>
          <Text style={styles.panelCopy}>
            Fetching the mobile asset list from the gateway.
          </Text>
        </View>
      ) : null}

      {viewState.kind === "empty" ? (
        <View style={styles.centerPanel}>
          <Text style={styles.panelTitle}>No assets yet</Text>
          <Text style={styles.panelCopy}>
            Your authenticated workspace does not have any assets to show.
          </Text>
        </View>
      ) : null}

      {viewState.kind === "error" ? (
        <View style={styles.centerPanel}>
          <Text style={styles.panelTitle}>Could not load assets</Text>
          <Text style={styles.panelCopy}>{viewState.message}</Text>
        </View>
      ) : null}

      {viewState.kind === "not_available" ? (
        <View style={styles.centerPanel}>
          <Text style={styles.panelTitle}>Asset list not available yet</Text>
          <Text style={styles.panelCopy}>
            The mobile list endpoint is not live on this backend surface yet.
          </Text>
        </View>
      ) : null}

      {viewState.kind === "ready" ? (
        <ScrollView contentContainerStyle={styles.listContent}>
          {viewState.assets.map((asset) => (
            <Pressable
              key={asset.id}
              onPress={() => onOpenAsset(asset)}
              style={styles.assetCard}
            >
              <Text style={styles.assetTitle}>{asset.title}</Text>
              <Text style={styles.assetMeta}>{formatStatus(asset.status)}</Text>
              <Text style={styles.assetMeta}>{asset.id}</Text>
            </Pressable>
          ))}
        </ScrollView>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#f2f4ee",
    padding: 24,
    gap: 20,
  },
  header: {
    marginTop: 24,
    gap: 10,
  },
  kicker: {
    fontSize: 12,
    fontWeight: "700",
    textTransform: "uppercase",
    color: "#537462",
  },
  title: {
    fontSize: 32,
    fontWeight: "700",
    color: "#10212a",
  },
  copy: {
    fontSize: 16,
    lineHeight: 24,
    color: "#425059",
  },
  centerPanel: {
    borderRadius: 10,
    backgroundColor: "#ffffff",
    borderWidth: 1,
    borderColor: "#d7dfd7",
    padding: 20,
    gap: 10,
  },
  panelTitle: {
    fontSize: 18,
    fontWeight: "700",
    color: "#10212a",
  },
  panelCopy: {
    fontSize: 15,
    lineHeight: 22,
    color: "#52616a",
  },
  listContent: {
    gap: 12,
    paddingBottom: 24,
  },
  assetCard: {
    borderRadius: 10,
    backgroundColor: "#ffffff",
    borderWidth: 1,
    borderColor: "#d7dfd7",
    padding: 16,
    gap: 8,
  },
  assetTitle: {
    fontSize: 18,
    fontWeight: "700",
    color: "#10212a",
  },
  assetMeta: {
    fontSize: 14,
    lineHeight: 20,
    color: "#5a6870",
  },
});
