jest.mock("bare-crypto", () => require("node:crypto"));

import { createCipheriv, createHash } from "node:crypto";

import {
  decryptProductFile,
  parseRangeHeader,
  rewriteHlsManifest,
} from "../../src/p2p/runtime/product-playback-runtime";

function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function encryptFixture(path: string, plaintext: Uint8Array) {
  const key = Buffer.alloc(32, 7);
  const nonce = Buffer.from("000102030405060708090a0b", "hex");
  const manifestBase = {
    asset_id: "asset-1",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    lineage_id: "lineage-1",
    manifest_version: "p2p-manifest-v1",
    publication_id: "publication-1",
  };
  const aad = JSON.stringify({
    aad_version: "p2p-aad-v1",
    asset_id: manifestBase.asset_id,
    lineage_id: manifestBase.lineage_id,
    manifest_version: manifestBase.manifest_version,
    path,
    publication_id: manifestBase.publication_id,
  });
  const cipher = createCipheriv("aes-256-gcm", key, nonce);
  cipher.setAAD(Buffer.from(aad));
  const encrypted = Buffer.concat([cipher.update(plaintext), cipher.final(), cipher.getAuthTag()]);
  const file = {
    path,
    plaintext_size: plaintext.byteLength,
    ciphertext_size: encrypted.byteLength,
    nonce_b64u: nonce.toString("base64url"),
    ciphertext_sha256: sha256(encrypted),
  };
  return { key, encrypted, file, manifestBase };
}

describe("P5 decrypt-on-read runtime", () => {
  it("decrypts the exact P2 AES-256-GCM/AAD layout and rejects corruption", () => {
    const plaintext = Buffer.from("segment payload");
    const fixture = encryptFixture("segments/000001.ts", plaintext);
    const manifest = { ...fixture.manifestBase, files: [fixture.file] };

    expect(Buffer.from(decryptProductFile(fixture.key, manifest, fixture.file, fixture.encrypted))).toEqual(plaintext);

    const corrupted = Buffer.from(fixture.encrypted);
    corrupted[0] ^= 1;
    expect(() => decryptProductFile(fixture.key, manifest, fixture.file, corrupted)).toThrow(
      "integrity check failed",
    );
  });

  it("rewrites HLS segment references to the randomized loopback session", () => {
    const token = "a".repeat(32);
    const manifest = {
      asset_id: "asset-1",
      cipher: "AES-256-GCM",
      digest: "SHA-256",
      lineage_id: "lineage-1",
      manifest_version: "p2p-manifest-v1",
      publication_id: "publication-1",
      files: [
        { path: "index.m3u8", plaintext_size: 1, ciphertext_size: 17, nonce_b64u: "AAECAwQFBgcICQoL", ciphertext_sha256: "a".repeat(64) },
        { path: "segments/000001.ts", plaintext_size: 1, ciphertext_size: 17, nonce_b64u: "AAECAwQFBgcICQoL", ciphertext_sha256: "b".repeat(64) },
      ],
    };
    const source = Buffer.from("#EXTM3U\n#EXTINF:4,\nprepared/asset/000001.ts\n#EXT-X-ENDLIST\n");
    const rewritten = Buffer.from(rewriteHlsManifest(source, manifest, token)).toString();

    expect(rewritten).toContain(`/${token}/segments/000001.ts`);
    expect(rewritten).not.toContain("prepared/asset/000001.ts");
  });

  it("supports bounded single byte ranges for expo-video seek requests", () => {
    expect(parseRangeHeader(null, 100)).toBeNull();
    expect(parseRangeHeader("bytes=10-19", 100)).toEqual({ start: 10, end: 19 });
    expect(parseRangeHeader("bytes=90-", 100)).toEqual({ start: 90, end: 99 });
    expect(() => parseRangeHeader("bytes=100-120", 100)).toThrow("byte range is invalid");
  });
});
