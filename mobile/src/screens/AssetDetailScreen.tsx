import { useState } from "react";
import { Pressable, StyleSheet, Text, View } from "react-native";

import { formatStatusLabel } from "../format";
import { ActionBar, ACTION_BAR_CONTENT_HEIGHT } from "../components/ActionBar";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { PlaybackStateView, type PlaybackViewState } from "../components/PlaybackStateView";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { color, space, type } from "../theme";
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

function AssetReadyContent({ asset, playbackState, onLoadPlayback, onOpenCompliance }: AssetReadyProps) {
  const [techExpanded, setTechExpanded] = useState(false);
  return (
    <>
      <Panel>
        <Text style={styles.assetTitle}>{asset.title}</Text>
        <Badge label={formatStatusLabel(asset.status)} tone={statusTone(asset.status)} />
        <Pressable testID="asset-tech-details-toggle" onPress={() => setTechExpanded((v) => !v)} accessibilityRole="button" accessibilityLabel="Technical details" accessibilityState={{ expanded: techExpanded }}>
          <Text style={styles.techToggle}>Technical details {techExpanded ? "▲" : "▼"}</Text>
        </Pressable>
        {techExpanded ? (
          <View testID="asset-tech-details" style={styles.techGroup}>
            <Text style={styles.metaLabel}>Asset ID</Text>
            <Text style={styles.metaValue} numberOfLines={1} ellipsizeMode="tail">{asset.id}</Text>
            <Text style={styles.metaLabel}>Uploader ID</Text>
            <Text style={styles.metaValue} numberOfLines={1} ellipsizeMode="tail">{asset.uploader_id}</Text>
          </View>
        ) : null}
      </Panel>
      {asset.status === "finalized" ? (
        <Panel>
          <Text style={styles.panelTitle}>Playback</Text>
          <PlaybackStateView state={playbackState} testIdPrefix="asset-playback" testIdPlayer="asset-inline-player" onRetry={onLoadPlayback} />
        </Panel>
      ) : null}
      <Panel>
        <Text style={styles.panelTitle}>Compliance and consent</Text>
        <Text style={styles.panelCopy}>Review the immutable audit trail, rights evidence, and voice consent ledger.</Text>
        <Button testID="asset-open-compliance" label="Open compliance center" onPress={onOpenCompliance} fullWidth />
      </Panel>
    </>
  );
}

function assetScreenPadding(viewState: AssetDetailViewState, actionBarHeight: number): number {
  return viewState.kind === "ready" && viewState.asset.status === "finalized" ? actionBarHeight : 0;
}

export function AssetDetailScreen({ assetId, gatewayBaseUrl, onOpenCompliance }: AssetDetailScreenProps) {
  const { viewState, playbackState, loadPlayback } = useAssetDetail(assetId, gatewayBaseUrl);
  const actionBarHeight = ACTION_BAR_CONTENT_HEIGHT + space.md * 2;
  const isFinalized = viewState.kind === "ready" && viewState.asset.status === "finalized";

  return (
    <View style={styles.container}>
      <Screen testID="asset-detail-screen" scroll extraBottomPadding={assetScreenPadding(viewState, actionBarHeight)}>
        <ScreenHeader kicker="Asset" title="Asset detail" />
        {viewState.kind === "loading" ? <StateView kind="loading" title="Loading asset detail…" /> : null}
        {viewState.kind === "error" ? <StateView kind="error" title="Could not load asset detail" message={viewState.message} /> : null}
        {viewState.kind === "not_available" ? <StateView kind="empty" title="Asset detail not available yet" message="This asset detail surface is not available on the current backend." /> : null}
        {viewState.kind === "ready" ? (
          <AssetReadyContent asset={viewState.asset} playbackState={playbackState} onLoadPlayback={() => void loadPlayback(viewState.asset)} onOpenCompliance={onOpenCompliance} />
        ) : null}
      </Screen>
      {isFinalized && viewState.kind === "ready" ? (
        <ActionBar>
          <Button testID="asset-play-button" label="Play" onPress={() => void loadPlayback(viewState.asset)} fullWidth loading={playbackState.kind === "loading"} disabled={playbackState.kind === "loading"} />
        </ActionBar>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: color.canvas },
  assetTitle: { ...type.title, color: color.ink900 },
  techToggle: { ...type.label, color: color.primary, marginTop: space.xs },
  techGroup: { gap: space.xs, marginTop: space.xs },
  metaLabel: { ...type.label, color: color.ink400 },
  metaValue: { ...type.meta, color: color.ink700 },
  panelTitle: { ...type.heading, color: color.ink900 },
  panelCopy: { ...type.body, color: color.ink500 },
});
