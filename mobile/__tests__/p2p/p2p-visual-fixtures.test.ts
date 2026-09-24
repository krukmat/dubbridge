import { buildP2PVisualFixtureSnapshots } from "../../src/p2p/development/P2PVisualFixtures";

describe("P2P visual fixture snapshots", () => {
  it("seeds only deterministic non-secret lifecycle states for the visual account", () => {
    const snapshots = buildP2PVisualFixtureSnapshots("viewer-visual");
    expect(snapshots.map((snapshot) => snapshot.phase)).toEqual([
      "DOWNLOADING",
      "FAILED",
      "READY",
    ]);
    expect(snapshots.map((snapshot) => snapshot.identity.publicationId)).toEqual([
      "pub-p2p-syncing",
      "pub-p2p-sync-error",
      "pub-p2p-available",
    ]);

    const ready = snapshots[2];
    expect(ready.manifestVerified).toBe(true);
    expect(ready.packageVerified).toBe(true);
    expect(ready.progress).toEqual({
      filesCompleted: 3,
      totalFiles: 3,
      bytesCompleted: 4096,
      totalBytes: 4096,
    });
  });
});
