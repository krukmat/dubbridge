export type P2pSyncPhase =
  | "IDLE"
  | "DISCOVERING"
  | "DOWNLOADING"
  | "VERIFYING"
  | "READY"
  | "RETRYING"
  | "CANCELLED"
  | "FAILED";

export type P2pSyncIdentity = Readonly<{
  accountScope: string;
  publicationId: string;
  lineageId: string;
}>;

export type P2pSyncProgress = Readonly<{
  filesCompleted: number;
  totalFiles: number;
  bytesCompleted: number;
  totalBytes: number;
}>;

export type P2pSyncSnapshot = Readonly<{
  schemaVersion: 1;
  identity: P2pSyncIdentity;
  phase: P2pSyncPhase;
  progress: P2pSyncProgress;
  manifestVerified: boolean;
  packageVerified: boolean;
  updatedAtUnixMs: number;
  lastError: string | null;
}>;

const TRANSITIONS: Readonly<Record<P2pSyncPhase, readonly P2pSyncPhase[]>> = {
  IDLE: ["DISCOVERING", "CANCELLED", "FAILED"],
  DISCOVERING: ["DOWNLOADING", "RETRYING", "CANCELLED", "FAILED"],
  DOWNLOADING: ["VERIFYING", "RETRYING", "CANCELLED", "FAILED"],
  VERIFYING: ["READY", "RETRYING", "CANCELLED", "FAILED"],
  READY: ["DISCOVERING"],
  RETRYING: ["DISCOVERING", "CANCELLED", "FAILED"],
  CANCELLED: ["DISCOVERING"],
  FAILED: ["DISCOVERING", "CANCELLED"],
};

export class InvalidSyncTransitionError extends Error {
  constructor(from: P2pSyncPhase, to: P2pSyncPhase, reason?: string) {
    super(reason ?? `Invalid P2P sync transition ${from} -> ${to}`);
    this.name = "InvalidSyncTransitionError";
  }
}

export function createSyncSnapshot(
  identity: P2pSyncIdentity,
  nowUnixMs = Date.now(),
): P2pSyncSnapshot {
  validateIdentity(identity);
  return {
    schemaVersion: 1,
    identity,
    phase: "IDLE",
    progress: emptyProgress(),
    manifestVerified: false,
    packageVerified: false,
    updatedAtUnixMs: nowUnixMs,
    lastError: null,
  };
}

export function transitionSync(
  current: P2pSyncSnapshot,
  phase: P2pSyncPhase,
  patch: Partial<Pick<P2pSyncSnapshot, "progress" | "manifestVerified" | "packageVerified" | "lastError">> = {},
  nowUnixMs = Date.now(),
): P2pSyncSnapshot {
  if (current.phase !== phase && !TRANSITIONS[current.phase].includes(phase)) {
    throw new InvalidSyncTransitionError(current.phase, phase);
  }

  const next: P2pSyncSnapshot = {
    ...current,
    ...patch,
    phase,
    updatedAtUnixMs: nowUnixMs,
  };
  validateProgress(next.progress);
  assertReadyInvariant(current.phase, next);
  return next;
}

export function restoreSyncSnapshot(
  value: unknown,
  expectedIdentity: P2pSyncIdentity,
): P2pSyncSnapshot {
  validateIdentity(expectedIdentity);
  const record = parseSnapshotRecord(value);
  const identity = parsePersistedIdentity(record.identity);
  assertIdentityMatches(identity, expectedIdentity);
  const phase = parseSyncPhase(record.phase);
  const snapshot: P2pSyncSnapshot = {
    schemaVersion: 1,
    identity,
    phase,
    progress: parseProgress(record.progress),
    manifestVerified: record.manifestVerified === true,
    packageVerified: record.packageVerified === true,
    updatedAtUnixMs: numberField(record.updatedAtUnixMs),
    lastError: typeof record.lastError === "string" ? record.lastError : null,
  };
  validateProgress(snapshot.progress);
  assertPersistedReadyInvariant(snapshot);
  return snapshot;
}

export function packageCacheKey(identity: P2pSyncIdentity): string {
  validateIdentity(identity);
  return `${identity.accountScope}/${identity.publicationId}/${identity.lineageId}`;
}

