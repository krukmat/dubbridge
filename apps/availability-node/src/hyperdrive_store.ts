// Persistent Corestore/Hyperdrive lifecycle for one publication's
// ciphertext-only drive.
//
// Scope note (P2.T3c-S4-c): this module currently owns only local,
// disk-backed open/reopen of a Hyperdrive keyed by publication_id — it does
// not announce on Hyperswarm, join a topic, or wait for a network flush.
// That remaining Complex-band residue (P2.T3c-S4-e) composes on top of
// `openDrive`'s result and is not implemented here.
//
// Design note: Corestore takes an exclusive file-descriptor lock on its
// storage root (RocksDB-backed), so a process may hold only one open
// `Corestore` per root at a time — verified empirically (a second
// concurrent `Corestore` instance against the same directory fails with
// "File descriptor could not be locked"). Multiple publications under the
// same root are therefore served by namespacing one shared `Corestore` per
// root, not by opening a fresh `Corestore` per publication_id.

import Corestore from "corestore";
import Hyperdrive from "hyperdrive";

type CorestoreInstance = InstanceType<typeof Corestore>;
type HyperdriveInstance = InstanceType<typeof Hyperdrive>;

const sharedStores = new Map<string, CorestoreInstance>();

function getSharedStore(storageRoot: string): CorestoreInstance {
  const existing = sharedStores.get(storageRoot);
  if (existing !== undefined) {
    return existing;
  }
  const store = new Corestore(storageRoot);
  sharedStores.set(storageRoot, store);
  return store;
}

export interface OpenDriveResult {
  readonly drive: HyperdriveInstance;
  readonly publicKeyHex: string;
}

/**
 * Opens (creating on first use, reopening deterministically thereafter) the
 * Hyperdrive for `publicationId` under `storageRoot`. Two calls with the
 * same `storageRoot` and `publicationId` — including across process
 * restarts — always resolve to the same drive public key, because Corestore
 * derives each namespaced core's key deterministically from the store's
 * seed plus the namespace name.
 *
 * A single `Corestore` is kept open per `storageRoot` for the lifetime of
 * the process (see design note above); `closeDrive` closes only the
 * returned drive, never the shared store.
 */
export async function openDrive(
  storageRoot: string,
  publicationId: string
): Promise<OpenDriveResult> {
  const store = getSharedStore(storageRoot);

  try {
    await store.ready();
  } catch (err) {
    sharedStores.delete(storageRoot);
    await store.close().catch(() => {});
    throw err;
  }

  const drive = new Hyperdrive(store.namespace(publicationId));

  try {
    await drive.ready();
  } catch (err) {
    await drive.close().catch(() => {});
    throw err;
  }

  return {
    drive,
    publicKeyHex: drive.key.toString("hex"),
  };
}

export async function closeDrive(opened: OpenDriveResult): Promise<void> {
  await opened.drive.close();
}

/**
 * Flushes pending writes durably to disk and then closes the drive.
 *
 * `Hyperdrive.flush()` (hyperdrive@13.3.3) delegates to `this.db.flush()`,
 * but the installed `hyperbee@2.27.3`'s `flush()` is defined only on its
 * internal `Batch` class, not on `Hyperbee` itself — calling
 * `drive.flush()` throws `this.db.flush is not a function` at runtime,
 * verified empirically against the exact installed dependency tree. Until
 * that upstream mismatch is fixed or worked around, `close()` is the only
 * durability primitive this codebase can rely on; `hyperdrive-store.test.js`
 * HP-2 already proves `put()` -> `close()` -> full shared-store teardown ->
 * reopen from disk recovers the written content, which is the durability
 * guarantee callers actually need.
 */
export async function flushDrive(opened: OpenDriveResult): Promise<void> {
  await opened.drive.close();
}

/**
 * Closes the shared `Corestore` for `storageRoot`, if one is open, and
 * evicts it from the process-wide cache. Callers must have already closed
 * every drive obtained from that root via `closeDrive`.
 */
export async function closeSharedStore(storageRoot: string): Promise<void> {
  const store = sharedStores.get(storageRoot);
  if (store === undefined) {
    return;
  }
  sharedStores.delete(storageRoot);
  await store.close();
}
