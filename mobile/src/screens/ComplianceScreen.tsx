import { useCallback, useEffect, useState } from "react";
import { Pressable, ScrollView, StyleSheet, Text, View } from "react-native";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";

type AuditEvent = {
  id: string;
  event_kind: string;
  detail: string | null;
  happened_at: string;
};

type AuditTimeline = { asset_id: string; events: AuditEvent[] };
type RightsRecord = {
  id: string;
  owner: string;
  license_type: string;
  source_type: string;
  proof_reference: string;
  created_at: string;
};
type RightsLedger = { asset_id: string; entries: RightsRecord[] };
type ConsentRow = {
  id: string;
  scope: string;
  status: "grant" | "revoke";
  evidence_ref: string | null;
  happened_at: string;
};
type ConsentLedger = {
  asset_id: string;
  current_status: "grant" | "revoke" | null;
  rows: ConsentRow[];
};

type ComplianceData = {
  audit: AuditTimeline;
  rights: RightsLedger;
  consent: ConsentLedger;
};

type Props = {
  assetId: string;
  gatewayBaseUrl: string;
  onManageConsent: () => void;
};

type ViewState =
  | { kind: "loading" }
  | { kind: "ready"; data: ComplianceData }
  | { kind: "error"; message: string };

export function ComplianceScreen({ assetId, gatewayBaseUrl, onManageConsent }: Props) {
  const auth = useAuth();
  const [viewState, setViewState] = useState<ViewState>({ kind: "loading" });

  const loadCompliance = useCallback(async () => {
    setViewState({ kind: "loading" });
    const client = createGatewayClient({ gatewayBaseUrl });
    const [audit, rights, consent] = await Promise.all([
      client.get<AuditTimeline>(`/api/assets/${assetId}/audit`, auth.sessionRef),
      client.get<RightsLedger>(`/api/assets/${assetId}/rights`, auth.sessionRef),
      client.get<ConsentLedger>(`/api/assets/${assetId}/consents`, auth.sessionRef),
    ]);

    const results = [audit, rights, consent];
    if (results.some((result) => !result.ok && result.error.kind === "session_expired")) {
      await auth.logout();
      return;
    }

    const failure = results.find((result) => !result.ok);
    if (failure && !failure.ok) {
      const message = failure.error.kind === "forbidden"
        ? "You do not have access to this asset's compliance data."
        : failure.error.kind === "network"
          ? failure.error.message
          : failure.error.kind === "http"
            ? `Request failed with status ${failure.error.status}.`
            : "Could not load compliance data.";
      setViewState({ kind: "error", message });
      return;
    }

    if (!audit.ok || !rights.ok || !consent.ok) return;
    await Promise.all([
      auth.onSessionRotation(audit.value.sessionRotation),
      auth.onSessionRotation(rights.value.sessionRotation),
      auth.onSessionRotation(consent.value.sessionRotation),
    ]);
    setViewState({
      kind: "ready",
      data: {
        audit: {
          ...audit.value.data,
          events: [...audit.value.data.events].sort((left, right) => left.happened_at.localeCompare(right.happened_at)),
        },
        rights: rights.value.data,
        consent: consent.value.data,
      },
    });
  }, [assetId, auth, gatewayBaseUrl]);

  useEffect(() => {
    void loadCompliance();
  }, [loadCompliance]);

  return (
    <ScrollView testID="compliance-screen" style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.header}>
        <Text style={styles.kicker}>Governance</Text>
        <Text style={styles.title}>Compliance center</Text>
        <Text style={styles.copy}>Audit history, rights evidence, and voice consent for this asset.</Text>
      </View>

      {viewState.kind === "loading" ? <Text>Loading compliance data...</Text> : null}
      {viewState.kind === "error" ? (
        <View style={styles.panel}>
          <Text style={styles.panelTitle}>Could not load compliance data</Text>
          <Text style={styles.copy}>{viewState.message}</Text>
          <Pressable testID="compliance-retry" onPress={() => void loadCompliance()} style={styles.secondaryButton}>
            <Text style={styles.secondaryButtonText}>Retry</Text>
          </Pressable>
        </View>
      ) : null}

      {viewState.kind === "ready" ? (
        <>
          <View testID="audit-timeline" style={styles.panel}>
            <Text style={styles.panelTitle}>Audit timeline</Text>
            {viewState.data.audit.events.length === 0 ? <Text testID="audit-empty">No audit events.</Text> : null}
            {viewState.data.audit.events.map((event) => (
              <View key={event.id} testID={`audit-event-${event.id}`} style={styles.row}>
                <Text style={styles.rowTitle}>{event.event_kind.replaceAll("_", " ")}</Text>
                <Text style={styles.meta}>{event.happened_at}</Text>
                {event.detail ? <Text style={styles.copy}>{event.detail}</Text> : null}
              </View>
            ))}
          </View>

          <View testID="rights-ledger" style={styles.panel}>
            <Text style={styles.panelTitle}>Rights ledger</Text>
            {viewState.data.rights.entries.length === 0 ? <Text testID="rights-empty">No rights records.</Text> : null}
            {viewState.data.rights.entries.map((entry) => (
              <View key={entry.id} testID={`rights-entry-${entry.id}`} style={styles.row}>
                <Text style={styles.rowTitle}>{entry.owner}</Text>
                <Text style={styles.copy}>{entry.license_type} / {entry.source_type}</Text>
                <Text style={styles.meta}>{entry.proof_reference}</Text>
              </View>
            ))}
          </View>

          <View style={styles.panel}>
            <Text style={styles.panelTitle}>Voice consent</Text>
            <Text testID="consent-current-status" style={styles.status}>
              {viewState.data.consent.current_status === "grant" ? "Active" : "Inactive"}
            </Text>
            <Text style={styles.copy}>{viewState.data.consent.rows.length} ledger entries</Text>
            <Pressable testID="consent-open" onPress={onManageConsent} style={styles.primaryButton}>
              <Text style={styles.primaryButtonText}>Manage consent</Text>
            </Pressable>
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
  copy: { color: "#52616a", fontSize: 15, lineHeight: 22 },
  panel: { backgroundColor: "#fff", borderColor: "#cbdad2", borderRadius: 10, borderWidth: 1, gap: 10, padding: 18 },
  panelTitle: { color: "#10212a", fontSize: 19, fontWeight: "700" },
  row: { borderTopColor: "#dce6e0", borderTopWidth: 1, gap: 4, paddingTop: 10 },
  rowTitle: { color: "#17372f", fontSize: 15, fontWeight: "700", textTransform: "capitalize" },
  meta: { color: "#61746c", fontSize: 12 },
  status: { color: "#1a6a58", fontSize: 24, fontWeight: "700" },
  primaryButton: { alignSelf: "flex-start", backgroundColor: "#1a5d50", borderRadius: 7, paddingHorizontal: 15, paddingVertical: 10 },
  primaryButtonText: { color: "#fff", fontSize: 14, fontWeight: "700" },
  secondaryButton: { alignSelf: "flex-start", backgroundColor: "#dfe8e5", borderRadius: 7, paddingHorizontal: 15, paddingVertical: 10 },
  secondaryButtonText: { color: "#14312d", fontSize: 14, fontWeight: "700" },
});
