import { useCallback, useEffect, useRef, useState } from "react";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import type { OrgRole } from "./OrganizationListScreen";

type OrgMember = { org_id: string; subject_id: string; role: OrgRole; joined_at: string };

export function useOrgMembers(gatewayBaseUrl: string, orgId: string) {
  const auth = useAuth();
  const [members, setMembers] = useState<OrgMember[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [subjectId, setSubjectId] = useState("");
  const [role, setRole] = useState<OrgRole>("viewer");
  const [addError, setAddError] = useState<string | null>(null);
  const [isAddingMemberLoading, setIsAddingMemberLoading] = useState(false);
  const isAddingMember = useRef(false);

  const loadMembers = useCallback(async () => {
    setLoading(true);
    setError(null);
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<OrgMember[]>(`/api/orgs/${orgId}/members`, auth.sessionRef);
    setLoading(false);
    if (result.ok) { await auth.onSessionRotation(result.value.sessionRotation); setMembers(result.value.data); return; }
    if (result.error.kind === "session_expired") { await auth.logout(); return; }
    setError(result.error.kind === "network" ? result.error.message : "Could not load members.");
  }, [auth, gatewayBaseUrl, orgId]);

  useEffect(() => { void loadMembers(); }, [loadMembers]);

  const addMember = useCallback(async () => {
    if (isAddingMember.current) return;
    const normalizedSubject = subjectId.trim();
    if (!normalizedSubject) { setAddError("Subject ID is required."); return; }
    setAddError(null);
    isAddingMember.current = true;
    setIsAddingMemberLoading(true);
    try {
      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await client.post<OrgMember>(`/api/orgs/${orgId}/members`, auth.sessionRef, { subject_id: normalizedSubject, role });
      if (result.ok) {
        await auth.onSessionRotation(result.value.sessionRotation);
        setMembers((current) => [...current.filter((m) => m.subject_id !== result.value.data.subject_id), result.value.data]);
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

  const onRetry = useCallback(() => { void loadMembers(); }, [loadMembers]);

  return { members, loading, error, subjectId, setSubjectId, role, setRole, addError, isAddingMemberLoading, addMember, onRetry };
}
