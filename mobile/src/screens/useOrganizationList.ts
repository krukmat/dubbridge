import { useCallback, useEffect, useState } from "react";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import type { OrganizationSummary } from "./OrganizationListScreen";

type ViewState =
  | { kind: "loading" }
  | { kind: "ready"; organizations: OrganizationSummary[] }
  | { kind: "error"; message: string };

function orgErrorMessage(kind: "forbidden" | "network" | "http", detail?: string | number) {
  if (kind === "forbidden") return "You do not have access to organizations.";
  if (kind === "network") return String(detail ?? "Network request failed.");
  return `Request failed with status ${detail}.`;
}

export function useOrganizationList(gatewayBaseUrl: string, onOpenProjects: (org: OrganizationSummary) => void) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<ViewState>({ kind: "loading" });
  const [name, setName] = useState("");
  const [createError, setCreateError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  const loadOrganizations = useCallback(async () => {
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<OrganizationSummary[]>("/api/orgs", auth.sessionRef);
    if (result.ok) { await auth.onSessionRotation(result.value.sessionRotation); setViewState({ kind: "ready", organizations: result.value.data }); return; }
    if (result.error.kind === "session_expired") { await auth.logout(); return; }
    const detail = result.error.kind === "network" ? result.error.message : result.error.kind === "http" ? result.error.status : undefined;
    setViewState({ kind: "error", message: orgErrorMessage(result.error.kind, detail) });
  }, [auth, gatewayBaseUrl]);

  useEffect(() => { void loadOrganizations(); }, [loadOrganizations]);

  const createOrganization = useCallback(async () => {
    const normalizedName = name.trim();
    if (!normalizedName) { setCreateError("Organization name is required."); return; }
    setCreating(true);
    setCreateError(null);
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.post<OrganizationSummary>("/api/orgs", auth.sessionRef, { name: normalizedName });
    setCreating(false);
    if (result.ok) { await auth.onSessionRotation(result.value.sessionRotation); setName(""); onOpenProjects(result.value.data); return; }
    if (result.error.kind === "session_expired") { await auth.logout(); return; }
    const detail = result.error.kind === "network" ? result.error.message : result.error.kind === "http" ? result.error.status : undefined;
    setCreateError(orgErrorMessage(result.error.kind, detail));
  }, [auth, gatewayBaseUrl, name, onOpenProjects]);

  const onRetry = useCallback(() => { setViewState({ kind: "loading" }); void loadOrganizations(); }, [loadOrganizations]);

  return { viewState, name, setName, createError, creating, createOrganization, onRetry };
}
