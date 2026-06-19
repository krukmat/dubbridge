import { useCallback, useEffect, useState } from "react";
import {
  FlatList,
  RefreshControl,
  StyleSheet,
  Text,
} from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { Card } from "../components/Card";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";

export type ProjectSummary = {
  id: string;
  org_id: string;
  name: string;
  created_at: string;
};

type ProjectListScreenProps = {
  gatewayBaseUrl: string;
  orgId: string;
  onOpenProject: (project: ProjectSummary) => void;
};

type ViewState =
  | { kind: "loading" }
  | { kind: "ready"; projects: ProjectSummary[] }
  | { kind: "empty" }
  | { kind: "error"; message: string };

export function ProjectListScreen({
  gatewayBaseUrl,
  orgId,
  onOpenProject,
}: ProjectListScreenProps) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<ViewState>({ kind: "loading" });
  const [refreshing, setRefreshing] = useState(false);

  const loadProjects = useCallback(async (): Promise<void> => {
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<ProjectSummary[]>(
      `/api/orgs/${orgId}/projects`,
      auth.sessionRef,
    );

    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      if (result.value.data.length === 0) {
        setViewState({ kind: "empty" });
      } else {
        setViewState({ kind: "ready", projects: result.value.data });
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
          ? "You do not have access to this organization's projects."
          : `Request failed with status ${result.error.status}.`;
    setViewState({ kind: "error", message });
  }, [auth, gatewayBaseUrl, orgId]);

  useEffect(() => {
    let isActive = true;

    void (async () => {
      setViewState({ kind: "loading" });
      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await client.get<ProjectSummary[]>(
        `/api/orgs/${orgId}/projects`,
        auth.sessionRef,
      );

      if (!isActive) return;

      if (result.ok) {
        await auth.onSessionRotation(result.value.sessionRotation);
        if (!isActive) return;
        if (result.value.data.length === 0) {
          setViewState({ kind: "empty" });
        } else {
          setViewState({ kind: "ready", projects: result.value.data });
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
            ? "You do not have access to this organization's projects."
            : `Request failed with status ${result.error.status}.`;
      setViewState({ kind: "error", message });
    })();

    return () => {
      isActive = false;
    };
  }, [auth, gatewayBaseUrl, orgId]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadProjects();
    setRefreshing(false);
  }, [loadProjects]);

  const onRetry = useCallback(() => {
    setViewState({ kind: "loading" });
    void loadProjects();
  }, [loadProjects]);

  return (
    <Screen testID="project-list-screen" edges={["bottom"]}>
      <ScreenHeader kicker="Organization projects" title="Projects" />

      {viewState.kind === "loading" ? (
        <StateView
          kind="loading"
          title="Loading projects…"
          message="Fetching your organization's projects from the gateway."
        />
      ) : null}

      {viewState.kind === "error" ? (
        <StateView
          kind="error"
          title="Could not load projects"
          message={viewState.message}
          onRetry={onRetry}
        />
      ) : null}

      {(viewState.kind === "ready" || viewState.kind === "empty") ? (
        <FlatList
          style={styles.scroll}
          contentContainerStyle={
            viewState.kind === "empty" ? styles.emptyContent : styles.listContent
          }
          data={viewState.kind === "ready" ? viewState.projects : []}
          keyExtractor={(project) => project.id}
          renderItem={({ item: project }) => (
            <Card
              testID={`project-card-${project.id}`}
              onPress={() => onOpenProject(project)}
              trailing="chevron"
            >
              <Text style={styles.projectName}>{project.name}</Text>
              <Text style={styles.projectMeta}>{project.id}</Text>
            </Card>
          )}
          ListEmptyComponent={
            <StateView
              testID="project-list-empty-state"
              kind="empty"
              title="No projects yet"
              message="This organization does not have any projects to show."
            />
          }
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }
        />
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  scroll: { flex: 1 },
  listContent: { gap: space.md, paddingBottom: space.xl },
  emptyContent: { flexGrow: 1 },
  projectName: { ...type.heading, color: color.ink900 },
  projectMeta: { ...type.meta, color: color.ink500 },
});
