import { useCallback, useEffect, useState } from "react";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";

type AuditEvent = { id: string; event_kind: string; detail: string | null; happened_at: string };
type AuditTimeline = { asset_id: string; events: AuditEvent[] };
type RightsRecord = { id: string; owner: string; license_type: string; source_type: string; proof_reference: string; created_at: string };
type RightsLedger = { asset_id: string; entries: RightsRecord[] };
type ConsentRow = { id: string; scope: string; status: "grant" | "revoke"; evidence_ref: string | null; happened_at: string };
type ConsentLedger = { asset_id: string; current_status: "grant" | "revoke" | null; rows: ConsentRow[] };

export type ComplianceData = { audit: AuditTimeline; rights: RightsLedger; consent: ConsentLedger };

export type ComplianceViewState =
  | { kind: "loading" }
  | { kind: "ready"; data: ComplianceData }
  | { kind: "error"; message: string };

function complianceErrorMessage(error: { kind: string; message?: string; status?: number }) {
  if (error.kind === "forbidden") return "You do not have access to this asset's compliance data.";
  if (error.kind === "network") return error.message ?? "Network request failed.";
  if (error.kind === "http") return `Request failed with status ${error.status}.`;
  return "Could not load compliance data.";
}

function sortAuditTimeline(audit: AuditTimeline): AuditTimeline {
  return { ...audit, events: [...audit.events].sort((a, b) => a.happened_at.localeCompare(b.happened_at)) };
}

async function fetchComplianceData(gatewayBaseUrl: string, assetId: string, sessionRef: string | null) {
  const client = createGatewayClient({ gatewayBaseUrl });
  const [audit, rights, consent] = await Promise.all([
    client.get<AuditTimeline>(`/api/assets/${assetId}/audit`, sessionRef),
    client.get<RightsLedger>(`/api/assets/${assetId}/rights`, sessionRef),
    client.get<ConsentLedger>(`/api/assets/${assetId}/consents`, sessionRef),
  ]);
  return { audit, rights, consent };
}

export function useComplianceLoader(assetId: string, gatewayBaseUrl: string) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<ComplianceViewState>({ kind: "loading" });

  const loadCompliance = useCallback(async () => {
    setViewState({ kind: "loading" });
    const { audit, rights, consent } = await fetchComplianceData(gatewayBaseUrl, assetId, auth.sessionRef);

    const results = [audit, rights, consent];
    if (results.some((r) => !r.ok && r.error.kind === "session_expired")) { await auth.logout(); return; }

    const failure = results.find((r) => !r.ok);
    if (failure && !failure.ok) { setViewState({ kind: "error", message: complianceErrorMessage(failure.error) }); return; }

    if (!audit.ok || !rights.ok || !consent.ok) return;
    await Promise.all([auth.onSessionRotation(audit.value.sessionRotation), auth.onSessionRotation(rights.value.sessionRotation), auth.onSessionRotation(consent.value.sessionRotation)]);
    setViewState({ kind: "ready", data: { audit: sortAuditTimeline(audit.value.data), rights: rights.value.data, consent: consent.value.data } });
  }, [assetId, auth, gatewayBaseUrl]);

  useEffect(() => { void loadCompliance(); }, [loadCompliance]);

  const onRetry = useCallback(() => { void loadCompliance(); }, [loadCompliance]);

  return { viewState, onRetry };
}
