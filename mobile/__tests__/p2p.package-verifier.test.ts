import b4a from "b4a";
import type { P2pReadyDescriptor } from "../src/api/p2p";
import { verifyP2pPackage } from "../src/p2p/sync/PackageVerifier";

const MANIFEST_DIGEST = "a".repeat(64);
const FILE_DIGEST = "b".repeat(64);
const fileBytes = b4a.from("ciphertext");

const manifest = {
  asset_id: "asset-1",
  cipher: "AES-256-GCM",
  digest: "SHA-256",
  files: [
    {
      ciphertext_sha256: FILE_DIGEST,
      ciphertext_size: fileBytes.byteLength,
      nonce_b64u: "AAECAwQFBgcICQoL",
      path: "segments/000001.ts",
      plaintext_size: 4,
    },
  ],
  lineage_id: "lineage-1",
  manifest_version: "p2p-manifest-v1",
  publication_id: "pub-1",
};
const manifestBytes = b4a.from(JSON.stringify(manifest));

const descriptor: P2pReadyDescriptor = {
  descriptorVersion: "p2p-ready-descriptor-v1",
  assetId: "asset-1",
  publicationId: "pub-1",
  lineageId: "lineage-1",
  manifestVersion: "p2p-manifest-v1",
  manifestDigestSha256: MANIFEST_DIGEST,
  externalPublicationId: "external-1",
  ckWrapRef: "p2p-k1-wrap/pub-1/lineage-1",
  kekId: "kek-1",
  kekVersion: 1,
  readyAt: "2026-09-15T00:00:00Z",
};

const sha256 = jest.fn(async (bytes: Uint8Array) => {
  if (b4a.equals(bytes, manifestBytes)) return MANIFEST_DIGEST;
  if (b4a.equals(bytes, fileBytes)) return FILE_DIGEST;
  return "c".repeat(64);
});

describe("P2P package verifier", () => {
  beforeEach(() => sha256.mockClear());

  it("verifies manifest identity plus every ciphertext file", async () => {
    const receipt = await verifyP2pPackage(
      descriptor,
      manifestBytes,
      async (path) => {
        expect(path).toBe("segments/000001.ts");
        return fileBytes;
      },
      sha256,
    );

    expect(receipt.manifestDigestSha256).toBe(MANIFEST_DIGEST);
    expect(receipt.filesVerified).toBe(2);
    expect(receipt.bytesVerified).toBe(manifestBytes.byteLength + fileBytes.byteLength);
  });

  it("fails closed on authoritative manifest digest mismatch", async () => {
    await expect(
      verifyP2pPackage(
        { ...descriptor, manifestDigestSha256: "d".repeat(64) },
        manifestBytes,
        async () => fileBytes,
        sha256,
      ),
    ).rejects.toThrow("Manifest digest does not match");
  });

  it("fails closed on publication or lineage mismatch", async () => {
    await expect(
      verifyP2pPackage(
        { ...descriptor, lineageId: "other-lineage" },
        manifestBytes,
        async () => fileBytes,
        sha256,
      ),
    ).rejects.toThrow("Manifest identity does not match");
  });

  it("fails closed when a ciphertext file is missing", async () => {
    await expect(
      verifyP2pPackage(
        descriptor,
        manifestBytes,
        async () => {
          throw new Error("ENOENT");
        },
        sha256,
      ),
    ).rejects.toThrow("Ciphertext file is missing");
  });

  it("rejects path traversal before reading ciphertext", async () => {
    const unsafeManifest = {
      ...manifest,
      files: [{ ...manifest.files[0], path: "../secret" }],
    };
    const unsafeBytes = b4a.from(JSON.stringify(unsafeManifest));
    const digest = "e".repeat(64);
    const unsafeSha = jest.fn(async (bytes: Uint8Array) =>
      b4a.equals(bytes, unsafeBytes) ? digest : FILE_DIGEST,
    );
    const read = jest.fn(async () => fileBytes);

    await expect(
      verifyP2pPackage(
        { ...descriptor, manifestDigestSha256: digest },
        unsafeBytes,
        read,
        unsafeSha,
      ),
    ).rejects.toThrow("unsafe segments");
    expect(read).not.toHaveBeenCalled();
  });
});
