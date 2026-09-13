// Real PublicationExecutor wiring (P2.T3c-S4-d3): composes independent
// package verification (S4-d1/d2), the decide-and-persist index policy
// (S2b), the per-publication_id concurrency lock (S4-b), and the persistent
// Hyperdrive open/reopen lifecycle (S4-c) into the executor `server.ts`
// dispatches into. Does not implement Hyperswarm announce/join/flush
// (S4-e, out of scope) — a drive is opened and its content written, but not
// yet advertised on the network.

import { randomUUID } from "node:crypto";
import type { PublicationRequest, PublicationHttpResponse } from "./contract.js";
import { PublicationContractError } from "./contract.js";
import type { PublicationRecord } from "./publication_record.js";
import { withPublicationLock } from "./publication_lock.js";
import { openDrive, closeDrive, flushDrive, type OpenDriveResult } from "./hyperdrive_store.js";
import { decideAndPersist } from "./publication_index.js";
import { indexEntryPath, readIndexEntry } from "./publication_index_io.js";
import { verifyPackage, type PackageVerificationResult } from "./package_verification.js";

export interface PublicationExecutorConfig {
  readonly packageRoot: string;
  readonly driveStorageRoot: string;
  readonly indexRoot: string;
}

/**
 * Builds the real `PublicationExecutor` used by
 * `createPrivatePublicationServer` in production. Every request for a given
 * `publication_id` is serialized through `withPublicationLock` so a
 * concurrent duplicate tuple cannot race the index decision or the drive
 * write.
 */
export function createPublicationExecutor(
  config: PublicationExecutorConfig
): (request: PublicationRequest) => Promise<PublicationHttpResponse> {
  return (request: PublicationRequest) =>
    withPublicationLock(request.publication_id, () => execute(config, request));
}

async function execute(
  config: PublicationExecutorConfig,
  request: PublicationRequest
): Promise<PublicationHttpResponse> {
  const verification = await verifyPackage(
    config.packageRoot,
    request.package_ref,
    request.lineage_id,
    request.manifest_digest_sha256
  );

  if (!verification.ok) {
    throw new PublicationContractError(verification.reason);
  }

  const entryPath = indexEntryPath(config.indexRoot, request.publication_id);
  const existing = await readIndexEntry(entryPath);

  if (existing !== null) {
    if (
      existing.lineage_id !== request.lineage_id ||
      existing.manifest_digest_sha256 !== request.manifest_digest_sha256
    ) {
      throw new PublicationContractError("publication_conflict");
    }
    // Replay of an already-accepted tuple: return the stable persisted
    // record without touching the drive again (criterion 2 — no second
    // drive, no identity rewrite). Still open/close the drive as a
    // liveness probe of the underlying storage.
    const opened = await openDrive(config.driveStorageRoot, request.publication_id);
    await closeDrive(opened);
    return { status: 200, evidence: existing };
  }

  // First-time acceptance: write the drive's content BEFORE persisting the
  // index record, so a write/open failure never leaves a committed success
  // record with no corresponding drive content (criterion 5). The bytes
  // written here are exactly the ones verifyPackage already hashed above —
  // never re-read from disk — so nothing can swap the package's content
  // between verification and publication (TOCTOU).
  if (request.publication_id !== verification.manifest.publication_id) {
    throw new PublicationContractError("publication_conflict");
  }

  const opened = await openDrive(config.driveStorageRoot, request.publication_id);

  const candidateRecord: PublicationRecord = {
    contract_version: request.contract_version,
    publication_id: request.publication_id,
    lineage_id: request.lineage_id,
    manifest_digest_sha256: request.manifest_digest_sha256,
    external_publication_id: opened.publicKeyHex,
    evidence_id: randomUUID(),
    confirmed_at: new Date().toISOString(),
  };

  try {
    await writeVerifiedPackageIntoDrive(opened, verification);
  } finally {
    // Flush (not merely close) so pending writes are durably committed to
    // disk before decideAndPersist can commit a success record — a flush
    // failure propagates here and never reaches the index (criterion 5).
    await flushDrive(opened);
  }

  const outcome = await decideAndPersist(config.indexRoot, candidateRecord);
  if (outcome.kind === "conflict") {
    // Another call for the same publication_id won the race to persist
    // first (should not happen under withPublicationLock, but stay
    // fail-closed rather than silently reporting our own record as
    // accepted if it does).
    throw new PublicationContractError("publication_conflict");
  }

  return { status: 201, evidence: candidateRecord };
}

async function writeVerifiedPackageIntoDrive(
  opened: OpenDriveResult,
  verification: Extract<PackageVerificationResult, { ok: true }>
): Promise<void> {
  await opened.drive.put("/manifest.json", verification.manifestBytes);
  for (const file of verification.files) {
    await opened.drive.put(`/${file.path}`, file.bytes);
  }
}
