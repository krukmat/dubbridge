import b4a from "b4a";
import type { P2pReadyDescriptor } from "../src/api/p2p";
import {
  P2pProductSync,
  P2pSyncCancelledError,
  type P2pPackageSource,
} from "../src/p2p/sync/P2pProductSync";
import { MemoryP2pSyncCache } from "../src/p2p/sync/SyncCache";

const accountScope = "viewer-1";
const manifestDigest = "a".repeat(64);
const firstDigest = "b".repeat(64);
const secondDigest = "c".repeat(64);
const first = b4a.from("first-ciphertext");
const second = b4a.from("second-ciphertext");
const manifest = {
  asset_id: "asset-1",
  cipher: "AES-256-GCM",
  digest: "SHA-256",
  files: [
    {
      ciphertext_sha256: firstDigest,
      ciphertext_size: first.byteLength,
      nonce_b64u: "AAECAwQFBgcICQoL",
      path: "index.m3u8",
      plaintext_size: 8,
    },
    {
      ciphertext_sha256: secondDigest,
      ciphertext_size: second.byteLength,
      nonce_b64u: "DA0ODxAREhMUFRYX",
      path: "segments/000001.ts",
      plaintext_size: 16,
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
  manifestDigestSha256: manifestDigest,
  externalPublicationId: "external-1",
  ckWrapRef: "p2p-k1-wrap/pub-1/lineage-1",
  kekId: "kek-1",
  kekVersion: 1,
  readyAt: "2026-09-15T00:00:00Z",
};

const sha256 = jest.fn(async (bytes: Uint8Array) => {
  if (b4a.equals(bytes, manifestBytes)) return manifestDigest;
  if (b4a.equals(bytes, first)) return firstDigest;
  if (b4a.equals(bytes, second)) return secondDigest;
  return "d".repeat(64);
});

function source(
  readCiphertext = jest.fn(async (path: string) => (path === "index.m3u8" ? first : second)),
) {
  const close = jest.fn(async () => undefined);
  const cancel = jest.fn(async () => undefined);
  const open = jest.fn(async () => ({
    readManifest: async () => manifestBytes,
    readCiphertext,
    cancel,
    close,
  }));
  return { value: { open } as P2pPackageSource, open, readCiphertext, cancel, close };
}

describe("P2P product sync", () => {
  beforeEach(() => sha256.mockClear());

  it("copies the complete encrypted package and only then becomes READY", async () => {
    const cache = new MemoryP2pSyncCache();
    const transport = source();
    const phases: string[] = [];
    const sync = new P2pProductSync(cache, transport.value, sha256, (state) =>
      phases.push(state.phase),
    );

    const state = await sync.sync(descriptor, accountScope);

    expect(state.phase).toBe("READY");
    expect(state.manifestVerified).toBe(true);
    expect(state.packageVerified).toBe(true);
    expect(state.progress.filesCompleted).toBe(3);
    expect(state.progress.filesCompleted).toBe(state.progress.totalFiles);
    expect(phases).toEqual([
      "DISCOVERING",
      "DOWNLOADING",
      "DOWNLOADING",
      "DOWNLOADING",
      "VERIFYING",
      "READY",
    ]);
    expect(transport.close).toHaveBeenCalledTimes(1);
  });

  it("reuses already verified ciphertext on restart instead of downloading it twice", async () => {
    const cache = new MemoryP2pSyncCache();
    const identity = { accountScope, publicationId: "pub-1", lineageId: "lineage-1" };
    await cache.writeCiphertext(identity, "index.m3u8", first);
    const transport = source();
    const sync = new P2pProductSync(cache, transport.value, sha256);

    await sync.sync(descriptor, accountScope);

    expect(transport.readCiphertext).toHaveBeenCalledTimes(1);
    expect(transport.readCiphertext).toHaveBeenCalledWith("segments/000001.ts");
  });

  it("never marks READY when source ciphertext fails digest verification", async () => {
    const cache = new MemoryP2pSyncCache();
    const corrupt = b4a.from("corrupt");
    const transport = source(jest.fn(async (_path: string) => corrupt));
    const sync = new P2pProductSync(cache, transport.value, sha256);

    await expect(sync.sync(descriptor, accountScope)).rejects.toThrow("Ciphertext");
    const state = await cache.readSnapshot({
      accountScope,
      publicationId: "pub-1",
      lineageId: "lineage-1",
    });
    expect(state?.phase).toBe("FAILED");
    expect(state?.packageVerified).toBe(false);
  });

  it("returns an already verified READY snapshot without reopening the network source", async () => {
    const cache = new MemoryP2pSyncCache();
    const firstTransport = source();
    await new P2pProductSync(cache, firstTransport.value, sha256).sync(descriptor, accountScope);

    const secondTransport = source();
    const state = await new P2pProductSync(cache, secondTransport.value, sha256).sync(
      descriptor,
      accountScope,
    );

    expect(state.phase).toBe("READY");
    expect(secondTransport.open).not.toHaveBeenCalled();
  });

  it("does not reuse READY lifecycle state across accounts", async () => {
    const cache = new MemoryP2pSyncCache();
    await new P2pProductSync(cache, source().value, sha256).sync(descriptor, "viewer-a");

    const viewerBTransport = source();
    const state = await new P2pProductSync(cache, viewerBTransport.value, sha256).sync(
      descriptor,
      "viewer-b",
    );

    expect(state.phase).toBe("READY");
    expect(viewerBTransport.open).toHaveBeenCalledTimes(1);
  });

  it("cancels an active source and prevents an in-flight sync from promoting READY", async () => {
    const cache = new MemoryP2pSyncCache();
    let rejectRead: (error: Error) => void = () => undefined;
    let markReadStarted: () => void = () => undefined;
    const readStarted = new Promise<void>((resolve) => {
      markReadStarted = resolve;
    });
    const blockedRead = new Promise<Uint8Array>((_resolve, reject) => {
      rejectRead = reject;
    });
    const cancel = jest.fn(async () => {
      rejectRead(new Error("transport cancelled"));
    });
    const close = jest.fn(async () => undefined);
    const transport: P2pPackageSource = {
      open: jest.fn(async () => ({
        readManifest: async () => manifestBytes,
        readCiphertext: async () => {
          markReadStarted();
          return blockedRead;
        },
        cancel,
        close,
      })),
    };
    const sync = new P2pProductSync(cache, transport, sha256);
    const identity = { accountScope, publicationId: "pub-1", lineageId: "lineage-1" } as const;

    const pending = sync.sync(descriptor, accountScope);
    await readStarted;
    const cancelled = await sync.cancel(identity);

    await expect(pending).rejects.toBeInstanceOf(P2pSyncCancelledError);
    expect(cancelled.phase).toBe("CANCELLED");
    expect((await cache.readSnapshot(identity))?.phase).toBe("CANCELLED");
    expect(cancel).toHaveBeenCalledTimes(1);
    expect(close).toHaveBeenCalledTimes(1);
  });

  it("clears only the signed-out account cache", async () => {
    const cache = new MemoryP2pSyncCache();
    const viewerA = { accountScope: "viewer-a", publicationId: "pub-1", lineageId: "lineage-1" };
    const viewerB = { accountScope: "viewer-b", publicationId: "pub-1", lineageId: "lineage-1" };
    await cache.writeCiphertext(viewerA, "index.m3u8", first);
    await cache.writeCiphertext(viewerB, "index.m3u8", second);
    const sync = new P2pProductSync(cache, source().value, sha256);

    await sync.clearAccount("viewer-a");

    expect(await cache.readCiphertext(viewerA, "index.m3u8")).toBeNull();
    expect(await cache.readCiphertext(viewerB, "index.m3u8")).toEqual(second);
  });
});
