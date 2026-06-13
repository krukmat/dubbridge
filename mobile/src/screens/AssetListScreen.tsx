import { useCallback, useEffect, useState } from "react";
import {
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
} from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { Badge, statusTone } from "../components/Badge";
import { Card } from "../components/Card";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";

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
    <Screen testID="asset-list-screen" edges={["bottom"]}>
      <ScreenHeader kicker="Assets" title="Asset list" />

      {viewState.kind === "loading" ? (
        <StateView
          kind="loading"
          title="Loading assets…"
          message="Fetching your assets from the gateway."
        />
      ) : null}

      {viewState.kind === "empty" ? (
        <ScrollView
          style={styles.scroll}
          contentContainerStyle={styles.listContent}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }
        >
          <StateView
            testID="asset-list-empty-state"
            kind="empty"
            title="No assets yet"
            message="Your workspace does not have any assets to show."
          />
        </ScrollView>
      ) : null}

      {viewState.kind === "error" ? (
        <StateView
          kind="error"
          title="Could not load assets"
          message={viewState.message}
          onRetry={onRetry}
        />
      ) : null}

      {viewState.kind === "ready" ? (
        <ScrollView
          style={styles.scroll}
          contentContainerStyle={styles.listContent}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }
        >
          {viewState.assets.map((asset) => (
            <Card
              key={asset.id}
              testID={`asset-card-${asset.id}`}
              onPress={() => onOpenAsset(asset)}
            >
              <Text style={styles.assetTitle}>{asset.title}</Text>
              <Badge
                label={formatStatus(asset.status)}
                tone={statusTone(asset.status)}
              />
              <Text style={styles.assetMeta}>{asset.id}</Text>
            </Card>
          ))}
        </ScrollView>
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  scroll: { flex: 1 },
  listContent: { gap: space.md, paddingBottom: space.xl },
  assetTitle: { ...type.heading, color: color.ink900 },
  assetMeta: { ...type.meta, color: color.ink500 },
});
