import { useCallback, useEffect, useState, useRef } from "react";
import { FlatList, StyleSheet, Text, TextInput, View } from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, fieldStyle, space, type } from "../theme";
import type { OrgRole } from "./OrganizationListScreen";

type OrgMember = {
  org_id: string;
  subject_id: string;
  role: OrgRole;
  joined_at: string;
};

type Props = {
  gatewayBaseUrl: string;
  orgId: string;
  viewerRole: OrgRole;
};

const ASSIGNABLE_ROLES: OrgRole[] = ["viewer", "reviewer", "editor", "admin"];

export function OrganizationMembersScreen({ gatewayBaseUrl, orgId, viewerRole }: Props) {
  const auth = useAuth();
  const [members, setMembers] = useState<OrgMember[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [subjectId, setSubjectId] = useState("");
  const [role, setRole] = useState<OrgRole>("viewer");
  const [addError, setAddError] = useState<string | null>(null);
  const [isAddingMemberLoading, setIsAddingMemberLoading] = useState(false);
  const isAddingMember = useRef(false);
  const canManage = viewerRole === "owner" || viewerRole === "admin";

  const loadMembers = useCallback(async () => {
    setLoading(true);
    setError(null);
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<OrgMember[]>(`/api/orgs/${orgId}/members`, auth.sessionRef);
    setLoading(false);

    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      setMembers(result.value.data);
      return;
    }
    if (result.error.kind === "session_expired") {
      await auth.logout();
      return;
    }
    setError(result.error.kind === "network" ? result.error.message : "Could not load members.");
  }, [auth, gatewayBaseUrl, orgId]);

  useEffect(() => {
    void loadMembers();
  }, [loadMembers]);

  const addMember = useCallback(async () => {
    if (isAddingMember.current) return;
    const normalizedSubject = subjectId.trim();
    if (!normalizedSubject) {
      setAddError("Subject ID is required.");
      return;
    }
    setAddError(null);
    isAddingMember.current = true;
    setIsAddingMemberLoading(true);
    try {
      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await client.post<OrgMember>(
        `/api/orgs/${orgId}/members`,
        auth.sessionRef,
        { subject_id: normalizedSubject, role },
      );

      if (result.ok) {
        await auth.onSessionRotation(result.value.sessionRotation);
        setMembers((current) => [...current.filter((item) => item.subject_id !== result.value.data.subject_id), result.value.data]);
        setSubjectId("");
      } else if (result.error.kind === "session_expired") {
        await auth.logout();
      } else {
        setAddError(result.error.kind === "forbidden" ? "You cannot manage members in this organization." : "Could not add member.");
      }
    } finally {
      isAddingMember.current = false;
      setIsAddingMemberLoading(false);
    }
  }, [auth, gatewayBaseUrl, orgId, role, subjectId]);

  const onRetry = useCallback(() => {
    void loadMembers();
  }, [loadMembers]);

  return (
    <Screen testID="organization-members-screen" edges={["bottom"]}>
      <ScreenHeader
        kicker="Organization"
        title="Members"
        copy={`Your role: ${viewerRole}`}
      />

      {canManage ? (
        <Panel testID="member-add-controls">
          <Text style={styles.sectionTitle}>Add member</Text>
          <TextInput
            testID="member-subject-input"
            accessibilityLabel="Subject ID"
            value={subjectId}
            onChangeText={setSubjectId}
            placeholder="User subject UUID"
            autoCapitalize="none"
            style={fieldStyle}
          />
          <View style={styles.roles}>
            {ASSIGNABLE_ROLES.map((candidate) => (
              <Button
                key={candidate}
                testID={`member-role-${candidate}`}
                label={candidate}
                onPress={() => setRole(candidate)}
                variant={role === candidate ? "primary" : "secondary"}
                size="sm"
              />
            ))}
          </View>
          {addError ? <Text style={styles.errorText}>{addError}</Text> : null}
          <Button
            testID="member-add"
            label="Add member"
            onPress={() => void addMember()}
            loading={isAddingMemberLoading}
            disabled={isAddingMemberLoading}
          />
        </Panel>
      ) : null}

      {loading ? (
        <StateView kind="loading" title="Loading members..." />
      ) : null}

      {!loading && error ? (
        <StateView
          kind="error"
          title="Could not load members"
          message={error}
          onRetry={onRetry}
        />
      ) : null}

      {!loading && !error && members.length === 0 ? (
        <StateView
          testID="member-list-empty"
          kind="empty"
          title="No members yet"
          message="This organization has no members."
        />
      ) : null}

      {!loading && !error && members.length > 0 ? (
        <FlatList
          contentContainerStyle={styles.list}
          data={members}
          keyExtractor={(member) => member.subject_id}
          renderItem={({ item: member }) => (
            <Panel testID={`member-row-${member.subject_id}`}>
              <Text style={styles.memberId}>{member.subject_id}</Text>
              <Text style={styles.memberRole}>{member.role}</Text>
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
  roles: { flexDirection: "row", flexWrap: "wrap", gap: space.sm },
  list: { gap: space.sm, paddingBottom: space.xl },
  memberId: { ...type.bodyStrong, color: color.ink900 },
  memberRole: { ...type.label, color: color.primary },
});
