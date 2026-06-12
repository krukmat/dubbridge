import { useCallback, useEffect, useState } from "react";
import {
  ActivityIndicator,
  Pressable,
  RefreshControl,
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
  | { kind: "error"; message: string };

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
  const [refreshing, setRefreshing] = useState(false);

  const loadAssets = useCallback(async (): Promise<void> => {
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<AssetSummary[]>(
      "/api/assets",
      auth.sessionRef,
    );

    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      if (result.value.data.length === 0) {
        setViewState({ kind: "empty" });
      } else {
        setViewState({ kind: "ready", assets: result.value.data });
      }
      return;
    }

    if (result.error.kind === "session_expired") {
      await auth.logout();
      return;
    }

    const message =
      result.error.kind === "network"
        ? result.error.message
        : result.error.kind === "forbidden"
          ? "You do not have access to the asset list."
          : `Request failed with status ${result.error.status}.`;
    setViewState({ kind: "error", message });
  }, [auth, gatewayBaseUrl]);

  useEffect(() => {
    let isActive = true;

    void (async () => {
      setViewState({ kind: "loading" });
      // Only apply state from this effect invocation if it's still current.
      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await client.get<AssetSummary[]>(
        "/api/assets",
        auth.sessionRef,
      );

      if (!isActive) return;

      if (result.ok) {
        await auth.onSessionRotation(result.value.sessionRotation);
        if (!isActive) return;
        if (result.value.data.length === 0) {
          setViewState({ kind: "empty" });
        } else {
          setViewState({ kind: "ready", assets: result.value.data });
        }
        return;
      }

      if (result.error.kind === "session_expired") {
        await auth.logout();
        return;
      }

      const message =
        result.error.kind === "network"
          ? result.error.message
          : result.error.kind === "forbidden"
            ? "You do not have access to the asset list."
            : `Request failed with status ${result.error.status}.`;
      setViewState({ kind: "error", message });
    })();

    return () => {
      isActive = false;
    };
  }, [auth, gatewayBaseUrl]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadAssets();
    setRefreshing(false);
  }, [loadAssets]);

  const onRetry = useCallback(() => {
    setViewState({ kind: "loading" });
    void loadAssets();
  }, [loadAssets]);

  return (
    <View testID="asset-list-screen" style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.kicker}>My assets</Text>
        <Text style={styles.title}>Asset list</Text>
      </View>

      {viewState.kind === "loading" ? (
        <View style={styles.centerPanel}>
          <ActivityIndicator size="small" color="#1a5d50" />
          <Text style={styles.panelTitle}>Loading assets…</Text>
          <Text style={styles.panelCopy}>
            Fetching your assets from the gateway.
          </Text>
        </View>
      ) : null}

      {viewState.kind === "empty" ? (
        <ScrollView
          contentContainerStyle={styles.centerPanelScroll}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }
        >
          <View testID="asset-list-empty-state" style={styles.centerPanel}>
            <Text style={styles.panelTitle}>No assets yet</Text>
            <Text style={styles.panelCopy}>
              Your authenticated workspace does not have any assets to show.
            </Text>
          </View>
        </ScrollView>
      ) : null}

      {viewState.kind === "error" ? (
        <View style={styles.centerPanel}>
          <Text style={styles.panelTitle}>Could not load assets</Text>
          <Text style={styles.panelCopy}>{viewState.message}</Text>
          <Pressable onPress={onRetry} style={styles.retryButton}>
            <Text style={styles.retryLabel}>Retry</Text>
          </Pressable>
        </View>
      ) : null}

      {viewState.kind === "ready" ? (
        <ScrollView
          contentContainerStyle={styles.listContent}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }
        >
          {viewState.assets.map((asset) => (
            <Pressable
              key={asset.id}
              testID={`asset-card-${asset.id}`}
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
  centerPanelScroll: {
    flexGrow: 1,
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
  retryButton: {
    marginTop: 4,
    alignSelf: "flex-start",
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
    backgroundColor: "#1a5d50",
  },
  retryLabel: {
    fontSize: 14,
    fontWeight: "600",
    color: "#ffffff",
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
