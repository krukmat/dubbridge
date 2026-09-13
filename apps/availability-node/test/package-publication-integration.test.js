// Proves that a package genuinely built and materialized by the real Rust
// production pipeline (`dubbridge_p2p::package_builder::build_package` +
// `dubbridge_p2p::package_writer::materialize`) is accepted end-to-end by
// the real Availability Node publication executor
// (`createPublicationExecutor`), rather than a hand-written JS fixture
// reproducing the on-disk shape without executing the actual Rust logic.

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { randomBytes, randomUUID } from "node:crypto";
import { fileURLToPath } from "node:url";
import { createPublicationExecutor } from "../dist/publication_executor.js";
import { openDrive, closeDrive, closeSharedStore } from "../dist/hyperdrive_store.js";

// Repository root: apps/availability-node/test -> apps/availability-node -> apps -> repo root.
const REPO_ROOT = join(fileURLToPath(new URL("../../..", import.meta.url)));

function makeConfig() {
  const packageRoot = mkdtempSync(join(tmpdir(), "pkg-publication-integration-pkg-"));
  const driveStorageRoot = mkdtempSync(join(tmpdir(), "pkg-publication-integration-drive-"));
  const indexRoot = mkdtempSync(join(tmpdir(), "pkg-publication-integration-index-"));
  return { packageRoot, driveStorageRoot, indexRoot };
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

// Invokes the real Rust production pipeline (build_package + materialize)
// via the test-only `package_build_and_materialize_fixture` binary
// (`crates/p2p/src/bin/package_build_and_materialize_fixture.rs`), which Cargo
// auto-discovers as a binary target with no Cargo.toml change. Always exits
// 0; callers must check the JSON `ok` field, not the exit code, so that an
// intentionally-invalid input can be asserted on without execFileSync
// throwing on a non-zero exit.
function runFixtureBinary({ root, assetId, publicationId, lineageId, ckHex, files }) {
  const args = [
    "run",
    "--quiet",
    "-p",
    "dubbridge-p2p",
    "--bin",
    "package_build_and_materialize_fixture",
    "--",
    "--root",
    root,
    "--asset-id",
    assetId,
    "--publication-id",
    publicationId,
    "--lineage-id",
    lineageId,
    "--ck-hex",
    ckHex,
  ];
  for (const file of files) {
    args.push("--file", `${file.path}=${file.content}`);
  }
  const stdout = execFileSync("cargo", args, { cwd: REPO_ROOT, encoding: "utf8" });
  return JSON.parse(stdout.trim());
}

function randomCkHex() {
  return randomBytes(32).toString("hex");
}

function defaultFiles(segmentContent = "segment-bytes-1") {
  return [
    { path: "index.m3u8", content: "#EXTM3U" },
    { path: "segments/000001.ts", content: segmentContent },
  ];
}

test("HP-T3c-1: a real Rust-built and materialized package is accepted end-to-end with 201 and stable evidence", async () => {
  const config = makeConfig();
  const assetId = randomUUID();
  const publicationId = randomUUID();
  const lineageId = randomUUID();

  const built = runFixtureBinary({
    root: config.packageRoot,
    assetId,
    publicationId,
    lineageId,
    ckHex: randomCkHex(),
    files: defaultFiles(),
  });

  assert.equal(built.ok, true);
  assert.equal(built.publication_id, publicationId);
  assert.equal(built.lineage_id, lineageId);
  assert.equal(typeof built.manifest_digest_sha256, "string");
  assert.equal(built.manifest_digest_sha256.length, 64);

  const executor = createPublicationExecutor(config);
  const result = await executor(makeRequest(publicationId, lineageId, built.manifest_digest_sha256));

  assert.equal(result.status, 201);
  assert.equal(result.evidence.publication_id, publicationId);
  assert.equal(result.evidence.lineage_id, lineageId);
  assert.equal(result.evidence.manifest_digest_sha256, built.manifest_digest_sha256);
  assert.equal(result.evidence.external_publication_id.length, 64);

  const opened = await openDrive(config.driveStorageRoot, publicationId);
  const manifestInDrive = await opened.drive.get("/manifest.json");
  const indexInDrive = await opened.drive.get("/index.m3u8");
  const segmentInDrive = await opened.drive.get("/segments/000001.ts");
  await closeDrive(opened);

  // index.m3u8 and the segment are ciphertext (AES-256-GCM), not plaintext —
  // the real build_package/materialize pipeline never stores plaintext
  // content. The meaningful assertion is that the drive holds the exact
  // ciphertext bytes the Rust pipeline materialized on disk, proving a
  // byte-for-byte copy rather than a re-derivation.
  const indexOnDisk = readFileSync(join(built.package_dir, "index.m3u8"));
  const segmentOnDisk = readFileSync(join(built.package_dir, "segments/000001.ts"));

  assert.ok(manifestInDrive !== null);
  assert.ok(indexInDrive !== null);
  assert.deepEqual(indexInDrive, indexOnDisk);
  assert.notEqual(indexInDrive.toString("utf8"), "#EXTM3U");
  assert.ok(segmentInDrive !== null);
  assert.deepEqual(segmentInDrive, segmentOnDisk);

  await closeSharedStore(config.driveStorageRoot);
});

test("HP-T3c-2: replaying the same real package after executor reconstruction returns 200 with stable evidence and no second drive write", async () => {
  const config = makeConfig();
  const assetId = randomUUID();
  const publicationId = randomUUID();
  const lineageId = randomUUID();

  const built = runFixtureBinary({
    root: config.packageRoot,
    assetId,
    publicationId,
    lineageId,
    ckHex: randomCkHex(),
    files: defaultFiles(),
  });
  assert.equal(built.ok, true);

  const executorA = createPublicationExecutor(config);
  const first = await executorA(makeRequest(publicationId, lineageId, built.manifest_digest_sha256));
  assert.equal(first.status, 201);

  const openedBefore = await openDrive(config.driveStorageRoot, publicationId);
  const coreLengthBefore = /** @type {any} */ (openedBefore.drive).core.length;
  await closeDrive(openedBefore);

  // Simulate executor reconstruction (process restart): a fresh executor
  // over the same persistent on-disk roots, not a fresh executor instance
  // that shares in-memory state with the first.
  const executorB = createPublicationExecutor(config);
  const second = await executorB(makeRequest(publicationId, lineageId, built.manifest_digest_sha256));

  assert.equal(second.status, 200);
  assert.deepEqual(second.evidence, first.evidence);

  const openedAfter = await openDrive(config.driveStorageRoot, publicationId);
  const coreLengthAfter = /** @type {any} */ (openedAfter.drive).core.length;
  await closeDrive(openedAfter);

  // The observable used here (`drive.core.length`, a Hypercore block count)
  // is a real property on the installed hyperdrive@13.3.3 (`this.core =
  // this.db.core` in its own source) rather than the local hand-written
  // `.d.ts`, which only declares the narrower surface this codebase already
  // calls elsewhere. Asserting it is unchanged proves no second write
  // occurred on replay, which unchanged file bytes alone would not: an
  // identical rewrite would still read back the same bytes.
  assert.equal(coreLengthAfter, coreLengthBefore);

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-T3c-1a: same publication_id with a different lineage_id (from two real builds) returns 409 with the original evidence unchanged", async () => {
  const config = makeConfig();
  const assetId = randomUUID();
  const publicationId = randomUUID();
  const lineageId = randomUUID();

  const builtFirst = runFixtureBinary({
    root: config.packageRoot,
    assetId,
    publicationId,
    lineageId,
    ckHex: randomCkHex(),
    files: defaultFiles(),
  });
  assert.equal(builtFirst.ok, true);

  const executor = createPublicationExecutor(config);
  const first = await executor(makeRequest(publicationId, lineageId, builtFirst.manifest_digest_sha256));
  assert.equal(first.status, 201);

  const openedBefore = await openDrive(config.driveStorageRoot, publicationId);
  const coreLengthBefore = /** @type {any} */ (openedBefore.drive).core.length;
  await closeDrive(openedBefore);

  // A second real build reusing the same publication_id but a different
  // lineage_id materializes to a *different* on-disk directory (materialize
  // is keyed by publication_id, and this fixture uses a fresh packageRoot
  // subtree per publication_id) is not possible here since materialize()
  // itself is keyed by publication_id under one root — so instead this
  // simulates the conflicting tuple the executor must reject by pointing a
  // second, distinct packageRoot at the same publication_id via a fresh
  // fixture invocation, then asserting the executor (which is the layer
  // under test, not materialize's own already-certified conflict handling)
  // rejects the mismatched lineage_id before ever reading that second
  // package's content.
  const otherLineageId = randomUUID();
  const otherPackageRoot = mkdtempSync(join(tmpdir(), "pkg-publication-integration-pkg-other-"));
  const builtSecond = runFixtureBinary({
    root: otherPackageRoot,
    assetId,
    publicationId,
    lineageId: otherLineageId,
    ckHex: randomCkHex(),
    files: defaultFiles("segment-bytes-2"),
  });
  assert.equal(builtSecond.ok, true);

  // Point config.packageRoot's publication_id directory at the second
  // build's manifest/content by using the second build's own root as the
  // executor's packageRoot for this one call — the executor only reads
  // config.packageRoot at request time, so swapping which built directory
  // exists at that publication_id path is exactly the wire-level conflict
  // being tested (two distinct real packages claiming the same
  // publication_id with different lineage_id).
  const conflictingExecutor = createPublicationExecutor({
    ...config,
    packageRoot: otherPackageRoot,
  });

  await assert.rejects(
    () => conflictingExecutor(makeRequest(publicationId, otherLineageId, builtSecond.manifest_digest_sha256)),
    (err) => {
      assert.equal(err.code, "publication_conflict");
      return true;
    }
  );

  const openedAfter = await openDrive(config.driveStorageRoot, publicationId);
  const coreLengthAfter = /** @type {any} */ (openedAfter.drive).core.length;
  const manifestAfter = await openedAfter.drive.get("/manifest.json");
  await closeDrive(openedAfter);

  assert.equal(coreLengthAfter, coreLengthBefore);
  assert.ok(manifestAfter !== null);

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-T3c-1b: same publication_id and lineage_id but a different real manifest digest returns 409 with the original evidence unchanged", async () => {
  const config = makeConfig();
  const assetId = randomUUID();
  const publicationId = randomUUID();
  const lineageId = randomUUID();

  const builtFirst = runFixtureBinary({
    root: config.packageRoot,
    assetId,
    publicationId,
    lineageId,
    ckHex: randomCkHex(),
    files: defaultFiles("segment-bytes-original"),
  });
  assert.equal(builtFirst.ok, true);

  const executor = createPublicationExecutor(config);
  const first = await executor(makeRequest(publicationId, lineageId, builtFirst.manifest_digest_sha256));
  assert.equal(first.status, 201);

  const openedBefore = await openDrive(config.driveStorageRoot, publicationId);
  const coreLengthBefore = /** @type {any} */ (openedBefore.drive).core.length;
  await closeDrive(openedBefore);

  // Different file content under the same publication_id/lineage_id changes
  // manifest_digest_sha256 while lineage stays fixed.
  const otherPackageRoot = mkdtempSync(join(tmpdir(), "pkg-publication-integration-pkg-digest-"));
  const builtSecond = runFixtureBinary({
    root: otherPackageRoot,
    assetId,
    publicationId,
    lineageId,
    ckHex: randomCkHex(),
    files: defaultFiles("segment-bytes-DIFFERENT"),
  });
  assert.equal(builtSecond.ok, true);
  assert.notEqual(builtSecond.manifest_digest_sha256, builtFirst.manifest_digest_sha256);

  const conflictingExecutor = createPublicationExecutor({
    ...config,
    packageRoot: otherPackageRoot,
  });

  await assert.rejects(
    () => conflictingExecutor(makeRequest(publicationId, lineageId, builtSecond.manifest_digest_sha256)),
    (err) => {
      assert.equal(err.code, "publication_conflict");
      return true;
    }
  );

  const openedAfter = await openDrive(config.driveStorageRoot, publicationId);
  const coreLengthAfter = /** @type {any} */ (openedAfter.drive).core.length;
  await closeDrive(openedAfter);

  assert.equal(coreLengthAfter, coreLengthBefore);

  await closeSharedStore(config.driveStorageRoot);
});

test("EC-T3c-2: tampering with a real materialized ciphertext file after the fact returns 422 and writes nothing to the drive", async () => {
  const config = makeConfig();
  const assetId = randomUUID();
  const publicationId = randomUUID();
  const lineageId = randomUUID();

  const built = runFixtureBinary({
    root: config.packageRoot,
    assetId,
    publicationId,
    lineageId,
    ckHex: randomCkHex(),
    files: defaultFiles(),
  });
  assert.equal(built.ok, true);

  // Tamper with the real Rust-materialized ciphertext file on disk after
  // the fact, exactly like publication-executor.test.js's EC-2, but here
  // the pre-tamper bytes are genuine AES-256-GCM ciphertext produced by
  // build_package, not hand-written JS bytes.
  const { writeFileSync } = await import("node:fs");
  writeFileSync(join(built.package_dir, "index.m3u8"), Buffer.from("tampered-after-real-materialize"));

  const executor = createPublicationExecutor(config);

  await assert.rejects(
    () => executor(makeRequest(publicationId, lineageId, built.manifest_digest_sha256)),
    (err) => {
      assert.equal(err.code, "package_invalid");
      return true;
    }
  );

  // No drive should have been created at all for a request that fails
  // verification before any write.
  const opened = await openDrive(config.driveStorageRoot, publicationId);
  const manifestInDrive = await opened.drive.get("/manifest.json");
  await closeDrive(opened);
  assert.equal(manifestInDrive, null);

  await closeSharedStore(config.driveStorageRoot);
});
