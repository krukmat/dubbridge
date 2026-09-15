import type { P2pReadyDescriptor } from "../../src/api/p2p";
import { createVerifiedPackageHandle } from "../../src/p2p/sync/VerifiedPackageHandle";
import {
  createSyncSnapshot,
  transitionSync,
  type P2pSyncSnapshot,
} from "../../src/p2p/sync/SyncState";

const descriptor: P2pReadyDescriptor = {
  descriptorVersion: "p2p-ready-descriptor-v1",
  assetId: "asset-1",
  publicationId: "pub-1",
  lineageId: "lineage-1",
  manifestVersion: "p2p-manifest-v1",
  manifestDigestSha256: "a".repeat(64),
  externalPublicationId: "b".repeat(64),
  ckWrapRef: "p2p-k1-wrap/pub-1/lineage-1",
  kekId: "kek-1",
  kekVersion: 1,
  readyAt: "2026-09-15T00:00:00Z",
};

function readySnapshot(): P2pSyncSnapshot {
  let state = createSyncSnapshot({
    accountScope: "viewer-1",
    publicationId: "pub-1",
    lineageId: "lineage-1",
  });
  state = transitionSync(state, "DISCOVERING");
  state = transitionSync(state, "DOWNLOADING", {
    manifestVerified: true,
    progress: { filesCompleted: 3, totalFiles: 3, bytesCompleted: 30, totalBytes: 30 },
  });
  state = transitionSync(state, "VERIFYING");
  return transitionSync(state, "READY", { packageVerified: true });
}

describe("P4 verified package handoff", () => {
  it("creates the P5 handle only from verified READY for the exact lineage", () => {
    expect(createVerifiedPackageHandle(descriptor, readySnapshot())).toEqual({
      accountScope: "viewer-1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
      manifestDigestSha256: "a".repeat(64),
    });
  });

  it("does not treat transport/download success as playback readiness", () => {
    let state = createSyncSnapshot({
      accountScope: "viewer-1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
    });
    state = transitionSync(state, "DISCOVERING");
    state = transitionSync(state, "DOWNLOADING", {
      manifestVerified: true,
      progress: { filesCompleted: 3, totalFiles: 3, bytesCompleted: 30, totalBytes: 30 },
    });

    expect(() => createVerifiedPackageHandle(descriptor, state)).toThrow("not verified READY");
  });

  it("rejects a READY snapshot from another lineage", () => {
    const state = readySnapshot();
    expect(() => createVerifiedPackageHandle({ ...descriptor, lineageId: "lineage-2" }, state)).toThrow(
      "does not match authoritative descriptor",
    );
  });
});
