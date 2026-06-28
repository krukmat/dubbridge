import { FlatList, StyleSheet, Text, TextInput, View } from "react-native";

import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, fieldStyle, space, type } from "../theme";
import { useOrganizationList } from "./useOrganizationList";

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

function OrganizationCreatePanel({ name, createError, creating, onChangeName, onCreate }: { name: string; createError: string | null; creating: boolean; onChangeName: (v: string) => void; onCreate: () => void }) {
  return (
    <Panel>
      <Text style={styles.sectionTitle}>Create organization</Text>
      <TextInput testID="organization-name-input" accessibilityLabel="Organization name" value={name} onChangeText={onChangeName} placeholder="Organization name" autoCapitalize="words" style={fieldStyle} />
      {createError ? <Text style={styles.errorText}>{createError}</Text> : null}
      <Button testID="organization-create" label={creating ? "Creating..." : "Create and open"} onPress={onCreate} loading={creating} disabled={creating} />
    </Panel>
  );
}

function OrganizationListBody({ organizations, onOpenProjects, onOpenMembers }: { organizations: OrganizationSummary[]; onOpenProjects: (o: OrganizationSummary) => void; onOpenMembers: (o: OrganizationSummary) => void }) {
  return (
    <FlatList
      contentContainerStyle={styles.list}
      data={organizations}
      keyExtractor={(o) => o.id}
      renderItem={({ item: org }) => (
        <Panel testID={`organization-card-${org.id}`}>
          <Text style={styles.cardTitle}>{org.name}</Text>
          <Badge label={org.viewer_role} tone={statusTone(org.viewer_role)} />
          <View style={styles.cardActions}>
            <Button testID={`organization-projects-${org.id}`} label="Projects" onPress={() => onOpenProjects(org)} size="sm" />
            <Button testID={`organization-members-${org.id}`} label="Members" onPress={() => onOpenMembers(org)} variant="secondary" size="sm" />
          </View>
        </Panel>
      )}
    />
  );
}

export function OrganizationListScreen({ gatewayBaseUrl, onOpenProjects, onOpenMembers }: Props) {
  const { viewState, name, setName, createError, creating, createOrganization, onRetry } = useOrganizationList(gatewayBaseUrl, onOpenProjects);
  return (
    <Screen testID="organization-list-screen">
      <ScreenHeader kicker="Workspace" title="Organizations" copy="Choose the organization whose projects and members you want to manage." />
      <OrganizationCreatePanel name={name} createError={createError} creating={creating} onChangeName={setName} onCreate={() => void createOrganization()} />
      {viewState.kind === "loading" ? <StateView kind="loading" title="Loading organizations..." /> : null}
      {viewState.kind === "error" ? <StateView testID="organization" kind="error" title="Could not load organizations" message={viewState.message} onRetry={onRetry} /> : null}
      {viewState.kind === "ready" && viewState.organizations.length === 0 ? <StateView testID="organization-list-empty" kind="empty" title="No organizations yet" message="Create your first organization above." /> : null}
      {viewState.kind === "ready" && viewState.organizations.length > 0 ? <OrganizationListBody organizations={viewState.organizations} onOpenProjects={onOpenProjects} onOpenMembers={onOpenMembers} /> : null}
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
