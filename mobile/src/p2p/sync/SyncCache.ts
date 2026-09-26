import { Directory, File, Paths } from "expo-file-system";
import { validateRelativePath } from "./PackageVerifier";
import {
  accountCachePrefix,
  packageCacheKey,
  restoreSyncSnapshot,
  type P2pSyncIdentity,
  type P2pSyncSnapshot,
} from "./SyncState";

const MANIFEST_FILE = "manifest.json";
const STATE_FILE = "sync-state.json";

export interface P2pSyncCache {
  readSnapshot(identity: P2pSyncIdentity): Promise<P2pSyncSnapshot | null>;
  writeSnapshot(snapshot: P2pSyncSnapshot): Promise<void>;
  readManifest(identity: P2pSyncIdentity): Promise<Uint8Array | null>;
  writeManifest(identity: P2pSyncIdentity, bytes: Uint8Array): Promise<void>;
  readCiphertext(identity: P2pSyncIdentity, path: string): Promise<Uint8Array | null>;
  writeCiphertext(identity: P2pSyncIdentity, path: string, bytes: Uint8Array): Promise<void>;
  clear(identity: P2pSyncIdentity): Promise<void>;
  clearAccount(accountScope: string): Promise<void>;
}

/**
 * Device cache for the complete encrypted package. Only canonical manifest,
 * ciphertext files, and non-secret lifecycle metadata are persisted here.
 */
export class ExpoP2pSyncCache implements P2pSyncCache {
  private readonly root: Directory;

  constructor(root: Directory = new Directory(Paths.document, "p2p-packages")) {
    this.root = root;
  }

  async readSnapshot(identity: P2pSyncIdentity): Promise<P2pSyncSnapshot | null> {
    const file = new File(this.packageDirectory(identity), STATE_FILE);
    if (!file.exists) return null;
    try {
      return restoreSyncSnapshot(JSON.parse(await file.text()), identity);
    } catch {
      return null;
    }
  }

  async writeSnapshot(snapshot: P2pSyncSnapshot): Promise<void> {
    const directory = this.ensurePackageDirectory(snapshot.identity);
    const file = new File(directory, STATE_FILE);
    file.create({ overwrite: true, intermediates: true });
    file.write(JSON.stringify(snapshot));
  }

  async readManifest(identity: P2pSyncIdentity): Promise<Uint8Array | null> {
    const file = new File(this.packageDirectory(identity), MANIFEST_FILE);
    return file.exists ? file.bytes() : null;
  }

  async writeManifest(identity: P2pSyncIdentity, bytes: Uint8Array): Promise<void> {
    const directory = this.ensurePackageDirectory(identity);
    const file = new File(directory, MANIFEST_FILE);
    file.create({ overwrite: true, intermediates: true });
    file.write(bytes);
  }

  async readCiphertext(identity: P2pSyncIdentity, path: string): Promise<Uint8Array | null> {
    const file = this.ciphertextFile(identity, path, false);
    return file.exists ? file.bytes() : null;
  }

  async writeCiphertext(identity: P2pSyncIdentity, path: string, bytes: Uint8Array): Promise<void> {
    const file = this.ciphertextFile(identity, path, true);
    file.create({ overwrite: true, intermediates: true });
    file.write(bytes);
  }

  async clear(identity: P2pSyncIdentity): Promise<void> {
    const directory = this.packageDirectory(identity);
    if (directory.exists) directory.delete();
  }

  async clearAccount(accountScope: string): Promise<void> {
    const [account] = accountCachePrefix(accountScope).split("/");
    const directory = new Directory(this.root, account);
    if (directory.exists) directory.delete();
  }

  private packageDirectory(identity: P2pSyncIdentity): Directory {
    const [accountScope, publicationId, lineageId] = packageCacheKey(identity).split("/");
    return new Directory(this.root, accountScope, publicationId, lineageId);
  }

  private ensurePackageDirectory(identity: P2pSyncIdentity): Directory {
    const directory = this.packageDirectory(identity);
    directory.create({ idempotent: true, intermediates: true });
    return directory;
  }

  private ciphertextFile(identity: P2pSyncIdentity, path: string, createParents: boolean): File {
    validateRelativePath(path);
    const segments = path.split("/");
    const name = segments.pop();
    if (!name) throw new Error("Invalid ciphertext path");
    const filesRoot = new Directory(this.packageDirectory(identity), "files");
    const parent = new Directory(filesRoot, ...segments);
    if (createParents) parent.create({ idempotent: true, intermediates: true });
    return new File(parent, name);
  }
}

export class MemoryP2pSyncCache implements P2pSyncCache {
  private readonly snapshots = new Map<string, P2pSyncSnapshot>();
  private readonly manifests = new Map<string, Uint8Array>();
  private readonly files = new Map<string, Uint8Array>();

  async readSnapshot(identity: P2pSyncIdentity): Promise<P2pSyncSnapshot | null> {
    return this.snapshots.get(packageCacheKey(identity)) ?? null;
  }

  async writeSnapshot(snapshot: P2pSyncSnapshot): Promise<void> {
    this.snapshots.set(packageCacheKey(snapshot.identity), snapshot);
  }

  async readManifest(identity: P2pSyncIdentity): Promise<Uint8Array | null> {
    return cloneBytes(this.manifests.get(packageCacheKey(identity)));
  }

  async writeManifest(identity: P2pSyncIdentity, bytes: Uint8Array): Promise<void> {
    this.manifests.set(packageCacheKey(identity), bytes.slice());
  }

  async readCiphertext(identity: P2pSyncIdentity, path: string): Promise<Uint8Array | null> {
    validateRelativePath(path);
    return cloneBytes(this.files.get(`${packageCacheKey(identity)}/${path}`));
  }

  async writeCiphertext(identity: P2pSyncIdentity, path: string, bytes: Uint8Array): Promise<void> {
    validateRelativePath(path);
    this.files.set(`${packageCacheKey(identity)}/${path}`, bytes.slice());
  }

  async clear(identity: P2pSyncIdentity): Promise<void> {
    const key = packageCacheKey(identity);
    this.snapshots.delete(key);
    this.manifests.delete(key);
    this.deleteFiles(`${key}/`);
  }

  async clearAccount(accountScope: string): Promise<void> {
    const prefix = accountCachePrefix(accountScope);
    for (const key of this.snapshots.keys()) {
      if (key.startsWith(prefix)) this.snapshots.delete(key);
    }
    for (const key of this.manifests.keys()) {
      if (key.startsWith(prefix)) this.manifests.delete(key);
    }
    this.deleteFiles(prefix);
  }

  private deleteFiles(prefix: string): void {
    for (const key of this.files.keys()) {
      if (key.startsWith(prefix)) this.files.delete(key);
    }
  }
}

function cloneBytes(value: Uint8Array | undefined): Uint8Array | null {
  return value ? value.slice() : null;
}
