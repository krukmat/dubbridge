import { StyleSheet, Text, View } from "react-native";

import { Badge, Card, Screen, ScreenHeader, StateView } from "../components";
import { space, type, color } from "../theme";
import {
  VIEWER_STATE_LABELS,
  VIEWER_STATE_TONES,
  type ViewerInboxProjection,
} from "../p2p/dashboard/InvitesModel";
import { useInvitesState } from "../p2p/dashboard/useInvitesState";

function InvitationRow({ projection }: { projection: ViewerInboxProjection }) {
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
    </Card>
  );
}

function ReadyInvitations({
  invitations,
}: {
  invitations: ViewerInboxProjection[];
}) {
  return (
    <View style={styles.list}>
      {invitations.map((projection) => (
        <InvitationRow
          key={projection.item.invitation.id}
          projection={projection}
        />
      ))}
    </View>
  );
}

export function InvitesScreen({ gatewayBaseUrl }: { gatewayBaseUrl: string }) {
  const { viewState, retry } = useInvitesState(gatewayBaseUrl);

  return (
    <Screen testID="invites-screen">
      <ScreenHeader
        kicker="P2P"
        title="Invites"
        copy="Your claimed invitations and local availability."
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
        <ReadyInvitations invitations={viewState.invitations} />
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
});
