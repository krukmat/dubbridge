import { StyleSheet, Text, TextInput, View } from "react-native";

import { Button, Card, Screen, ScreenHeader, StateView } from "../components";
import { color, fieldStyle, space, type } from "../theme";
import { type ViewerInboxProjection } from "../p2p/dashboard/InvitesModel";
import { P2pStatusBadge } from "../p2p/dashboard/P2pStatusBadge";
import { useInvitesActions } from "../p2p/dashboard/useInvitesActions";
import { useInvitesState } from "../p2p/dashboard/useInvitesState";
import { P2PPlaybackSessionView } from "../p2p/playback/P2PPlaybackSessionView";

function InvitationRow({
  projection, syncing, syncError, onSync, playing, playError, onPlay,
}: {
  projection: ViewerInboxProjection; syncing: boolean; syncError: string | null;
  onSync: () => void; playing: boolean; playError: string | null; onPlay: () => void;
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
        <P2pStatusBadge
          surface="viewer"
          state={projection.state}
          testID={`invite-state-${invitation.id}`}
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
      {projection.action === "play" ? (
        <Button
          testID={`invite-play-${invitation.id}`}
          label="Play"
          onPress={onPlay}
          loading={playing}
          disabled={playing}
          size="sm"
        />
      ) : null}
      {playError ? (
        <Text testID={`invite-play-error-${invitation.id}`} style={styles.error}>
          {playError}
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
  busyPlayInvites,
  playErrors,
  onPlay,
}: {
  invitations: ViewerInboxProjection[];
  busyInvites: ReadonlySet<string>;
  syncErrors: Readonly<Record<string, string>>;
  onSync: (projection: ViewerInboxProjection) => void;
  busyPlayInvites: ReadonlySet<string>;
  playErrors: Readonly<Record<string, string>>;
  onPlay: (projection: ViewerInboxProjection) => void;
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
          playing={busyPlayInvites.has(projection.item.invitation.id)}
          playError={playErrors[projection.item.invitation.id] ?? null}
          onPlay={() => onPlay(projection)}
        />
      ))}
    </View>
  );
}

function InvitesBody({
  viewState,
  retry,
  claim,
}: {
  viewState: ReturnType<typeof useInvitesState>["viewState"];
  retry: () => void;
  claim: ReturnType<typeof useInvitesActions>;
}) {
  if (viewState.kind === "loading") {
    return <StateView testID="invites-loading" kind="loading" title="Loading invitations…" />;
  }
  if (viewState.kind === "error") {
    return (
      <StateView testID="invites-error" kind="error" title="Could not load invitations"
        message={viewState.message} onRetry={retry} />
    );
  }
  if (viewState.kind === "empty") {
    return (
      <StateView testID="invites-empty" kind="empty" title="No P2P invitations"
        message="Claimed invitations will appear here." />
    );
  }
  return (
    <ReadyInvitations
      invitations={viewState.invitations}
      busyInvites={claim.busyInvites}
      syncErrors={claim.syncErrors}
      onSync={(projection) => void claim.syncInvitation(projection)}
      busyPlayInvites={claim.busyPlayInvites}
      playErrors={claim.playErrors}
      onPlay={(projection) => void claim.playInvitation(projection)}
    />
  );
}

export function InvitesScreen({
  gatewayBaseUrl,
  onBack,
}: {
  gatewayBaseUrl: string;
  onBack?: () => void;
}) {
  const { viewState, retry, refresh } = useInvitesState(gatewayBaseUrl);
  const claim = useInvitesActions(gatewayBaseUrl, refresh);

  return (
    <Screen testID="invites-screen">
      <ScreenHeader
        kicker="P2P" title="Invites"
        copy="Your claimed invitations and local availability."
      />
      {onBack ? (
        <Button
          testID="invites-back"
          label="Back to home"
          variant="secondary"
          size="sm"
          onPress={onBack}
        />
      ) : null}
      <ClaimInvitationForm
        token={claim.claimToken} error={claim.claimError}
        isClaiming={claim.isClaiming} canClaim={claim.canClaim}
        onChangeToken={claim.updateClaimToken} onClaim={() => void claim.claim()}
      />
      <InvitesBody viewState={viewState} retry={retry} claim={claim} />
      {claim.playbackSession ? (
        <P2PPlaybackSessionView
          testID="p2p-player"
          session={claim.playbackSession.session}
          controller={claim.playbackController}
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
