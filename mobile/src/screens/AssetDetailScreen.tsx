import { useState } from "react";
import { Pressable, StyleSheet, Text, View } from "react-native";

import { formatId, formatRelative, formatStatusLabel } from "../format";
import { ActionBar, ACTION_BAR_CONTENT_HEIGHT } from "../components/ActionBar";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { PlaybackStateView, type PlaybackViewState } from "../components/PlaybackStateView";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, radius, space, type } from "../theme";
import { useAssetDetail, type AssetDetailViewState } from "./useAssetDetail";
import type { AssetSummary } from "./AssetListScreen";

type AssetDetailScreenProps = {
  assetId: string;
  gatewayBaseUrl: string;
  onOpenCompliance: () => void;
};

type AssetReadyProps = {
  asset: AssetSummary;
  playbackState: PlaybackViewState;
  onLoadPlayback: () => void;
  onOpenCompliance: () => void;
};

function assetDisplayTitle(title: string): string {
  const value = title.trim();
  return value.length > 0 ? value : "Untitled asset";
}

function assetUploaderLabel(uploaderId: string): string {
  return formatId(uploaderId, { max: 24 }) || "Uploader TBD";
}

function assetUpdatedLabel(updatedAt: string): string {
  return formatRelative(updatedAt) || "Recently updated";
}

function playbackAvailabilityCopy(statusLabel: string, isFinalized: boolean): string {
  if (isFinalized) {
    return "Play the original track inline before opening compliance or sharing the next action.";
  }
  return [
    `This asset is ${statusLabel.toLowerCase()}.`,
    "The original track will appear here when playback is ready.",
  ].join(" ");
}

function AssetMediaPanel({
  asset,
  playbackState,
  onLoadPlayback,
}: Omit<AssetReadyProps, "onOpenCompliance">) {
  const title = assetDisplayTitle(asset.title);
  const statusLabel = formatStatusLabel(asset.status);
  const isFinalized = asset.status === "finalized";

  return (
    <Panel testID="asset-media-panel">
      <View style={styles.mediaHeader}>
        <View style={styles.mediaHeaderCopy}>
          <Text style={styles.mediaEyebrow}>Original track</Text>
          <Text style={styles.mediaTitle} numberOfLines={2}>{title}</Text>
          <Text style={styles.mediaBody}>{playbackAvailabilityCopy(statusLabel, isFinalized)}</Text>
        </View>
        <Badge label={statusLabel} tone={statusTone(asset.status)} />
      </View>
      {isFinalized ? (
        playbackState.kind === "idle" ? (
          <View testID="asset-playback-idle" style={styles.mediaFrame}>
            <Text style={styles.mediaFrameLabel}>Playback ready</Text>
            <Text style={styles.mediaFrameTitle}>Tap Play to load the original track inline.</Text>
          </View>
        ) : (
          <PlaybackStateView
            state={playbackState}
            testIdPrefix="asset-playback"
            testIdPlayer="asset-inline-player"
            onRetry={onLoadPlayback}
          />
        )
      ) : (
        <View testID="asset-playback-unavailable" style={styles.mediaFrame}>
          <Text style={styles.mediaFrameLabel}>Playback unavailable</Text>
          <Text style={styles.mediaFrameTitle}>
            This media surface stays reserved so the preview does not disappear from the page
            hierarchy.
          </Text>
        </View>
      )}
    </Panel>
  );
}

function AssetSummaryPanel({ asset }: { asset: AssetSummary }) {
  const [techExpanded, setTechExpanded] = useState(false);
  const title = assetDisplayTitle(asset.title);

  return (
    <Panel testID="asset-summary-panel">
      <Text style={styles.panelTitle}>Asset summary</Text>
      <Text style={styles.assetTitle}>{title}</Text>
      <View style={styles.summaryGrid}>
        <View style={styles.summaryTile}>
          <Text style={styles.metaLabel}>Status</Text>
          <Badge label={formatStatusLabel(asset.status)} tone={statusTone(asset.status)} />
        </View>
        <View style={styles.summaryTile}>
          <Text style={styles.metaLabel}>Updated</Text>
          <Text style={styles.summaryValue}>{assetUpdatedLabel(asset.updated_at)}</Text>
        </View>
        <View style={styles.summaryTile}>
          <Text style={styles.metaLabel}>Uploaded by</Text>
          <Text style={styles.summaryValue}>{assetUploaderLabel(asset.uploader_id)}</Text>
        </View>
      </View>
      <Pressable
        testID="asset-tech-details-toggle"
        onPress={() => setTechExpanded((v) => !v)}
        accessibilityRole="button"
        accessibilityLabel="Technical details"
        accessibilityState={{ expanded: techExpanded }}
      >
        <Text style={styles.techToggle}>Technical details {techExpanded ? "▲" : "▼"}</Text>
      </Pressable>
      {techExpanded ? (
        <View testID="asset-tech-details" style={styles.techGroup}>
          <Text style={styles.metaLabel}>Asset ID</Text>
          <Text style={styles.metaValue} numberOfLines={1} ellipsizeMode="tail">{asset.id}</Text>
          <Text style={styles.metaLabel}>Uploader ID</Text>
          <Text style={styles.metaValue} numberOfLines={1} ellipsizeMode="tail">
            {formatId(asset.uploader_id) || "Uploader TBD"}
          </Text>
        </View>
      ) : null}
    </Panel>
  );
}

