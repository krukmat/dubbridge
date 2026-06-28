import { useCallback, useEffect, useState } from "react";
import { FlatList, ScrollView, StyleSheet, Text, View } from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { formatStatusLabel } from "../format";
import { Badge, statusTone } from "../components/Badge";
import { Card } from "../components/Card";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";

export type ProjectAssetSummary = { id: string; title: string; status: string };

export type ProjectDetail = {
  id: string;
  org_id: string;
  name: string;
  assets: ProjectAssetSummary[];
  target_languages: Array<{ id: string; project_id: string; source_lang: string; target_lang: string; created_at: string }>;
  created_at: string;
  updated_at: string;
};

type ProjectDetailScreenProps = {
  gatewayBaseUrl: string;
  orgId: string;
  projectId: string;
  onOpenAsset: (assetId: string, assetTitle: string) => void;
};

type ViewState = { kind: "loading" } | { kind: "ready"; detail: ProjectDetail } | { kind: "error"; message: string };

function projectDetailError(error: { kind: string; message?: string; status?: number }) {
  return error.kind === "network" ? error.message ?? "Network request failed." : error.kind === "forbidden" ? "You do not have access to this project." : `Request failed with status ${error.status}.`;
}

function ProjectAssetList({ assets, onOpenAsset }: { assets: ProjectAssetSummary[]; onOpenAsset: (id: string, title: string) => void }) {
  return (
    <FlatList data={assets} keyExtractor={(a) => a.id} scrollEnabled={false} ItemSeparatorComponent={() => <View style={styles.separator} />}
      renderItem={({ item: asset }) => (
        <Card testID={`asset-row-${asset.id}`} onPress={() => onOpenAsset(asset.id, asset.title)} trailing="chevron">
          <Text style={styles.assetTitle}>{asset.title}</Text>
          <Badge label={formatStatusLabel(asset.status)} tone={statusTone(asset.status)} />
        </Card>
      )}
      ListEmptyComponent={<StateView testID="project-detail-empty-assets" kind="empty" title="No assets linked" message="This project does not have any linked assets yet." />}
    />
  );
}

function TargetLanguageList({ targetLanguages }: { targetLanguages: ProjectDetail["target_languages"] }) {
  return (
    <FlatList data={targetLanguages} keyExtractor={(l) => l.id} scrollEnabled={false} ItemSeparatorComponent={() => <View style={styles.separator} />}
      renderItem={({ item: language }) => (
        <Panel testID={`target-language-${language.id}`}>
          <Text style={styles.assetTitle}>{language.source_lang} to {language.target_lang}</Text>
        </Panel>
      )}
      ListEmptyComponent={<StateView testID="project-detail-empty-languages" kind="empty" title="No target languages" message="This project has no target languages configured." />}
    />
  );
}

function ProjectDetailBody({ detail, onOpenAsset }: { detail: ProjectDetail; onOpenAsset: (id: string, title: string) => void }) {
  return (
    <ScrollView style={styles.scroll} contentContainerStyle={styles.listContent}>
      <ScreenHeader kicker="Project" title={detail.name} />
      <Text style={styles.sectionHeader}>Linked assets</Text>
      <ProjectAssetList assets={detail.assets} onOpenAsset={onOpenAsset} />
      <Text style={styles.sectionHeader}>Target languages</Text>
      <TargetLanguageList targetLanguages={detail.target_languages} />
    </ScrollView>
  );
}

export function ProjectDetailScreen({ gatewayBaseUrl, orgId, projectId, onOpenAsset }: ProjectDetailScreenProps) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<ViewState>({ kind: "loading" });

  const loadDetail = useCallback(async (signal?: { active: boolean }): Promise<void> => {
    setViewState({ kind: "loading" });
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<ProjectDetail>(`/api/orgs/${orgId}/projects/${projectId}`, auth.sessionRef);
    if (signal && !signal.active) return;
    if (result.ok) { await auth.onSessionRotation(result.value.sessionRotation); setViewState({ kind: "ready", detail: result.value.data }); return; }
    if (result.error.kind === "session_expired") { await auth.logout(); return; }
    setViewState({ kind: "error", message: projectDetailError(result.error) });
  }, [auth, gatewayBaseUrl, orgId, projectId]);

  useEffect(() => {
    const signal = { active: true };
    void loadDetail(signal);
    return () => { signal.active = false; };
  }, [loadDetail]);

  const onRetry = useCallback(() => { void loadDetail(); }, [loadDetail]);

  return (
    <Screen testID="project-detail-screen" edges={["bottom"]}>
      {viewState.kind === "loading" ? <StateView kind="loading" title="Loading project…" message="Fetching project details from the gateway." /> : null}
      {viewState.kind === "error" ? <StateView kind="error" title="Could not load project" message={viewState.message} onRetry={onRetry} /> : null}
      {viewState.kind === "ready" ? <ProjectDetailBody detail={viewState.detail} onOpenAsset={onOpenAsset} /> : null}
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
