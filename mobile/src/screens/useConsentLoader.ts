import { useCallback, useEffect, useState } from "react";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";

export type ConsentStatus = "grant" | "revoke";
export type ConsentScope = "voice_clone" | "tts_synthesis";
export type ConsentRow = { id: string; scope: ConsentScope; status: ConsentStatus; evidence_ref: string | null; happened_at: string };
export type ConsentLedger = { current_status: ConsentStatus | null; rows: ConsentRow[] };

function consentLoadError(error: { kind: string; message?: string }) {
  if (error.kind === "forbidden") return "You do not have access to this asset.";
  if (error.kind === "network") return error.message ?? "Network request failed.";
  return "Could not load consent.";
}

function consentMutationError(kind: string) {
  return kind === "forbidden" ? "You cannot change consent for this asset." : "Could not update consent.";
}

export function useConsentLoader(assetId: string, gatewayBaseUrl: string) {
  const auth = useAuth();
  const [ledger, setLedger] = useState<ConsentLedger | null>(null);
  const [scope, setScope] = useState<ConsentScope>("voice_clone");
  const [evidenceRef, setEvidenceRef] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  const loadLedger = useCallback(async () => {
    setLoading(true);
    setError(null);
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<ConsentLedger>(`/api/assets/${assetId}/consents`, auth.sessionRef);
    setLoading(false);
    if (result.ok) { await auth.onSessionRotation(result.value.sessionRotation); setLedger(result.value.data); return; }
    if (result.error.kind === "session_expired") { await auth.logout(); return; }
    setError(consentLoadError(result.error));
  }, [assetId, auth, gatewayBaseUrl]);

  useEffect(() => { void loadLedger(); }, [loadLedger]);

  const mutate = useCallback(async (status: ConsentStatus) => {
    const normalizedEvidence = evidenceRef.trim();
    if (status === "grant" && !normalizedEvidence) { setError("Evidence reference is required to grant consent."); return; }
    setError(null);
    setSubmitting(true);
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.post("/api/consents", auth.sessionRef, {
      asset_id: assetId, scope, status, evidence_ref: status === "grant" ? normalizedEvidence : null,
    });
    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      setEvidenceRef("");
      await loadLedger();
      setSubmitting(false);
      return;
    }
    setSubmitting(false);
    if (result.error.kind === "session_expired") { await auth.logout(); return; }
    setError(consentMutationError(result.error.kind));
  }, [assetId, auth, evidenceRef, gatewayBaseUrl, loadLedger, scope]);

  const onRetry = useCallback(() => { void loadLedger(); }, [loadLedger]);

  return { ledger, scope, setScope, evidenceRef, setEvidenceRef, error, loading, submitting, mutate, onRetry };
}
