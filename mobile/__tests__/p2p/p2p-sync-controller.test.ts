import { createHash } from "node:crypto";

import type { P2pReadyDescriptor } from "../../src/api/p2p";
import { P2PService } from "../../src/p2p/P2PService";
import { P2PSyncController } from "../../src/p2p/sync/P2PSyncController";
import { MemoryP2pSyncCache } from "../../src/p2p/sync/SyncCache";

const ciphertext = new TextEncoder().encode("encrypted-segment");
const ciphertextDigest = digest(ciphertext);
const manifest = {
  asset_id: "asset-1",
  cipher: "AES-256-GCM",
  digest: "SHA-256",
  files: [
    {
      ciphertext_sha256: ciphertextDigest,
      ciphertext_size: ciphertext.byteLength,
      nonce_b64u: "AAECAwQFBgcICQoL",
      path: "segments/000001.ts",
      plaintext_size: 16,
    },
  ],
  lineage_id: "lineage-1",
  manifest_version: "p2p-manifest-v1",
  publication_id: "pub-1",
} as const;
const manifestBytes = new TextEncoder().encode(JSON.stringify(manifest));
const manifestDigest = digest(manifestBytes);

const descriptor: P2pReadyDescriptor = {
  descriptorVersion: "p2p-ready-descriptor-v1",
  assetId: "asset-1",
  publicationId: "pub-1",
  lineageId: "lineage-1",
  manifestVersion: "p2p-manifest-v1",
  manifestDigestSha256: manifestDigest,
  externalPublicationId: "a".repeat(64),
  ckWrapRef: "p2p-k1-wrap/pub-1/lineage-1",
  kekId: "kek-1",
  kekVersion: 1,
  readyAt: "2026-09-15T00:00:00Z",
};

function digest(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function runtime() {
  let state: "stopped" | "ready" = "stopped";
  return {
    get currentState() {
      return state;
    },
    initialize: jest.fn(async () => {
      state = "ready";
      return {
        protocolVersion: 1 as const,
        runtimeVersion: "test",
        capabilities: [
          "ping",
          "lifecycle:suspend",
          "lifecycle:resume",
          "fatal",
          "shutdown",
          "product-package:v1",
        ] as const,
      };
    }),
    ping: jest.fn(async () => "pong" as const),
    openProductPackage: jest.fn(async (_accountScope: string, _id: string) => undefined),
    readProductFile: jest.fn(async (path: string) => {
      if (path === "manifest.json") return manifestBytes;
      if (path === "segments/000001.ts") return ciphertext;
      throw new Error("missing product file");
    }),
    hashProductBytes: jest.fn(async (bytes: Uint8Array) => digest(bytes)),
    closeProductPackage: jest.fn(async () => undefined),
    cancelProductPackage: jest.fn(async () => undefined),
    shutdown: jest.fn(async () => {
      state = "stopped";
    }),
  };
}

describe("P4 product sync controller", () => {
  it("explicitly starts the runtime, verifies the whole package, and emits a narrow P5 handle", async () => {
    const cache = new MemoryP2pSyncCache();
    const productRuntime = runtime();
    const controller = new P2PSyncController(new P2PService(productRuntime), cache);

    const state = await controller.startSync(descriptor, "viewer-1");
    const handle = await controller.getVerifiedPackageHandle(descriptor, "viewer-1");

    expect(state).toMatchObject({ phase: "READY", manifestVerified: true, packageVerified: true });
    expect(handle).toEqual({
      accountScope: "viewer-1",
      publicationId: "pub-1",
      lineageId: "lineage-1",
      manifestDigestSha256: manifestDigest,
    });
    expect(Object.keys(handle).sort()).toEqual([
      "accountScope",
      "lineageId",
      "manifestDigestSha256",
      "publicationId",
    ]);
    expect(productRuntime.initialize).toHaveBeenCalledTimes(1);
    expect(productRuntime.openProductPackage).toHaveBeenCalledWith("viewer-1", descriptor.externalPublicationId);
    expect(productRuntime.hashProductBytes).toHaveBeenCalled();
    expect(productRuntime.closeProductPackage).toHaveBeenCalledTimes(1);
  });

  it("does not create a P5 handle from transport identity without verified local state", async () => {
    const controller = new P2PSyncController(new P2PService(runtime()), new MemoryP2pSyncCache());

    await expect(controller.getVerifiedPackageHandle(descriptor, "viewer-1")).rejects.toThrow(
      "no local sync state",
    );
  });

  it("keeps verified state account-scoped and wipes only the signed-out account", async () => {
    const cache = new MemoryP2pSyncCache();
    const controller = new P2PSyncController(new P2PService(runtime()), cache);
    await controller.startSync(descriptor, "viewer-a");

    await expect(controller.getVerifiedPackageHandle(descriptor, "viewer-b")).rejects.toThrow(
      "no local sync state",
    );
    await controller.clearAccount("viewer-a");
    await expect(controller.getVerifiedPackageHandle(descriptor, "viewer-a")).rejects.toThrow(
      "no local sync state",
    );
  });
});
