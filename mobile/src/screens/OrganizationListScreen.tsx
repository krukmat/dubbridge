import { useCallback, useEffect, useState } from "react";
import {
  FlatList,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, fieldStyle, space, type } from "../theme";

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

  const onRetry = useCallback(() => {
    setViewState({ kind: "loading" });
    void loadOrganizations();
  }, [loadOrganizations]);

  return (
    <Screen testID="organization-list-screen" edges={["bottom"]}>
      <ScreenHeader
        kicker="Workspace"
        title="Organizations"
        copy="Choose the organization whose projects and members you want to manage."
      />

      <Panel>
        <Text style={styles.sectionTitle}>Create organization</Text>
        <TextInput
          testID="organization-name-input"
          accessibilityLabel="Organization name"
          value={name}
          onChangeText={setName}
          placeholder="Organization name"
          autoCapitalize="words"
          style={fieldStyle}
        />
        {createError ? <Text style={styles.errorText}>{createError}</Text> : null}
        <Button
          testID="organization-create"
          label={creating ? "Creating..." : "Create and open"}
          onPress={() => void createOrganization()}
          loading={creating}
          disabled={creating}
        />
      </Panel>

      {viewState.kind === "loading" ? (
        <StateView kind="loading" title="Loading organizations..." />
      ) : null}

      {viewState.kind === "error" ? (
        <StateView
          testID="organization"
          kind="error"
          title="Could not load organizations"
          message={viewState.message}
          onRetry={onRetry}
        />
      ) : null}

      {viewState.kind === "ready" && viewState.organizations.length === 0 ? (
        <StateView
          testID="organization-list-empty"
          kind="empty"
          title="No organizations yet"
          message="Create your first organization above."
        />
      ) : null}

      {viewState.kind === "ready" && viewState.organizations.length > 0 ? (
        <FlatList
          contentContainerStyle={styles.list}
          data={viewState.organizations}
          keyExtractor={(organization) => organization.id}
          renderItem={({ item: organization }) => (
            <Panel testID={`organization-card-${organization.id}`}>
              <Text style={styles.cardTitle}>{organization.name}</Text>
              <Badge
                label={organization.viewer_role}
                tone={statusTone(organization.viewer_role)}
              />
              <View style={styles.cardActions}>
                <Button
                  testID={`organization-projects-${organization.id}`}
                  label="Projects"
                  onPress={() => onOpenProjects(organization)}
                  size="sm"
                />
                <Button
                  testID={`organization-members-${organization.id}`}
                  label="Members"
                  onPress={() => onOpenMembers(organization)}
                  variant="secondary"
                  size="sm"
                />
              </View>
            </Panel>
          )}
        />
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  sectionTitle: { ...type.heading, color: color.ink900 },
  errorText: { ...type.meta, color: color.danger },
  list: { gap: space.md, paddingBottom: space.xl },
  cardTitle: { ...type.title, color: color.ink900 },
  cardActions: { flexDirection: "row", gap: space.sm },
});
