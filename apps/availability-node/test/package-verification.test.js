import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, symlinkSync, chmodSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createHash } from "node:crypto";
import {
  decodeManifest,
  verifyFileAgainstManifest,
  verifyPackage,
} from "../dist/package_verification.js";

function sha256Hex(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function writePackage(root, publicationId, { lineageId, files, manifestOverride } = {}) {
  const packageDir = join(root, publicationId);
  mkdirSync(packageDir, { recursive: true });

  const fileEntries = (files ?? [{ path: "index.m3u8", content: Buffer.from("ciphertext-a") }]).map(
    (f) => {
      const bytes = f.content;
      const fullPath = join(packageDir, f.path);
      mkdirSync(join(fullPath, ".."), { recursive: true });
      writeFileSync(fullPath, bytes);
      return {
        path: f.path,
        plaintext_size: f.plaintextSize ?? Math.max(0, bytes.length - 16),
        ciphertext_size: bytes.length,
        nonce_b64u: "AAECAwQFBgcICQoL",
        ciphertext_sha256: f.hashOverride ?? sha256Hex(bytes),
      };
    }
  );

  const manifest = manifestOverride ?? {
    asset_id: "11111111-1111-4111-8111-111111111111",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: fileEntries,
    lineage_id: lineageId ?? "33333333-3333-4333-8333-333333333333",
    manifest_version: "p2p-manifest-v1",
    publication_id: publicationId,
  };

  const manifestJson = JSON.stringify(manifest);
  writeFileSync(join(packageDir, "manifest.json"), manifestJson);
  const manifestDigest = sha256Hex(Buffer.from(manifestJson));

  return { packageDir, manifestJson, manifestDigest, lineageId: manifest.lineage_id };
}

test("HP-1: a well-formed package with matching manifest digest, lineage, and file hashes verifies ok", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-hp1-"));
  const { manifestDigest, lineageId } = writePackage(root, "pub-hp1");

  const result = await verifyPackage(root, "pub-hp1", lineageId, manifestDigest);

  assert.equal(result.ok, true);
  assert.equal(result.manifest.publication_id, "pub-hp1");
  assert.equal(result.manifest.files.length, 1);
});

test("HP-2: decodeManifest accepts a structurally valid manifest object", () => {
  const manifest = decodeManifest({
    asset_id: "a",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: [
      {
        path: "a.ts",
        plaintext_size: 10,
        ciphertext_size: 26,
        nonce_b64u: "x",
        ciphertext_sha256: "0".repeat(64),
      },
    ],
    lineage_id: "l",
    manifest_version: "p2p-manifest-v1",
    publication_id: "p",
  });

  assert.notEqual(manifest, null);
  assert.equal(manifest.files.length, 1);
});

test("EC-1: manifest digest mismatch (tampered manifest bytes) returns package_invalid", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec1-"));
  const { lineageId } = writePackage(root, "pub-ec1");

  const result = await verifyPackage(root, "pub-ec1", lineageId, "f".repeat(64));

  assert.equal(result.ok, false);
  assert.equal(result.reason, "package_invalid");
});

test("EC-2: same publication_id with a different lineage_id returns publication_conflict", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec2-"));
  const { manifestDigest } = writePackage(root, "pub-ec2", {
    lineageId: "33333333-3333-4333-8333-333333333333",
  });

  const result = await verifyPackage(
    root,
    "pub-ec2",
    "44444444-4444-4444-4444-444444444444",
    manifestDigest
  );

  assert.equal(result.ok, false);
  assert.equal(result.reason, "publication_conflict");
});

test("EC-3: a ciphertext file whose bytes don't match its declared hash returns package_invalid", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec3-"));
  const packageDir = join(root, "pub-ec3");
  mkdirSync(packageDir, { recursive: true });
  writeFileSync(join(packageDir, "index.m3u8"), Buffer.from("real-bytes-on-disk"));

  const manifest = {
    asset_id: "a",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: [
      {
        path: "index.m3u8",
        plaintext_size: 2,
        ciphertext_size: 18,
        nonce_b64u: "AAECAwQFBgcICQoL",
        ciphertext_sha256: sha256Hex(Buffer.from("different-bytes")),
      },
    ],
    lineage_id: "l1",
    manifest_version: "p2p-manifest-v1",
    publication_id: "pub-ec3",
  };
  const manifestJson = JSON.stringify(manifest);
  writeFileSync(join(packageDir, "manifest.json"), manifestJson);
  const manifestDigest = sha256Hex(Buffer.from(manifestJson));

  const result = await verifyPackage(root, "pub-ec3", "l1", manifestDigest);

  assert.equal(result.ok, false);
  assert.equal(result.reason, "package_invalid");
});

test("EC-4: a declared file size that doesn't match the on-disk file size returns package_invalid", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec4-"));
  const packageDir = join(root, "pub-ec4");
  mkdirSync(packageDir, { recursive: true });
  const bytes = Buffer.from("some-ciphertext-bytes");
  writeFileSync(join(packageDir, "index.m3u8"), bytes);

  const manifest = {
    asset_id: "a",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: [
      {
        path: "index.m3u8",
        plaintext_size: 5,
        ciphertext_size: bytes.length + 1,
        nonce_b64u: "AAECAwQFBgcICQoL",
        ciphertext_sha256: sha256Hex(bytes),
      },
    ],
    lineage_id: "l1",
    manifest_version: "p2p-manifest-v1",
    publication_id: "pub-ec4",
  };
  const manifestJson = JSON.stringify(manifest);
  writeFileSync(join(packageDir, "manifest.json"), manifestJson);
  const manifestDigest = sha256Hex(Buffer.from(manifestJson));

  const result = await verifyPackage(root, "pub-ec4", "l1", manifestDigest);

  assert.equal(result.ok, false);
  assert.equal(result.reason, "package_invalid");
});

