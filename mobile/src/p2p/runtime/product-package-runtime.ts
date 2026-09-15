import { RuntimeProtocolError } from "./protocol";
import type { WorkletRuntime } from "./transient-drive";

const DRIVE_KEY = /^[0-9a-f]{64}$/;
const DEFAULT_IO_TIMEOUT_MS = 30_000;

interface ProductStore {
  close(): Promise<void>;
  replicate(connection: unknown): unknown;
}

interface ProductDrive {
  readonly discoveryKey: Buffer;
  ready(): Promise<void>;
  get(path: string): Promise<Uint8Array | null>;
  close(): Promise<void>;
}

interface ProductDiscovery {
  flushed(): Promise<boolean>;
  destroy(): Promise<void>;
}

interface ProductSwarm {
  on(event: "connection", listener: (connection: unknown) => void): void;
  join(topic: Buffer, options: { server: boolean; client: boolean }): ProductDiscovery;
  destroy(): Promise<void>;
}

interface ProductDependencies {
  Corestore: new (storage: string) => ProductStore;
  Hyperdrive: new (store: ProductStore, key: Buffer) => ProductDrive;
  Hyperswarm: new () => ProductSwarm;
}

type ActiveProductPackage = {
  store: ProductStore;
  drive: ProductDrive;
  swarm: ProductSwarm;
  discovery: ProductDiscovery;
};

export class ProductPackageRuntime {
  private active: ActiveProductPackage | null = null;

  async open(runtime: WorkletRuntime, externalPublicationId: string): Promise<void> {
    if (this.active !== null) {
      throw new RuntimeProtocolError("PRODUCT_PACKAGE_OPEN_FAILED", "Product package is already open");
    }
    validateDriveKey(externalPublicationId);
    const active = await openPackage(runtimeStorageUri(runtime), externalPublicationId);
    this.active = active;
  }

  async read(path: string): Promise<Uint8Array> {
    validatePackagePath(path);
    const active = this.requireActive();
    const bytes = await withTimeout(
      active.drive.get(`/${path}`),
      DEFAULT_IO_TIMEOUT_MS,
      "PRODUCT_PACKAGE_READ_FAILED",
      "Product package read timed out",
    );
    if (bytes === null) {
      throw new RuntimeProtocolError("PRODUCT_PACKAGE_READ_FAILED", "Product package file is unavailable");
    }
    return bytes;
  }

  async close(): Promise<void> {
    const active = this.active;
    this.active = null;
    if (active === null) return;
    await closePackage(active);
  }

  async cancel(): Promise<void> {
    await this.close();
  }

  private requireActive(): ActiveProductPackage {
    if (this.active === null) {
      throw new RuntimeProtocolError("PRODUCT_PACKAGE_NOT_OPEN", "Product package is not open");
    }
    return this.active;
  }
}

async function openPackage(storageUri: string, externalPublicationId: string): Promise<ActiveProductPackage> {
  const { Corestore, Hyperdrive, Hyperswarm } = loadDependencies();
  const store = new Corestore(storageUri);
  const drive = new Hyperdrive(store, Buffer.from(externalPublicationId, "hex"));
  const swarm = new Hyperswarm();
  swarm.on("connection", (connection) => store.replicate(connection));

  try {
    await drive.ready();
    const discovery = swarm.join(drive.discoveryKey, { server: false, client: true });
    const flushed = await withTimeout(
      discovery.flushed(),
      DEFAULT_IO_TIMEOUT_MS,
      "REPLICATION_DISCOVERY_FAILED",
      "Product package discovery timed out",
    );
    if (!flushed) {
      throw new RuntimeProtocolError("REPLICATION_DISCOVERY_FAILED", "Product package discovery failed");
    }
    return { store, drive, swarm, discovery };
  } catch (error) {
    await Promise.allSettled([swarm.destroy(), drive.close(), store.close()]);
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError("PRODUCT_PACKAGE_OPEN_FAILED", "Product package could not be opened");
  }
}

async function closePackage(active: ActiveProductPackage): Promise<void> {
  const results = await Promise.allSettled([
    active.discovery.destroy(),
    active.swarm.destroy(),
    active.drive.close(),
    active.store.close(),
  ]);
  if (results.some((result) => result.status === "rejected")) {
    throw new RuntimeProtocolError("PRODUCT_PACKAGE_CLOSE_FAILED", "Product package could not be closed");
  }
}

function runtimeStorageUri(runtime: WorkletRuntime): string {
  const uri = runtime.argv?.[0];
  if (typeof uri !== "string" || !uri.startsWith("file:") || uri.length <= 5) {
    throw new RuntimeProtocolError("PRODUCT_STORAGE_CONFIG_INVALID", "Product storage configuration is invalid");
  }
  return uri;
}

function validateDriveKey(value: string): void {
  if (!DRIVE_KEY.test(value)) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product package identity is invalid");
  }
}

export function validatePackagePath(path: string): void {
  if (!path || path.startsWith("/") || path.includes("\\")) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product package path is invalid");
  }
  if (path.split("/").some((segment) => !segment || segment === "." || segment === "..")) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product package path is invalid");
  }
}

function loadDependencies(): ProductDependencies {
  try {
    return {
      Corestore: require("corestore") as ProductDependencies["Corestore"],
      Hyperdrive: require("hyperdrive") as ProductDependencies["Hyperdrive"],
      Hyperswarm: require("hyperswarm") as ProductDependencies["Hyperswarm"],
    };
  } catch {
    throw new RuntimeProtocolError(
      "TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED",
      "Product package dependencies could not be loaded",
    );
  }
}

async function withTimeout<T>(
  operation: Promise<T>,
  timeoutMs: number,
  code: "REPLICATION_DISCOVERY_FAILED" | "PRODUCT_PACKAGE_READ_FAILED",
  message: string,
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      operation,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new RuntimeProtocolError(code, message)), timeoutMs);
      }),
    ]);
  } finally {
    if (timer !== undefined) clearTimeout(timer);
  }
}
