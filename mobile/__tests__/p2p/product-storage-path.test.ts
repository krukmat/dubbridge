/** @jest-environment node */

import { access, mkdtemp, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

import type { WorkletRuntime } from "../../src/p2p/runtime/transient-drive";

const DRIVE_KEY = "a".repeat(64);

function runtimeArg(storageUri: string): WorkletRuntime {
  return {
    argv: [storageUri],
    on: jest.fn(),
  } as unknown as WorkletRuntime;
}

function loadRealRuntime() {
  jest.resetModules();
  jest.dontMock("corestore");
  jest.dontMock("hyperdrive");
  jest.dontMock("hyperswarm");
  jest.dontMock("bare-url");
  jest.doMock("bare-url", () => require("node:url"));

  class FakeHyperswarm {
    on(_event: string, _listener: (connection: unknown) => void): void {}

    async flush(): Promise<boolean> { return true; }

    join(
      _topic: Buffer,
      _options: { server: boolean; client: boolean },
    ): { flushed(): Promise<boolean>; destroy(): Promise<void> } {
      return {
        flushed: async () => true,
        destroy: async () => undefined,
      };
    }

    async destroy(): Promise<void> {}
  }

  jest.doMock("hyperswarm", () => FakeHyperswarm);

  return require("../../src/p2p/runtime/product-package-runtime") as typeof import("../../src/p2p/runtime/product-package-runtime");
}

function loadRuntimeWithDependencySpies() {
  jest.resetModules();
  jest.dontMock("bare-url");
  jest.doMock("bare-url", () => require("node:url"));

  const store = {
    close: jest.fn(async () => undefined),
    replicate: jest.fn(),
  };
  const drive = {
    discoveryKey: Buffer.alloc(32, 1),
    ready: jest.fn(async () => undefined),
    findingPeers: jest.fn(() => jest.fn()),
    get: jest.fn(async () => null),
    close: jest.fn(async () => undefined),
  };
  const discovery = {
    flushed: jest.fn(async () => true),
    destroy: jest.fn(async () => undefined),
  };
  const swarm = {
    on: jest.fn(),
    flush: jest.fn(async () => true),
    join: jest.fn(() => discovery),
    destroy: jest.fn(async () => undefined),
  };

  const Corestore = jest.fn().mockImplementation(() => store);
  const Hyperdrive = jest.fn().mockImplementation(() => drive);
  const Hyperswarm = jest.fn().mockImplementation(() => swarm);

  jest.doMock("corestore", () => Corestore);
  jest.doMock("hyperdrive", () => Hyperdrive);
  jest.doMock("hyperswarm", () => Hyperswarm);

  const module = require("../../src/p2p/runtime/product-package-runtime") as typeof import("../../src/p2p/runtime/product-package-runtime");
  return { ...module, Corestore, Hyperdrive, Hyperswarm };
}

describe("P4.T1-r1 product storage URI boundary", () => {
  afterEach(() => {
    jest.resetModules();
    jest.dontMock("corestore");
    jest.dontMock("hyperdrive");
    jest.dontMock("hyperswarm");
    jest.dontMock("bare-url");
  });

  it("opens real Corestore/Hyperdrive at the decoded account path without creating a relative file: tree", async () => {
    const root = await mkdtemp(path.join(tmpdir(), "dubbridge storage "));
    const cwd = await mkdtemp(path.join(tmpdir(), "dubbridge-cwd-"));
    const previousCwd = process.cwd();

    try {
      process.chdir(cwd);
      const { ProductPackageRuntime } = loadRealRuntime();
      const product = new ProductPackageRuntime();
      const storageUri = pathToFileURL(root).href;

      expect(storageUri).toContain("%20");

      await product.open(runtimeArg(storageUri), "viewer-a", DRIVE_KEY);
      await product.close();

      await expect(access(path.join(root, "accounts", "viewer-a"))).resolves.toBeUndefined();
      expect(await readdir(cwd)).not.toContain("file:");
    } finally {
      process.chdir(previousCwd);
      await rm(root, { recursive: true, force: true });
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("keeps distinct account scopes in distinct decoded filesystem directories", async () => {
    const root = await mkdtemp(path.join(tmpdir(), "dubbridge scopes "));

    try {
      const { ProductPackageRuntime } = loadRealRuntime();
      const product = new ProductPackageRuntime();
      const storageUri = pathToFileURL(root).href;

      await product.open(runtimeArg(storageUri), "viewer-a", DRIVE_KEY);
      await product.close();
      await product.open(runtimeArg(storageUri), "viewer-b", DRIVE_KEY);
      await product.close();

      await expect(access(path.join(root, "accounts", "viewer-a"))).resolves.toBeUndefined();
      await expect(access(path.join(root, "accounts", "viewer-b"))).resolves.toBeUndefined();
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it.each([
    "file://evil/tmp/dubbridge",
    "file:///tmp/dubbridge%2Fescape",
    "file:///tmp/dubbridge%00bad",
    "https://example.test/dubbridge",
  ])("rejects invalid storage URI %s before constructing product dependencies", async (storageUri) => {
    const { ProductPackageRuntime, Corestore, Hyperdrive, Hyperswarm } =
      loadRuntimeWithDependencySpies();
    const product = new ProductPackageRuntime();

    await expect(
      product.open(runtimeArg(storageUri), "viewer-a", DRIVE_KEY),
    ).rejects.toMatchObject({ code: "PRODUCT_STORAGE_CONFIG_INVALID" });

    expect(Corestore).not.toHaveBeenCalled();
    expect(Hyperdrive).not.toHaveBeenCalled();
    expect(Hyperswarm).not.toHaveBeenCalled();
  });

  it("decodes a percent-encoded path exactly once", async () => {
    const { ProductPackageRuntime, Corestore } = loadRuntimeWithDependencySpies();
    const product = new ProductPackageRuntime();

    await product.open(
      runtimeArg("file:///tmp/dubbridge%2520root"),
      "viewer-a",
      DRIVE_KEY,
    );
    await product.close();

    expect(Corestore).toHaveBeenCalledWith(
      "/tmp/dubbridge%20root/accounts/viewer-a",
    );
  });
});
