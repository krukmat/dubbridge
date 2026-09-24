import { StyleSheet, Text, TextInput, View } from "react-native";

import { Badge, Button, Card, Screen, ScreenHeader, StateView } from "../components";
import { color, fieldStyle, space, type } from "../theme";
import {
  VIEWER_STATE_LABELS,
  VIEWER_STATE_TONES,
  type ViewerInboxProjection,
} from "../p2p/dashboard/InvitesModel";
import { useInvitesActions } from "../p2p/dashboard/useInvitesActions";
import { useInvitesState } from "../p2p/dashboard/useInvitesState";

function InvitationRow({
  projection,
  syncing,
  syncError,
  onSync,
}: {
  projection: ViewerInboxProjection;
  syncing: boolean;
  syncError: string | null;
  onSync: () => void;
}) {
  const { invitation, authorization } = projection.item;
  return (
    <Card testID={`invite-row-${invitation.id}`}>
      <View style={styles.header}>
        <View style={styles.text}>
          <Text style={styles.title}>P2P invitation</Text>
          <Text style={styles.meta}>Asset {invitation.assetId}</Text>
          <Text style={styles.meta}>Publication {authorization.publicationId}</Text>
        </View>
        <Badge
          testID={`invite-state-${invitation.id}`}
          label={VIEWER_STATE_LABELS[projection.state]}
          tone={VIEWER_STATE_TONES[projection.state]}
        />
      </View>
      {projection.action === "sync" || projection.action === "retry_sync" ? (
        <Button
          testID={`invite-sync-${invitation.id}`}
          label={projection.action === "retry_sync" ? "Retry Sync" : "Sync"}
          onPress={onSync}
          loading={syncing}
          disabled={syncing}
          variant="secondary"
          size="sm"
        />
      ) : null}
      {syncError ? (
        <Text testID={`invite-sync-error-${invitation.id}`} style={styles.error}>
          {syncError}
        </Text>
      ) : null}
    </Card>
  );
}

function ClaimInvitationForm({
  token,
  error,
  isClaiming,
  canClaim,
  onChangeToken,
  onClaim,
}: {
  token: string;
  error: string | null;
  isClaiming: boolean;
  canClaim: boolean;
  onChangeToken: (value: string) => void;
  onClaim: () => void;
}) {
  return (
    <Card testID="invites-claim-card">
      <Text style={styles.title}>Claim invitation</Text>
      <TextInput
        testID="invites-claim-token"
        style={fieldStyle}
        value={token}
        onChangeText={onChangeToken}
        autoCapitalize="none"
        autoCorrect={false}
        placeholder="Paste invitation token"
        placeholderTextColor={color.ink400}
      />
      {error ? <Text testID="invites-claim-error" style={styles.error}>{error}</Text> : null}
      <Button
        testID="invites-claim-submit"
        label="Claim"
        onPress={onClaim}
        loading={isClaiming}
        disabled={!canClaim}
      />
    </Card>
  );
}

function ReadyInvitations({
  invitations,
  busyInvites,
  syncErrors,
  onSync,
}: {
  invitations: ViewerInboxProjection[];
  busyInvites: ReadonlySet<string>;
  syncErrors: Readonly<Record<string, string>>;
  onSync: (projection: ViewerInboxProjection) => void;
}) {
  return (
    <View style={styles.list}>
      {invitations.map((projection) => (
        <InvitationRow
          key={projection.item.invitation.id}
          projection={projection}
          syncing={busyInvites.has(projection.item.invitation.id)}
          syncError={syncErrors[projection.item.invitation.id] ?? null}
          onSync={() => onSync(projection)}
        />
      ))}
    </View>
  );
}

export function InvitesScreen({ gatewayBaseUrl }: { gatewayBaseUrl: string }) {
  const { viewState, retry, refresh } = useInvitesState(gatewayBaseUrl);
  const claim = useInvitesActions(gatewayBaseUrl, refresh);

  return (
    <Screen testID="invites-screen">
      <ScreenHeader
        kicker="P2P"
        title="Invites"
        copy="Your claimed invitations and local availability."
      />

      <ClaimInvitationForm
        token={claim.claimToken}
        error={claim.claimError}
        isClaiming={claim.isClaiming}
        canClaim={claim.canClaim}
        onChangeToken={claim.updateClaimToken}
        onClaim={() => void claim.claim()}
      />

      {viewState.kind === "loading" ? (
        <StateView
          testID="invites-loading"
          kind="loading"
          title="Loading invitations…"
        />
      ) : null}
      {viewState.kind === "error" ? (
        <StateView
          testID="invites-error"
          kind="error"
          title="Could not load invitations"
          message={viewState.message}
          onRetry={retry}
        />
      ) : null}
      {viewState.kind === "empty" ? (
        <StateView
          testID="invites-empty"
          kind="empty"
          title="No P2P invitations"
          message="Claimed invitations will appear here."
        />
      ) : null}
      {viewState.kind === "ready" ? (
        <ReadyInvitations
          invitations={viewState.invitations}
          busyInvites={claim.busyInvites}
          syncErrors={claim.syncErrors}
          onSync={(projection) => void claim.syncInvitation(projection)}
        />
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  list: { gap: space.md, paddingBottom: space.xl },
  header: {
    flexDirection: "row",
    alignItems: "flex-start",
    gap: space.md,
  },
  text: { flex: 1, gap: space.xs },
  title: { ...type.heading, color: color.ink900 },
  meta: { ...type.meta, color: color.ink500 },
  error: { ...type.meta, color: color.danger },
});
