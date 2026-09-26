// Independent verification of a referenced ciphertext package against its
// canonical manifest, for the case where no upstream caller (T4c, not yet
// built) has already validated the package before handing this node a
// publication request. Scope note (P2.T3c-S4-d1/d2): this module owns
// manifest decode, per-file ciphertext hash/size verification, and
// containment — it does not open Hyperdrive, persist anything, or decide
// HTTP status; callers map its result to the wire contract.

import * as fs from "node:fs/promises";
import * as crypto from "node:crypto";
import { verifyContainedRealpath, ContainmentError } from "./containment.js";

const MANIFEST_FILE_NAME = "manifest.json";
const MANIFEST_VERSION = "p2p-manifest-v1";
const SHA256_HEX_REGEX = /^[0-9a-f]{64}$/;

export interface ManifestFile {
  readonly path: string;
  readonly plaintext_size: number;
  readonly ciphertext_size: number;
  readonly nonce_b64u: string;
  readonly ciphertext_sha256: string;
}

export interface Manifest {
  readonly asset_id: string;
  readonly cipher: string;
  readonly digest: string;
  readonly files: readonly ManifestFile[];
  readonly lineage_id: string;
  readonly manifest_version: string;
  readonly publication_id: string;
}

export type PackageVerificationFailure =
  | "package_invalid"
  | "publication_conflict";

export interface VerifiedPackageFile {
  readonly path: string;
  readonly bytes: Buffer;
}

export type PackageVerificationResult =
  | {
      readonly ok: true;
      readonly manifest: Manifest;
      readonly manifestBytes: Buffer;
      readonly files: readonly VerifiedPackageFile[];
    }
  | { readonly ok: false; readonly reason: PackageVerificationFailure };

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function isNonNegativeInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0;
}

function hasValidTopLevelShape(
  obj: Record<string, unknown>
): obj is Record<string, unknown> & {
  asset_id: string;
  cipher: string;
  digest: string;
  lineage_id: string;
  manifest_version: string;
  publication_id: string;
} {
  return (
    isNonEmptyString(obj.asset_id) &&
    isNonEmptyString(obj.cipher) &&
    isNonEmptyString(obj.digest) &&
    isNonEmptyString(obj.lineage_id) &&
    isNonEmptyString(obj.manifest_version) &&
    isNonEmptyString(obj.publication_id)
  );
}

// Rejects absolute paths, backslashes, and `.`/`..` segments up front —
// verifyContainedRealpath still performs the authoritative filesystem
// containment/symlink check per file later, but a manifest listing an
// unsafe path is itself a malformed manifest, not merely an escape attempt
// discovered late.
function isSafeRelativePath(path: string): boolean {
  if (path.startsWith("/") || path.includes("\\")) {
    return false;
  }
  return path.split("/").every((seg) => seg !== "." && seg !== ".." && seg !== "");
}

function hasValidFileShape(
  f: Record<string, unknown>
): f is Record<string, unknown> & {
  path: string;
  nonce_b64u: string;
  ciphertext_sha256: string;
  plaintext_size: number;
  ciphertext_size: number;
} {
  return (
    isNonEmptyString(f.path) &&
    isNonEmptyString(f.nonce_b64u) &&
    isNonEmptyString(f.ciphertext_sha256) &&
    isNonNegativeInteger(f.plaintext_size) &&
    isNonNegativeInteger(f.ciphertext_size)
  );
}

function decodeManifestFile(rawFile: unknown, seenPaths: Set<string>): ManifestFile | null {
  if (rawFile === null || typeof rawFile !== "object" || Array.isArray(rawFile)) {
    return null;
  }
  const f = rawFile as Record<string, unknown>;

  if (!hasValidFileShape(f)) {
    return null;
  }
  if (!SHA256_HEX_REGEX.test(f.ciphertext_sha256)) {
    return null;
  }
  if (!isSafeRelativePath(f.path)) {
    return null;
  }
  if (seenPaths.has(f.path)) {
    return null;
  }
  seenPaths.add(f.path);

  return {
    path: f.path,
    plaintext_size: f.plaintext_size,
    ciphertext_size: f.ciphertext_size,
    nonce_b64u: f.nonce_b64u,
    ciphertext_sha256: f.ciphertext_sha256,
  };
}

/**
 * Decodes and structurally validates a manifest object already parsed from
 * JSON. Does not touch the filesystem or check digests — pure shape
 * validation against the frozen `p2p-manifest-v1` contract
 * (`crates/p2p/src/manifest.rs::Manifest`/`ManifestFile`).
 */
