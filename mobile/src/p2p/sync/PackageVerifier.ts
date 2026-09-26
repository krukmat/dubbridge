import b4a from "b4a";
import type { P2pReadyDescriptor } from "../../api/p2p";

export type P2pManifestFile = Readonly<{
  ciphertext_sha256: string;
  ciphertext_size: number;
  nonce_b64u: string;
  path: string;
  plaintext_size: number;
}>;

export type P2pManifest = Readonly<{
  asset_id: string;
  cipher: "AES-256-GCM";
  digest: "SHA-256";
  files: readonly P2pManifestFile[];
  lineage_id: string;
  manifest_version: "p2p-manifest-v1";
  publication_id: string;
}>;

export type ManifestVerificationReceipt = Readonly<{
  manifest: P2pManifest;
  manifestDigestSha256: string;
}>;

export type PackageVerificationReceipt = ManifestVerificationReceipt & Readonly<{
  filesVerified: number;
  bytesVerified: number;
}>;

export type Sha256Hex = (bytes: Uint8Array) => Promise<string>;
export type ReadCiphertextFile = (path: string) => Promise<Uint8Array>;

export class PackageVerificationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "PackageVerificationError";
  }
}

export async function verifyManifestAgainstDescriptor(
  descriptor: P2pReadyDescriptor,
  manifestBytes: Uint8Array,
  sha256Hex: Sha256Hex,
): Promise<ManifestVerificationReceipt> {
  const manifestDigest = normalizeDigest(await sha256Hex(manifestBytes));
  assertManifestDigest(descriptor, manifestDigest);
  const manifest = parseManifest(manifestBytes);
  assertManifestIdentity(descriptor, manifest);
  assertManifestProfile(manifest);
  validateManifestFiles(manifest.files);
  return { manifest, manifestDigestSha256: manifestDigest };
}

/**
 * Fail-closed package verifier used by the product sync path. READY must only
 * be reached after this succeeds against the authoritative API descriptor.
 */
export async function verifyP2pPackage(
  descriptor: P2pReadyDescriptor,
  manifestBytes: Uint8Array,
  readCiphertext: ReadCiphertextFile,
  sha256Hex: Sha256Hex,
): Promise<PackageVerificationReceipt> {
  const verifiedManifest = await verifyManifestAgainstDescriptor(descriptor, manifestBytes, sha256Hex);
  let bytesVerified = manifestBytes.byteLength;
  for (const file of verifiedManifest.manifest.files) {
    const ciphertext = await readVerifiedCiphertext(file, readCiphertext, sha256Hex);
    bytesVerified += ciphertext.byteLength;
  }

  return {
    ...verifiedManifest,
    filesVerified: verifiedManifest.manifest.files.length + 1,
    bytesVerified,
  };
}

export function parseManifest(bytes: Uint8Array): P2pManifest {
  const value = parseManifestJson(bytes);
  assertManifestShape(value);
  const files = value.files.map((file) => parseManifestFile(file));
  return {
    asset_id: value.asset_id as string,
    cipher: value.cipher as "AES-256-GCM",
    digest: value.digest as "SHA-256",
    files,
    lineage_id: value.lineage_id as string,
    manifest_version: value.manifest_version as "p2p-manifest-v1",
    publication_id: value.publication_id as string,
  };
}

function assertManifestDigest(descriptor: P2pReadyDescriptor, actual: string): void {
  if (actual !== descriptor.manifestDigestSha256) {
    throw new PackageVerificationError("Manifest digest does not match authoritative descriptor");
  }
}

function assertManifestIdentity(descriptor: P2pReadyDescriptor, manifest: P2pManifest): void {
  const expected = [descriptor.manifestVersion, descriptor.assetId, descriptor.publicationId, descriptor.lineageId];
  const actual = [manifest.manifest_version, manifest.asset_id, manifest.publication_id, manifest.lineage_id];
  if (actual.some((value, index) => value !== expected[index])) {
    throw new PackageVerificationError("Manifest identity does not match authoritative descriptor");
  }
}

function assertManifestProfile(manifest: P2pManifest): void {
  if (manifest.cipher !== "AES-256-GCM" || manifest.digest !== "SHA-256") {
    throw new PackageVerificationError("Manifest cryptographic profile is unsupported");
  }
}

function validateManifestFiles(files: readonly P2pManifestFile[]): void {
  if (files.length === 0) {
    throw new PackageVerificationError("Manifest contains no ciphertext files");
  }
  const paths = new Set<string>();
  for (const file of files) {
    validateManifestFile(file);
    if (paths.has(file.path)) {
      throw new PackageVerificationError("Manifest contains duplicate file paths");
    }
    paths.add(file.path);
  }
}

