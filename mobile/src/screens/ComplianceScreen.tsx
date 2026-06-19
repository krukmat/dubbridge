import { useCallback, useEffect, useState } from "react";
import { ScrollView, StyleSheet, Text, View } from "react-native";

import { formatTimestamp } from "../format";

import { createGatewayClient } from "../api/client";
import { useAuth } from "../auth/AuthProvider";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";

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

  const onRetry = useCallback(() => {
    void loadCompliance();
  }, [loadCompliance]);

  return (
    <Screen testID="compliance-screen" scroll edges={["bottom"]}>
      <ScreenHeader
        kicker="Governance"
        title="Compliance center"
        copy="Audit history, rights evidence, and voice consent for this asset."
      />

      {viewState.kind === "loading" ? (
        <StateView kind="loading" title="Loading compliance data..." />
      ) : null}

      {viewState.kind === "error" ? (
        <StateView
          testID="compliance"
          kind="error"
          title="Could not load compliance data"
          message={viewState.message}
          onRetry={onRetry}
        />
      ) : null}

      {viewState.kind === "ready" ? (
        <>
          <Panel testID="audit-timeline">
            <Text style={styles.panelTitle}>Audit timeline</Text>
            {viewState.data.audit.events.length === 0 ? (
              <Text testID="audit-empty" style={styles.emptyText}>No audit events.</Text>
            ) : null}
            {viewState.data.audit.events.map((event) => (
              <View key={event.id} testID={`audit-event-${event.id}`} style={styles.row}>
                <Text style={styles.rowTitle}>{event.event_kind.replaceAll("_", " ")}</Text>
                <Text style={styles.meta}>{formatTimestamp(event.happened_at)}</Text>
                {event.detail ? <Text style={styles.copy}>{event.detail}</Text> : null}
              </View>
            ))}
          </Panel>

          <Panel testID="rights-ledger">
            <Text style={styles.panelTitle}>Rights ledger</Text>
            {viewState.data.rights.entries.length === 0 ? (
              <Text testID="rights-empty" style={styles.emptyText}>No rights records.</Text>
            ) : null}
            {viewState.data.rights.entries.map((entry) => (
              <View key={entry.id} testID={`rights-entry-${entry.id}`} style={styles.row}>
                <Text style={styles.rowTitle}>{entry.owner}</Text>
                <Text style={styles.copy}>{entry.license_type} / {entry.source_type}</Text>
                <Text style={styles.meta}>{entry.proof_reference}</Text>
              </View>
            ))}
          </Panel>

          <Panel>
            <Text style={styles.panelTitle}>Voice consent</Text>
            <Badge
              testID="consent-current-status"
              label={viewState.data.consent.current_status === "grant" ? "Active" : "Inactive"}
              tone={statusTone(viewState.data.consent.current_status)}
            />
            <Text style={styles.copy}>{viewState.data.consent.rows.length} ledger entries</Text>
            <Button
              testID="consent-open"
              label="Manage consent"
              onPress={onManageConsent}
            />
          </Panel>
        </>
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  panelTitle: { ...type.heading, color: color.ink900 },
  emptyText: { ...type.body, color: color.ink500 },
  copy: { ...type.body, color: color.ink500 },
  row: { borderTopColor: color.border, borderTopWidth: 1, gap: space.xs, paddingTop: space.sm },
  rowTitle: { ...type.bodyStrong, color: color.ink900, textTransform: "capitalize" },
  meta: { ...type.meta, color: color.ink400 },
});
