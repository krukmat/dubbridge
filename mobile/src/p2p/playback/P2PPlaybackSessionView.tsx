import { useCallback, useEffect, useMemo } from "react";

import { VideoPlayer, type VideoPlayerProps } from "../../components/VideoPlayer";
import type { P2PPlaybackController, P2PPlaybackSession } from "./P2PPlaybackController";
import { createP2PPlaybackLease } from "./P2PPlaybackLease";

type PlaybackStopper = Pick<P2PPlaybackController, "stop">;
type P2PVideoPlayerProps = Omit<VideoPlayerProps, "source" | "onRetry" | "onPlaybackError">;

export type P2PPlaybackSessionViewProps = P2PVideoPlayerProps & {
  session: P2PPlaybackSession;
  controller: PlaybackStopper;
  onRetry?: () => void;
};

/**
 * Owns one P5 loopback playback lease and connects it to the existing player.
 * Player error, retry and React unmount all converge on the same idempotent
 * release operation; retry stays fail-closed if teardown cannot complete.
 */
export function P2PPlaybackSessionView({
  session,
  controller,
  onRetry,
  ...playerProps
}: P2PPlaybackSessionViewProps) {
  const lease = useMemo(
    () => createP2PPlaybackLease(controller),
    [controller, session.playbackUrl],
  );

  useEffect(() => () => {
    void lease.release().catch(() => undefined);
  }, [lease]);

  const handlePlaybackError = useCallback(() => {
    void lease.release().catch(() => undefined);
  }, [lease]);

  const handleRetry = useCallback(() => {
    if (!onRetry) return;
    void lease.retryAfterRelease(onRetry).catch(() => undefined);
  }, [lease, onRetry]);

  return (
    <VideoPlayer
      {...playerProps}
      source={session.playbackUrl}
      onPlaybackError={handlePlaybackError}
      onRetry={onRetry ? handleRetry : undefined}
    />
  );
}
