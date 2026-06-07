import { useEffect, useState } from "react";
import {
  ActivityIndicator,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import type { AssetSummary } from "./AssetListScreen";

type AssetDetailScreenProps = {
  assetId: string;
  gatewayBaseUrl: string;
};

type AssetDetailViewState =
  | { kind: "loading" }
  | { kind: "ready"; asset: AssetSummary }
  | { kind: "error"; message: string }
  | { kind: "not_available" };

function formatStatus(status: string): string {
  return status
    .split("_")
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");
}

export function AssetDetailScreen({
  assetId,
  gatewayBaseUrl,
}: AssetDetailScreenProps) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<AssetDetailViewState>({
    kind: "loading",
  });

  useEffect(() => {
    let isActive = true;

    async function loadAssetDetail(): Promise<void> {
      setViewState({ kind: "loading" });

      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await client.get<AssetSummary>(
        `/api/assets/${assetId}`,
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
              ? "You do not have access to this asset."
              : `Gateway request failed with status ${result.error.status}.`;
        setViewState({ kind: "error", message });
        return;
      }

      await auth.onSessionRotation(result.value.sessionRotation);

      if (!isActive) {
        return;
      }

      setViewState({ kind: "ready", asset: result.value.data });
    }

    void loadAssetDetail();

    return () => {
      isActive = false;
    };
  }, [assetId, auth, gatewayBaseUrl]);

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.header}>
        <Text style={styles.kicker}>Gateway asset detail</Text>
        <Text style={styles.title}>Asset detail</Text>
        <Text style={styles.copy}>
          This screen reads the current S1 asset summary through the session
          gateway.
        </Text>
      </View>

      {viewState.kind === "loading" ? (
        <View style={styles.panel}>
          <ActivityIndicator size="small" color="#855f19" />
          <Text style={styles.panelTitle}>Loading asset detail…</Text>
        </View>
      ) : null}

      {viewState.kind === "error" ? (
        <View style={styles.panel}>
          <Text style={styles.panelTitle}>Could not load asset detail</Text>
          <Text style={styles.panelCopy}>{viewState.message}</Text>
        </View>
      ) : null}

      {viewState.kind === "not_available" ? (
        <View style={styles.panel}>
          <Text style={styles.panelTitle}>Asset detail not available yet</Text>
          <Text style={styles.panelCopy}>
            This asset detail surface is not available on the current backend.
          </Text>
        </View>
      ) : null}

      {viewState.kind === "ready" ? (
        <>
          <View style={styles.panel}>
            <Text style={styles.assetTitle}>{viewState.asset.title}</Text>
            <Text style={styles.metaLabel}>Status</Text>
            <Text style={styles.metaValue}>
              {formatStatus(viewState.asset.status)}
            </Text>
            <Text style={styles.metaLabel}>Asset ID</Text>
            <Text style={styles.metaValue}>{viewState.asset.id}</Text>
            <Text style={styles.metaLabel}>Uploader ID</Text>
            <Text style={styles.metaValue}>{viewState.asset.uploader_id}</Text>
          </View>

          <View style={styles.panel}>
            <Text style={styles.panelTitle}>Downstream processing</Text>
            <Text style={styles.panelCopy}>
              Not available yet. S4–S9 product surfaces have not been delivered
              on this mobile client.
            </Text>
          </View>
        </>
      ) : null}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#f7f0e4",
  },
  content: {
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
    color: "#855f19",
  },
  title: {
    fontSize: 32,
    fontWeight: "700",
    color: "#1f1305",
  },
  copy: {
    fontSize: 16,
    lineHeight: 24,
    color: "#594b39",
  },
  panel: {
    borderRadius: 10,
    backgroundColor: "#fffaf2",
    borderWidth: 1,
    borderColor: "#e8d9c4",
    padding: 20,
    gap: 8,
  },
  panelTitle: {
    fontSize: 18,
    fontWeight: "700",
    color: "#1f1305",
  },
  panelCopy: {
    fontSize: 15,
    lineHeight: 22,
    color: "#6a5d4a",
  },
  assetTitle: {
    fontSize: 24,
    fontWeight: "700",
    color: "#1f1305",
    marginBottom: 6,
  },
  metaLabel: {
    fontSize: 12,
    fontWeight: "700",
    textTransform: "uppercase",
    color: "#8c6d34",
  },
  metaValue: {
    fontSize: 15,
    lineHeight: 22,
    color: "#3f3324",
  },
});
