import { useCallback, useEffect, useState } from "react";
import { StyleSheet, Text, View } from "react-native";

import { createGatewayClient } from "../api/client";
import { listNotifications, type NotificationItem } from "../api/notifications";
import { useAuth } from "../auth/AuthProvider";
import { formatStatusLabel } from "../format";
import { Badge, statusTone } from "../components/Badge";
import { Card } from "../components/Card";
import { Button } from "../components/Button";
import { IconBadge } from "../components/IconBadge";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";
import type { AssetSummary } from "./AssetListScreen";

const NAV_CARDS = [
  {
    testID: "home-open-assets" as const,
    title: "Browse assets",
    subtitle: "View and manage uploaded media",
    key: "assets",
    symbol: "AS",
    tone: "primary" as const,
  },
  {
    testID: "home-open-upload" as const,
    title: "Upload asset",
    subtitle: "Add new media to your workspace",
    key: "upload",
    symbol: "UP",
    tone: "success" as const,
  },
  {
    testID: "home-open-review" as const,
    title: "Review inbox",
    subtitle: "Approve or reject pending review tasks",
    key: "review",
    symbol: "RV",
    tone: "info" as const,
  },
  {
    testID: "home-open-organizations" as const,
    title: "Organizations and projects",
    subtitle: "Manage teams and project workspaces",
    key: "organizations",
    symbol: "OR",
    tone: "neutral" as const,
  },
] as const;

type HomeDashboardData = {
  recentAssets: AssetSummary[];
  pendingReviewCount: number;
};

type HomeDashboardState =
  | { kind: "loading" }
  | { kind: "ready"; data: HomeDashboardData }
  | { kind: "error"; message: string };

function dashboardErrorMessage(error: { kind: string; message?: string; status?: number }) {
  return error.kind === "network"
    ? error.message ?? "Network request failed."
    : error.kind === "forbidden"
      ? "You do not have access to the asset list."
      : `Could not load dashboard (${error.status}).`;
}

async function loadDashboardData(
  gatewayBaseUrl: string,
  sessionRef: string | null,
) {
  const client = createGatewayClient({ gatewayBaseUrl });
  const [assetsResult, notifResult] = await Promise.all([
    client.get<AssetSummary[]>("/api/assets", sessionRef),
    listNotifications(client, sessionRef),
  ]);
  return { assetsResult, notifResult };
}

function countPendingReviews(notifications: NotificationItem[]) {
  return notifications.filter(
    (n) => n.ref_entity_type === "review_task" && n.read_at === null,
  ).length;
}

function useDashboardState(
  gatewayBaseUrl: string,
  sessionRef: string | null,
  logout: () => Promise<void>,
  onSessionRotation: (rotation: string | null) => Promise<void>,
) {
  const [dashState, setDashState] = useState<HomeDashboardState>({ kind: "loading" });

  const load = useCallback(async (): Promise<void> => {
    setDashState({ kind: "loading" });
    const { assetsResult, notifResult } = await loadDashboardData(gatewayBaseUrl, sessionRef);

    if (!assetsResult.ok) {
      if (assetsResult.error.kind === "session_expired") {
        await logout();
        return;
      }
      setDashState({ kind: "error", message: dashboardErrorMessage(assetsResult.error) });
      return;
    }

    await onSessionRotation(assetsResult.value.sessionRotation);

    let pendingReviewCount = 0;
    if (notifResult.ok) {
      await onSessionRotation(notifResult.value.sessionRotation);
      pendingReviewCount = countPendingReviews(notifResult.value.data.notifications);
    } else if (notifResult.error.kind === "session_expired") {
      await logout();
      return;
    }

    setDashState({
      kind: "ready",
      data: {
        recentAssets: assetsResult.value.data.slice(0, 3),
        pendingReviewCount,
      },
    });
  }, [gatewayBaseUrl, logout, onSessionRotation, sessionRef]);

  useEffect(() => {
    void load();
  }, [load]);

  return { dashState, load };
}

function RecentAssetRow({
  asset,
  onOpenAssets,
}: {
  asset: AssetSummary;
  onOpenAssets: () => void;
}) {
  return (
    <Card
      key={asset.id}
      testID={`home-recent-asset-${asset.id}`}
      onPress={onOpenAssets}
      trailing="chevron"
      mediaTone={statusTone(asset.status)}
      accessibilityLabel={asset.title}
    >
      <Text style={styles.assetTitle} numberOfLines={1}>{asset.title}</Text>
      <Badge
        label={formatStatusLabel(asset.status)}
        tone={statusTone(asset.status)}
      />
    </Card>
  );
}

function ReviewSummarySection({
  pendingReviewCount,
  onOpenReview,
}: {
  pendingReviewCount: number;
  onOpenReview: () => void;
}) {
  if (pendingReviewCount <= 0) return null;
  return (
    <View style={styles.section}>
      <Text style={styles.sectionTitle}>Review inbox</Text>
      <Card
        testID="home-pending-review-summary"
        onPress={onOpenReview}
        trailing="chevron"
        accessibilityLabel={`Review inbox, ${pendingReviewCount} pending`}
      >
        <Text style={styles.summaryCount}>
          {pendingReviewCount} pending
        </Text>
      </Card>
    </View>
  );
}

