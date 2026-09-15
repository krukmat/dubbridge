import type { P2pReadyDescriptor } from "../../api/p2p";
import {
  PackageVerificationError,
  verifyManifestAgainstDescriptor,
  verifyP2pPackage,
  type Sha256Hex,
} from "./PackageVerifier";
import type { P2pSyncCache } from "./SyncCache";
import {
  createSyncSnapshot,
  transitionSync,
  type P2pSyncIdentity,
  type P2pSyncSnapshot,
} from "./SyncState";

export interface P2pPackageSourceSession {
  readManifest(): Promise<Uint8Array>;
  readCiphertext(path: string): Promise<Uint8Array>;
  close(): Promise<void>;
}

export interface P2pPackageSource {
  open(descriptor: P2pReadyDescriptor): Promise<P2pPackageSourceSession>;
}

export type SyncProgressObserver = (snapshot: P2pSyncSnapshot) => void;

/**
 * Product P2P sync: complete ciphertext package copy first, then full integrity
 * verification, then READY. No playback/progressive-read path is exposed here.
 */
export class P2pProductSync {
  constructor(
    private readonly cache: P2pSyncCache,
    private readonly source: P2pPackageSource,
    private readonly sha256Hex: Sha256Hex,
    private readonly onProgress: SyncProgressObserver = () => undefined,
  ) {}

  async sync(descriptor: P2pReadyDescriptor): Promise<P2pSyncSnapshot> {
    const identity = descriptorIdentity(descriptor);
    let state = (await this.cache.readSnapshot(identity)) ?? createSyncSnapshot(identity);
    if (state.phase === "READY") return state;

    if (state.phase !== "IDLE" && state.phase !== "RETRYING" && state.phase !== "FAILED" && state.phase !== "CANCELLED") {
      state = await this.persist(transitionSync(state, "RETRYING", { lastError: "Resuming interrupted sync" }));
    }
    state = await this.persist(transitionSync(state, "DISCOVERING", { lastError: null }));

    let session: P2pPackageSourceSession | null = null;
    try {
      session = await this.source.open(descriptor);
      const manifestBytes = await session.readManifest();
      const { manifest } = await verifyManifestAgainstDescriptor(descriptor, manifestBytes, this.sha256Hex);
      await this.cache.writeManifest(identity, manifestBytes);

      const totalFiles = manifest.files.length + 1;
      const totalBytes = manifestBytes.byteLength + manifest.files.reduce((sum, file) => sum + file.ciphertext_size, 0);
      state = await this.persist(transitionSync(state, "DOWNLOADING", {
        manifestVerified: true,
        packageVerified: false,
        progress: {
          filesCompleted: 1,
          totalFiles,
          bytesCompleted: manifestBytes.byteLength,
          totalBytes,
        },
      }));

      for (const file of manifest.files) {
        let ciphertext = await this.cache.readCiphertext(identity, file.path);
        const cachedValid = ciphertext !== null &&
          ciphertext.byteLength === file.ciphertext_size &&
          (await this.sha256Hex(ciphertext)).toLowerCase() === file.ciphertext_sha256;
        if (!cachedValid) {
          ciphertext = await session.readCiphertext(file.path);
          if (ciphertext.byteLength !== file.ciphertext_size) {
            throw new PackageVerificationError(`Ciphertext size mismatch: ${file.path}`);
          }
          if ((await this.sha256Hex(ciphertext)).toLowerCase() !== file.ciphertext_sha256) {
            throw new PackageVerificationError(`Ciphertext digest mismatch: ${file.path}`);
          }
          await this.cache.writeCiphertext(identity, file.path, ciphertext);
        }

        state = await this.persist(transitionSync(state, "DOWNLOADING", {
          progress: {
            ...state.progress,
            filesCompleted: state.progress.filesCompleted + 1,
            bytesCompleted: state.progress.bytesCompleted + file.ciphertext_size,
          },
        }));
      }

      state = await this.persist(transitionSync(state, "VERIFYING"));
      const cachedManifest = await this.cache.readManifest(identity);
      if (cachedManifest === null) {
        throw new PackageVerificationError("Cached manifest disappeared before package verification");
      }
      await verifyP2pPackage(
        descriptor,
        cachedManifest,
        async (path) => {
          const bytes = await this.cache.readCiphertext(identity, path);
          if (bytes === null) throw new Error("missing ciphertext");
          return bytes;
        },
        this.sha256Hex,
      );

      state = await this.persist(transitionSync(state, "READY", { packageVerified: true, lastError: null }));
      return state;
    } catch (error) {
      const message = error instanceof Error ? error.message : "P2P sync failed";
      const target = error instanceof PackageVerificationError ? "FAILED" : "RETRYING";
      if (state.phase !== "READY") {
        state = await this.persist(transitionSync(state, target, { lastError: message }));
      }
      throw error;
    } finally {
      await session?.close().catch(() => undefined);
    }
  }

  async cancel(identity: P2pSyncIdentity): Promise<P2pSyncSnapshot> {
    const current = (await this.cache.readSnapshot(identity)) ?? createSyncSnapshot(identity);
    if (current.phase === "READY") return current;
    return this.persist(transitionSync(current, "CANCELLED", { lastError: null }));
  }

  private async persist(snapshot: P2pSyncSnapshot): Promise<P2pSyncSnapshot> {
    await this.cache.writeSnapshot(snapshot);
    this.onProgress(snapshot);
    return snapshot;
  }
}

export function descriptorIdentity(descriptor: P2pReadyDescriptor): P2pSyncIdentity {
  return { publicationId: descriptor.publicationId, lineageId: descriptor.lineageId };
}
