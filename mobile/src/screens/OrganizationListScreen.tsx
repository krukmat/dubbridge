import { useCallback, useEffect, useState } from "react";
import {
  ActivityIndicator,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";

export type OrgRole = "owner" | "admin" | "editor" | "reviewer" | "viewer";

export type OrganizationSummary = {
  id: string;
  name: string;
  viewer_role: OrgRole;
  created_at: string;
  updated_at: string;
};

type Props = {
  gatewayBaseUrl: string;
  onOpenProjects: (organization: OrganizationSummary) => void;
  onOpenMembers: (organization: OrganizationSummary) => void;
};

type ViewState =
  | { kind: "loading" }
  | { kind: "ready"; organizations: OrganizationSummary[] }
  | { kind: "error"; message: string };

function errorMessage(kind: "forbidden" | "network" | "http", detail?: string | number) {
  if (kind === "forbidden") return "You do not have access to organizations.";
  if (kind === "network") return String(detail ?? "Network request failed.");
  return `Request failed with status ${detail}.`;
}

export function OrganizationListScreen({
  gatewayBaseUrl,
  onOpenProjects,
  onOpenMembers,
}: Props) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<ViewState>({ kind: "loading" });
  const [name, setName] = useState("");
  const [createError, setCreateError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  const loadOrganizations = useCallback(async () => {
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<OrganizationSummary[]>("/api/orgs", auth.sessionRef);

    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      setViewState({ kind: "ready", organizations: result.value.data });
      return;
    }

    if (result.error.kind === "session_expired") {
      await auth.logout();
      return;
    }

    setViewState({
      kind: "error",
      message: errorMessage(
        result.error.kind,
        result.error.kind === "network" ? result.error.message : result.error.kind === "http" ? result.error.status : undefined,
      ),
    });
  }, [auth, gatewayBaseUrl]);

  useEffect(() => {
    void loadOrganizations();
  }, [loadOrganizations]);

  const createOrganization = useCallback(async () => {
    const normalizedName = name.trim();
    if (!normalizedName) {
      setCreateError("Organization name is required.");
      return;
    }

    setCreating(true);
    setCreateError(null);
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.post<OrganizationSummary>(
      "/api/orgs",
      auth.sessionRef,
      { name: normalizedName },
    );
    setCreating(false);

    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      setName("");
      onOpenProjects(result.value.data);
      return;
    }

    if (result.error.kind === "session_expired") {
      await auth.logout();
      return;
    }

    setCreateError(
      errorMessage(
        result.error.kind,
        result.error.kind === "network" ? result.error.message : result.error.kind === "http" ? result.error.status : undefined,
      ),
    );
  }, [auth, gatewayBaseUrl, name, onOpenProjects]);

  return (
    <View testID="organization-list-screen" style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.kicker}>Workspace</Text>
        <Text style={styles.title}>Organizations</Text>
        <Text style={styles.copy}>Choose the organization whose projects and members you want to manage.</Text>
      </View>

      <View style={styles.createPanel}>
        <Text style={styles.sectionTitle}>Create organization</Text>
        <TextInput
          testID="organization-name-input"
          accessibilityLabel="Organization name"
          value={name}
          onChangeText={setName}
          placeholder="Organization name"
          autoCapitalize="words"
          style={styles.input}
        />
        {createError ? <Text style={styles.errorText}>{createError}</Text> : null}
        <Pressable
          testID="organization-create"
          disabled={creating}
          onPress={() => void createOrganization()}
          style={[styles.primaryButton, creating && styles.disabledButton]}
        >
          <Text style={styles.primaryButtonText}>{creating ? "Creating..." : "Create and open"}</Text>
        </Pressable>
      </View>

      {viewState.kind === "loading" ? (
        <View style={styles.panel}>
          <ActivityIndicator color="#1a5d50" />
          <Text style={styles.panelTitle}>Loading organizations...</Text>
        </View>
      ) : null}

      {viewState.kind === "error" ? (
        <View style={styles.panel}>
          <Text style={styles.panelTitle}>Could not load organizations</Text>
          <Text style={styles.copy}>{viewState.message}</Text>
          <Pressable testID="organization-retry" onPress={() => void loadOrganizations()} style={styles.secondaryButton}>
            <Text style={styles.secondaryButtonText}>Retry</Text>
          </Pressable>
        </View>
      ) : null}

      {viewState.kind === "ready" && viewState.organizations.length === 0 ? (
        <View testID="organization-list-empty" style={styles.panel}>
          <Text style={styles.panelTitle}>No organizations yet</Text>
          <Text style={styles.copy}>Create your first organization above.</Text>
        </View>
      ) : null}

      {viewState.kind === "ready" && viewState.organizations.length > 0 ? (
        <ScrollView contentContainerStyle={styles.list}>
          {viewState.organizations.map((organization) => (
            <View key={organization.id} testID={`organization-card-${organization.id}`} style={styles.card}>
              <Text style={styles.cardTitle}>{organization.name}</Text>
              <Text style={styles.role}>{organization.viewer_role}</Text>
              <View style={styles.cardActions}>
                <Pressable
                  testID={`organization-projects-${organization.id}`}
                  onPress={() => onOpenProjects(organization)}
                  style={styles.primaryButton}
                >
                  <Text style={styles.primaryButtonText}>Projects</Text>
                </Pressable>
                <Pressable
                  testID={`organization-members-${organization.id}`}
                  onPress={() => onOpenMembers(organization)}
                  style={styles.secondaryButton}
                >
                  <Text style={styles.secondaryButtonText}>Members</Text>
                </Pressable>
              </View>
            </View>
          ))}
        </ScrollView>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "#f2f4ee", padding: 24, gap: 18 },
  header: { marginTop: 20, gap: 8 },
  kicker: { color: "#537462", fontSize: 12, fontWeight: "700", textTransform: "uppercase" },
  title: { color: "#10212a", fontSize: 32, fontWeight: "700" },
  copy: { color: "#52616a", fontSize: 15, lineHeight: 22 },
  createPanel: { backgroundColor: "#fff", borderColor: "#d7dfd7", borderRadius: 10, borderWidth: 1, gap: 10, padding: 16 },
  sectionTitle: { color: "#10212a", fontSize: 18, fontWeight: "700" },
  input: { borderColor: "#aebdb5", borderRadius: 7, borderWidth: 1, color: "#10212a", paddingHorizontal: 12, paddingVertical: 10 },
  errorText: { color: "#9f2d24", fontSize: 14 },
  panel: { backgroundColor: "#fff", borderColor: "#d7dfd7", borderRadius: 10, borderWidth: 1, gap: 10, padding: 18 },
  panelTitle: { color: "#10212a", fontSize: 18, fontWeight: "700" },
  list: { gap: 12, paddingBottom: 24 },
  card: { backgroundColor: "#fff", borderColor: "#d7dfd7", borderRadius: 10, borderWidth: 1, gap: 8, padding: 16 },
  cardTitle: { color: "#10212a", fontSize: 20, fontWeight: "700" },
  role: { color: "#537462", fontSize: 13, fontWeight: "700", textTransform: "uppercase" },
  cardActions: { flexDirection: "row", gap: 10, marginTop: 6 },
  primaryButton: { alignSelf: "flex-start", backgroundColor: "#1a5d50", borderRadius: 7, paddingHorizontal: 15, paddingVertical: 10 },
  primaryButtonText: { color: "#fff", fontSize: 14, fontWeight: "700" },
  secondaryButton: { alignSelf: "flex-start", backgroundColor: "#dfe8e5", borderRadius: 7, paddingHorizontal: 15, paddingVertical: 10 },
  secondaryButtonText: { color: "#14312d", fontSize: 14, fontWeight: "700" },
  disabledButton: { opacity: 0.55 },
});

