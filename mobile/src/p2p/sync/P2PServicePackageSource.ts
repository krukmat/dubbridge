import type { P2pReadyDescriptor } from "../../api/p2p";
import type { P2PService } from "../P2PService";
import type { P2pPackageSource, P2pPackageSourceSession } from "./P2pProductSync";

export type P2PPackageRuntimeService = Pick<
  P2PService,
  "openProductPackage" | "readProductFile" | "closeProductPackage" | "cancelProductPackage"
>;

/** Bridges product sync to the single product Bare runtime owned by P2PService. */
export class P2PServicePackageSource implements P2pPackageSource {
  constructor(private readonly service: P2PPackageRuntimeService) {}

  async open(descriptor: P2pReadyDescriptor): Promise<P2pPackageSourceSession> {
    await this.service.openProductPackage(descriptor.externalPublicationId);
    return new ServicePackageSession(this.service);
  }
}

class ServicePackageSession implements P2pPackageSourceSession {
  private closed = false;

  constructor(private readonly service: P2PPackageRuntimeService) {}

  readManifest(): Promise<Uint8Array> {
    return this.read("manifest.json");
  }

  readCiphertext(path: string): Promise<Uint8Array> {
    return this.read(path);
  }

  async cancel(): Promise<void> {
    if (this.closed) return;
    this.closed = true;
    await this.service.cancelProductPackage();
  }

  async close(): Promise<void> {
    if (this.closed) return;
    this.closed = true;
    await this.service.closeProductPackage();
  }

  private read(path: string): Promise<Uint8Array> {
    if (this.closed) return Promise.reject(new Error("P2P package source session is closed"));
    return this.service.readProductFile(path);
  }
}