async function readVerifiedCiphertext(
  file: P2pManifestFile,
  readCiphertext: ReadCiphertextFile,
  sha256Hex: Sha256Hex,
): Promise<Uint8Array> {
  const ciphertext = await readCiphertextOrThrow(file.path, readCiphertext);
  if (ciphertext.byteLength !== file.ciphertext_size) {
    throw new PackageVerificationError(`Ciphertext size mismatch: ${file.path}`);
  }
  const ciphertextDigest = normalizeDigest(await sha256Hex(ciphertext));
  if (ciphertextDigest !== file.ciphertext_sha256) {
    throw new PackageVerificationError(`Ciphertext digest mismatch: ${file.path}`);
  }
  return ciphertext;
}

async function readCiphertextOrThrow(
  path: string,
  readCiphertext: ReadCiphertextFile,
): Promise<Uint8Array> {
  try {
    return await readCiphertext(path);
  } catch {
    throw new PackageVerificationError(`Ciphertext file is missing: ${path}`);
  }
}

function parseManifestJson(bytes: Uint8Array): unknown {
  try {
    return JSON.parse(b4a.toString(bytes, "utf8"));
  } catch {
    throw new PackageVerificationError("Manifest is not valid UTF-8 JSON");
  }
}

function assertManifestShape(value: unknown): asserts value is Record<string, unknown> & { files: unknown[] } {
  if (!isRecord(value) || !Array.isArray(value.files)) {
    throw new PackageVerificationError("Manifest shape is invalid");
  }
  const requiredStrings = [
    value.asset_id,
    value.cipher,
    value.digest,
    value.lineage_id,
    value.manifest_version,
    value.publication_id,
  ];
  if (requiredStrings.some((field) => typeof field !== "string" || field.length === 0)) {
    throw new PackageVerificationError("Manifest identity/profile fields are invalid");
  }
}

function parseManifestFile(value: unknown): P2pManifestFile {
  if (!isManifestFileRecord(value)) {
    throw new PackageVerificationError("Manifest file entry is invalid");
  }
  const file: P2pManifestFile = {
    path: value.path,
    nonce_b64u: value.nonce_b64u,
    ciphertext_sha256: normalizeDigest(value.ciphertext_sha256),
    ciphertext_size: value.ciphertext_size,
    plaintext_size: value.plaintext_size,
  };
  validateManifestFile(file);
  return file;
}

function isManifestFileRecord(value: unknown): value is Record<string, unknown> & {
  path: string;
  nonce_b64u: string;
  ciphertext_sha256: string;
  ciphertext_size: number;
  plaintext_size: number;
} {
  if (!isRecord(value)) return false;
  return (
    typeof value.path === "string" &&
    typeof value.nonce_b64u === "string" &&
    typeof value.ciphertext_sha256 === "string" &&
    typeof value.ciphertext_size === "number" &&
    typeof value.plaintext_size === "number"
  );
}

function validateManifestFile(file: P2pManifestFile): void {
  validateRelativePath(file.path);
  if (!/^[0-9a-f]{64}$/.test(file.ciphertext_sha256)) {
    throw new PackageVerificationError(`Invalid ciphertext digest: ${file.path}`);
  }
  if (!isNonNegativeSafeInteger(file.ciphertext_size) || file.ciphertext_size === 0) {
    throw new PackageVerificationError(`Invalid ciphertext size: ${file.path}`);
  }
  if (!isNonNegativeSafeInteger(file.plaintext_size)) {
    throw new PackageVerificationError(`Invalid plaintext size: ${file.path}`);
  }
  if (!/^[A-Za-z0-9_-]+$/.test(file.nonce_b64u)) {
    throw new PackageVerificationError(`Invalid nonce encoding: ${file.path}`);
  }
}

export function validateRelativePath(path: string): void {
  if (!path || path.startsWith("/") || path.includes("\\")) {
    throw new PackageVerificationError("Package path is not a safe relative path");
  }
  for (const segment of path.split("/")) {
    if (!segment || segment === "." || segment === "..") {
      throw new PackageVerificationError("Package path contains unsafe segments");
    }
  }
}

function normalizeDigest(value: string): string {
  const digest = value.toLowerCase();
  if (!/^[0-9a-f]{64}$/.test(digest)) {
    throw new PackageVerificationError("SHA-256 digest is invalid");
  }
  return digest;
}

function isNonNegativeSafeInteger(value: number): boolean {
  return Number.isSafeInteger(value) && value >= 0;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
