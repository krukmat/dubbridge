import {
  createSyncSnapshot,
  packageCacheKey,
  restoreSyncSnapshot,
  transitionSync,
} from "../src/p2p/sync/SyncState";

const identity = {
  accountScope: "viewer-1",
  publicationId: "pub-1",
  lineageId: "lineage-1",
} as const;

describe("P2P product sync lifecycle", () => {
  it("reaches READY only after complete package verification", () => {
    let state = createSyncSnapshot(identity, 1);
    state = transitionSync(state, "DISCOVERING", {}, 2);
    state = transitionSync(
      state,
      "DOWNLOADING",
      {
        progress: { filesCompleted: 1, totalFiles: 3, bytesCompleted: 10, totalBytes: 30 },
      },
      3,
    );
    state = transitionSync(
      state,
      "VERIFYING",
      {
        progress: { filesCompleted: 3, totalFiles: 3, bytesCompleted: 30, totalBytes: 30 },
        manifestVerified: true,
      },
      4,
    );
    state = transitionSync(state, "READY", { packageVerified: true }, 5);

    expect(state.phase).toBe("READY");
    expect(state.packageVerified).toBe(true);
  });

  it("rejects false READY when package verification is incomplete", () => {
    let state = createSyncSnapshot(identity);
    state = transitionSync(state, "DISCOVERING");
    state = transitionSync(state, "DOWNLOADING", {
      progress: { filesCompleted: 1, totalFiles: 2, bytesCompleted: 10, totalBytes: 20 },
    });
    state = transitionSync(state, "VERIFYING", { manifestVerified: true });

    expect(() => transitionSync(state, "READY", { packageVerified: true })).toThrow(
      "cannot become READY",
    );
  });

  it("rejects illegal lifecycle jumps", () => {
    const state = createSyncSnapshot(identity);
    expect(() => transitionSync(state, "READY")).toThrow("Invalid P2P sync transition");
  });

  it("restores a retryable snapshot only for the same account publication lineage", () => {
    let state = createSyncSnapshot(identity, 1);
    state = transitionSync(state, "DISCOVERING", {}, 2);
    state = transitionSync(
      state,
      "DOWNLOADING",
      {
        progress: { filesCompleted: 1, totalFiles: 4, bytesCompleted: 100, totalBytes: 400 },
      },
      3,
    );

    expect(restoreSyncSnapshot(JSON.parse(JSON.stringify(state)), identity)).toEqual(state);
    expect(() => restoreSyncSnapshot(state, { ...identity, lineageId: "other" })).toThrow(
      "does not match",
    );
    expect(() => restoreSyncSnapshot(state, { ...identity, accountScope: "viewer-2" })).toThrow(
      "does not match",
    );
  });

  it("keys cache by exact account, publication and lineage", () => {
    expect(packageCacheKey(identity)).toBe("viewer-1/pub-1/lineage-1");
    expect(packageCacheKey({ ...identity, accountScope: "viewer-2" })).toBe(
      "viewer-2/pub-1/lineage-1",
    );
    expect(() => packageCacheKey({ ...identity, publicationId: "../escape" })).toThrow(
      "Invalid P2P sync identity",
    );
    expect(() => packageCacheKey({ ...identity, accountScope: "../viewer" })).toThrow(
      "Invalid P2P sync identity",
    );
  });
});
