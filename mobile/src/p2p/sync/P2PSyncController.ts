import type { P2pReadyDescriptor } from "../../api/p2p";
import type { P2PService } from "../P2PService";
import { P2PServicePackageSource } from "./P2PServicePackageSource";
import { P2pProductSync, type SyncProgressObserver } from "./P2pProductSync";
import { ExpoP2pSyncCache, type P2pSyncCache } from "./SyncCache";
import type { P2pSyncIdentity, P2pSyncSnapshot } from "./SyncState";
import {
  createVerifiedPackageHandle,
  type VerifiedP2pPackageHandle,
} from "./VerifiedPackageHandle";

/**
 * Product-facing P4 façade. Runtime startup remains inert until startSync is
 * explicitly invoked; transport bytes alone never produce a P5 handle.
 */
export class P2PSyncController {
  private readonly sync: P2pProductSync;

  constructor(
    private readonly service: P2PService,
    private readonly cache: P2pSyncCache = new ExpoP2pSyncCache(),
    onProgress: SyncProgressObserver = () => undefined,
  ) {
    this.sync = new P2pProductSync(
      cache,
      new P2PServicePackageSource(service),
      (bytes) => service.hashProductBytes(bytes),
      onProgress,
    );
  }

  async startSync(
    descriptor: P2pReadyDescriptor,
    accountScope: string,
  ): Promise<P2pSyncSnapshot> {
    await this.ensureRuntimeReady();
    return this.sync.sync(descriptor, accountScope);
  }

  getSyncState(identity: P2pSyncIdentity): Promise<P2pSyncSnapshot | null> {
    return this.cache.readSnapshot(identity);
  }

  cancel(identity: P2pSyncIdentity): Promise<P2pSyncSnapshot> {
    return this.sync.cancel(identity);
  }

  async clearAccount(accountScope: string): Promise<void> {
    await this.sync.clearAccount(accountScope);
    await this.service.clearProductAccount(accountScope);
  }

  async getVerifiedPackageHandle(
    descriptor: P2pReadyDescriptor,
    accountScope: string,
  ): Promise<VerifiedP2pPackageHandle> {
    const identity = {
      accountScope,
      publicationId: descriptor.publicationId,
      lineageId: descriptor.lineageId,
    };
    const snapshot = await this.cache.readSnapshot(identity);
    if (snapshot === null) throw new Error("P2P package has no local sync state");
    return createVerifiedPackageHandle(descriptor, snapshot);
  }

  private async ensureRuntimeReady(): Promise<void> {
    const state = this.service.getSnapshot().runtimeState;
    if (state === "ready") return;
    if (state === "failed") await this.service.shutdown();
    await this.service.initialize();
  }
}