function AssetReadyContent({
  asset,
  playbackState,
  onLoadPlayback,
  onOpenCompliance,
}: AssetReadyProps) {
  return (
    <>
      <AssetMediaPanel asset={asset} playbackState={playbackState} onLoadPlayback={onLoadPlayback} />
      <AssetSummaryPanel asset={asset} />
      <Panel>
        <Text style={styles.panelTitle}>Compliance and consent</Text>
        <Text style={styles.panelCopy}>
          Review audit trail, rights evidence, and voice consent without leaving this asset.
        </Text>
        <Button
          testID="asset-open-compliance"
          label="Open compliance center"
          variant="secondary"
          onPress={onOpenCompliance}
          fullWidth
        />
      </Panel>
    </>
  );
}

function assetScreenPadding(viewState: AssetDetailViewState, actionBarHeight: number): number {
  return viewState.kind === "ready" && viewState.asset.status === "finalized" ? actionBarHeight : 0;
}

export function AssetDetailScreen({
  assetId,
  gatewayBaseUrl,
  onOpenCompliance,
}: AssetDetailScreenProps) {
  const { viewState, playbackState, loadPlayback } = useAssetDetail(assetId, gatewayBaseUrl);
  const actionBarHeight = ACTION_BAR_CONTENT_HEIGHT + space.md * 2;
  const isFinalized = viewState.kind === "ready" && viewState.asset.status === "finalized";

  return (
    <View style={styles.container}>
      <Screen
        testID="asset-detail-screen"
        scroll
        extraBottomPadding={assetScreenPadding(viewState, actionBarHeight)}
      >
        <ScreenHeader kicker="Asset" title="Asset detail" />
        {viewState.kind === "loading" ? <StateView kind="loading" title="Loading asset detail…" /> : null}
        {viewState.kind === "error" ? (
          <StateView kind="error" title="Could not load asset detail" message={viewState.message} />
        ) : null}
        {viewState.kind === "not_available" ? (
          <StateView
            kind="empty"
            title="Asset detail not available yet"
            message="This asset detail surface is not available on the current backend."
          />
        ) : null}
        {viewState.kind === "ready" ? (
          <AssetReadyContent
            asset={viewState.asset}
            playbackState={playbackState}
            onLoadPlayback={() => void loadPlayback(viewState.asset)}
            onOpenCompliance={onOpenCompliance}
          />
        ) : null}
      </Screen>
      {isFinalized && viewState.kind === "ready" ? (
        <ActionBar>
          <Button
            testID="asset-play-button"
            label="Play"
            onPress={() => void loadPlayback(viewState.asset)}
            fullWidth
            loading={playbackState.kind === "loading"}
            disabled={playbackState.kind === "loading"}
          />
        </ActionBar>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: color.canvas },
  assetTitle: { ...type.title, color: color.ink900 },
  mediaHeader: { gap: space.md },
  mediaHeaderCopy: { gap: space.sm },
  mediaEyebrow: { ...type.label, color: color.primary },
  mediaTitle: { ...type.title, color: color.ink900 },
  mediaBody: { ...type.body, color: color.ink500 },
  mediaFrame: {
    minHeight: 220,
    borderRadius: radius.lg,
    backgroundColor: color.ink900,
    padding: space.xl,
    justifyContent: "space-between",
    gap: space.md,
  },
  mediaFrameLabel: {
    ...type.label,
    color: color.primary,
    alignSelf: "flex-start",
    backgroundColor: color.raised,
    borderRadius: radius.pill,
    overflow: "hidden",
    paddingHorizontal: space.md,
    paddingVertical: space.xs,
  },
  mediaFrameTitle: { ...type.heading, color: color.onPrimary, maxWidth: "85%" },
  summaryGrid: { gap: space.md },
  summaryTile: {
    gap: space.xs,
    padding: space.md,
    borderRadius: radius.lg,
    borderWidth: 1,
    borderColor: color.border,
    backgroundColor: color.sunken,
  },
  summaryValue: { ...type.body, color: color.ink700 },
  techToggle: { ...type.label, color: color.primary, marginTop: space.xs },
  techGroup: { gap: space.xs, marginTop: space.xs },
  metaLabel: { ...type.label, color: color.ink400 },
  metaValue: { ...type.meta, color: color.ink700 },
  panelTitle: { ...type.heading, color: color.ink900 },
  panelCopy: { ...type.body, color: color.ink500 },
});