export function decodeManifest(input: unknown): Manifest | null {
  if (input === null || typeof input !== "object" || Array.isArray(input)) {
    return null;
  }
  const obj = input as Record<string, unknown>;

  if (!hasValidTopLevelShape(obj)) {
    return null;
  }
  if (obj.manifest_version !== MANIFEST_VERSION) {
    return null;
  }
  if (!Array.isArray(obj.files) || obj.files.length === 0) {
    return null;
  }

  const files: ManifestFile[] = [];
  const seenPaths = new Set<string>();
  for (const rawFile of obj.files) {
    const file = decodeManifestFile(rawFile, seenPaths);
    if (file === null) {
      return null;
    }
    files.push(file);
  }

  return {
    asset_id: obj.asset_id,
    cipher: obj.cipher,
    digest: obj.digest,
    files,
    lineage_id: obj.lineage_id,
    manifest_version: obj.manifest_version,
    publication_id: obj.publication_id,
  };
}

/**
 * Mechanical, per-file check: does the ciphertext on disk at `filePath`
 * match the manifest's declared size and SHA-256 hash exactly? No manifest
 * structure, no containment — a pure content check against one already-
 * resolved, already-contained path. Returns the read bytes on success so
 * callers that need to persist exactly the bytes that were verified (e.g.
 * writing into a Hyperdrive) never have to re-read the file and reopen a
 * verify-then-use TOCTOU window.
 */
async function readAndVerifyFileBytes(
  filePath: string,
  expected: ManifestFile
): Promise<Buffer | null> {
  let stat;
  try {
    stat = await fs.stat(filePath);
  } catch {
    return null;
  }
  if (!stat.isFile() || stat.size !== expected.ciphertext_size) {
    return null;
  }

  let bytes: Buffer;
  try {
    bytes = await fs.readFile(filePath);
  } catch {
    return null;
  }
  const actualHash = crypto.createHash("sha256").update(bytes).digest("hex");
  return actualHash === expected.ciphertext_sha256 ? bytes : null;
}

export async function verifyFileAgainstManifest(
  filePath: string,
  expected: ManifestFile
): Promise<boolean> {
  return (await readAndVerifyFileBytes(filePath, expected)) !== null;
}

/**
 * Resolves `packageRef` (the publication's directory name) under
 * `packageRoot`, decodes its `manifest.json`, checks it against the request's
 * expected `lineage_id`/`manifest_digest_sha256`, verifies every listed file's
 * ciphertext hash/size, and verifies every path — including the manifest file
 * itself — stays contained under the package directory (no traversal, no
 * symlink escape).
 *
 * Returns `publication_conflict` only for the one case that is not a
 * malformed/tampered package: the manifest decodes fine and matches its own
 * internal digest, but its `lineage_id` disagrees with what the request
 * declared for the same `publication_id`. Every other failure — missing
 * manifest, bad JSON, structural mismatch, digest mismatch, size/hash
 * mismatch, containment/symlink escape — is `package_invalid`.
 */
export async function verifyPackage(
  packageRoot: string,
  packageRef: string,
  expectedLineageId: string,
  expectedManifestDigestSha256: string
): Promise<PackageVerificationResult> {
  let packageDir: string;
  try {
    packageDir = verifyContainedRealpath(packageRoot, packageRef);
  } catch (err) {
    if (err instanceof ContainmentError) {
      return { ok: false, reason: "package_invalid" };
    }
    throw err;
  }

  let manifestPath: string;
  try {
    manifestPath = verifyContainedRealpath(packageDir, MANIFEST_FILE_NAME);
  } catch (err) {
    if (err instanceof ContainmentError) {
      return { ok: false, reason: "package_invalid" };
    }
    throw err;
  }

  let manifestBytes: Buffer;
  try {
    manifestBytes = await fs.readFile(manifestPath);
  } catch {
    return { ok: false, reason: "package_invalid" };
  }

  const manifestDigest = crypto
    .createHash("sha256")
    .update(manifestBytes)
    .digest("hex");
  if (manifestDigest !== expectedManifestDigestSha256) {
    return { ok: false, reason: "package_invalid" };
  }

  let parsedJson: unknown;
  try {
    parsedJson = JSON.parse(manifestBytes.toString("utf8"));
  } catch {
    return { ok: false, reason: "package_invalid" };
  }

  const manifest = decodeManifest(parsedJson);
  if (manifest === null) {
    return { ok: false, reason: "package_invalid" };
  }

  if (manifest.lineage_id !== expectedLineageId) {
    return { ok: false, reason: "publication_conflict" };
  }

  const files: VerifiedPackageFile[] = [];
  for (const file of manifest.files) {
    let filePath: string;
    try {
      filePath = verifyContainedRealpath(packageDir, file.path);
    } catch (err) {
      if (err instanceof ContainmentError) {
        return { ok: false, reason: "package_invalid" };
      }
      throw err;
    }

    const bytes = await readAndVerifyFileBytes(filePath, file);
    if (bytes === null) {
      return { ok: false, reason: "package_invalid" };
    }
    files.push({ path: file.path, bytes });
  }

  return { ok: true, manifest, manifestBytes, files };
}
