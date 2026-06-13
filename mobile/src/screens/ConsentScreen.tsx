import { useCallback, useEffect, useState } from "react";
import { Pressable, ScrollView, StyleSheet, Text, TextInput, View } from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";

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

  return (
    <ScrollView testID="consent-screen" style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.header}>
        <Text style={styles.kicker}>Append-only ledger</Text>
        <Text style={styles.title}>Voice consent</Text>
      </View>
      {loading ? <Text>Loading consent...</Text> : null}
      {error ? <Text style={styles.error}>{error}</Text> : null}
      {ledger ? (
        <>
          <View style={styles.panel}>
            <Text style={styles.panelTitle}>Current status</Text>
            <Text testID="consent-status" style={styles.status}>{ledger.current_status === "grant" ? "Active" : "Inactive"}</Text>
          </View>
          <View style={styles.panel}>
            <Text style={styles.panelTitle}>Scope</Text>
            <View style={styles.scopeButtons}>
              {(["voice_clone", "tts_synthesis"] as ConsentScope[]).map((candidate) => (
                <Pressable
                  key={candidate}
                  testID={`consent-scope-${candidate}`}
                  onPress={() => setScope(candidate)}
                  style={[styles.scopeButton, scope === candidate && styles.scopeSelected]}
                >
                  <Text>{candidate.replaceAll("_", " ")}</Text>
                </Pressable>
              ))}
            </View>
            <TextInput
              testID="consent-evidence-input"
              accessibilityLabel="Evidence reference"
              value={evidenceRef}
              onChangeText={setEvidenceRef}
              placeholder="Evidence URI or reference ID"
              autoCapitalize="none"
              style={styles.input}
            />
            <View style={styles.actions}>
              <Pressable testID="consent-grant" onPress={() => void mutate("grant")} style={styles.grantButton}>
                <Text style={styles.buttonText}>Grant</Text>
              </Pressable>
              <Pressable testID="consent-revoke" onPress={() => void mutate("revoke")} style={styles.revokeButton}>
                <Text style={styles.buttonText}>Revoke</Text>
              </Pressable>
            </View>
          </View>
          <View style={styles.panel}>
            <Text style={styles.panelTitle}>History</Text>
            {ledger.rows.length === 0 ? <Text testID="consent-history-empty">No consent history.</Text> : null}
            {ledger.rows.map((row) => (
              <View key={row.id} testID={`consent-row-${row.id}`} style={styles.row}>
                <Text style={styles.rowTitle}>{row.status} / {row.scope}</Text>
                <Text style={styles.meta}>{row.happened_at}</Text>
                {row.evidence_ref ? <Text style={styles.meta}>{row.evidence_ref}</Text> : null}
              </View>
            ))}
          </View>
        </>
      ) : null}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "#edf3ef" },
  content: { gap: 16, padding: 24, paddingBottom: 40 },
  header: { gap: 8, marginTop: 20 },
  kicker: { color: "#1a6a58", fontSize: 12, fontWeight: "700", textTransform: "uppercase" },
  title: { color: "#10212a", fontSize: 32, fontWeight: "700" },
  panel: { backgroundColor: "#fff", borderColor: "#cbdad2", borderRadius: 10, borderWidth: 1, gap: 10, padding: 18 },
  panelTitle: { color: "#10212a", fontSize: 19, fontWeight: "700" },
  status: { color: "#1a6a58", fontSize: 28, fontWeight: "700" },
  error: { color: "#9f2d24", fontSize: 14 },
  scopeButtons: { flexDirection: "row", gap: 8 },
  scopeButton: { backgroundColor: "#e8eeeb", borderRadius: 6, paddingHorizontal: 12, paddingVertical: 9 },
  scopeSelected: { backgroundColor: "#bcd3ca" },
  input: { borderColor: "#aebdb5", borderRadius: 7, borderWidth: 1, color: "#10212a", paddingHorizontal: 12, paddingVertical: 10 },
  actions: { flexDirection: "row", gap: 10 },
  grantButton: { backgroundColor: "#1a6a58", borderRadius: 7, paddingHorizontal: 16, paddingVertical: 11 },
  revokeButton: { backgroundColor: "#8b342d", borderRadius: 7, paddingHorizontal: 16, paddingVertical: 11 },
  buttonText: { color: "#fff", fontWeight: "700" },
  row: { borderTopColor: "#dce6e0", borderTopWidth: 1, gap: 4, paddingTop: 10 },
  rowTitle: { color: "#17372f", fontSize: 15, fontWeight: "700" },
  meta: { color: "#61746c", fontSize: 12 },
});

