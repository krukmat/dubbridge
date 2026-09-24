import {
  createSyncSnapshot,
  transitionSync,
  type P2pSyncSnapshot,
} from "../sync/SyncState";

const FIXTURE_TIME = 1_790_236_800_000;

function identity(accountScope: string, key: string) {
  return {
    accountScope,
    publicationId: `pub-p2p-${key}`,
    lineageId: `line-p2p-${key}`,
  };
}

function buildSyncing(accountScope: string): P2pSyncSnapshot {
  const idle = createSyncSnapshot(identity(accountScope, "syncing"), FIXTURE_TIME);
  const discovering = transitionSync(idle, "DISCOVERING", {}, FIXTURE_TIME + 1);
  return transitionSync(
    discovering,
    "DOWNLOADING",
    {
      progress: {
        filesCompleted: 1,
        totalFiles: 3,
        bytesCompleted: 1024,
        totalBytes: 4096,
      },
    },
    FIXTURE_TIME + 2,
  );
}

function buildFailed(accountScope: string): P2pSyncSnapshot {
  const idle = createSyncSnapshot(identity(accountScope, "sync-error"), FIXTURE_TIME);
  const discovering = transitionSync(idle, "DISCOVERING", {}, FIXTURE_TIME + 1);
  return transitionSync(
    discovering,
    "FAILED",
    { lastError: "visual_fixture_failure" },
    FIXTURE_TIME + 2,
  );
}

function buildAvailable(accountScope: string): P2pSyncSnapshot {
  const progress = {
    filesCompleted: 3,
    totalFiles: 3,
    bytesCompleted: 4096,
    totalBytes: 4096,
  };
  const idle = createSyncSnapshot(identity(accountScope, "available"), FIXTURE_TIME);
  const discovering = transitionSync(idle, "DISCOVERING", {}, FIXTURE_TIME + 1);
  const downloading = transitionSync(
    discovering,
    "DOWNLOADING",
    { progress },
    FIXTURE_TIME + 2,
  );
  const verifying = transitionSync(
    downloading,
    "VERIFYING",
    { progress },
    FIXTURE_TIME + 3,
  );
  return transitionSync(
    verifying,
    "READY",
    { progress, manifestVerified: true, packageVerified: true },
    FIXTURE_TIME + 4,
  );
}

export function buildP2PVisualFixtureSnapshots(
  accountScope: string,
): readonly P2pSyncSnapshot[] {
  return [
    buildSyncing(accountScope),
    buildFailed(accountScope),
    buildAvailable(accountScope),
  ];
}
