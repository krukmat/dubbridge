import { useEffect, useEffectEvent, useState } from "react";

import { createGatewayClient } from "../api/client";
import { buildManifestUrl, issuePlaybackGrant, resolvePlaybackErrorMessage } from "../api/playback";
import { useAuth } from "../auth/AuthProvider";
import { type PlaybackViewState } from "../components/PlaybackStateView";
import type { AssetSummary } from "./AssetListScreen";

export type AssetDetailViewState =
  | { kind: "loading" }
  | { kind: "ready"; asset: AssetSummary }
  | { kind: "error"; message: string }
  | { kind: "not_available" };

type GatewayClient = ReturnType<typeof createGatewayClient>;
type Auth = ReturnType<typeof useAuth>;

function assetLoadErrorMessage(error: { kind: string; message?: string; status?: number }): string {
  if (error.kind === "network") return error.message ?? "Network error.";
  if (error.kind === "forbidden") return "You do not have access to this asset.";
  return `Gateway request failed with status ${error.status}.`;
}

async function fetchAsset(
  client: GatewayClient,
  auth: Auth,
  assetId: string,
  onLogout: () => Promise<void>,
  onRotation: (r: string | null) => Promise<void>,
): Promise<AssetDetailViewState> {
  const result = await client.get<AssetSummary>(`/api/assets/${assetId}`, auth.sessionRef);
  if (!result.ok) {
    if (result.error.kind === "session_expired") { await onLogout(); return { kind: "loading" }; }
    if (result.error.kind === "http" && result.error.status === 404) return { kind: "not_available" };
    return { kind: "error", message: assetLoadErrorMessage(result.error) };
  }
  await onRotation(result.value.sessionRotation);
  return { kind: "ready", asset: result.value.data };
}

async function fetchPlayback(
  client: GatewayClient,
  auth: Auth,
  gatewayBaseUrl: string,
  asset: AssetSummary,
  onLogout: () => Promise<void>,
  onRotation: (r: string | null) => Promise<void>,
): Promise<PlaybackViewState> {
  const result = await issuePlaybackGrant(client, auth.sessionRef, asset.id);
  if (!result.ok) {
    if (result.error.kind === "session_expired") { await onLogout(); return { kind: "idle" }; }
    if (result.error.kind === "http" && (result.error.status === 409 || result.error.status === 422)) return { kind: "not_ready" };
    return { kind: "error", message: resolvePlaybackErrorMessage(result.error) };
  }
  await onRotation(result.value.sessionRotation);
  return { kind: "ready", source: buildManifestUrl(gatewayBaseUrl, asset.id, result.value.data.grantId) };
}

export function useAssetDetail(assetId: string, gatewayBaseUrl: string) {
  const auth = useAuth();
  const onLogout = useEffectEvent(async () => { await auth.logout(); });
  const onRotation = useEffectEvent(async (r: string | null) => { await auth.onSessionRotation(r); });

  const [viewState, setViewState] = useState<AssetDetailViewState>({ kind: "loading" });
  const [playbackState, setPlaybackState] = useState<PlaybackViewState>({ kind: "idle" });

  useEffect(() => { setPlaybackState({ kind: "idle" }); }, [assetId]);

  useEffect(() => {
    let isActive = true;
    const client = createGatewayClient({ gatewayBaseUrl });
    fetchAsset(client, auth, assetId, onLogout, onRotation).then((state) => {
      if (isActive) setViewState(state);
    });
    return () => { isActive = false; };
  }, [assetId, gatewayBaseUrl]);

  async function loadPlayback(asset: AssetSummary): Promise<void> {
    if (playbackState.kind === "loading") return;
    setPlaybackState({ kind: "loading" });
    const client = createGatewayClient({ gatewayBaseUrl });
    const state = await fetchPlayback(client, auth, gatewayBaseUrl, asset, onLogout, onRotation);
    setPlaybackState(state);
  }

  return { viewState, playbackState, loadPlayback };
}
