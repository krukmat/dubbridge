import { useCallback, useEffect, useMemo, useState } from "react";
import { StyleSheet, Text, View } from "react-native";

import { createGatewayClient } from "../api/client";
import type { P2pOwnerContent, P2pOwnerContentState } from "../api/p2pDashboard";
import { useAuth } from "../auth/AuthProvider";
import { Badge, type BadgeTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { P2PDashboardService } from "../p2p/P2PDashboardService";
import { color, space, type } from "../theme";

type MyContentScreenProps = {
  gatewayBaseUrl: string;
  onCreateInvite: (content: P2pOwnerContent) => void;
};

type MyContentViewState =
  | { kind: "loading" }
  | { kind: "ready"; content: P2pOwnerContent[] }
  | { kind: "empty" }
  | { kind: "error"; message: string };

const STATE_LABELS: Record<P2pOwnerContentState, string> = {
  processing: "Processing",
  ready: "Ready",
  failed: "Failed",
};

const STATE_TONES: Record<P2pOwnerContentState, BadgeTone> = {
  processing: "info",
  ready: "success",
  failed: "danger",
};

export function canCreateP2pInvite(content: P2pOwnerContent): boolean {
  const descriptor = content.descriptor;
  return (
    content.state === "ready" &&
    descriptor !== null &&
    descriptor.assetId === content.assetId &&
    descriptor.publicationId === content.publicationId &&
    descriptor.lineageId === content.lineageId
  );
}

function toViewState(content: P2pOwnerContent[]): MyContentViewState {
  return content.length === 0 ? { kind: "empty" } : { kind: "ready", content };
}

function errorMessage(error: { kind: string; message?: string; status?: number }): string {
  if (error.kind === "network") return error.message ?? "Network request failed.";
  if (error.kind === "forbidden") return "You do not have access to P2P content.";
  return error.status ? `Request failed with status ${error.status}.` : "Could not load P2P content.";
}

function useMyContentState(gatewayBaseUrl: string) {
  const auth = useAuth();
  const dashboard = useMemo(
    () => new P2PDashboardService(createGatewayClient({ gatewayBaseUrl })),
    [gatewayBaseUrl],
  );
  const [viewState, setViewState] = useState<MyContentViewState>({ kind: "loading" });

  const load = useCallback(async () => {
    if (!auth.sessionRef) {
      setViewState({ kind: "error", message: "Session unavailable." });
      return;
    }

    const result = await dashboard.listOwnerContent(auth.sessionRef);
    if (result.ok) {
      await auth.onSessionRotation(result.value.sessionRotation);
      setViewState(toViewState(result.value.data));
      return;
    }
    if (result.error.kind === "session_expired") {
      await auth.logout();
      return;
    }
    setViewState({ kind: "error", message: errorMessage(result.error) });
  }, [auth.logout, auth.onSessionRotation, auth.sessionRef, dashboard]);

  useEffect(() => {
    void load();
  }, [load]);

  const retry = useCallback(() => {
    setViewState({ kind: "loading" });
    void load();
  }, [load]);

  return { viewState, retry };
}

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
          label={STATE_LABELS[content.state]}
          tone={STATE_TONES[content.state]}
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
