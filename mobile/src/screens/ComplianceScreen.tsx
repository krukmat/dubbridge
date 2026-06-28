import { StyleSheet, Text, View } from "react-native";

import { formatStatusLabel, formatTimestamp } from "../format";

import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";
import { type ComplianceData, useComplianceLoader } from "./useComplianceLoader";

type Props = { assetId: string; gatewayBaseUrl: string; onManageConsent: () => void };

function ComplianceReady({ data, onManageConsent }: { data: ComplianceData; onManageConsent: () => void }) {
  return (
    <>
      <Panel testID="audit-timeline">
        <Text style={styles.panelTitle}>Audit timeline</Text>
        {data.audit.events.length === 0 ? <Text testID="audit-empty" style={styles.emptyText}>No audit events.</Text> : null}
        {data.audit.events.map((event) => (
          <View key={event.id} testID={`audit-event-${event.id}`} style={styles.row}>
            <Text style={styles.rowTitle}>{event.event_kind.replaceAll("_", " ")}</Text>
            <Text style={styles.meta}>{formatTimestamp(event.happened_at)}</Text>
            {event.detail ? <Text style={styles.copy}>{event.detail}</Text> : null}
          </View>
        ))}
      </Panel>
      <Panel testID="rights-ledger">
        <Text style={styles.panelTitle}>Rights ledger</Text>
        {data.rights.entries.length === 0 ? <Text testID="rights-empty" style={styles.emptyText}>No rights records.</Text> : null}
        {data.rights.entries.map((entry) => (
          <View key={entry.id} testID={`rights-entry-${entry.id}`} style={styles.row}>
            <Text style={styles.rowTitle}>{entry.owner}</Text>
            <Text style={styles.copy}>{entry.license_type} / {entry.source_type}</Text>
            <Text style={styles.meta}>{entry.proof_reference}</Text>
          </View>
        ))}
      </Panel>
      <Panel>
        <Text style={styles.panelTitle}>Voice consent</Text>
        <Badge testID="consent-current-status" label={formatStatusLabel(data.consent.current_status, "consent")} tone={statusTone(data.consent.current_status)} />
        <Text style={styles.copy}>{data.consent.rows.length} ledger entries</Text>
        <Button testID="consent-open" label="Manage consent" onPress={onManageConsent} />
      </Panel>
    </>
  );
}

export function ComplianceScreen({ assetId, gatewayBaseUrl, onManageConsent }: Props) {
  const { viewState, onRetry } = useComplianceLoader(assetId, gatewayBaseUrl);
  return (
    <Screen testID="compliance-screen" scroll>
      <ScreenHeader kicker="Governance" title="Compliance center" copy="Audit history, rights evidence, and voice consent for this asset." />
      {viewState.kind === "loading" ? <StateView kind="loading" title="Loading compliance data..." /> : null}
      {viewState.kind === "error" ? <StateView testID="compliance" kind="error" title="Could not load compliance data" message={viewState.message} onRetry={onRetry} /> : null}
      {viewState.kind === "ready" ? <ComplianceReady data={viewState.data} onManageConsent={onManageConsent} /> : null}
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
