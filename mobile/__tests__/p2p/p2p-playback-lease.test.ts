import { createP2PPlaybackLease } from "../../src/p2p/playback/P2PPlaybackLease";

describe("P5 playback lease", () => {
  it("shares one stop operation across concurrent terminal paths", async () => {
    let resolveStop: (() => void) | undefined;
    const stop = jest.fn(
      () => new Promise<void>((resolve) => {
        resolveStop = resolve;
      }),
    );
    const lease = createP2PPlaybackLease({ stop });

    const first = lease.release();
    const second = lease.release();

    expect(first).toBe(second);
    expect(stop).toHaveBeenCalledTimes(1);

    resolveStop?.();
    await first;
  });

  it("waits for teardown before retrying", async () => {
    let resolveStop: (() => void) | undefined;
    const stop = jest.fn(
      () => new Promise<void>((resolve) => {
        resolveStop = resolve;
      }),
    );
    const retry = jest.fn();
    const lease = createP2PPlaybackLease({ stop });

    const retryOperation = lease.retryAfterRelease(retry);
    expect(stop).toHaveBeenCalledTimes(1);
    expect(retry).not.toHaveBeenCalled();

    resolveStop?.();
    await retryOperation;

    expect(retry).toHaveBeenCalledTimes(1);
    expect(stop).toHaveBeenCalledTimes(1);
  });

  it("keeps retry fail-closed when teardown rejects", async () => {
    const stop = jest.fn(async () => {
      throw new Error("stop failed");
    });
    const retry = jest.fn();
    const lease = createP2PPlaybackLease({ stop });

    await expect(lease.retryAfterRelease(retry)).rejects.toThrow("stop failed");
    expect(retry).not.toHaveBeenCalled();
    expect(stop).toHaveBeenCalledTimes(1);
  });
});
