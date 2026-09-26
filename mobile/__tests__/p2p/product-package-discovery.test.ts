/** @jest-environment node */

import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

import type { WorkletRuntime } from "../../src/p2p/runtime/transient-drive";

describe("product package client discovery", () => {
  afterEach(() => {
    jest.useRealTimers();
    jest.resetModules();
    jest.dontMock("corestore");
    jest.dontMock("hyperdrive");
    jest.dontMock("hyperswarm");
    jest.dontMock("bare-url");
  });

  it.each(["timeout", "false", "reject"])("releases discovery resources on %s", async (mode) => {
    jest.resetModules();
    jest.useFakeTimers();
    const done = jest.fn();
    const store = { close: jest.fn(async () => undefined), replicate: jest.fn() };
    const drive = {
      discoveryKey: Buffer.alloc(32),
      ready: async () => undefined,
      findingPeers: () => done,
      close: jest.fn(async () => undefined),
    };
    const swarm = {
      on: jest.fn(),
      join: () => ({ destroy: jest.fn() }),
      flush: () => mode === "timeout" ? new Promise<boolean>(() => undefined)
        : mode === "false" ? Promise.resolve(false) : Promise.reject(new Error("discovery failed")),
      destroy: jest.fn(async () => undefined),
    };
    jest.doMock("corestore", () => jest.fn(() => store));
    jest.doMock("hyperdrive", () => jest.fn(() => drive));
    jest.doMock("hyperswarm", () => jest.fn(() => swarm));
    jest.doMock("bare-url", () => require("node:url"));
    const { ProductPackageRuntime } = require("../../src/p2p/runtime/product-package-runtime");
    const product = new ProductPackageRuntime();
    const opened = product.open({ argv: ["file:///tmp/p5-test"] }, "viewer", "a".repeat(64));
    const rejection = expect(opened).rejects.toMatchObject({
      code: mode === "reject" ? "PRODUCT_PACKAGE_OPEN_FAILED" : "REPLICATION_DISCOVERY_FAILED",
    });
    await jest.advanceTimersByTimeAsync(30_000);
    await rejection;
    expect(done).toHaveBeenCalledTimes(1);
    expect(swarm.destroy).toHaveBeenCalledTimes(1);
    expect(drive.close).toHaveBeenCalledTimes(1);
    expect(store.close).toHaveBeenCalledTimes(1);
    expect(product.isOpen).toBe(false);
  });

  it("reads a cold manifest after a peer connects later than the DHT announcement", async () => {
    jest.resetModules();
    const Corestore = require("corestore");
    const Hyperdrive = require("hyperdrive");
    const Hypercore = require("hypercore");
    const root = await mkdtemp(path.join(tmpdir(), "p5-discovery-"));
    const seedStore = new Corestore(path.join(root, "seed"));
    const seed = new Hyperdrive(seedStore);
    const manifest = Buffer.from('{"ciphertext":"test manifest"}');
    const connections: Array<{ destroy(): void }> = [];
    let connected = false;

    class DelayedSwarm {
      listener: ((connection: unknown) => void) | undefined;
      on(_event: string, listener: (connection: unknown) => void) {
        this.listener = listener;
      }
      join() {
        return { flushed: async () => true, destroy: async () => undefined };
      }
      async flush() {
        await new Promise((resolve) => setTimeout(resolve, 25));
        const reader = Hypercore.createProtocolStream(false);
        const writer = seedStore.replicate(true);
        connections.push(reader, writer);
        this.listener!(reader);
        writer.pipe(reader).pipe(writer);
        await reader.noiseStream.opened;
        connected = true;
        return true;
      }
      async destroy() {
        for (const connection of connections) connection.destroy();
      }
    }

    jest.doMock("bare-url", () => require("node:url"));
    jest.doMock("hyperswarm", () => DelayedSwarm);
    const { ProductPackageRuntime } = require("../../src/p2p/runtime/product-package-runtime");
    const product = new ProductPackageRuntime();
    try {
      await seed.put("/manifest.json", manifest);
      await product.open(
        { argv: [pathToFileURL(path.join(root, "reader")).href] } as WorkletRuntime,
        "viewer",
        seed.key.toString("hex"),
      );
      await expect(product.read("manifest.json")).resolves.toEqual(manifest);
      expect(connected).toBe(true);
      await expect(product.read("missing.json")).rejects.toMatchObject({
        code: "PRODUCT_PACKAGE_READ_FAILED",
      });
    } finally {
      await product.close();
      for (const connection of connections) connection.destroy();
      await seed.close();
      await seedStore.close();
      await rm(root, { recursive: true, force: true });
    }
  });
});