test("EC-5: corrupt (non-JSON) manifest bytes return package_invalid without throwing", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec5-"));
  const packageDir = join(root, "pub-ec5");
  mkdirSync(packageDir, { recursive: true });
  const garbage = Buffer.from("{not valid json");
  writeFileSync(join(packageDir, "manifest.json"), garbage);
  const manifestDigest = sha256Hex(garbage);

  const result = await verifyPackage(root, "pub-ec5", "l1", manifestDigest);

  assert.equal(result.ok, false);
  assert.equal(result.reason, "package_invalid");
});

test("EC-6: a package_ref containing a parent-directory traversal segment is rejected as package_invalid", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec6-"));
  writePackage(root, "victim");

  const result = await verifyPackage(root, "../victim", "l1", "0".repeat(64));

  assert.equal(result.ok, false);
  assert.equal(result.reason, "package_invalid");
});

test("EC-7: a manifest listing a file path that escapes the package directory via '..' is rejected as package_invalid", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec7-"));
  const packageDir = join(root, "pub-ec7");
  mkdirSync(packageDir, { recursive: true });

  const manifest = {
    asset_id: "a",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: [
      {
        path: "../escape.ts",
        plaintext_size: 1,
        ciphertext_size: 1,
        nonce_b64u: "x",
        ciphertext_sha256: "0".repeat(64),
      },
    ],
    lineage_id: "l1",
    manifest_version: "p2p-manifest-v1",
    publication_id: "pub-ec7",
  };
  const manifestJson = JSON.stringify(manifest);
  writeFileSync(join(packageDir, "manifest.json"), manifestJson);
  const manifestDigest = sha256Hex(Buffer.from(manifestJson));

  const result = await verifyPackage(root, "pub-ec7", "l1", manifestDigest);

  assert.equal(result.ok, false);
  assert.equal(result.reason, "package_invalid");
});

test("EC-8: a symlinked package directory escaping the root is rejected as package_invalid", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec8-"));
  const outsideDir = mkdtempSync(join(tmpdir(), "pkg-verify-ec8-outside-"));
  writePackage(outsideDir, "real-pub");

  symlinkSync(join(outsideDir, "real-pub"), join(root, "linked-pub"));

  const result = await verifyPackage(root, "linked-pub", "l1", "0".repeat(64));

  assert.equal(result.ok, false);
  assert.equal(result.reason, "package_invalid");
});

test("EC-9: verifyFileAgainstManifest returns false for a missing file rather than throwing", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec9-"));
  const missingPath = join(root, "does-not-exist.ts");

  const result = await verifyFileAgainstManifest(missingPath, {
    path: "does-not-exist.ts",
    plaintext_size: 1,
    ciphertext_size: 1,
    nonce_b64u: "x",
    ciphertext_sha256: "0".repeat(64),
  });

  assert.equal(result, false);
});

test("EC-10: decodeManifest rejects a manifest missing a required field", () => {
  const manifest = decodeManifest({
    asset_id: "a",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: [],
    lineage_id: "l",
    manifest_version: "p2p-manifest-v1",
    // publication_id missing
  });

  assert.equal(manifest, null);
});

test("EC-11: decodeManifest rejects a wrong manifest_version", () => {
  const manifest = decodeManifest({
    asset_id: "a",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: [{ path: "a.ts", plaintext_size: 1, ciphertext_size: 1, nonce_b64u: "x", ciphertext_sha256: "0".repeat(64) }],
    lineage_id: "l",
    manifest_version: "p2p-manifest-v0",
    publication_id: "p",
  });

  assert.equal(manifest, null);
});

test("EC-13: a ciphertext file that becomes unreadable after the stat check returns package_invalid rather than throwing", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec13-"));
  const { manifestDigest, lineageId } = writePackage(root, "pub-ec13");
  const filePath = join(root, "pub-ec13", "index.m3u8");

  chmodSync(filePath, 0o000);
  try {
    const result = await verifyPackage(root, "pub-ec13", lineageId, manifestDigest);

    assert.equal(result.ok, false);
    assert.equal(result.reason, "package_invalid");
  } finally {
    chmodSync(filePath, 0o644);
  }
});

test("EC-14: a manifest file path containing a backslash is rejected as package_invalid", async () => {
  const root = mkdtempSync(join(tmpdir(), "pkg-verify-ec14-"));
  const packageDir = join(root, "pub-ec14");
  mkdirSync(packageDir, { recursive: true });

  const manifest = {
    asset_id: "a",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: [
      {
        path: "sub\\evil.ts",
        plaintext_size: 1,
        ciphertext_size: 1,
        nonce_b64u: "x",
        ciphertext_sha256: "0".repeat(64),
      },
    ],
    lineage_id: "l1",
    manifest_version: "p2p-manifest-v1",
    publication_id: "pub-ec14",
  };
  const manifestJson = JSON.stringify(manifest);
  writeFileSync(join(packageDir, "manifest.json"), manifestJson);
  const manifestDigest = sha256Hex(Buffer.from(manifestJson));

  const result = await verifyPackage(root, "pub-ec14", "l1", manifestDigest);

  assert.equal(result.ok, false);
  assert.equal(result.reason, "package_invalid");
});

test("EC-12: decodeManifest rejects duplicate file paths within one manifest", () => {
  const entry = { path: "a.ts", plaintext_size: 1, ciphertext_size: 1, nonce_b64u: "x", ciphertext_sha256: "0".repeat(64) };
  const manifest = decodeManifest({
    asset_id: "a",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    files: [entry, entry],
    lineage_id: "l",
    manifest_version: "p2p-manifest-v1",
    publication_id: "p",
  });

  assert.equal(manifest, null);
});
