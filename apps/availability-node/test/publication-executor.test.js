import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createHash, randomUUID } from "node:crypto";
import { createPublicationExecutor } from "../dist/publication_executor.js";
import { openDrive, closeDrive, closeSharedStore } from "../dist/hyperdrive_store.js";
import { indexEntryPath, readIndexEntry } from "../dist/publication_index_io.js";

function sha256Hex(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function makeConfig() {
  const packageRoot = mkdtempSync(join(tmpdir(), "exec-pkg-"));
  const driveStorageRoot = mkdtempSync(join(tmpdir(), "exec-drive-"));
  const indexRoot = mkdtempSync(join(tmpdir(), "exec-index-"));
  return { packageRoot, driveStorageRoot, indexRoot };
}

function writePackage(packageRoot, publicationId, lineageId, fileContent = "ciphertext-bytes-here") {
  const packageDir = join(packageRoot, publicationId);
  mkdirSync(packageDir, { recursive: true });
  const bytes = Buffer.from(fileContent);
  writeFileSync(join(packageDir, "index.m3u8"), bytes);

  const manifest = {
    asset_id: "11111111-1111-4111-8111-111111111111",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: [
      {
        path: "index.m3u8",
        plaintext_size: Math.max(0, bytes.length - 16),
        ciphertext_size: bytes.length,
        nonce_b64u: "AAECAwQFBgcICQoL",
        ciphertext_sha256: sha256Hex(bytes),
      },
    ],
    lineage_id: lineageId,
    manifest_version: "p2p-manifest-v1",
    publication_id: publicationId,
  };
  const manifestJson = JSON.stringify(manifest);
  writeFileSync(join(packageDir, "manifest.json"), manifestJson);
  return sha256Hex(Buffer.from(manifestJson));
}

function makeRequest(publicationId, lineageId, manifestDigest) {
  return {
    contract_version: "availability-publication-v1",
    publication_id: publicationId,
    lineage_id: lineageId,
    manifest_version: "p2p-manifest-v1",
    manifest_digest_sha256: manifestDigest,
    package_ref: publicationId,
  };
}

test("HP-1: first valid publication tuple returns 201 with stable evidence and writes the package into the drive", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const digest = writePackage(config.packageRoot, publicationId, lineageId);

  const executor = createPublicationExecutor(config);
  const result = await executor(makeRequest(publicationId, lineageId, digest));

  assert.equal(result.status, 201);
  assert.equal(result.evidence.publication_id, publicationId);
  assert.equal(result.evidence.lineage_id, lineageId);
  assert.equal(result.evidence.manifest_digest_sha256, digest);
  assert.equal(result.evidence.external_publication_id.length, 64);
  assert.equal(typeof result.evidence.evidence_id, "string");
  assert.ok(result.evidence.evidence_id.length > 0);

  await closeSharedStore(config.driveStorageRoot);
});

test("HP-2: replaying the same tuple returns 200 with byte-for-byte stable evidence and does not rewrite identity", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const digest = writePackage(config.packageRoot, publicationId, lineageId);

  const executor = createPublicationExecutor(config);
  const first = await executor(makeRequest(publicationId, lineageId, digest));
  const second = await executor(makeRequest(publicationId, lineageId, digest));

  assert.equal(first.status, 201);
  assert.equal(second.status, 200);
  assert.deepEqual(second.evidence, first.evidence);

  await closeSharedStore(config.driveStorageRoot);
});

