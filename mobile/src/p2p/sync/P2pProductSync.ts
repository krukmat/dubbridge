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
  packageCacheKey,
  transitionSync,
  type P2pSyncIdentity,
  type P2pSyncSnapshot,
} from "./SyncState";

export interface P2pPackageSourceSession {
  readManifest(): Promise<Uint8Array>;
  readCiphertext(path: string): Promise<Uint8Array>;
  cancel?(): Promise<void>;
  close(): Promise<void>;
}

export interface P2pPackageSource {
  open(descriptor: P2pReadyDescriptor): Promise<P2pPackageSourceSession>;
}

export type SyncProgressObserver = (snapshot: P2pSyncSnapshot) => void;

type ActiveSyncRun = {
  readonly identity: P2pSyncIdentity;
  cancelled: boolean;
  session: P2pPackageSourceSession | null;
  writeTail: Promise<void>;
};

export class P2pSyncCancelledError extends Error {
  constructor() {
    super("P2P sync cancelled");
    this.name = "P2pSyncCancelledError";
  }
}

/**
 * Product P2P sync: complete ciphertext package copy first, then full integrity
 * verification, then READY. No playback/progressive-read path is exposed here.
 * Cache identity is account-scoped so a stale/sign-out session cannot reuse a
 * different account's package lifecycle state.
 */
export class P2pProductSync {
  private readonly activeRuns = new Map<string, ActiveSyncRun>();

  constructor(
    private readonly cache: P2pSyncCache,
    private readonly source: P2pPackageSource,
    private readonly sha256Hex: Sha256Hex,
    private readonly onProgress: SyncProgressObserver = () => undefined,
  ) {}

  async sync(descriptor: P2pReadyDescriptor, accountScope: string): Promise<P2pSyncSnapshot> {
    const identity = descriptorIdentity(descriptor, accountScope);
    const runKey = packageCacheKey(identity);
    if (this.activeRuns.has(runKey)) {
      throw new Error("P2P sync is already active for this account publication lineage");
    }

    const run: ActiveSyncRun = {
      identity,
      cancelled: false,
      session: null,
      writeTail: Promise.resolve(),
    };
    this.activeRuns.set(runKey, run);

    let state = (await this.cache.readSnapshot(identity)) ?? createSyncSnapshot(identity);
    if (state.phase === "READY") {
      this.activeRuns.delete(runKey);
      return state;
    }

    try {
      if (
        state.phase !== "IDLE" &&
        state.phase !== "RETRYING" &&
        state.phase !== "FAILED" &&
        state.phase !== "CANCELLED"
      ) {
        state = await this.persistForRun(
          transitionSync(state, "RETRYING", { lastError: "Resuming interrupted sync" }),
          run,
        );
      }
      state = await this.persistForRun(
        transitionSync(state, "DISCOVERING", { lastError: null }),
        run,
      );

      run.session = await this.source.open(descriptor);
      this.assertRunActive(run);
      const manifestBytes = await run.session.readManifest();
      this.assertRunActive(run);
      const { manifest } = await verifyManifestAgainstDescriptor(
        descriptor,
        manifestBytes,
        this.sha256Hex,
      );
      this.assertRunActive(run);
      await this.cache.writeManifest(identity, manifestBytes);
      this.assertRunActive(run);

      const totalFiles = manifest.files.length + 1;
      const totalBytes =
        manifestBytes.byteLength +
        manifest.files.reduce((sum, file) => sum + file.ciphertext_size, 0);
      state = await this.persistForRun(
        transitionSync(state, "DOWNLOADING", {
          manifestVerified: true,
          packageVerified: false,
          progress: {
            filesCompleted: 1,
            totalFiles,
            bytesCompleted: manifestBytes.byteLength,
            totalBytes,
          },
        }),
        run,
      );

      for (const file of manifest.files) {
        this.assertRunActive(run);
        let ciphertext = await this.cache.readCiphertext(identity, file.path);
        this.assertRunActive(run);
        const cachedValid =
          ciphertext !== null &&
          ciphertext.byteLength === file.ciphertext_size &&
          (await this.sha256Hex(ciphertext)).toLowerCase() === file.ciphertext_sha256;
        this.assertRunActive(run);
        if (!cachedValid) {
          ciphertext = await run.session.readCiphertext(file.path);
          this.assertRunActive(run);
          if (ciphertext.byteLength !== file.ciphertext_size) {
            throw new PackageVerificationError(`Ciphertext size mismatch: ${file.path}`);
          }
          if ((await this.sha256Hex(ciphertext)).toLowerCase() !== file.ciphertext_sha256) {
            throw new PackageVerificationError(`Ciphertext digest mismatch: ${file.path}`);
          }
          this.assertRunActive(run);
          await this.cache.writeCiphertext(identity, file.path, ciphertext);
          this.assertRunActive(run);
        }

        state = await this.persistForRun(
          transitionSync(state, "DOWNLOADING", {
            progress: {
              ...state.progress,
              filesCompleted: state.progress.filesCompleted + 1,
              bytesCompleted: state.progress.bytesCompleted + file.ciphertext_size,
            },
          }),
          run,
        );
      }

      state = await this.persistForRun(transitionSync(state, "VERIFYING"), run);
      const cachedManifest = await this.cache.readManifest(identity);
      this.assertRunActive(run);
      if (cachedManifest === null) {
        throw new PackageVerificationError("Cached manifest disappeared before package verification");
      }
      await verifyP2pPackage(
        descriptor,
        cachedManifest,
        async (path) => {
          this.assertRunActive(run);
          const bytes = await this.cache.readCiphertext(identity, path);
          this.assertRunActive(run);
          if (bytes === null) throw new Error("missing ciphertext");
          return bytes;
        },
        this.sha256Hex,
      );
      this.assertRunActive(run);

      state = await this.persistForRun(
        transitionSync(state, "READY", { packageVerified: true, lastError: null }),
        run,
      );
      return state;
    } catch (error) {
      if (run.cancelled || error instanceof P2pSyncCancelledError) {
        await run.writeTail;
        throw error instanceof P2pSyncCancelledError ? error : new P2pSyncCancelledError();
      }

      const message = error instanceof Error ? error.message : "P2P sync failed";
      const target = error instanceof PackageVerificationError ? "FAILED" : "RETRYING";
      if (state.phase !== "READY") {
        state = await this.persistForRun(transitionSync(state, target, { lastError: message }), run);
      }
      throw error;
    } finally {
      await run.session?.close().catch(() => undefined);
      if (this.activeRuns.get(runKey) === run) this.activeRuns.delete(runKey);
    }
  }

