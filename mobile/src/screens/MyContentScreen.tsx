import { StyleSheet, Text, View } from "react-native";

import type { P2pOwnerContent } from "../api/p2pDashboard";
import { Badge, Button, Card, Screen, ScreenHeader, StateView } from "../components";
import {
  canCreateP2pInvite,
  MY_CONTENT_STATE_LABELS,
  MY_CONTENT_STATE_TONES,
} from "../p2p/dashboard/MyContentModel";
import {
  type MyContentInviteState,
  useMyContentInvite,
} from "../p2p/dashboard/useMyContentInvite";
import { useMyContentState } from "../p2p/dashboard/useMyContentState";
import { color, radius, space, type } from "../theme";

type CreatedInviteState = Extract<MyContentInviteState, { kind: "created" }>;

function MyContentCard({
  content,
  onCreateInvite,
  creating,
  inviteLocked,
}: {
  content: P2pOwnerContent;
  onCreateInvite: (content: P2pOwnerContent) => void;
  creating: boolean;
  inviteLocked: boolean;
}) {
  const inviteEligible = canCreateP2pInvite(content);
  return (
    <Card testID={`my-content-card-${content.assetId}`}>
      <View style={styles.cardHeader}>
        <View style={styles.cardText}>
          <Text style={styles.title} numberOfLines={2}>
            {content.title}
          </Text>
          <Text style={styles.meta} numberOfLines={1}>
            {content.publicationId}
          </Text>
        </View>
        <Badge
          testID={`my-content-state-${content.assetId}`}
          label={MY_CONTENT_STATE_LABELS[content.state]}
          tone={MY_CONTENT_STATE_TONES[content.state]}
        />
      </View>
      {inviteEligible ? (
        <Button
          testID={`my-content-create-invite-${content.assetId}`}
          label="Create invite"
          variant="secondary"
          size="sm"
          loading={creating}
          disabled={inviteLocked && !creating}
          onPress={() => onCreateInvite(content)}
        />
      ) : null}
    </Card>
  );
}

function InviteTokenCard({
  state,
  onCopy,
  onDismiss,
}: {
  state: CreatedInviteState;
  onCopy: () => Promise<void>;
  onDismiss: () => void;
}) {
  return (
    <Card testID="my-content-invite-token">
      <Text style={styles.title}>Invite ready</Text>
      <Text style={styles.body}>
        This token is shown only in this screen. Copy it before closing.
      </Text>
      <Text testID="my-content-invite-token-value" selectable style={styles.token}>
        {state.token}
      </Text>
      {state.copyError ? (
        <Text testID="my-content-invite-copy-error" style={styles.error}>
          {state.copyError}
        </Text>
      ) : null}
      <View style={styles.actions}>
        <Button
          testID="my-content-copy-invite"
          label={state.copied ? "Copied" : "Copy invite"}
          variant="secondary"
          size="sm"
          onPress={() => void onCopy()}
        />
        <Button
          testID="my-content-dismiss-invite"
          label="Done"
          variant="secondary"
          size="sm"
          onPress={onDismiss}
        />
      </View>
    </Card>
  );
}

function ReadyContent({
  content,
  onCreateInvite,
  inviteState,
}: {
  content: P2pOwnerContent[];
  onCreateInvite: (content: P2pOwnerContent) => void;
  inviteState: MyContentInviteState;
}) {
  const inviteLocked = inviteState.kind === "creating" || inviteState.kind === "created";
  return (
    <View style={styles.list} testID="my-content-list">
      {content.map((item) => (
        <MyContentCard
          key={item.publicationId}
          content={item}
          onCreateInvite={onCreateInvite}
          creating={inviteState.kind === "creating" && inviteState.assetId === item.assetId}
          inviteLocked={inviteLocked}
        />
      ))}
    </View>
  );
}

type MyContentScreenProps = { gatewayBaseUrl: string; onBack?: () => void };

function BackToHome({ onBack }: { onBack?: () => void }) {
  return onBack ? (
    <Button
      testID="my-content-back"
      label="Back to home"
      variant="secondary"
      onPress={onBack}
    />
  ) : null;
}

export function MyContentScreen({ gatewayBaseUrl, onBack }: MyContentScreenProps) {
  const { viewState, retry, refresh } = useMyContentState(gatewayBaseUrl);
  const { inviteState, createInvite, copyInvite, dismissInvite } =
    useMyContentInvite(gatewayBaseUrl, refresh);

  return (
    <Screen testID="my-content-screen" scroll>
      <ScreenHeader
        kicker="P2P"
        title="My content"
        copy="Track publication readiness and create invitations for verified packages."
      />
      {inviteState.kind === "created" ? (
        <InviteTokenCard state={inviteState} onCopy={copyInvite} onDismiss={dismissInvite} />
      ) : null}
      {inviteState.kind === "error" ? (
        <StateView
          testID="my-content-invite-error"
          kind="error"
          title="Invite not created"
          message={inviteState.message}
        />
      ) : null}
      {viewState.kind === "loading" ? (
        <StateView
          testID="my-content-loading"
          kind="loading"
          title="Loading content…"
          message="Fetching your P2P publication state."
        />
      ) : null}
      {viewState.kind === "error" ? (
        <StateView
          testID="my-content-error"
          kind="error"
          title="Could not load content"
          message={viewState.message}
          onRetry={retry}
        />
      ) : null}
      {viewState.kind === "empty" ? (
        <StateView
          testID="my-content-empty"
          kind="empty"
          title="No P2P content yet"
          message="Ready and processing P2P publications will appear here."
        />
      ) : null}
      {viewState.kind === "ready" ? (
        <ReadyContent
          content={viewState.content}
          onCreateInvite={(content) => void createInvite(content)}
          inviteState={inviteState}
        />
      ) : null}
      <BackToHome onBack={onBack} />
    </Screen>
  );
}

const styles = StyleSheet.create({
  list: { gap: space.md, paddingBottom: space.xl },
  cardHeader: {
    flexDirection: "row",
    alignItems: "flex-start",
    gap: space.md,
  },
  cardText: { flex: 1, gap: space.xs },
  title: { ...type.heading, color: color.ink900 },
  body: { ...type.body, color: color.ink500 },
  meta: { ...type.meta, color: color.ink500 },
  token: {
    ...type.bodyStrong,
    color: color.ink900,
    backgroundColor: color.sunken,
    borderRadius: radius.sm,
    padding: space.md,
  },
  error: { ...type.meta, color: color.danger },
  actions: { flexDirection: "row", flexWrap: "wrap", gap: space.sm },
});
