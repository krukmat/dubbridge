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

function toProjectViewState(projects: ProjectSummary[]): ViewState {
  return projects.length === 0 ? { kind: "empty" } : { kind: "ready", projects };
}

function projectListErrorMessage(error: { kind: string; message?: string; status?: number }) {
  return error.kind === "network"
    ? error.message ?? "Network request failed."
    : error.kind === "forbidden"
      ? "You do not have access to this organization's projects."
      : `Request failed with status ${error.status}.`;
}

async function fetchProjects(
  gatewayBaseUrl: string,
  orgId: string,
  sessionRef: string | null,
) {
  const client = createGatewayClient({ gatewayBaseUrl });
  return client.get<ProjectSummary[]>(`/api/orgs/${orgId}/projects`, sessionRef);
}

function ProjectListBody({
  viewState,
  refreshing,
  onRefresh,
  onOpenProject,
}: {
  viewState: Extract<ViewState, { kind: "ready" } | { kind: "empty" }>;
  refreshing: boolean;
  onRefresh: () => void;
  onOpenProject: (project: ProjectSummary) => void;
}) {
  return (
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
  );
}

function useProjectListState(
  gatewayBaseUrl: string,
  orgId: string,
  sessionRef: string | null,
  logout: () => Promise<void>,
  onSessionRotation: (rotation: string | null) => Promise<void>,
) {
  const [viewState, setViewState] = useState<ViewState>({ kind: "loading" });
  const [refreshing, setRefreshing] = useState(false);

  const loadProjects = useCallback(async (): Promise<void> => {
    const result = await fetchProjects(gatewayBaseUrl, orgId, sessionRef);

    if (result.ok) {
      await onSessionRotation(result.value.sessionRotation);
      setViewState(toProjectViewState(result.value.data));
      return;
    }
    if (result.error.kind === "session_expired") {
      await logout();
      return;
    }
    setViewState({ kind: "error", message: projectListErrorMessage(result.error) });
  }, [gatewayBaseUrl, logout, onSessionRotation, orgId, sessionRef]);

  useEffect(() => {
    void (async () => {
      setViewState({ kind: "loading" });
      await loadProjects();
    })();
  }, [loadProjects]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadProjects();
    setRefreshing(false);
  }, [loadProjects]);

  const onRetry = useCallback(() => {
    setViewState({ kind: "loading" });
    void loadProjects();
  }, [loadProjects]);

  return { viewState, refreshing, onRefresh, onRetry };
}

export function ProjectListScreen({
  gatewayBaseUrl,
  orgId,
  onOpenProject,
}: ProjectListScreenProps) {
  const auth = useAuth();
  const { viewState, refreshing, onRefresh, onRetry } = useProjectListState(
    gatewayBaseUrl,
    orgId,
    auth.sessionRef,
    auth.logout,
    auth.onSessionRotation,
  );

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
        <ProjectListBody
          viewState={viewState}
          refreshing={refreshing}
          onRefresh={onRefresh}
          onOpenProject={onOpenProject}
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
