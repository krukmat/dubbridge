export type P2PPlaybackStopper = Readonly<{
  stop: () => Promise<void>;
}>;

export type P2PPlaybackLease = Readonly<{
  release: () => Promise<void>;
  retryAfterRelease: (retry: () => void) => Promise<void>;
}>;

/**
 * Idempotent owner for a single P5 playback session. Every terminal path shares
 * the same stop operation, so concurrent error/retry/unmount paths cannot extend
 * the loopback listener or transient CK lifetime.
 */
export function createP2PPlaybackLease(
  controller: P2PPlaybackStopper,
): P2PPlaybackLease {
  let releaseOperation: Promise<void> | null = null;

  const release = () => {
    if (releaseOperation === null) releaseOperation = controller.stop();
    return releaseOperation;
  };

  return {
    release,
    retryAfterRelease: async (retry) => {
      await release();
      retry();
    },
  };
}
