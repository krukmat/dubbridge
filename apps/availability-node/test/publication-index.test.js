import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, statSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { decideAndPersist } from "../dist/publication_index.js";
import { indexEntryPath, readIndexEntry } from "../dist/publication_index_io.js";
import { ContainmentError } from "../dist/containment.js";

const VALID_RECORD = {
  contract_version: "availability-publication-v1",
  publication_id: "01234567-89ab-cdef-0123-456789abcdef",
  lineage_id: "fedcba98-7654-3210-fedc-ba9876543210",
  manifest_digest_sha256:
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  external_publication_id: "ext-pub-1",
  evidence_id: "evidence-1",
  confirmed_at: "2026-09-12T00:00:00Z",
};

test("HP-1: first write of a new tuple returns written and persists the record", async () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const outcome = await decideAndPersist(dir, VALID_RECORD);
    assert.deepEqual(outcome, { kind: "written" });

    const entryPath = indexEntryPath(dir, VALID_RECORD.publication_id);
    const stored = await readIndexEntry(entryPath);
    assert.deepEqual(stored, VALID_RECORD);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("HP-2: replaying the identical tuple returns replayed without rewriting the file", async () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const first = await decideAndPersist(dir, VALID_RECORD);
    assert.deepEqual(first, { kind: "written" });

    const entryPath = indexEntryPath(dir, VALID_RECORD.publication_id);
    const statBefore = statSync(entryPath);

    const second = await decideAndPersist(dir, { ...VALID_RECORD });
    assert.deepEqual(second, { kind: "replayed" });

    const statAfter = statSync(entryPath);
    assert.equal(statAfter.mtimeMs, statBefore.mtimeMs);
    assert.equal(statAfter.ino, statBefore.ino);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("EC-1: differing lineage_id for the same publication_id returns conflict without writing", async () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const first = await decideAndPersist(dir, VALID_RECORD);
    assert.deepEqual(first, { kind: "written" });

    const entryPath = indexEntryPath(dir, VALID_RECORD.publication_id);
    const statBefore = statSync(entryPath);

    const conflicting = {
      ...VALID_RECORD,
      lineage_id: "00000000-0000-0000-0000-000000000000",
    };
    const second = await decideAndPersist(dir, conflicting);
    assert.deepEqual(second, { kind: "conflict", existing: VALID_RECORD });

    const statAfter = statSync(entryPath);
    assert.equal(statAfter.mtimeMs, statBefore.mtimeMs);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("EC-1b: differing manifest_digest_sha256 for the same publication_id returns conflict without writing", async () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const first = await decideAndPersist(dir, VALID_RECORD);
    assert.deepEqual(first, { kind: "written" });

    const conflicting = {
      ...VALID_RECORD,
      manifest_digest_sha256:
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    };
    const second = await decideAndPersist(dir, conflicting);
    assert.deepEqual(second, { kind: "conflict", existing: VALID_RECORD });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("EC-2: path traversal in publication_id propagates ContainmentError unchanged", async () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const malicious = { ...VALID_RECORD, publication_id: "../../etc/passwd" };
    await assert.rejects(() => decideAndPersist(dir, malicious), ContainmentError);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("EC-3: corrupt existing entry propagates the decode error unchanged", async () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const entryPath = indexEntryPath(dir, VALID_RECORD.publication_id);
    writeFileSync(entryPath, "not valid json or encoded record");

    await assert.rejects(() => decideAndPersist(dir, VALID_RECORD));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
