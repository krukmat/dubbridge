import { RuntimeProtocolError } from "./protocol";
import type { WorkletRuntime } from "./transient-drive";

const DRIVE_KEY = /^[0-9a-f]{64}$/;
const ACCOUNT_SCOPE = /^[A-Za-z0-9._-]{1,128}$/;
const DEFAULT_IO_TIMEOUT_MS = 30_000;

interface ProductStore {
  close(): Promise<void>;
  replicate(connection: unknown): unknown;
}

interface ProductDrive {
  readonly discoveryKey: Buffer;
  readonly version: number;
  ready(): Promise<void>;
  findingPeers(): () => void;
  update(options: { wait: boolean }): Promise<boolean>;
  get(path: string): Promise<Uint8Array | null>;
  close(): Promise<void>;
}

interface ProductDiscovery {
  destroy(): Promise<void>;
}

interface ProductSwarm {
  on(event: "connection", listener: (connection: unknown) => void): void;
  join(topic: Buffer, options: { server: boolean; client: boolean }): ProductDiscovery;
  flush(): Promise<boolean>;
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

  get isOpen(): boolean {
    return this.active !== null;
  }

  async open(
    runtime: WorkletRuntime,
    accountScope: string,
    externalPublicationId: string,
  ): Promise<void> {
    if (this.active !== null) {
      throw new RuntimeProtocolError("PRODUCT_PACKAGE_OPEN_FAILED", "Product package is already open");
    }
    validateDriveKey(externalPublicationId);
    const active = await openPackage(productAccountStorageUri(runtime, accountScope), externalPublicationId);
    this.active = active;
  }

  async read(path: string): Promise<Uint8Array> {
    validatePackagePath(path);
    const active = this.requireActive();
    const bytes = await withTimeout(
      readDiscoveredFile(active.drive, path),
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
  const storagePath = productStoragePath(storageUri);
  const { Corestore, Hyperdrive, Hyperswarm } = loadDependencies();
  const store = new Corestore(storagePath);
  const drive = new Hyperdrive(store, Buffer.from(externalPublicationId, "hex"));
  const swarm = new Hyperswarm();
  swarm.on("connection", (connection) => store.replicate(connection));

  try {
    await drive.ready();
    const doneFindingPeers = drive.findingPeers();
    let discovery: ProductDiscovery;
    try {
      discovery = swarm.join(drive.discoveryKey, { server: false, client: true });
      // Client reads must wait for pending peer connections. discovery.flushed()
      // only waits for a server announcement and can leave a cold drive empty.
      const flushed = await withTimeout(
        swarm.flush(),
        DEFAULT_IO_TIMEOUT_MS,
        "REPLICATION_DISCOVERY_FAILED",
        "Product package discovery timed out",
      );
      if (!flushed) {
        throw new RuntimeProtocolError("REPLICATION_DISCOVERY_FAILED", "Product package discovery failed");
      }
    } finally {
      doneFindingPeers();
    }
    return { store, drive, swarm, discovery };
  } catch (error) {
    await Promise.allSettled([swarm.destroy(), drive.close(), store.close()]);
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError("PRODUCT_PACKAGE_OPEN_FAILED", "Product package could not be opened");
  }
}

async function readDiscoveredFile(drive: ProductDrive, path: string): Promise<Uint8Array | null> {
  // A connected peer may not have delivered its first metadata proof yet.
  // Hyperbee otherwise treats a cold version as an empty (missing-file) tree.
  if (drive.version < 2) await drive.update({ wait: true });
  return drive.get(`/${path}`);
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

export function productAccountStorageUri(runtime: WorkletRuntime, accountScope: string): string {
  validateAccountScope(accountScope);
  return `${runtimeStorageUri(runtime)}/accounts/${accountScope}`;
}

function runtimeStorageUri(runtime: WorkletRuntime): string {
  const uri = runtime.argv?.[0];
  if (typeof uri !== "string" || !uri.startsWith("file:") || uri.length <= 5) {
    throw new RuntimeProtocolError("PRODUCT_STORAGE_CONFIG_INVALID", "Product storage configuration is invalid");
  }
  return uri.replace(/\/$/, "");
}

function productStoragePath(storageUri: string): string {
  try {
    const bareUrl = require("bare-url") as { fileURLToPath(url: string): string };
    const storagePath = bareUrl.fileURLToPath(storageUri);
    if (storagePath.includes("\0")) {
      throw new Error("Product storage path contains NUL");
    }
    return storagePath;
  } catch {
    throw new RuntimeProtocolError(
      "PRODUCT_STORAGE_CONFIG_INVALID",
      "Product storage configuration is invalid",
    );
  }
}

function validateAccountScope(value: string): void {
  if (!ACCOUNT_SCOPE.test(value) || value === "." || value === "..") {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product account scope is invalid");
  }
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
