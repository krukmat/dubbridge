import { useCallback, useEffect, useState } from "react";
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

export type ProjectAssetSummary = {
  id: string;
  title: string;
  status: string;
};

export type ProjectDetail = {
  id: string;
  org_id: string;
  name: string;
  asset_summaries: ProjectAssetSummary[];
  created_at: string;
};

type ProjectDetailScreenProps = {
  gatewayBaseUrl: string;
  orgId: string;
  projectId: string;
  onOpenAsset: (assetId: string, assetTitle: string) => void;
};

type ViewState =
  | { kind: "loading" }
  | { kind: "ready"; detail: ProjectDetail }
  | { kind: "error"; message: string };

export function ProjectDetailScreen({
  gatewayBaseUrl,
  orgId,
  projectId,
  onOpenAsset,
}: ProjectDetailScreenProps) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<ViewState>({ kind: "loading" });

  const loadDetail = useCallback(async (): Promise<void> => {
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<ProjectDetail>(
      `/api/orgs/${orgId}/projects/${projectId}`,
      auth.sessionRef,
    );

    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      setViewState({ kind: "ready", detail: result.value.data });
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
          ? "You do not have access to this project."
          : `Request failed with status ${result.error.status}.`;
    setViewState({ kind: "error", message });
  }, [auth, gatewayBaseUrl, orgId, projectId]);

  useEffect(() => {
    let isActive = true;

    void (async () => {
      setViewState({ kind: "loading" });
      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await client.get<ProjectDetail>(
        `/api/orgs/${orgId}/projects/${projectId}`,
        auth.sessionRef,
      );

      if (!isActive) return;

      if (result.ok) {
        await auth.onSessionRotation(result.value.sessionRotation);
        if (!isActive) return;
        setViewState({ kind: "ready", detail: result.value.data });
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
            ? "You do not have access to this project."
            : `Request failed with status ${result.error.status}.`;
      setViewState({ kind: "error", message });
    })();

    return () => {
      isActive = false;
    };
  }, [auth, gatewayBaseUrl, orgId, projectId]);

  const onRetry = useCallback(() => {
    setViewState({ kind: "loading" });
    void loadDetail();
  }, [loadDetail]);

  return (
    <View testID="project-detail-screen" style={styles.container}>
      {viewState.kind === "loading" ? (
        <View style={styles.centerPanel}>
          <ActivityIndicator size="small" color="#1a5d50" />
          <Text style={styles.panelTitle}>Loading project…</Text>
          <Text style={styles.panelCopy}>
            Fetching project details from the gateway.
          </Text>
        </View>
      ) : null}

      {viewState.kind === "error" ? (
        <View style={styles.centerPanel}>
          <Text style={styles.panelTitle}>Could not load project</Text>
          <Text style={styles.panelCopy}>{viewState.message}</Text>
          <Pressable onPress={onRetry} style={styles.retryButton}>
            <Text style={styles.retryLabel}>Retry</Text>
          </Pressable>
        </View>
      ) : null}

      {viewState.kind === "ready" ? (
        <ScrollView contentContainerStyle={styles.listContent}>
          <View style={styles.header}>
            <Text style={styles.kicker}>Project</Text>
            <Text style={styles.title}>{viewState.detail.name}</Text>
          </View>

          <Text style={styles.sectionHeader}>Linked assets</Text>

          {viewState.detail.asset_summaries.length === 0 ? (
            <View testID="project-detail-empty-assets" style={styles.emptyPanel}>
              <Text style={styles.panelTitle}>No assets linked</Text>
              <Text style={styles.panelCopy}>
                This project does not have any linked assets yet.
              </Text>
            </View>
          ) : (
            viewState.detail.asset_summaries.map((asset) => (
              <Pressable
                key={asset.id}
                testID={`asset-row-${asset.id}`}
                onPress={() => onOpenAsset(asset.id, asset.title)}
                style={styles.assetCard}
              >
                <Text style={styles.assetTitle}>{asset.title}</Text>
                <Text style={styles.assetMeta}>{asset.status}</Text>
              </Pressable>
            ))
          )}
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
    marginBottom: 8,
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
  sectionHeader: {
    fontSize: 16,
    fontWeight: "700",
    color: "#10212a",
    marginBottom: 4,
  },
  centerPanel: {
    borderRadius: 10,
    backgroundColor: "#ffffff",
    borderWidth: 1,
    borderColor: "#d7dfd7",
    padding: 20,
    gap: 10,
  },
  emptyPanel: {
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
