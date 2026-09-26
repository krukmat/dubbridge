import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  indexEntryPath,
  readIndexEntry,
  writeIndexEntry,
} from "../dist/publication_index_io.js";
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

test("HP-1: indexEntryPath returns correct path", () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const root = dir;
    const id = "01234567-89ab-cdef-0123-456789abcdef";
    const path = indexEntryPath(root, id);
    assert.equal(path, join(root, `${id}.json`));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("HP-2: writeIndexEntry followed by readIndexEntry returns deep-equal record", async () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const path = indexEntryPath(dir, VALID_RECORD.publication_id);
    await writeIndexEntry(path, VALID_RECORD);
    const read = await readIndexEntry(path);
    assert.deepEqual(read, VALID_RECORD);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("EC-1: readIndexEntry on non-existent file returns null", async () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const path = join(dir, "non-existent.json");
    const result = await readIndexEntry(path);
    assert.equal(result, null);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("EC-2: readIndexEntry on corrupt file throws", async () => {
  const dir = mkdtempSync(join(tmpdir(), "pub-index-test-"));
  try {
    const path = join(dir, "corrupt.json");
    writeFileSync(path, "not valid json or encoded record");
    await assert.rejects(() => readIndexEntry(path));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("EC-3: indexEntryPath with path traversal throws ContainmentError", () => {
  const root = "/tmp/test-index";
  const maliciousId = "../../etc/passwd";
  assert.throws(() => indexEntryPath(root, maliciousId), ContainmentError);
});