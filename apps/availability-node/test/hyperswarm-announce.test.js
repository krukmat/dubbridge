import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import Hyperswarm from "hyperswarm";
import {
  openDrive,
  closeDrive,
  closeSharedStore,
  announceOnSwarm,
  HyperswarmAnnounceTimeoutError,
  HyperswarmAnnounceFailedError,
} from "../dist/hyperdrive_store.js";

test("HP-1: announceOnSwarm joins and flushes within a generous timeout", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-swarm-hp1-"));
  const opened = await openDrive(root, "33333333-3333-3333-3333-333333333333");

  await announceOnSwarm(root, opened, 30_000);

  await closeDrive(opened);
  await closeSharedStore(root);
});

test("EC-1: announceOnSwarm rejects with HyperswarmAnnounceTimeoutError when the timeout is effectively zero", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-swarm-ec1-"));
  const opened = await openDrive(root, "44444444-4444-4444-4444-444444444444");

  await assert.rejects(
    () => announceOnSwarm(root, opened, 1),
    HyperswarmAnnounceTimeoutError
  );

  await closeDrive(opened);
  await closeSharedStore(root);
});

test("EC-2: announceOnSwarm rejects with HyperswarmAnnounceFailedError when flushed() resolves false, and destroys the discovery session", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-swarm-ec2-"));
  const opened = await openDrive(root, "55555555-5555-5555-5555-555555555555");

  const originalJoin = Hyperswarm.prototype.join;
  let destroyCalled = false;
  Hyperswarm.prototype.join = function fakeJoin(topic, opts) {
    const discovery = originalJoin.call(this, topic, opts);
    const originalDestroy = discovery.destroy.bind(discovery);
    discovery.flushed = async () => false;
    discovery.destroy = async (...args) => {
      destroyCalled = true;
      return originalDestroy(...args);
    };
    return discovery;
  };

  try {
    await assert.rejects(
      () => announceOnSwarm(root, opened, 30_000),
      HyperswarmAnnounceFailedError
    );
    assert.equal(destroyCalled, true);
  } finally {
    Hyperswarm.prototype.join = originalJoin;
  }

  await closeDrive(opened);
  await closeSharedStore(root);
});

test("EC-3: announceOnSwarm propagates a synchronous swarm.join() failure without hanging", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-swarm-ec3-"));
  const opened = await openDrive(root, "66666666-6666-6666-6666-666666666666");

  const originalJoin = Hyperswarm.prototype.join;
  Hyperswarm.prototype.join = function throwingJoin() {
    throw new Error("synthetic synchronous join failure");
  };

  try {
    await assert.rejects(() => announceOnSwarm(root, opened, 30_000), /synthetic synchronous join failure/);
  } finally {
    Hyperswarm.prototype.join = originalJoin;
  }

  await closeDrive(opened);
  await closeSharedStore(root);
});

test("EC-4: a timed-out announce does not leak a joined discovery session (destroy() is called on timeout)", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-swarm-ec4-"));
  const opened = await openDrive(root, "77777777-7777-7777-7777-777777777777");

  const originalJoin = Hyperswarm.prototype.join;
  let destroyCalled = false;
  Hyperswarm.prototype.join = function fakeJoin(topic, opts) {
    const discovery = originalJoin.call(this, topic, opts);
    const originalDestroy = discovery.destroy.bind(discovery);
    // Never resolves, forcing the timeout branch to win the race.
    discovery.flushed = () => new Promise(() => {});
    discovery.destroy = async (...args) => {
      destroyCalled = true;
      return originalDestroy(...args);
    };
    return discovery;
  };

  try {
    await assert.rejects(() => announceOnSwarm(root, opened, 20), HyperswarmAnnounceTimeoutError);
    assert.equal(destroyCalled, true);
  } finally {
    Hyperswarm.prototype.join = originalJoin;
  }

  await closeDrive(opened);
  await closeSharedStore(root);
});

