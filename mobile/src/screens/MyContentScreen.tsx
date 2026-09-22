import { StyleSheet, Text, View } from "react-native";

import type { P2pOwnerContent } from "../api/p2pDashboard";
import { Badge, Button, Card, Screen, ScreenHeader, StateView } from "../components";
import {
  canCreateP2pInvite,
  MY_CONTENT_STATE_LABELS,
  MY_CONTENT_STATE_TONES,
} from "../p2p/dashboard/MyContentModel";
import { useMyContentState } from "../p2p/dashboard/useMyContentState";
import { color, space, type } from "../theme";

type MyContentScreenProps = {
  gatewayBaseUrl: string;
  onCreateInvite: (content: P2pOwnerContent) => void;
};

function MyContentCard({
  content,
  onCreateInvite,
}: {
  content: P2pOwnerContent;
  onCreateInvite: (content: P2pOwnerContent) => void;
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
          onPress={() => onCreateInvite(content)}
        />
      ) : null}
    </Card>
  );
}

function ReadyContent({
  content,
  onCreateInvite,
}: {
  content: P2pOwnerContent[];
  onCreateInvite: (content: P2pOwnerContent) => void;
}) {
  return (
    <View style={styles.list} testID="my-content-list">
      {content.map((item) => (
        <MyContentCard key={item.publicationId} content={item} onCreateInvite={onCreateInvite} />
      ))}
    </View>
  );
}

export function MyContentScreen({ gatewayBaseUrl, onCreateInvite }: MyContentScreenProps) {
  const { viewState, retry } = useMyContentState(gatewayBaseUrl);

  return (
    <Screen testID="my-content-screen" scroll>
      <ScreenHeader
        kicker="P2P"
        title="My content"
        copy="Track publication readiness and create invitations for verified packages."
      />
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
        <ReadyContent content={viewState.content} onCreateInvite={onCreateInvite} />
      ) : null}
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
  meta: { ...type.meta, color: color.ink500 },
});
