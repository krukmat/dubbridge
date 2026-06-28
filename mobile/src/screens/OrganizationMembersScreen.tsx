import { FlatList, StyleSheet, Text, TextInput, View } from "react-native";

import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, fieldStyle, space, type } from "../theme";
import type { OrgRole } from "./OrganizationListScreen";
import { useOrgMembers } from "./useOrgMembers";

type Props = { gatewayBaseUrl: string; orgId: string; viewerRole: OrgRole };

const ASSIGNABLE_ROLES: OrgRole[] = ["viewer", "reviewer", "editor", "admin"];

function AddMemberPanel({ subjectId, role, addError, isAddingMemberLoading, onChangeSubjectId, onChangeRole, onAddMember }: { subjectId: string; role: OrgRole; addError: string | null; isAddingMemberLoading: boolean; onChangeSubjectId: (v: string) => void; onChangeRole: (v: OrgRole) => void; onAddMember: () => void }) {
  return (
    <Panel testID="member-add-controls">
      <Text style={styles.sectionTitle}>Add member</Text>
      <TextInput testID="member-subject-input" accessibilityLabel="Subject ID" value={subjectId} onChangeText={onChangeSubjectId} placeholder="User subject UUID" autoCapitalize="none" style={fieldStyle} />
      <View style={styles.roles}>
        {ASSIGNABLE_ROLES.map((candidate) => (
          <Button key={candidate} testID={`member-role-${candidate}`} label={candidate} onPress={() => onChangeRole(candidate)} variant={role === candidate ? "primary" : "secondary"} size="sm" />
        ))}
      </View>
      {addError ? <Text style={styles.errorText}>{addError}</Text> : null}
      <Button testID="member-add" label="Add member" onPress={onAddMember} loading={isAddingMemberLoading} disabled={isAddingMemberLoading} />
    </Panel>
  );
}

type OrgMember = { subject_id: string; role: OrgRole };

function MemberListBody({ loading, error, members, onRetry }: { loading: boolean; error: string | null; members: OrgMember[]; onRetry: () => void }) {
  if (loading) return <StateView kind="loading" title="Loading members..." />;
  if (error) return <StateView kind="error" title="Could not load members" message={error} onRetry={onRetry} />;
  if (members.length === 0) return <StateView testID="member-list-empty" kind="empty" title="No members yet" message="This organization has no members." />;
  return (
    <FlatList
      contentContainerStyle={styles.list}
      data={members}
      keyExtractor={(m) => m.subject_id}
      renderItem={({ item: member }) => (
        <Panel testID={`member-row-${member.subject_id}`}>
          <Text style={styles.memberId}>{member.subject_id}</Text>
          <Text style={styles.memberRole}>{member.role}</Text>
        </Panel>
      )}
    />
  );
}

export function OrganizationMembersScreen({ gatewayBaseUrl, orgId, viewerRole }: Props) {
  const { members, loading, error, subjectId, setSubjectId, role, setRole, addError, isAddingMemberLoading, addMember, onRetry } = useOrgMembers(gatewayBaseUrl, orgId);
  const canManage = viewerRole === "owner" || viewerRole === "admin";

  return (
    <Screen testID="organization-members-screen">
      <ScreenHeader kicker="Organization" title="Members" copy={`Your role: ${viewerRole}`} />
      {canManage ? <AddMemberPanel subjectId={subjectId} role={role} addError={addError} isAddingMemberLoading={isAddingMemberLoading} onChangeSubjectId={setSubjectId} onChangeRole={setRole} onAddMember={() => void addMember()} /> : null}
      <MemberListBody loading={loading} error={error} members={members} onRetry={onRetry} />
    </Screen>
  );
}

const styles = StyleSheet.create({
  sectionTitle: { ...type.heading, color: color.ink900 },
  errorText: { ...type.meta, color: color.danger },
  roles: { flexDirection: "row", flexWrap: "wrap", gap: space.sm },
  list: { gap: space.sm, paddingBottom: space.xl },
  memberId: { ...type.bodyStrong, color: color.ink900 },
  memberRole: { ...type.label, color: color.primary },
});
