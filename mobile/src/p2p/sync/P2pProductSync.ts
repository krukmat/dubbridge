import type { P2pReadyDescriptor } from "../../api/p2p";
import {
  PackageVerificationError,
  verifyManifestAgainstDescriptor,
  verifyP2pPackage,
  type P2pManifest,
  type P2pManifestFile,
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
  open(descriptor: P2pReadyDescriptor, accountScope: string): Promise<P2pPackageSourceSession>;
}

export type SyncProgressObserver = (snapshot: P2pSyncSnapshot) => void;

type ActiveSyncRun = {
  readonly identity: P2pSyncIdentity;
  readonly key: string;
  cancelled: boolean;
  session: P2pPackageSourceSession | null;
  state: P2pSyncSnapshot;
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
    const state = (await this.cache.readSnapshot(identity)) ?? createSyncSnapshot(identity);
    if (state.phase === "READY") return state;

    const run = this.beginRun(identity, state);
    try {
      return await this.executeSync(descriptor, run);
    } catch (error) {
      await this.handleSyncFailure(run, error);
      throw error;
    } finally {
      await run.session?.close().catch(() => undefined);
      this.releaseRun(run);
    }
  }

  async cancel(identity: P2pSyncIdentity): Promise<P2pSyncSnapshot> {
    const run = this.activeRuns.get(packageCacheKey(identity));
    if (run === undefined) return this.persistCancellation(identity);

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

  async clearAccount(accountScope: string): Promise<void> {
    const identities = [...this.activeRuns.values()]
      .filter((run) => run.identity.accountScope === accountScope)
      .map((run) => run.identity);
    await Promise.all(identities.map((identity) => this.cancel(identity)));
    await this.cache.clearAccount(accountScope);
  }

  private beginRun(identity: P2pSyncIdentity, state: P2pSyncSnapshot): ActiveSyncRun {
    const key = packageCacheKey(identity);
    if (this.activeRuns.has(key)) {
      throw new Error("P2P sync is already active for this account publication lineage");
    }
    const run: ActiveSyncRun = {
      identity,
      key,
      cancelled: false,
      session: null,
      state,
      writeTail: Promise.resolve(),
    };
    this.activeRuns.set(key, run);
    return run;
  }

  private async executeSync(
    descriptor: P2pReadyDescriptor,
    run: ActiveSyncRun,
  ): Promise<P2pSyncSnapshot> {
    await this.prepareRun(run);
    run.session = await this.source.open(descriptor, run.identity.accountScope);
    this.assertRunActive(run);
    const { manifestBytes, manifest } = await this.loadManifest(descriptor, run);
    await this.copyPackage(run, manifestBytes, manifest);
    await this.verifyCachedPackage(descriptor, run);
    run.state = await this.persistForRun(
      transitionSync(run.state, "READY", { packageVerified: true, lastError: null }),
      run,
    );
    return run.state;
  }

  private async prepareRun(run: ActiveSyncRun): Promise<void> {
    if (!isRestartablePhase(run.state.phase)) {
      run.state = await this.persistForRun(
        transitionSync(run.state, "RETRYING", { lastError: "Resuming interrupted sync" }),
        run,
      );
    }
    run.state = await this.persistForRun(
      transitionSync(run.state, "DISCOVERING", { lastError: null }),
      run,
    );
  }

  private async loadManifest(
    descriptor: P2pReadyDescriptor,
    run: ActiveSyncRun,
  ): Promise<{ manifestBytes: Uint8Array; manifest: P2pManifest }> {
    const session = this.requireSession(run);
    const manifestBytes = await session.readManifest();
    this.assertRunActive(run);
    const { manifest } = await verifyManifestAgainstDescriptor(
      descriptor,
      manifestBytes,
      this.sha256Hex,
    );
    this.assertRunActive(run);
    await this.cache.writeManifest(run.identity, manifestBytes);
    this.assertRunActive(run);
    return { manifestBytes, manifest };
  }

  private async copyPackage(
    run: ActiveSyncRun,
    manifestBytes: Uint8Array,
    manifest: P2pManifest,
  ): Promise<void> {
    const totalBytes = manifestBytes.byteLength + sumCiphertextBytes(manifest.files);
    run.state = await this.persistForRun(
      transitionSync(run.state, "DOWNLOADING", {
        manifestVerified: true,
        packageVerified: false,
        progress: {
          filesCompleted: 1,
          totalFiles: manifest.files.length + 1,
          bytesCompleted: manifestBytes.byteLength,
          totalBytes,
        },
      }),
      run,
    );

    for (const file of manifest.files) {
      await this.copyFile(run, file);
    }
  }

  private async copyFile(run: ActiveSyncRun, file: P2pManifestFile): Promise<void> {
    this.assertRunActive(run);
    const cached = await this.cache.readCiphertext(run.identity, file.path);
    this.assertRunActive(run);
    const ciphertext = await this.resolveCiphertext(run, file, cached);
    this.assertRunActive(run);
    if (cached !== ciphertext) {
      await this.cache.writeCiphertext(run.identity, file.path, ciphertext);
      this.assertRunActive(run);
    }
    run.state = await this.persistForRun(
      transitionSync(run.state, "DOWNLOADING", {
        progress: {
          ...run.state.progress,
          filesCompleted: run.state.progress.filesCompleted + 1,
          bytesCompleted: run.state.progress.bytesCompleted + file.ciphertext_size,
        },
      }),
      run,
    );
  }

  private async resolveCiphertext(
    run: ActiveSyncRun,
    file: P2pManifestFile,
    cached: Uint8Array | null,
  ): Promise<Uint8Array> {
    if (cached !== null && (await this.matchesFile(file, cached))) return cached;
    const ciphertext = await this.requireSession(run).readCiphertext(file.path);
    this.assertRunActive(run);
    if (!(await this.matchesFile(file, ciphertext))) {
      throw new PackageVerificationError(`Ciphertext verification failed: ${file.path}`);
    }
    return ciphertext;
  }

  private async matchesFile(file: P2pManifestFile, bytes: Uint8Array): Promise<boolean> {
    if (bytes.byteLength !== file.ciphertext_size) return false;
    return (await this.sha256Hex(bytes)).toLowerCase() === file.ciphertext_sha256;
  }

  private async verifyCachedPackage(
    descriptor: P2pReadyDescriptor,
    run: ActiveSyncRun,
  ): Promise<void> {
    run.state = await this.persistForRun(transitionSync(run.state, "VERIFYING"), run);
    const manifest = await this.cache.readManifest(run.identity);
    this.assertRunActive(run);
    if (manifest === null) {
      throw new PackageVerificationError("Cached manifest disappeared before package verification");
    }
    await verifyP2pPackage(
      descriptor,
      manifest,
      (path) => this.readCachedCiphertext(run, path),
      this.sha256Hex,
    );
    this.assertRunActive(run);
  }

  private async readCachedCiphertext(run: ActiveSyncRun, path: string): Promise<Uint8Array> {
    this.assertRunActive(run);
    const bytes = await this.cache.readCiphertext(run.identity, path);
    this.assertRunActive(run);
    if (bytes === null) throw new Error("missing ciphertext");
    return bytes;
  }

  private async handleSyncFailure(run: ActiveSyncRun, error: unknown): Promise<void> {
    if (run.cancelled || error instanceof P2pSyncCancelledError) {
      await run.writeTail;
      if (!(error instanceof P2pSyncCancelledError)) throw new P2pSyncCancelledError();
      return;
    }
    if (run.state.phase === "READY") return;
    const target = error instanceof PackageVerificationError ? "FAILED" : "RETRYING";
    const message = error instanceof Error ? error.message : "P2P sync failed";
    run.state = await this.persistForRun(
      transitionSync(run.state, target, { lastError: message }),
      run,
    );
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

  private requireSession(run: ActiveSyncRun): P2pPackageSourceSession {
    if (run.session === null) throw new Error("P2P source session is not open");
    return run.session;
  }

  private releaseRun(run: ActiveSyncRun): void {
    if (this.activeRuns.get(run.key) === run) this.activeRuns.delete(run.key);
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

function isRestartablePhase(phase: P2pSyncSnapshot["phase"]): boolean {
  return phase === "IDLE" || phase === "RETRYING" || phase === "FAILED" || phase === "CANCELLED";
}

function sumCiphertextBytes(files: readonly P2pManifestFile[]): number {
  return files.reduce((sum, file) => sum + file.ciphertext_size, 0);
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