function CommunityModuleSlot() {
  return <View testID="home-community-slot" />;
}

function RecentAssetsSection({
  recentAssets,
  onOpenAssets,
}: {
  recentAssets: AssetSummary[];
  onOpenAssets: () => void;
}) {
  return (
    <View style={styles.section}>
      <Text style={styles.sectionTitle}>Recent assets</Text>
      {recentAssets.length > 0 ? (
        <View style={styles.assetList}>
          {recentAssets.map((asset) => (
            <RecentAssetRow key={asset.id} asset={asset} onOpenAssets={onOpenAssets} />
          ))}
        </View>
      ) : (
        <Text style={styles.emptyHint}>No recent assets.</Text>
      )}
    </View>
  );
}

function QuickActionsSection({
  onOpenAssets,
  onOpenUpload,
  onOpenReview,
  onOpenOrganizations,
}: {
  onOpenAssets: () => void;
  onOpenUpload: () => void;
  onOpenReview: () => void;
  onOpenOrganizations: () => void;
}) {
  const callbacks: Record<string, () => void> = {
    assets: onOpenAssets,
    upload: onOpenUpload,
    review: onOpenReview,
    organizations: onOpenOrganizations,
  };

  return (
    <View style={styles.section}>
      <Text style={styles.sectionTitle}>Quick actions</Text>
      <View style={styles.navCards}>
        {NAV_CARDS.map((card) => (
          <Card
            key={card.key}
            testID={card.testID}
            title={card.title}
            subtitle={card.subtitle}
            leadingAdornment={<IconBadge symbol={card.symbol} tone={card.tone} />}
            trailing="chevron"
            onPress={callbacks[card.key]}
            accessibilityLabel={card.title}
          />
        ))}
      </View>
    </View>
  );
}

function AccountSection({ onLogout }: { onLogout: () => Promise<void> }) {
  return (
    <View style={styles.section}>
      <Text style={styles.sectionTitle}>Account</Text>
      <Card
        testID="home-account-card"
        title="Signed-in workspace"
        subtitle="Manage your current session"
        leadingAdornment={<IconBadge symbol="ME" tone="neutral" testID="home-account-icon" />}
      >
        <Button
          testID="home-sign-out"
          label="Sign out"
          variant="secondary"
          onPress={() => void onLogout()}
        />
      </Card>
    </View>
  );
}

function DashboardContent({
  dashState,
  onOpenAssets,
  onOpenUpload,
  onOpenReview,
  onOpenOrganizations,
  onLogout,
}: {
  dashState: Extract<HomeDashboardState, { kind: "ready" }>;
  onOpenAssets: () => void;
  onOpenUpload: () => void;
  onOpenReview: () => void;
  onOpenOrganizations: () => void;
  onLogout: () => Promise<void>;
}) {
  return (
    <>
      <ReviewSummarySection
        pendingReviewCount={dashState.data.pendingReviewCount}
        onOpenReview={onOpenReview}
      />
      <RecentAssetsSection
        recentAssets={dashState.data.recentAssets}
        onOpenAssets={onOpenAssets}
      />
      <CommunityModuleSlot />
      <QuickActionsSection
        onOpenAssets={onOpenAssets}
        onOpenUpload={onOpenUpload}
        onOpenReview={onOpenReview}
        onOpenOrganizations={onOpenOrganizations}
      />
      <AccountSection onLogout={onLogout} />
    </>
  );
}

export function HomeScreen({
  dubbridgeEnv: _dubbridgeEnv,
  gatewayBaseUrl,
  onOpenAssets,
  onOpenUpload,
  onOpenReview,
  onOpenOrganizations,
}: {
  dubbridgeEnv: string;
  gatewayBaseUrl: string;
  onOpenAssets: () => void;
  onOpenUpload: () => void;
  onOpenReview: () => void;
  onOpenOrganizations: () => void;
}) {
  const auth = useAuth();
  const { dashState, load } = useDashboardState(
    gatewayBaseUrl,
    auth.sessionRef,
    auth.logout,
    auth.onSessionRotation,
  );

  return (
    <Screen testID="home-screen">
      <ScreenHeader
        kicker="DubBridge"
        title="Your workspace"
        copy="Pick up where you left off."
      />

      {dashState.kind === "loading" ? (
        <StateView kind="loading" title="Loading dashboard…" />
      ) : null}

      {dashState.kind === "error" ? (
        <StateView
          kind="error"
          title="Could not load dashboard"
          message={dashState.message}
          onRetry={() => void load()}
        />
      ) : null}

      {dashState.kind === "ready" ? (
        <DashboardContent
          dashState={dashState}
          onOpenAssets={onOpenAssets}
          onOpenUpload={onOpenUpload}
          onOpenReview={onOpenReview}
          onOpenOrganizations={onOpenOrganizations}
          onLogout={auth.logout}
        />
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  section: { gap: space.sm },
  sectionTitle: { ...type.label, color: color.ink400 },
  navCards: { gap: space.md },
  assetList: { gap: space.md },
  assetTitle: { ...type.heading, color: color.ink900 },
  summaryCount: { ...type.bodyStrong, color: color.ink900 },
  emptyHint: { ...type.meta, color: color.ink400 },
});
