import type { P2pReadyDescriptor } from "../../src/api/p2p";
import { P2PServicePackageSource } from "../../src/p2p/sync/P2PServicePackageSource";

const descriptor: P2pReadyDescriptor = {
  descriptorVersion: "p2p-ready-descriptor-v1",
  assetId: "asset-1",
  publicationId: "pub-1",
  lineageId: "lineage-1",
  manifestVersion: "p2p-manifest-v1",
  manifestDigestSha256: "a".repeat(64),
  externalPublicationId: "b".repeat(64),
  ckWrapRef: "p2p-k1-wrap/pub-1/lineage-1",
  kekId: "kek-1",
  kekVersion: 1,
  readyAt: "2026-09-15T00:00:00Z",
};

function createService() {
  return {
    openProductPackage: jest.fn(async (_id: string) => undefined),
    readProductFile: jest.fn(async (path: string) => new TextEncoder().encode(path)),
    closeProductPackage: jest.fn(async () => undefined),
    cancelProductPackage: jest.fn(async () => undefined),
  };
}

describe("P2PServicePackageSource", () => {
  it("opens the authoritative Hyperdrive identity and maps manifest/files through P2PService", async () => {
    const service = createService();
    const source = new P2PServicePackageSource(service);

    const session = await source.open(descriptor);
    await expect(session.readManifest()).resolves.toEqual(new TextEncoder().encode("manifest.json"));
    await expect(session.readCiphertext("segments/000001.ts")).resolves.toEqual(
      new TextEncoder().encode("segments/000001.ts"),
    );
    await session.close();

    expect(service.openProductPackage).toHaveBeenCalledWith(descriptor.externalPublicationId);
    expect(service.readProductFile).toHaveBeenNthCalledWith(1, "manifest.json");
    expect(service.readProductFile).toHaveBeenNthCalledWith(2, "segments/000001.ts");
    expect(service.closeProductPackage).toHaveBeenCalledTimes(1);
  });

  it("cancels once and prevents stale reads or a later duplicate close", async () => {
    const service = createService();
    const session = await new P2PServicePackageSource(service).open(descriptor);

    await session.cancel?.();
    await session.cancel?.();
    await session.close();

    await expect(session.readCiphertext("index.m3u8")).rejects.toThrow("session is closed");
    expect(service.cancelProductPackage).toHaveBeenCalledTimes(1);
    expect(service.closeProductPackage).not.toHaveBeenCalled();
  });
});
