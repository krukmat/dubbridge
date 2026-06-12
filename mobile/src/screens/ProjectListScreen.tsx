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
    <View testID="project-list-screen" style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.kicker}>Organization projects</Text>
        <Text style={styles.title}>Projects</Text>
      </View>

      {viewState.kind === "loading" ? (
        <View style={styles.centerPanel}>
          <ActivityIndicator size="small" color="#1a5d50" />
          <Text style={styles.panelTitle}>Loading projects…</Text>
          <Text style={styles.panelCopy}>
            Fetching your organization's projects from the gateway.
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
          <View testID="project-list-empty-state" style={styles.centerPanel}>
            <Text style={styles.panelTitle}>No projects yet</Text>
            <Text style={styles.panelCopy}>
              This organization does not have any projects to show.
            </Text>
          </View>
        </ScrollView>
      ) : null}

      {viewState.kind === "error" ? (
        <View style={styles.centerPanel}>
          <Text style={styles.panelTitle}>Could not load projects</Text>
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
          {viewState.projects.map((project) => (
            <Pressable
              key={project.id}
              testID={`project-card-${project.id}`}
              onPress={() => onOpenProject(project)}
              style={styles.projectCard}
            >
              <Text style={styles.projectName}>{project.name}</Text>
              <Text style={styles.projectMeta}>{project.id}</Text>
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
  projectCard: {
    borderRadius: 10,
    backgroundColor: "#ffffff",
    borderWidth: 1,
    borderColor: "#d7dfd7",
    padding: 16,
    gap: 8,
  },
  projectName: {
    fontSize: 18,
    fontWeight: "700",
    color: "#10212a",
  },
  projectMeta: {
    fontSize: 14,
    lineHeight: 20,
    color: "#5a6870",
  },
});
