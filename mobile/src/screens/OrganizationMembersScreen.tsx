import { useCallback, useEffect, useState } from "react";
import { Pressable, ScrollView, StyleSheet, Text, TextInput, View } from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
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
    const normalizedSubject = subjectId.trim();
    if (!normalizedSubject) {
      setAddError("Subject ID is required.");
      return;
    }
    setAddError(null);
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
      return;
    }
    if (result.error.kind === "session_expired") {
      await auth.logout();
      return;
    }
    setAddError(result.error.kind === "forbidden" ? "You cannot manage members in this organization." : "Could not add member.");
  }, [auth, gatewayBaseUrl, orgId, role, subjectId]);

  return (
    <View testID="organization-members-screen" style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.kicker}>Organization</Text>
        <Text style={styles.title}>Members</Text>
        <Text style={styles.copy}>Your role: {viewerRole}</Text>
      </View>

      {canManage ? (
        <View testID="member-add-controls" style={styles.panel}>
          <Text style={styles.sectionTitle}>Add member</Text>
          <TextInput
            testID="member-subject-input"
            accessibilityLabel="Subject ID"
            value={subjectId}
            onChangeText={setSubjectId}
            placeholder="User subject UUID"
            autoCapitalize="none"
            style={styles.input}
          />
          <View style={styles.roles}>
            {ASSIGNABLE_ROLES.map((candidate) => (
              <Pressable
                key={candidate}
                testID={`member-role-${candidate}`}
                onPress={() => setRole(candidate)}
                style={[styles.roleButton, role === candidate && styles.roleButtonSelected]}
              >
                <Text style={styles.roleText}>{candidate}</Text>
              </Pressable>
            ))}
          </View>
          {addError ? <Text style={styles.errorText}>{addError}</Text> : null}
          <Pressable testID="member-add" onPress={() => void addMember()} style={styles.primaryButton}>
            <Text style={styles.primaryButtonText}>Add member</Text>
          </Pressable>
        </View>
      ) : null}

      {loading ? <Text>Loading members...</Text> : null}
      {error ? (
        <View style={styles.panel}>
          <Text style={styles.errorText}>{error}</Text>
          <Pressable onPress={() => void loadMembers()} style={styles.secondaryButton}>
            <Text style={styles.secondaryButtonText}>Retry</Text>
          </Pressable>
        </View>
      ) : null}
      {!loading && !error && members.length === 0 ? <Text testID="member-list-empty">No members yet.</Text> : null}
      {!loading && !error ? (
        <ScrollView contentContainerStyle={styles.list}>
          {members.map((member) => (
            <View key={member.subject_id} testID={`member-row-${member.subject_id}`} style={styles.memberCard}>
              <Text style={styles.memberId}>{member.subject_id}</Text>
              <Text style={styles.memberRole}>{member.role}</Text>
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
  copy: { color: "#52616a", fontSize: 15 },
  panel: { backgroundColor: "#fff", borderColor: "#d7dfd7", borderRadius: 10, borderWidth: 1, gap: 10, padding: 16 },
  sectionTitle: { color: "#10212a", fontSize: 18, fontWeight: "700" },
  input: { borderColor: "#aebdb5", borderRadius: 7, borderWidth: 1, color: "#10212a", paddingHorizontal: 12, paddingVertical: 10 },
  roles: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  roleButton: { backgroundColor: "#e8eeeb", borderRadius: 6, paddingHorizontal: 10, paddingVertical: 8 },
  roleButtonSelected: { backgroundColor: "#bcd3ca" },
  roleText: { color: "#14312d", fontSize: 13, fontWeight: "700", textTransform: "capitalize" },
  errorText: { color: "#9f2d24", fontSize: 14 },
  primaryButton: { alignSelf: "flex-start", backgroundColor: "#1a5d50", borderRadius: 7, paddingHorizontal: 15, paddingVertical: 10 },
  primaryButtonText: { color: "#fff", fontSize: 14, fontWeight: "700" },
  secondaryButton: { alignSelf: "flex-start", backgroundColor: "#dfe8e5", borderRadius: 7, paddingHorizontal: 15, paddingVertical: 10 },
  secondaryButtonText: { color: "#14312d", fontSize: 14, fontWeight: "700" },
  list: { gap: 10, paddingBottom: 24 },
  memberCard: { backgroundColor: "#fff", borderColor: "#d7dfd7", borderRadius: 9, borderWidth: 1, gap: 5, padding: 14 },
  memberId: { color: "#10212a", fontSize: 14, fontWeight: "600" },
  memberRole: { color: "#537462", fontSize: 13, fontWeight: "700", textTransform: "uppercase" },
});

