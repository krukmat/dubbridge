import { useEffect, useEffectEvent, useState } from "react";
import { StyleSheet, Text, View } from "react-native";

import { createGatewayClient } from "../api/client";
import { buildManifestUrl, issuePlaybackGrant } from "../api/playback";
import { useAuth } from "../auth/AuthProvider";
import { Badge, statusTone } from "../components/Badge";
import { Button } from "../components/Button";
import { Panel } from "../components/Panel";
import { Screen } from "../components/Screen";
import { ScreenHeader } from "../components/ScreenHeader";
import { StateView } from "../components/StateView";
import { VideoPlayer } from "../components/VideoPlayer";
import { color, space, type } from "../theme";
import type { AssetSummary } from "./AssetListScreen";

type AssetDetailScreenProps = {
  assetId: string;
  gatewayBaseUrl: string;
  onOpenCompliance: () => void;
};

type AssetDetailViewState =
  | { kind: "loading" }
  | { kind: "ready"; asset: AssetSummary }
  | { kind: "error"; message: string }
  | { kind: "not_available" };

type AssetPlaybackState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; source: string }
  | { kind: "not_ready" }
  | { kind: "error"; message: string };

function formatStatus(status: string): string {
  return status
    .split("_")
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");
}

export function AssetDetailScreen({
  assetId,
  gatewayBaseUrl,
  onOpenCompliance,
}: AssetDetailScreenProps) {
  const auth = useAuth();
  const handleLogout = useEffectEvent(async () => {
    await auth.logout();
  });
  const handleSessionRotation = useEffectEvent(async (rotation: string | null) => {
    await auth.onSessionRotation(rotation);
  });
  const [viewState, setViewState] = useState<AssetDetailViewState>({
    kind: "loading",
  });
  const [playbackState, setPlaybackState] = useState<AssetPlaybackState>({
    kind: "idle",
  });

  useEffect(() => {
    setPlaybackState({ kind: "idle" });
  }, [assetId]);

  useEffect(() => {
    let isActive = true;

    async function loadAssetDetail(): Promise<void> {
      setViewState({ kind: "loading" });

      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await client.get<AssetSummary>(
        `/api/assets/${assetId}`,
        auth.sessionRef,
      );

      if (!isActive) {
        return;
      }

      if (!result.ok) {
        if (result.error.kind === "session_expired") {
          await handleLogout();
          return;
        }

        if (result.error.kind === "http" && result.error.status === 404) {
          setViewState({ kind: "not_available" });
          return;
        }

        const message =
          result.error.kind === "network"
            ? result.error.message
            : result.error.kind === "forbidden"
              ? "You do not have access to this asset."
              : `Gateway request failed with status ${result.error.status}.`;
        setViewState({ kind: "error", message });
        return;
      }

      await handleSessionRotation(result.value.sessionRotation);

      if (!isActive) {
        return;
      }

      setViewState({ kind: "ready", asset: result.value.data });
    }

    void loadAssetDetail();

    return () => {
      isActive = false;
    };
  }, [assetId, gatewayBaseUrl]);

  async function loadPlayback(asset: AssetSummary): Promise<void> {
    if (playbackState.kind === "loading") {
      return;
    }

    setPlaybackState({ kind: "loading" });

    const client = createGatewayClient({ gatewayBaseUrl });
    const result = await issuePlaybackGrant(client, auth.sessionRef, asset.id);

    if (!result.ok) {
      if (result.error.kind === "session_expired") {
        await handleLogout();
        return;
      }

      if (result.error.kind === "http" && (result.error.status === 409 || result.error.status === 422)) {
        setPlaybackState({ kind: "not_ready" });
        return;
      }

      const message =
        result.error.kind === "forbidden"
          ? "You do not have access to this playback stream."
          : result.error.kind === "network"
            ? result.error.message
            : `Could not load playback (${result.error.status}).`;
      setPlaybackState({ kind: "error", message });
      return;
    }

    await handleSessionRotation(result.value.sessionRotation);
    setPlaybackState({
      kind: "ready",
      source: buildManifestUrl(gatewayBaseUrl, asset.id, result.value.data.grantId),
    });
  }

  return (
    <Screen testID="asset-detail-screen" scroll edges={["bottom"]}>
      <ScreenHeader kicker="Asset" title="Asset detail" />

      {viewState.kind === "loading" ? (
        <StateView kind="loading" title="Loading asset detail…" />
      ) : null}

      {viewState.kind === "error" ? (
        <StateView
          kind="error"
          title="Could not load asset detail"
          message={viewState.message}
        />
      ) : null}

      {viewState.kind === "not_available" ? (
        <StateView
          kind="empty"
          title="Asset detail not available yet"
          message="This asset detail surface is not available on the current backend."
        />
      ) : null}

      {viewState.kind === "ready" ? (
        <>
          <Panel>
            <Text style={styles.assetTitle}>{viewState.asset.title}</Text>
            <Text style={styles.metaLabel}>Status</Text>
            <Badge
              label={formatStatus(viewState.asset.status)}
              tone={statusTone(viewState.asset.status)}
            />
            <Text style={styles.metaLabel}>Asset ID</Text>
            <Text style={styles.metaValue}>{viewState.asset.id}</Text>
            <Text style={styles.metaLabel}>Uploader ID</Text>
            <Text style={styles.metaValue}>{viewState.asset.uploader_id}</Text>
          </Panel>

          {viewState.asset.status === "finalized" ? (
            <Panel>
              <Text style={styles.panelTitle}>Playback</Text>
              <Text style={styles.panelCopy}>
                Open the original track inline when playback is available.
              </Text>
              <Button
                testID="asset-play-button"
                label="Play"
                onPress={() => void loadPlayback(viewState.asset)}
                fullWidth
                loading={playbackState.kind === "loading"}
                disabled={playbackState.kind === "loading"}
              />

              {playbackState.kind === "loading" ? (
                <View style={styles.playbackSurface}>
                  <StateView
                    testID="asset-playback-loading"
                    kind="loading"
                    title="Loading playback…"
                    message="Preparing the original track."
                  />
                </View>
              ) : null}

              {playbackState.kind === "not_ready" ? (
                <View style={styles.playbackSurface}>
                  <StateView
                    testID="asset-playback-empty"
                    kind="empty"
                    title="Media not ready yet"
                    message="Playback is not available for this asset yet."
                  />
                </View>
              ) : null}

              {playbackState.kind === "error" ? (
                <View style={styles.playbackSurface}>
                  <StateView
                    testID="asset-playback-error"
                    kind="error"
                    title="Could not load playback"
                    message={playbackState.message}
                    onRetry={() => void loadPlayback(viewState.asset)}
                  />
                </View>
              ) : null}

              {playbackState.kind === "ready" ? (
                <VideoPlayer
                  testID="asset-inline-player"
                  source={playbackState.source}
                  onRetry={() => void loadPlayback(viewState.asset)}
                />
              ) : null}
            </Panel>
          ) : null}

          <Panel>
            <Text style={styles.panelTitle}>Compliance and consent</Text>
            <Text style={styles.panelCopy}>
              Review the immutable audit trail, rights evidence, and voice consent ledger.
            </Text>
            <Button
              testID="asset-open-compliance"
              label="Open compliance center"
              onPress={onOpenCompliance}
              fullWidth
            />
          </Panel>
        </>
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  assetTitle: { ...type.title, color: color.ink900 },
  metaLabel: { ...type.label, color: color.ink400 },
  metaValue: { ...type.meta, color: color.ink700 },
  panelTitle: { ...type.heading, color: color.ink900 },
  panelCopy: { ...type.body, color: color.ink500 },
  playbackSurface: { minHeight: 220, marginTop: space.md },
});
