// Persistent Corestore/Hyperdrive lifecycle for one publication's
// ciphertext-only drive, plus Hyperswarm announce/join for that drive's
// discovery key (P2.T3c-S4-e).
//
// Design note: Corestore takes an exclusive file-descriptor lock on its
// storage root (RocksDB-backed), so a process may hold only one open
// `Corestore` per root at a time — verified empirically (a second
// concurrent `Corestore` instance against the same directory fails with
// "File descriptor could not be locked"). Multiple publications under the
// same root are therefore served by namespacing one shared `Corestore` per
// root, not by opening a fresh `Corestore` per publication_id. One shared
// `Hyperswarm` per storage root follows the same pattern: a single DHT/swarm
// identity serves every publication under that root.

import Corestore from "corestore";
import Hyperdrive from "hyperdrive";
import Hyperswarm from "hyperswarm";

type CorestoreInstance = InstanceType<typeof Corestore>;
type HyperdriveInstance = InstanceType<typeof Hyperdrive>;
type HyperswarmInstance = InstanceType<typeof Hyperswarm>;

const sharedStores = new Map<string, CorestoreInstance>();
const sharedSwarms = new Map<string, HyperswarmInstance>();

function getSharedStore(storageRoot: string): CorestoreInstance {
  const existing = sharedStores.get(storageRoot);
  if (existing !== undefined) {
    return existing;
  }
  const store = new Corestore(storageRoot);
  sharedStores.set(storageRoot, store);
  return store;
}

function getSharedSwarm(storageRoot: string): HyperswarmInstance {
  const existing = sharedSwarms.get(storageRoot);
  if (existing !== undefined) {
    return existing;
  }
  const swarm = new Hyperswarm();
  const store = getSharedStore(storageRoot);
  // Hyperswarm only establishes peer sockets; without replicating the
  // shared Corestore over each connection, an announced/joined drive is
  // discoverable but cannot actually serve its content to peers.
  swarm.on("connection", (connection) => store.replicate(connection));
  sharedSwarms.set(storageRoot, swarm);
  return swarm;
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

export class HyperswarmAnnounceTimeoutError extends Error {
  constructor() {
    super("Hyperswarm join/announce did not complete within the configured timeout");
    this.name = "HyperswarmAnnounceTimeoutError";
  }
}

export class HyperswarmAnnounceFailedError extends Error {
  constructor() {
    super("Hyperswarm announce round failed (flushed() resolved false)");
    this.name = "HyperswarmAnnounceFailedError";
  }
}

/**
 * Announces `opened`'s drive on the shared Hyperswarm for `storageRoot` and
 * waits for the join to flush (the DHT has finished the initial announce
 * round for the drive's discovery key), bounded by `timeoutMs`.
 *
 * Idempotent per topic: if the drive's discovery key is already tracked by
 * the shared swarm (`swarm.status()` returns non-null — e.g. a prior
 * successful announce, or a replay-path re-announce for a publication this
 * process already joined), this reuses that existing discovery instead of
 * calling `swarm.join()` again. `swarm.join()` on an already-tracked topic
 * would create a brand new `PeerDiscoverySession` that this function never
 * releases (destroying it on success would call `swarm.leave()` and
 * silently un-announce the drive if it were the last session, but keeping
 * it alive without limit accumulates one session per call) — every caller
 * in this codebase (the first-time-publication path and the replay path in
 * `publication_executor.ts`) calls this once per request for the same
 * long-lived publication, so avoiding the duplicate join sidesteps the leak
 * entirely rather than trying to track a destroy-safe reference count here.
 *
 * Fail-closed by design (criterion 5): `flushed()` resolves a boolean rather
 * than rejecting on a failed announce round (hyperswarm@4.17.1's
 * `PeerDiscovery.flushed()` catches the internal refresh rejection and
 * returns `false`) — a `false` resolution is treated as a failure and
 * throws `HyperswarmAnnounceFailedError`. A timeout rejects with
 * `HyperswarmAnnounceTimeoutError`. On a **first-time join** that then fails
 * or times out, the newly created discovery is destroyed before returning
 * (nothing else references it, so `swarm.leave()` firing is correct and
 * required — otherwise the topic would stay joined with no way to reach it
 * again short of `closeSharedStore`). On an **already-tracked** topic that
 * fails or times out, the existing discovery is left alone: destroying it
 * would unjoin a topic that may still be validly serving other requests
 * for the same publication, and the caller (whose own record predates this
 * call) still fails the current request closed regardless.
 */
export async function announceOnSwarm(
  storageRoot: string,
  opened: OpenDriveResult,
  timeoutMs: number
): Promise<void> {
  const swarm = getSharedSwarm(storageRoot);
  const topic = opened.drive.discoveryKey;
  const alreadyTracked = swarm.status(topic) !== null;
  const discovery = alreadyTracked
    ? swarm.status(topic)!
    : swarm.join(topic, { server: true, client: false });

  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new HyperswarmAnnounceTimeoutError()), timeoutMs);
  });

  try {
    const flushed = await Promise.race([discovery.flushed(), timeout]);
    if (!flushed) {
      throw new HyperswarmAnnounceFailedError();
    }
  } catch (err) {
    if (!alreadyTracked) {
      await discovery.destroy().catch(() => {});
    }
    throw err;
  } finally {
    clearTimeout(timer);
  }
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
 * Closes the shared `Corestore` and `Hyperswarm` for `storageRoot`, if
 * either is open, and evicts both from the process-wide caches. Callers
 * must have already closed every drive obtained from that root via
 * `closeDrive`.
 *
 * Both resources are evicted from the cache before either close is
 * attempted, and both close attempts run regardless of whether the other
 * fails, so a rejecting `swarm.destroy()` can never leave a stale cached
 * Corestore reference behind. Any close failure is surfaced to the caller
 * after both attempts complete.
 */
export async function closeSharedStore(storageRoot: string): Promise<void> {
  const swarm = sharedSwarms.get(storageRoot);
  sharedSwarms.delete(storageRoot);

  const store = sharedStores.get(storageRoot);
  sharedStores.delete(storageRoot);

  const results = await Promise.allSettled([
    swarm !== undefined ? swarm.destroy() : Promise.resolve(),
    store !== undefined ? store.close() : Promise.resolve(),
  ]);

  for (const result of results) {
    if (result.status === "rejected") {
      throw result.reason;
    }
  }
}