  async cancel(identity: P2pSyncIdentity): Promise<P2pSyncSnapshot> {
    const run = this.activeRuns.get(packageCacheKey(identity));
    if (run !== undefined) {
      run.cancelled = true;
      const abort = this.abortSession(run.session);
      const cancellation = run.writeTail.then(() => this.persistCancellation(identity));
      run.writeTail = cancellation.then(
        () => undefined,
        () => undefined,
      );
      const state = await cancellation;
      await abort;
      return state;
    }
    return this.persistCancellation(identity);
  }

  async clearAccount(accountScope: string): Promise<void> {
    const active = [...this.activeRuns.values()]
      .filter((run) => run.identity.accountScope === accountScope)
      .map((run) => run.identity);
    await Promise.all(active.map((identity) => this.cancel(identity)));
    await this.cache.clearAccount(accountScope);
  }

  private async persistForRun(
    snapshot: P2pSyncSnapshot,
    run: ActiveSyncRun,
  ): Promise<P2pSyncSnapshot> {
    this.assertRunActive(run);
    const operation = run.writeTail.then(async () => {
      this.assertRunActive(run);
      await this.cache.writeSnapshot(snapshot);
      this.assertRunActive(run);
      this.onProgress(snapshot);
    });
    run.writeTail = operation.then(
      () => undefined,
      () => undefined,
    );
    await operation;
    return snapshot;
  }

  private async persistCancellation(identity: P2pSyncIdentity): Promise<P2pSyncSnapshot> {
    const current = (await this.cache.readSnapshot(identity)) ?? createSyncSnapshot(identity);
    if (current.phase === "READY") return current;
    const cancelled = transitionSync(current, "CANCELLED", { lastError: null });
    await this.cache.writeSnapshot(cancelled);
    this.onProgress(cancelled);
    return cancelled;
  }

  private assertRunActive(run: ActiveSyncRun): void {
    if (run.cancelled) throw new P2pSyncCancelledError();
  }

  private async abortSession(session: P2pPackageSourceSession | null): Promise<void> {
    if (session === null) return;
    if (session.cancel !== undefined) {
      await session.cancel().catch(() => undefined);
      return;
    }
    await session.close().catch(() => undefined);
  }
}

export function descriptorIdentity(
  descriptor: P2pReadyDescriptor,
  accountScope: string,
): P2pSyncIdentity {
  return {
    accountScope,
    publicationId: descriptor.publicationId,
    lineageId: descriptor.lineageId,
  };
}
