import { useCallback, useEffect, useMemo } from "react";

import { VideoPlayer, type VideoPlayerProps } from "../../components/VideoPlayer";
import type { P2PPlaybackController, P2PPlaybackSession } from "./P2PPlaybackController";

type PlaybackStopper = Pick<P2PPlaybackController, "stop">;
type P2PVideoPlayerProps = Omit<VideoPlayerProps, "source" | "onRetry" | "onPlaybackError">;

export type P2PPlaybackSessionViewProps = P2PVideoPlayerProps & {
  session: P2PPlaybackSession;
  controller: PlaybackStopper;
  onRetry?: () => void;
};

type ReleaseSession = () => Promise<void>;

function createReleaseSession(controller: PlaybackStopper): ReleaseSession {
  let releaseOperation: Promise<void> | null = null;
  return () => {
    if (releaseOperation === null) releaseOperation = controller.stop();
    return releaseOperation;
  };
}

/**
 * Owns one P5 loopback playback lease and connects it to the existing player.
 * Teardown is intentionally idempotent so player error, retry and React unmount
 * can race without issuing duplicate stop commands or extending CK lifetime.
 */
export function P2PPlaybackSessionView({
  session,
  controller,
  onRetry,
  ...playerProps
}: P2PPlaybackSessionViewProps) {
  const releaseSession = useMemo(
    () => createReleaseSession(controller),
    [controller, session.playbackUrl],
  );

  useEffect(() => () => {
    void releaseSession().catch(() => undefined);
  }, [releaseSession]);

  const handlePlaybackError = useCallback(() => {
    void releaseSession().catch(() => undefined);
  }, [releaseSession]);

  const handleRetry = useCallback(() => {
    if (!onRetry) return;
    void releaseSession().then(onRetry, onRetry);
  }, [onRetry, releaseSession]);

  return (
    <VideoPlayer
      {...playerProps}
      source={session.playbackUrl}
      onPlaybackError={handlePlaybackError}
      onRetry={onRetry ? handleRetry : undefined}
    />
  );
}
