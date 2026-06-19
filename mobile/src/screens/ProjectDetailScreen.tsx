import { useCallback, useEffect, useState } from "react";
import {
  FlatList,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { Badge, statusTone } from "../components/Badge";
import { Card } from "../components/Card";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";

export type ProjectAssetSummary = {
  id: string;
  title: string;
  status: string;
};

export type ProjectDetail = {
  id: string;
  org_id: string;
  name: string;
  assets: ProjectAssetSummary[];
  target_languages: Array<{
    id: string;
    project_id: string;
    source_lang: string;
    target_lang: string;
    created_at: string;
  }>;
  created_at: string;
  updated_at: string;
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

function formatStatus(status: string): string {
  return status
    .split("_")
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");
}

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
    <Screen testID="project-detail-screen" edges={["bottom"]}>
      {viewState.kind === "loading" ? (
        <StateView
          kind="loading"
          title="Loading project…"
          message="Fetching project details from the gateway."
        />
      ) : null}

      {viewState.kind === "error" ? (
        <StateView
          kind="error"
          title="Could not load project"
          message={viewState.message}
          onRetry={onRetry}
        />
      ) : null}

      {viewState.kind === "ready" ? (
        <ScrollView style={styles.scroll} contentContainerStyle={styles.listContent}>
          <ScreenHeader kicker="Project" title={viewState.detail.name} />

          <Text style={styles.sectionHeader}>Linked assets</Text>

          <FlatList
            data={viewState.detail.assets}
            keyExtractor={(asset) => asset.id}
            renderItem={({ item: asset }) => (
              <Card
                testID={`asset-row-${asset.id}`}
                onPress={() => onOpenAsset(asset.id, asset.title)}
                trailing="chevron"
              >
                <Text style={styles.assetTitle}>{asset.title}</Text>
                <Badge label={formatStatus(asset.status)} tone={statusTone(asset.status)} />
              </Card>
            )}
            ListEmptyComponent={
              <StateView
                testID="project-detail-empty-assets"
                kind="empty"
                title="No assets linked"
                message="This project does not have any linked assets yet."
              />
            }
            scrollEnabled={false}
            ItemSeparatorComponent={() => <View style={styles.separator} />}
          />

          <Text style={styles.sectionHeader}>Target languages</Text>

          <FlatList
            data={viewState.detail.target_languages}
            keyExtractor={(language) => language.id}
            renderItem={({ item: language }) => (
              <Panel testID={`target-language-${language.id}`}>
                <Text style={styles.assetTitle}>{language.source_lang} to {language.target_lang}</Text>
              </Panel>
            )}
            ListEmptyComponent={
              <StateView
                testID="project-detail-empty-languages"
                kind="empty"
                title="No target languages"
                message="This project has no target languages configured."
              />
            }
            scrollEnabled={false}
            ItemSeparatorComponent={() => <View style={styles.separator} />}
          />
        </ScrollView>
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  scroll: { flex: 1 },
  listContent: { gap: space.md, paddingBottom: space.xl },
  sectionHeader: { ...type.bodyStrong, color: color.ink900 },
  assetTitle: { ...type.heading, color: color.ink900 },
  separator: { height: space.md, backgroundColor: "transparent" },
});