test("EC-5: repeated announceOnSwarm calls for the same drive reuse the existing discovery instead of accumulating sessions", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-swarm-ec5-"));
  const opened = await openDrive(root, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");

  const originalJoin = Hyperswarm.prototype.join;
  let joinCallCount = 0;
  Hyperswarm.prototype.join = function countingJoin(topic, opts) {
    joinCallCount += 1;
    return originalJoin.call(this, topic, opts);
  };

  try {
    await announceOnSwarm(root, opened, 30_000);
    await announceOnSwarm(root, opened, 30_000);
    await announceOnSwarm(root, opened, 30_000);

    assert.equal(joinCallCount, 1, "swarm.join() must only be called once for a repeatedly-announced topic");
  } finally {
    Hyperswarm.prototype.join = originalJoin;
  }

  await closeDrive(opened);
  await closeSharedStore(root);
});

test("HP-2: an inbound peer replicates and downloads the published drive's content over Hyperswarm", async () => {
  const seederRoot = mkdtempSync(join(tmpdir(), "hd-swarm-seed-"));
  const leecherRoot = mkdtempSync(join(tmpdir(), "hd-swarm-leech-"));
  const publicationId = "88888888-8888-8888-8888-888888888888";

  const seeded = await openDrive(seederRoot, publicationId);
  await seeded.drive.put("/hello.txt", Buffer.from("peer-replicated-content"));
  await announceOnSwarm(seederRoot, seeded, 30_000);

  const leechSwarm = new Hyperswarm();
  try {
    const Hyperdrive = (await import("hyperdrive")).default;
    const Corestore = (await import("corestore")).default;
    const leechStore = new Corestore(leecherRoot);
    await leechStore.ready();
    // Register the connection listener before join()/flushed(): a connection
    // can fire as soon as the join round completes, and `swarm.on("connection", ...)`
    // only catches connections that occur after it is registered — attaching
    // it later can miss the only connection this test gets.
    leechSwarm.on("connection", (connection) => leechStore.replicate(connection));

    const leechDiscovery = leechSwarm.join(seeded.drive.discoveryKey, { server: false, client: true });
    await leechDiscovery.flushed();

    const leechDrive = new Hyperdrive(leechStore, seeded.drive.key);
    await leechDrive.ready();

    // A freshly-opened Hyperbee core over a live connection does not know
    // its own length until an update round-trip completes; without this,
    // db.get()-backed lookups (entry()/get()) see length 0 and resolve null
    // even though replication itself is working. Real callers of
    // hyperdrive_store.ts never hit this path (they only ever open a drive
    // they already possess locally), so this is a test-only synchronization
    // step, not a production code fix.
    await Promise.race([
      leechDrive.db.core.update({ wait: true }),
      new Promise((_, reject) => setTimeout(() => reject(new Error("peer core update timed out")), 30_000)),
    ]);

    const downloaded = await Promise.race([
      leechDrive.get("/hello.txt"),
      new Promise((_, reject) => setTimeout(() => reject(new Error("peer download timed out")), 30_000)),
    ]);

    assert.equal(downloaded?.toString(), "peer-replicated-content");

    await leechDrive.close();
    await leechStore.close();
  } finally {
    await leechSwarm.destroy();
  }

  await closeDrive(seeded);
  await closeSharedStore(seederRoot);
});

test("HP-3: after closeSharedStore + a fresh executor-level re-announce, the same drive is discoverable again", async () => {
  const root = mkdtempSync(join(tmpdir(), "hd-swarm-restart-"));
  const publicationId = "99999999-9999-9999-9999-999999999999";

  const first = await openDrive(root, publicationId);
  await announceOnSwarm(root, first, 30_000);
  await closeDrive(first);
  // Simulates process teardown: destroys every joined session.
  await closeSharedStore(root);

  // A fresh "restart" must explicitly re-announce — closeSharedStore does
  // not leave anything joined.
  const second = await openDrive(root, publicationId);
  await announceOnSwarm(root, second, 30_000);

  const leechSwarm = new Hyperswarm();
  try {
    const leechDiscovery = leechSwarm.join(second.drive.discoveryKey, { server: false, client: true });
    const flushed = await leechDiscovery.flushed();
    assert.equal(flushed, true);
  } finally {
    await leechSwarm.destroy();
  }

  await closeDrive(second);
  await closeSharedStore(root);
});
