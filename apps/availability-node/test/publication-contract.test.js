import { after, test } from "node:test";
import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import {
  mkdirSync,
  readFileSync,
  readdirSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";

import { EVIDENCE_FIELDS, parsePublicationResponse } from "../dist/contract.js";
import { openDrive, closeDrive, closeSharedStore } from "../dist/hyperdrive_store.js";
import { indexEntryPath, readIndexEntry } from "../dist/publication_index_io.js";
import { verifyPackage } from "../dist/package_verification.js";
import { createPublicationExecutor } from "../dist/publication_executor.js";
import { createPrivatePublicationServer } from "../dist/server.js";
import {
  MAX_REQUEST_BODY_BYTES,
  cleanupPublicationRoots,
  createConsoleCapture,
  createMtlsFixture,
  createPublicationRoots,
  defaultFiles,
  httpsPublicationRequest,
  loadSecretDenyList,
  makeRequest,
  runFixtureBinary,
  startServer,
  stopServer,
} from "./fixtures.js";

const mtls = createMtlsFixture();
after(() => mtls.cleanup());

function realServer(config, publisher = createPublicationExecutor(config)) {
  return createPrivatePublicationServer(
    mtls.credentials,
    [mtls.allowedFingerprint],
    publisher
  );
}

async function allowedRequest(port, request, overrides = {}) {
  return httpsPublicationRequest({
    port,
    ca: mtls.ca,
    client: mtls.allowed,
    publicationId: request.publication_id,
    body: request,
    ...overrides,
  });
}

function assertContractError(response, status, code) {
  assert.equal(response.status, status);
  assert.deepEqual(response.body, {
    contract_version: "availability-publication-v1",
    code,
  });
}

async function driveBlockCount(config, publicationId) {
  const opened = await openDrive(config.driveStorageRoot, publicationId);
  try {
    return opened.drive.core.length;
  } finally {
    await closeDrive(opened);
  }
}

test("HP-T3d-1/2 + EC-T3d-3 secrecy: real mTLS publication replays after reconstruction and exposes byte-identical ciphertext only", async () => {
  const config = createPublicationRoots();
  const assetId = randomUUID();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const built = runFixtureBinary({
    root: config.packageRoot,
    assetId,
    publicationId,
    lineageId,
    files: defaultFiles(),
  });
  const request = makeRequest(publicationId, lineageId, built.manifest_digest_sha256);
  const verified = await verifyPackage(
    config.packageRoot,
    request.package_ref,
    request.lineage_id,
    request.manifest_digest_sha256
  );
  assert.equal(verified.ok, true);

  const denyList = loadSecretDenyList();
  const secretCanaries = Object.fromEntries(
    denyList.map(field => [field, `T3D_CANARY_${field.toUpperCase()}`])
  );
  const capture = createConsoleCapture();
  let serverA;
  let serverB;

  try {
    serverA = realServer(config);
    const portA = await startServer(serverA);
    const first = await allowedRequest(portA, request);
    assert.equal(first.status, 201);
    assert.deepEqual(Object.keys(first.body).sort(), [...EVIDENCE_FIELDS].sort());
    assert.equal(first.body.publication_id, publicationId);
    assert.equal(first.body.lineage_id, lineageId);
    assert.equal(first.body.manifest_digest_sha256, built.manifest_digest_sha256);
    assert.equal(first.body.external_publication_id.length, 64);
    assert.ok(first.body.evidence_id.length > 0);
    assert.doesNotThrow(() => parsePublicationResponse(first.status, first.body, request));
    assert.equal("ready" in first.body, false);
    assert.equal("authorized" in first.body, false);
    assert.equal("business_authorization" in first.body, false);

    const opened = await openDrive(config.driveStorageRoot, publicationId);
    const rawDriveBytes = [];
    let coreLengthBeforeReplay;
    try {
      assert.equal(opened.publicKeyHex, first.body.external_publication_id);
      const manifestInDrive = await opened.drive.get("/manifest.json");
      assert.ok(manifestInDrive !== null);
      assert.deepEqual(manifestInDrive, verified.manifestBytes);
      rawDriveBytes.push(manifestInDrive);

      for (const file of verified.files) {
        const bytesInDrive = await opened.drive.get(`/${file.path}`);
        assert.ok(bytesInDrive !== null);
        assert.deepEqual(bytesInDrive, file.bytes);
        rawDriveBytes.push(bytesInDrive);
      }
      coreLengthBeforeReplay = opened.drive.core.length;
    } finally {
      await closeDrive(opened);
    }

    const malformedWithSecretFields = await allowedRequest(portA, request, {
      body: { ...request, ...secretCanaries },
    });
    assertContractError(malformedWithSecretFields, 400, "invalid_contract");

    const indexPath = indexEntryPath(config.indexRoot, publicationId);
    const indexBytes = readFileSync(indexPath);
    const persisted = await readIndexEntry(indexPath);
    assert.deepEqual(persisted, first.body);

    await stopServer(serverA);
    serverA = undefined;
    await closeSharedStore(config.driveStorageRoot);

    serverB = realServer(config);
    const portB = await startServer(serverB);
    const replay = await allowedRequest(portB, request);
    assert.equal(replay.status, 200);
    assert.deepEqual(replay.body, first.body);
    assert.equal(await driveBlockCount(config, publicationId), coreLengthBeforeReplay);

    const surfaces = [
      first.rawBody,
      malformedWithSecretFields.rawBody,
      replay.rawBody,
      capture.entries.join("\n"),
      indexBytes.toString("utf8"),
      ...rawDriveBytes.map(bytes => bytes.toString("utf8")),
    ].join("\n");
    for (const [field, canary] of Object.entries(secretCanaries)) {
      assert.equal(surfaces.includes(field), false, `${field} leaked into a server-owned surface`);
      assert.equal(surfaces.includes(canary), false, `${canary} leaked into a server-owned surface`);
    }
  } finally {
    capture.restore();
    if (serverA !== undefined) await stopServer(serverA);
    if (serverB !== undefined) await stopServer(serverB);
    await cleanupPublicationRoots(config);
  }
});

test("EC-T3d-1: absent, rogue, and CA-trusted-but-unlisted client identities cannot invoke the real executor", async () => {
  const config = createPublicationRoots();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const built = runFixtureBinary({
    root: config.packageRoot,
    assetId: randomUUID(),
    publicationId,
    lineageId,
    files: defaultFiles(),
  });
  const request = makeRequest(publicationId, lineageId, built.manifest_digest_sha256);
  const executor = createPublicationExecutor(config);
  let executorCalls = 0;
  const publisher = async value => {
    executorCalls += 1;
    return executor(value);
  };
  const server = realServer(config, publisher);

  try {
    const port = await startServer(server);
    await assert.rejects(
      httpsPublicationRequest({
        port,
        ca: mtls.ca,
        publicationId,
        body: request,
      })
    );
    assert.equal(executorCalls, 0);

    await assert.rejects(
      httpsPublicationRequest({
        port,
        ca: mtls.ca,
        client: mtls.rogue,
        publicationId,
        body: request,
      })
    );
    assert.equal(executorCalls, 0);

    const unlisted = await httpsPublicationRequest({
      port,
      ca: mtls.ca,
      client: mtls.unlisted,
      publicationId,
      body: request,
    });
    assertContractError(unlisted, 403, "service_identity_rejected");
    assert.equal(executorCalls, 0);

    const allowed = await allowedRequest(port, request);
    assert.equal(allowed.status, 201);
    assert.equal(executorCalls, 1);
  } finally {
    await stopServer(server);
    await cleanupPublicationRoots(config);
  }
});

test("EC-T3d-2 conflicts: different lineage or digest returns 409 with stable original evidence and no second drive write", async () => {
  const config = createPublicationRoots();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const firstBuilt = runFixtureBinary({
    root: config.packageRoot,
    assetId: randomUUID(),
    publicationId,
    lineageId,
    files: defaultFiles("original-segment"),
  });
  const firstRequest = makeRequest(publicationId, lineageId, firstBuilt.manifest_digest_sha256);
  let server;

  try {
    server = realServer(config);
    let port = await startServer(server);
    const first = await allowedRequest(port, firstRequest);
    assert.equal(first.status, 201);
    const blockCount = await driveBlockCount(config, publicationId);
    await stopServer(server);
    server = undefined;

    const lineagePackageRoot = join(config.root, "lineage-conflict-root");
    mkdirSync(lineagePackageRoot);
    const otherLineageId = randomUUID();
    const lineageBuilt = runFixtureBinary({
      root: lineagePackageRoot,
      assetId: randomUUID(),
      publicationId,
      lineageId: otherLineageId,
      files: defaultFiles("different-lineage-segment"),
    });
    server = realServer({ ...config, packageRoot: lineagePackageRoot });
    port = await startServer(server);
    const lineageConflict = await allowedRequest(
      port,
      makeRequest(publicationId, otherLineageId, lineageBuilt.manifest_digest_sha256)
    );
    assertContractError(lineageConflict, 409, "publication_conflict");
    await stopServer(server);
    server = undefined;

    const digestPackageRoot = join(config.root, "digest-conflict-root");
    mkdirSync(digestPackageRoot);
    const digestBuilt = runFixtureBinary({
      root: digestPackageRoot,
      assetId: randomUUID(),
      publicationId,
      lineageId,
      files: defaultFiles("different-digest-segment"),
    });
    assert.notEqual(digestBuilt.manifest_digest_sha256, firstBuilt.manifest_digest_sha256);
    server = realServer({ ...config, packageRoot: digestPackageRoot });
    port = await startServer(server);
    const digestConflict = await allowedRequest(
      port,
      makeRequest(publicationId, lineageId, digestBuilt.manifest_digest_sha256)
    );
    assertContractError(digestConflict, 409, "publication_conflict");

    assert.equal(await driveBlockCount(config, publicationId), blockCount);
    const persisted = await readIndexEntry(indexEntryPath(config.indexRoot, publicationId));
    assert.deepEqual(persisted, first.body);
  } finally {
    if (server !== undefined) await stopServer(server);
    await cleanupPublicationRoots(config);
  }
});

test("EC-T3d-2 integrity and containment: tampered ciphertext and a symlink escape return 422 before drive creation", async t => {
  await t.test("tampered ciphertext", async () => {
    const config = createPublicationRoots();
    const publicationId = randomUUID();
    const lineageId = randomUUID();
    const built = runFixtureBinary({
      root: config.packageRoot,
      assetId: randomUUID(),
      publicationId,
      lineageId,
      files: defaultFiles(),
    });
    writeFileSync(join(built.package_dir, "index.m3u8"), "tampered-after-materialize");
    const request = makeRequest(publicationId, lineageId, built.manifest_digest_sha256);
    const server = realServer(config);
    try {
      const response = await allowedRequest(await startServer(server), request);
      assertContractError(response, 422, "package_invalid");
      assert.deepEqual(readdirSync(config.driveStorageRoot), []);
      assert.equal(await readIndexEntry(indexEntryPath(config.indexRoot, publicationId)), null);
    } finally {
      await stopServer(server);
      await cleanupPublicationRoots(config);
    }
  });

  await t.test("symlink escape", async () => {
    const config = createPublicationRoots();
    const outsideRoot = join(config.root, "outside-package-root");
    mkdirSync(outsideRoot);
    const publicationId = randomUUID();
    const lineageId = randomUUID();
    const built = runFixtureBinary({
      root: outsideRoot,
      assetId: randomUUID(),
      publicationId,
      lineageId,
      files: defaultFiles(),
    });
    const publicationDir = join(config.packageRoot, "packages", publicationId);
    mkdirSync(publicationDir, { recursive: true });
    symlinkSync(built.package_dir, join(publicationDir, lineageId), "dir");
    const request = makeRequest(publicationId, lineageId, built.manifest_digest_sha256);
    const server = realServer(config);
    try {
      const response = await allowedRequest(await startServer(server), request);
      assertContractError(response, 422, "package_invalid");
      assert.deepEqual(readdirSync(config.driveStorageRoot), []);
      assert.equal(await readIndexEntry(indexEntryPath(config.indexRoot, publicationId)), null);
    } finally {
      await stopServer(server);
      await cleanupPublicationRoots(config);
    }
  });
});

test("EC-T3d-2 request framing: malformed JSON, wrong content type, and oversized bodies return 400 without invoking the real executor", async () => {
  const config = createPublicationRoots();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const request = makeRequest(publicationId, lineageId, "0".repeat(64));
  const executor = createPublicationExecutor(config);
  let executorCalls = 0;
  const server = realServer(config, async value => {
    executorCalls += 1;
    return executor(value);
  });

  try {
    const port = await startServer(server);
    const malformed = await allowedRequest(port, request, { body: "{not-json" });
    assertContractError(malformed, 400, "invalid_contract");

    const wrongContentType = await allowedRequest(port, request, {
      contentType: "text/plain",
    });
    assertContractError(wrongContentType, 400, "invalid_contract");

    const oversized = await allowedRequest(port, request, {
      body: Buffer.alloc(MAX_REQUEST_BODY_BYTES + 1, "x"),
    });
    assertContractError(oversized, 400, "invalid_contract");
    assert.equal(executorCalls, 0);
  } finally {
    await stopServer(server);
    await cleanupPublicationRoots(config);
  }
});

test("EC-T3d-3: a forced Hyperswarm join timeout returns 503 and commits no success record", async () => {
  const config = createPublicationRoots();
  const publicationId = randomUUID();
  const lineageId = randomUUID();
  const built = runFixtureBinary({
    root: config.packageRoot,
    assetId: randomUUID(),
    publicationId,
    lineageId,
    files: defaultFiles(),
  });
  const request = makeRequest(publicationId, lineageId, built.manifest_digest_sha256);
  const server = realServer({ ...config, hyperswarmJoinTimeoutMs: 0 });

  try {
    const response = await allowedRequest(await startServer(server), request);
    assertContractError(response, 503, "publication_unavailable");
    assert.equal(await readIndexEntry(indexEntryPath(config.indexRoot, publicationId)), null);
  } finally {
    await stopServer(server);
    await cleanupPublicationRoots(config);
  }
});
