import { test } from "node:test";
import assert from "node:assert/strict";

async function loadPublicationRecord() {
  const mod = await import("../dist/publication_record.js");
  return mod;
}

const VALID_RECORD = {
  contract_version: "availability-publication-v1",
  publication_id: "01234567-89ab-cdef-0123-456789abcdef",
  lineage_id: "fedcba98-7654-3210-fedc-ba9876543210",
  manifest_digest_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  external_publication_id: "ext-pub-1",
  evidence_id: "evidence-1",
  confirmed_at: "2026-09-12T00:00:00Z",
};

test("HP: valid evidence encodes in frozen field order and round-trips byte-for-byte", async () => {
  const { encodePublicationRecord, decodePublicationRecord } = await loadPublicationRecord();
  const encoded = encodePublicationRecord(VALID_RECORD);
  assert.ok(encoded instanceof Uint8Array);
  const decoded = decodePublicationRecord(encoded);
  assert.deepStrictEqual(decoded, VALID_RECORD);
  const parsed = JSON.parse(new TextDecoder().decode(encoded));
  assert.deepStrictEqual(Object.keys(parsed), [
    "contract_version",
    "publication_id",
    "lineage_id",
    "manifest_digest_sha256",
    "external_publication_id",
    "evidence_id",
    "confirmed_at",
  ]);
});

test("HP: repeated encoding of the same logical record is byte-identical", async () => {
  const { encodePublicationRecord } = await loadPublicationRecord();
  const a = encodePublicationRecord(VALID_RECORD);
  const b = encodePublicationRecord(VALID_RECORD);
  assert.ok(Buffer.from(a).equals(Buffer.from(b)));
});

test("EC: malformed UTF-8, malformed JSON, and invalid/missing fields are all rejected as invalid_contract", async () => {
  const { decodePublicationRecord } = await loadPublicationRecord();

  const malformedUtf8 = new Uint8Array([0xff, 0xfe, 0xfd]);
  assert.throws(() => decodePublicationRecord(malformedUtf8), (err) => err.code === "invalid_contract");

  const malformedJson = new TextEncoder().encode("{not valid json");
  assert.throws(() => decodePublicationRecord(malformedJson), (err) => err.code === "invalid_contract");

  const missingField = { ...VALID_RECORD };
  delete missingField.evidence_id;
  const missingBytes = new TextEncoder().encode(JSON.stringify(missingField));
  assert.throws(() => decodePublicationRecord(missingBytes), (err) => err.code === "invalid_contract");

  const extraField = { ...VALID_RECORD, extra: "unexpected" };
  const extraBytes = new TextEncoder().encode(JSON.stringify(extraField));
  assert.throws(() => decodePublicationRecord(extraBytes), (err) => err.code === "invalid_contract");

  const badDate = { ...VALID_RECORD, confirmed_at: "not-a-date" };
  const badDateBytes = new TextEncoder().encode(JSON.stringify(badDate));
  assert.throws(() => decodePublicationRecord(badDateBytes), (err) => err.code === "invalid_contract");

  const wrongVersion = { ...VALID_RECORD, contract_version: "wrong-version" };
  const wrongVersionBytes = new TextEncoder().encode(JSON.stringify(wrongVersion));
  assert.throws(() => decodePublicationRecord(wrongVersionBytes), (err) => err.code === "invalid_contract");
});

test("EC: semantically equivalent but non-canonical JSON is rejected", async () => {
  const { decodePublicationRecord, encodePublicationRecord } = await loadPublicationRecord();

  const reversed = {
    confirmed_at: VALID_RECORD.confirmed_at,
    evidence_id: VALID_RECORD.evidence_id,
    external_publication_id: VALID_RECORD.external_publication_id,
    manifest_digest_sha256: VALID_RECORD.manifest_digest_sha256,
    lineage_id: VALID_RECORD.lineage_id,
    publication_id: VALID_RECORD.publication_id,
    contract_version: VALID_RECORD.contract_version,
  };
  const reversedBytes = new TextEncoder().encode(JSON.stringify(reversed));
  assert.throws(() => decodePublicationRecord(reversedBytes), (err) => err.code === "invalid_contract");

  const canonical = encodePublicationRecord(VALID_RECORD);
  const canonicalStr = new TextDecoder().decode(canonical);
  const pretty = JSON.stringify(JSON.parse(canonicalStr), null, 2);
  const prettyBytes = new TextEncoder().encode(pretty);
  assert.throws(() => decodePublicationRecord(prettyBytes), (err) => err.code === "invalid_contract");

  const trailingNewline = new TextEncoder().encode(canonicalStr + "\n");
  assert.throws(() => decodePublicationRecord(trailingNewline), (err) => err.code === "invalid_contract");
});