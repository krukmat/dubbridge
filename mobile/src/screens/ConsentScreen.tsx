import { Alert, StyleSheet, Text, View } from "react-native";

import { formatStatusLabel, formatTimestamp } from "../format";

import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, fieldStyle, space, type } from "../theme";
import { type ConsentLedger, type ConsentScope, useConsentLoader } from "./useConsentLoader";
import { TextInput } from "react-native";

type Props = { assetId: string; gatewayBaseUrl: string };

function ScopeButtons({ scope, onChange }: { scope: ConsentScope; onChange: (v: ConsentScope) => void }) {
  return (
    <View style={styles.scopeButtons}>
      {(["voice_clone", "tts_synthesis"] as ConsentScope[]).map((candidate) => (
        <Button key={candidate} testID={`consent-scope-${candidate}`} label={candidate.replaceAll("_", " ")} onPress={() => onChange(candidate)} variant={scope === candidate ? "primary" : "secondary"} size="sm" selected={scope === candidate} />
      ))}
    </View>
  );
}

function ConsentHistoryPanel({ ledger }: { ledger: ConsentLedger }) {
  return (
    <Panel>
      <Text style={styles.panelTitle}>History</Text>
      {ledger.rows.length === 0 ? <Text testID="consent-history-empty" style={styles.emptyText}>No consent history.</Text> : null}
      {ledger.rows.map((row) => (
        <View key={row.id} testID={`consent-row-${row.id}`} style={styles.row}>
          <Text style={styles.rowTitle}>{row.status} / {row.scope}</Text>
          <Text style={styles.meta}>{formatTimestamp(row.happened_at)}</Text>
          {row.evidence_ref ? <Text style={styles.meta}>{row.evidence_ref}</Text> : null}
        </View>
      ))}
    </Panel>
  );
}

type LedgerViewProps = { ledger: ConsentLedger; scope: ConsentScope; evidenceRef: string; error: string | null; submitting: boolean; onChangeScope: (v: ConsentScope) => void; onChangeEvidenceRef: (v: string) => void; onGrant: () => void; onRevoke: () => void };

function ConsentLedgerView({ ledger, scope, evidenceRef, error, submitting, onChangeScope, onChangeEvidenceRef, onGrant, onRevoke }: LedgerViewProps) {
  return (
    <>
      <Panel>
        <Text style={styles.panelTitle}>Current status</Text>
        <Badge testID="consent-status" label={formatStatusLabel(ledger.current_status, "consent")} tone={statusTone(ledger.current_status)} />
      </Panel>
      <Panel>
        <Text style={styles.panelTitle}>Scope</Text>
        <ScopeButtons scope={scope} onChange={onChangeScope} />
        <TextInput testID="consent-evidence-input" accessibilityLabel="Evidence reference" value={evidenceRef} onChangeText={onChangeEvidenceRef} placeholder="Evidence URI or reference ID" autoCapitalize="none" style={fieldStyle} />
        {error ? <Text style={styles.errorText}>{error}</Text> : null}
        <View style={styles.actions}>
          <Button testID="consent-grant" label="Grant" onPress={onGrant} disabled={submitting} />
          <Button testID="consent-revoke" label="Revoke" onPress={onRevoke} variant="danger" disabled={submitting} />
        </View>
      </Panel>
      <ConsentHistoryPanel ledger={ledger} />
    </>
  );
}

export function ConsentScreen({ assetId, gatewayBaseUrl }: Props) {
  const { ledger, scope, setScope, evidenceRef, setEvidenceRef, error, loading, submitting, mutate, onRetry } = useConsentLoader(assetId, gatewayBaseUrl);

  const onPressRevoke = () => {
    Alert.alert("Revoke consent", "This will add an irrevocable revoke entry to the ledger. This action cannot be undone.", [
      { text: "Cancel", style: "cancel" },
      { text: "Revoke", style: "destructive", onPress: () => void mutate("revoke") },
    ]);
  };

  return (
    <Screen testID="consent-screen" scroll edges={["bottom"]}>
      <ScreenHeader kicker="Append-only ledger" title="Voice consent" />
      {loading && !ledger ? <StateView kind="loading" title="Loading consent..." /> : null}
      {!loading && error && !ledger ? <StateView kind="error" title="Could not load consent" message={error} onRetry={onRetry} /> : null}
      {ledger ? <ConsentLedgerView ledger={ledger} scope={scope} evidenceRef={evidenceRef} error={error} submitting={submitting} onChangeScope={setScope} onChangeEvidenceRef={setEvidenceRef} onGrant={() => void mutate("grant")} onRevoke={onPressRevoke} /> : null}
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
