import { useEffect, useEffectEvent, useState } from "react";

import { createGatewayClient } from "../api/client";
import { buildManifestUrl, issuePlaybackGrant, resolvePlaybackErrorMessage } from "../api/playback";
import { useAuth } from "../auth/AuthProvider";
import type { PlaybackViewState } from "../components/PlaybackStateView";

type UsePlaybackLoaderOptions = {
  assetId: string;
  gatewayBaseUrl: string;
  attempt: number;
};

export function usePlaybackLoader({ assetId, gatewayBaseUrl, attempt }: UsePlaybackLoaderOptions): PlaybackViewState {
  const auth = useAuth();
  const [state, setState] = useState<PlaybackViewState>({ kind: "loading" });

  const handleLogout = useEffectEvent(async () => {
    await auth.logout();
  });
  const handleSessionRotation = useEffectEvent(async (rotation: string | null) => {
    await auth.onSessionRotation(rotation);
  });

  useEffect(() => {
    let isActive = true;

    async function load(): Promise<void> {
      setState({ kind: "loading" });
      const client = createGatewayClient({ gatewayBaseUrl });
      const result = await issuePlaybackGrant(client, auth.sessionRef, assetId);

      if (!isActive) return;

      if (!result.ok) {
        if (result.error.kind === "session_expired") {
          await handleLogout();
          return;
        }
        if (result.error.kind === "http" && (result.error.status === 409 || result.error.status === 422)) {
          setState({ kind: "not_ready" });
          return;
        }
        setState({ kind: "error", message: resolvePlaybackErrorMessage(result.error) });
        return;
      }

      await handleSessionRotation(result.value.sessionRotation);
      if (!isActive) return;

      setState({
        kind: "ready",
        source: buildManifestUrl(gatewayBaseUrl, assetId, result.value.data.grantId),
      });
    }

    void load();
    return () => { isActive = false; };
  }, [auth.sessionRef, gatewayBaseUrl, assetId, attempt]);

  return state;
}