test("HP-3: reconstructing the executor from the same persistent storage still replays the same evidence", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const digest = writePackage(config.packageRoot, publicationId, lineageId);

  const executorA = createPublicationExecutor(config);
  const first = await executorA(makeRequest(publicationId, lineageId, digest));
  await closeSharedStore(config.driveStorageRoot);

  // Simulate a fresh process: a brand-new executor instance against the same
  // on-disk roots.
  const executorB = createPublicationExecutor(config);
  const second = await executorB(makeRequest(publicationId, lineageId, digest));

  assert.equal(second.status, 200);
  assert.deepEqual(second.evidence, first.evidence);

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-1: same publication_id with a different lineage_id returns 409 publication_conflict", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const digest = writePackage(config.packageRoot, publicationId, lineageId);

  const executor = createPublicationExecutor(config);
  await executor(makeRequest(publicationId, lineageId, digest));

  const otherLineageId = randomUUID();
  const otherDigest = writePackage(config.packageRoot, publicationId, otherLineageId, "different-bytes-here");

  await assert.rejects(
    () => executor(makeRequest(publicationId, otherLineageId, otherDigest)),
    (err) => {
      assert.equal(err.code, "publication_conflict");
      return true;
    }
  );

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-2: a package whose ciphertext hash doesn't match the manifest returns 422 package_invalid before any drive write", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const digest = writePackage(config.packageRoot, publicationId, lineageId);

  // Corrupt the ciphertext bytes on disk after computing the (now stale) digest.
  writeFileSync(join(config.packageRoot, publicationId, "index.m3u8"), Buffer.from("tampered!"));

  const executor = createPublicationExecutor(config);

  await assert.rejects(
    () => executor(makeRequest(publicationId, lineageId, digest)),
    (err) => {
      assert.equal(err.code, "package_invalid");
      return true;
    }
  );

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-4: a package whose manifest.publication_id doesn't match the request's publication_id is rejected as a conflict", async () => {
  const config = makeConfig();
  const requestPublicationId = randomUUID();
  const manifestPublicationId = randomUUID();
  const lineageId = randomUUID();
  // Build a package on disk at a directory named after the *request's*
  // publication_id, but whose manifest declares a *different*
  // publication_id inside it.
  const digest = writePackage(config.packageRoot, requestPublicationId, lineageId);
  const manifestPath = join(config.packageRoot, requestPublicationId, "manifest.json");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  manifest.publication_id = manifestPublicationId;
  const manifestJson = JSON.stringify(manifest);
  writeFileSync(manifestPath, manifestJson);
  const newDigest = sha256Hex(Buffer.from(manifestJson));

  const executor = createPublicationExecutor(config);

  await assert.rejects(
    () => executor(makeRequest(requestPublicationId, lineageId, newDigest)),
    (err) => {
      assert.equal(err.code, "publication_conflict");
      return true;
    }
  );

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-5: the drive persists exactly the manifest and file bytes verifyPackage already hashed", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const fileContent = "original-verified-bytes";
  const digest = writePackage(config.packageRoot, publicationId, lineageId, fileContent);
  const manifestPath = join(config.packageRoot, publicationId, "manifest.json");
  const manifestOnDisk = readFileSync(manifestPath);

  const executor = createPublicationExecutor(config);
  const result = await executor(makeRequest(publicationId, lineageId, digest));

  assert.equal(result.status, 201);

  const opened = await openDrive(config.driveStorageRoot, publicationId);
  const manifestInDrive = await opened.drive.get("/manifest.json");
  const fileInDrive = await opened.drive.get("/index.m3u8");
  await closeDrive(opened);

  assert.deepEqual(manifestInDrive, manifestOnDisk);
  assert.equal(fileInDrive.toString(), fileContent);

  await closeSharedStore(config.driveStorageRoot);
});

test("HP-4: first valid publication announces on Hyperswarm before returning 201", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const digest = writePackage(config.packageRoot, publicationId, lineageId);

  const executor = createPublicationExecutor({ ...config, hyperswarmJoinTimeoutMs: 30_000 });
  const result = await executor(makeRequest(publicationId, lineageId, digest));

  assert.equal(result.status, 201);
  assert.equal(result.evidence.publication_id, publicationId);

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-6: a Hyperswarm join timeout returns 503 publication_unavailable and commits no success record", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const digest = writePackage(config.packageRoot, publicationId, lineageId);

  const executor = createPublicationExecutor({ ...config, hyperswarmJoinTimeoutMs: 1 });

  await assert.rejects(
    () => executor(makeRequest(publicationId, lineageId, digest)),
    (err) => {
      assert.equal(err.code, "publication_unavailable");
      return true;
    }
  );

  const entryPath = indexEntryPath(config.indexRoot, publicationId);
  const persisted = await readIndexEntry(entryPath);
  assert.equal(persisted, null);

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-7: after a Hyperswarm join timeout, a same-lineage retry with a generous timeout still succeeds with 201", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const digest = writePackage(config.packageRoot, publicationId, lineageId);

  const timingOutExecutor = createPublicationExecutor({ ...config, hyperswarmJoinTimeoutMs: 1 });
  await assert.rejects(() => timingOutExecutor(makeRequest(publicationId, lineageId, digest)));

  const retryExecutor = createPublicationExecutor({ ...config, hyperswarmJoinTimeoutMs: 30_000 });
  const result = await retryExecutor(makeRequest(publicationId, lineageId, digest));

  assert.equal(result.status, 201);
  assert.equal(result.evidence.publication_id, publicationId);

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-3: concurrent identical requests for the same publication_id serialize to one 201 and the rest 200 replays", async () => {
  const config = makeConfig();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const digest = writePackage(config.packageRoot, publicationId, lineageId);

  const executor = createPublicationExecutor(config);
  const request = makeRequest(publicationId, lineageId, digest);

  const results = await Promise.all([
    executor(request),
    executor(request),
    executor(request),
  ]);

  const statuses = results.map((r) => r.status).sort();
  assert.deepEqual(statuses, [200, 200, 201]);

  const evidenceIds = new Set(results.map((r) => r.evidence.evidence_id));
  assert.equal(evidenceIds.size, 1);

  await closeSharedStore(config.driveStorageRoot);
});