export function accountCachePrefix(accountScope: string): string {
  validateSafeSegment(accountScope);
  return `${accountScope}/`;
}

function parseSnapshotRecord(value: unknown): Record<string, unknown> {
  if (!isRecord(value) || value.schemaVersion !== 1 || !isRecord(value.identity)) {
    throw new Error("Invalid persisted P2P sync snapshot");
  }
  return value;
}

function parsePersistedIdentity(value: unknown): P2pSyncIdentity {
  if (!isRecord(value)) throw new Error("Invalid persisted P2P sync snapshot");
  const identity = {
    accountScope: stringField(value.accountScope),
    publicationId: stringField(value.publicationId),
    lineageId: stringField(value.lineageId),
  };
  validateIdentity(identity);
  return identity;
}

function assertIdentityMatches(actual: P2pSyncIdentity, expected: P2pSyncIdentity): void {
  if (
    actual.accountScope !== expected.accountScope ||
    actual.publicationId !== expected.publicationId ||
    actual.lineageId !== expected.lineageId
  ) {
    throw new Error("Persisted P2P sync identity does not match requested account publication lineage");
  }
}

function parseProgress(value: unknown): P2pSyncProgress {
  if (!isRecord(value)) throw new Error("Invalid persisted P2P sync snapshot");
  return {
    filesCompleted: numberField(value.filesCompleted),
    totalFiles: numberField(value.totalFiles),
    bytesCompleted: numberField(value.bytesCompleted),
    totalBytes: numberField(value.totalBytes),
  };
}

function parseSyncPhase(value: unknown): P2pSyncPhase {
  if (!isSyncPhase(value)) throw new Error("Invalid persisted P2P sync snapshot");
  return value;
}

function assertReadyInvariant(from: P2pSyncPhase, snapshot: P2pSyncSnapshot): void {
  if (snapshot.phase !== "READY") return;
  if (snapshot.manifestVerified && snapshot.packageVerified && isProgressComplete(snapshot.progress)) {
    return;
  }
  throw new InvalidSyncTransitionError(
    from,
    snapshot.phase,
    "P2P sync cannot become READY before manifest, package, and progress verification complete",
  );
}

function assertPersistedReadyInvariant(snapshot: P2pSyncSnapshot): void {
  if (snapshot.phase !== "READY") return;
  if (snapshot.manifestVerified && snapshot.packageVerified && isProgressComplete(snapshot.progress)) {
    return;
  }
  throw new Error("Persisted READY P2P sync snapshot is incomplete");
}

function emptyProgress(): P2pSyncProgress {
  return { filesCompleted: 0, totalFiles: 0, bytesCompleted: 0, totalBytes: 0 };
}

function isProgressComplete(progress: P2pSyncProgress): boolean {
  return (
    progress.totalFiles > 0 &&
    progress.filesCompleted === progress.totalFiles &&
    progress.bytesCompleted === progress.totalBytes
  );
}

function validateIdentity(identity: P2pSyncIdentity): void {
  for (const value of [identity.accountScope, identity.publicationId, identity.lineageId]) {
    validateSafeSegment(value);
  }
}

function validateSafeSegment(value: string): void {
  if (!value || value.includes("/") || value.includes("\\") || value === "." || value === "..") {
    throw new Error("Invalid P2P sync identity");
  }
}

function validateProgress(progress: P2pSyncProgress): void {
  const values = [progress.filesCompleted, progress.totalFiles, progress.bytesCompleted, progress.totalBytes];
  if (values.some((value) => !Number.isSafeInteger(value) || value < 0)) {
    throw new Error("Invalid P2P sync progress");
  }
  if (progress.filesCompleted > progress.totalFiles || progress.bytesCompleted > progress.totalBytes) {
    throw new Error("P2P sync progress exceeds declared totals");
  }
}

function isSyncPhase(value: unknown): value is P2pSyncPhase {
  return typeof value === "string" && Object.prototype.hasOwnProperty.call(TRANSITIONS, value);
}

function stringField(value: unknown): string {
  if (typeof value !== "string") throw new Error("Invalid persisted P2P sync snapshot");
  return value;
}

function numberField(value: unknown): number {
  if (typeof value !== "number") throw new Error("Invalid persisted P2P sync snapshot");
  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
