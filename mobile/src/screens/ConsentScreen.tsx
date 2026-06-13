import { useCallback, useEffect, useState } from "react";
import { ScrollView, StyleSheet, Text, TextInput, View } from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, fieldStyle, space, type } from "../theme";

type ConsentStatus = "grant" | "revoke";
type ConsentScope = "voice_clone" | "tts_synthesis";
type ConsentRow = {
  id: string;
  scope: ConsentScope;
  status: ConsentStatus;
  evidence_ref: string | null;
  happened_at: string;
};
type ConsentLedger = { current_status: ConsentStatus | null; rows: ConsentRow[] };

type Props = { assetId: string; gatewayBaseUrl: string };

export function ConsentScreen({ assetId, gatewayBaseUrl }: Props) {
  const auth = useAuth();
  const [ledger, setLedger] = useState<ConsentLedger | null>(null);
  const [scope, setScope] = useState<ConsentScope>("voice_clone");
  const [evidenceRef, setEvidenceRef] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const loadLedger = useCallback(async () => {
    setLoading(true);
    setError(null);
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.get<ConsentLedger>(`/api/assets/${assetId}/consents`, auth.sessionRef);
    setLoading(false);
    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      setLedger(result.value.data);
      return;
    }
    if (result.error.kind === "session_expired") {
      await auth.logout();
      return;
    }
    setError(result.error.kind === "forbidden" ? "You do not have access to this asset." : result.error.kind === "network" ? result.error.message : "Could not load consent.");
  }, [assetId, auth, gatewayBaseUrl]);

  useEffect(() => {
    void loadLedger();
  }, [loadLedger]);

  const mutate = useCallback(async (status: ConsentStatus) => {
    const normalizedEvidence = evidenceRef.trim();
    if (status === "grant" && !normalizedEvidence) {
      setError("Evidence reference is required to grant consent.");
      return;
    }
    setError(null);
    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await client.post(
      "/api/consents",
      auth.sessionRef,
      {
        asset_id: assetId,
        scope,
        status,
        evidence_ref: status === "grant" ? normalizedEvidence : null,
      },
    );
    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      setEvidenceRef("");
      await loadLedger();
      return;
    }
    if (result.error.kind === "session_expired") {
      await auth.logout();
      return;
    }
    setError(result.error.kind === "forbidden" ? "You cannot change consent for this asset." : "Could not update consent.");
  }, [assetId, auth, evidenceRef, gatewayBaseUrl, loadLedger, scope]);

  const onRetry = useCallback(() => {
    void loadLedger();
  }, [loadLedger]);

  return (
    <Screen testID="consent-screen" scroll edges={["bottom"]}>
      <ScreenHeader kicker="Append-only ledger" title="Voice consent" />

      {loading && !ledger ? (
        <StateView kind="loading" title="Loading consent..." />
      ) : null}

      {!loading && error && !ledger ? (
        <StateView
          kind="error"
          title="Could not load consent"
          message={error}
          onRetry={onRetry}
        />
      ) : null}

      {ledger ? (
        <>
          <Panel>
            <Text style={styles.panelTitle}>Current status</Text>
            <Badge
              testID="consent-status"
              label={ledger.current_status === "grant" ? "Active" : "Inactive"}
              tone={statusTone(ledger.current_status)}
            />
          </Panel>

          <Panel>
            <Text style={styles.panelTitle}>Scope</Text>
            <View style={styles.scopeButtons}>
              {(["voice_clone", "tts_synthesis"] as ConsentScope[]).map((candidate) => (
                <Button
                  key={candidate}
                  testID={`consent-scope-${candidate}`}
                  label={candidate.replaceAll("_", " ")}
                  onPress={() => setScope(candidate)}
                  variant={scope === candidate ? "primary" : "secondary"}
                  size="sm"
                />
              ))}
            </View>
            <TextInput
              testID="consent-evidence-input"
              accessibilityLabel="Evidence reference"
              value={evidenceRef}
              onChangeText={setEvidenceRef}
              placeholder="Evidence URI or reference ID"
              autoCapitalize="none"
              style={fieldStyle}
            />
            {error ? <Text style={styles.errorText}>{error}</Text> : null}
            <View style={styles.actions}>
              <Button
                testID="consent-grant"
                label="Grant"
                onPress={() => void mutate("grant")}
              />
              <Button
                testID="consent-revoke"
                label="Revoke"
                onPress={() => void mutate("revoke")}
                variant="danger"
              />
            </View>
          </Panel>

          <Panel>
            <Text style={styles.panelTitle}>History</Text>
            {ledger.rows.length === 0 ? (
              <Text testID="consent-history-empty" style={styles.emptyText}>No consent history.</Text>
            ) : null}
            {ledger.rows.map((row) => (
              <View key={row.id} testID={`consent-row-${row.id}`} style={styles.row}>
                <Text style={styles.rowTitle}>{row.status} / {row.scope}</Text>
                <Text style={styles.meta}>{row.happened_at}</Text>
                {row.evidence_ref ? <Text style={styles.meta}>{row.evidence_ref}</Text> : null}
              </View>
            ))}
          </Panel>
        </>
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  panelTitle: { ...type.heading, color: color.ink900 },
  emptyText: { ...type.body, color: color.ink500 },
  errorText: { ...type.meta, color: color.danger },
  scopeButtons: { flexDirection: "row", gap: space.sm },
  actions: { flexDirection: "row", gap: space.sm },
  row: { borderTopColor: color.border, borderTopWidth: 1, gap: space.xs, paddingTop: space.sm },
  rowTitle: { ...type.bodyStrong, color: color.ink900 },
  meta: { ...type.meta, color: color.ink400 },
});
