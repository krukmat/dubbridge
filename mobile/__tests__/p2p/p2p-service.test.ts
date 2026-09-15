jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

import {
  BareRuntimeClient,
  BareRuntimeClientError,
  type BareRuntimeProtocol,
  type BareRuntimeWorklet,
} from "../../src/p2p/runtime/BareRuntimeClient";
import { P2PService } from "../../src/p2p/P2PService";
import type { RuntimeHandshake } from "../../src/p2p/runtime/protocol";

const handshake: RuntimeHandshake = {
  protocolVersion: 1,
  runtimeVersion: "test",
  capabilities: [
    "ping",
    "lifecycle:suspend",
    "lifecycle:resume",
    "fatal",
    "shutdown",
    "product-package:v1",
    "product-playback:v1",
  ],
};

const playbackInput = {
  accountScope: "viewer-1",
  assetId: "asset-1",
  externalPublicationId: "a".repeat(64),
  lineageId: "lineage-1",
  manifestDigestSha256: "b".repeat(64),
  publicationId: "publication-1",
  ckBase64: Buffer.alloc(32, 7).toString("base64"),
};

const playbackReceipt = {
  capability: "product-playback" as const,
  schema_version: 1 as const,
  playback_url: `http://127.0.0.1:12345/${"c".repeat(32)}/index.m3u8`,
};

function createRuntime() {
  let state: "stopped" | "starting" | "ready" | "failed" = "stopped";
  return {
    get currentState() {
      return state;
    },
    initialize: jest.fn(async () => {
      state = "ready";
      return handshake;
    }),
    ping: jest.fn(async () => "pong" as const),
    openProductPackage: jest.fn(async (_accountScope: string, _externalPublicationId: string) => undefined),
    readProductFile: jest.fn(async (_path: string) => new Uint8Array([1, 2, 3])),
    hashProductBytes: jest.fn(async (_bytes: Uint8Array) => "d".repeat(64)),
    closeProductPackage: jest.fn(async () => undefined),
    cancelProductPackage: jest.fn(async () => undefined),
    clearProductAccount: jest.fn(async (_accountScope: string) => undefined),
    startProductPlayback: jest.fn(async () => playbackReceipt),
    stopProductPlayback: jest.fn(async () => undefined),
    shutdown: jest.fn(async () => {
      state = "stopped";
    }),
  };
}

function createProtocol(overrides: Partial<BareRuntimeProtocol> = {}): BareRuntimeProtocol {
  return {
    handshake: jest.fn(async () => handshake),
    ping: jest.fn(async () => "pong" as const),
    openProductPackage: jest.fn(async () => undefined),
    readProductFile: jest.fn(async () => new Uint8Array([1, 2, 3])),
    hashProductBytes: jest.fn(async () => "d".repeat(64)),
    closeProductPackage: jest.fn(async () => undefined),
    cancelProductPackage: jest.fn(async () => undefined),
    startProductPlayback: jest.fn(async () => playbackReceipt),
    stopProductPlayback: jest.fn(async () => undefined),
    shutdown: jest.fn(async () => undefined),
    ...overrides,
  };
}

function createWorklet(): BareRuntimeWorklet {
  return { IPC: {}, start: jest.fn(), terminate: jest.fn() } as unknown as BareRuntimeWorklet;
}

describe("P2PService", () => {
  it("HP-F2 keeps construction inert and publishes only explicit lifecycle work", async () => {
    const runtime = createRuntime();
    const service = new P2PService(runtime);
    const snapshots: string[] = [];
    const unsubscribe = service.subscribe(() => snapshots.push(service.getSnapshot().runtimeState));

    expect(service.getSnapshot()).toEqual({ runtimeState: "stopped", lastError: null });
    expect(runtime.initialize).not.toHaveBeenCalled();

    await expect(service.initialize()).resolves.toEqual(handshake);
    await expect(service.ping()).resolves.toBe("pong");
    await expect(service.shutdown()).resolves.toBeUndefined();

    expect(runtime.initialize).toHaveBeenCalledTimes(1);
    expect(runtime.ping).toHaveBeenCalledTimes(1);
    expect(runtime.shutdown).toHaveBeenCalledTimes(1);
    expect(snapshots).toEqual(["starting", "ready", "stopped"]);
    unsubscribe();
  });

  it("P4/P5 delegates package, playback, verification, and cleanup operations", async () => {
    const runtime = createRuntime();
    const service = new P2PService(runtime);
    const bytes = new Uint8Array([1, 2, 3]);
    await service.initialize();

    await service.openProductPackage("viewer-1", "a".repeat(64));
    await expect(service.readProductFile("manifest.json")).resolves.toEqual(bytes);
    await expect(service.hashProductBytes(bytes)).resolves.toBe("d".repeat(64));
    await service.cancelProductPackage();
    await expect(service.startProductPlayback(playbackInput)).resolves.toEqual(playbackReceipt);
    await service.stopProductPlayback();
    await service.clearProductAccount("viewer-1");

    expect(runtime.startProductPlayback).toHaveBeenCalledWith(playbackInput);
    expect(runtime.stopProductPlayback).toHaveBeenCalledTimes(1);
    expect(runtime.clearProductAccount).toHaveBeenCalledWith("viewer-1");
  });

  it("EC-F2 preserves a typed invalid lifecycle error and its snapshot", async () => {
    const runtime = createRuntime();
    runtime.ping.mockRejectedValueOnce(new BareRuntimeClientError("INVALID_STATE", "Cannot ping while runtime is stopped"));
    const service = new P2PService(runtime);

    await expect(service.ping()).rejects.toMatchObject({ code: "INVALID_STATE" });
    expect(service.getSnapshot()).toEqual({
      runtimeState: "stopped",
      lastError: "Cannot ping while runtime is stopped",
    });
  });

  it("EC-F2 shares concurrent initialization and shutdown operations", async () => {
    const runtime = createRuntime();
    const service = new P2PService(runtime);

    await expect(Promise.all([service.initialize(), service.initialize()])).resolves.toEqual([
      handshake,
      handshake,
    ]);
    await expect(Promise.all([service.shutdown(), service.shutdown()])).resolves.toEqual([
      undefined,
      undefined,
    ]);

    expect(runtime.initialize).toHaveBeenCalledTimes(1);
    expect(runtime.shutdown).toHaveBeenCalledTimes(1);
  });

  it("EC-F2 clears failed lifecycle operations and publishes their typed errors", async () => {
    const runtime = createRuntime();
    const service = new P2PService(runtime);
    runtime.initialize.mockRejectedValueOnce(new BareRuntimeClientError("START_FAILED", "start failed"));
    runtime.shutdown.mockRejectedValueOnce(new BareRuntimeClientError("INVALID_STATE", "shutdown failed"));

    await expect(service.initialize()).rejects.toMatchObject({ code: "START_FAILED" });
    expect(service.getSnapshot()).toEqual({ runtimeState: "stopped", lastError: "start failed" });
    await expect(service.shutdown()).rejects.toMatchObject({ code: "INVALID_STATE" });
    expect(service.getSnapshot()).toEqual({ runtimeState: "stopped", lastError: "shutdown failed" });
  });

  it("EC-F2 does not publish an unchanged stopped snapshot", async () => {
    const runtime = createRuntime();
    const service = new P2PService(runtime);
    const listener = jest.fn();
    service.subscribe(listener);

    await service.shutdown();

    expect(listener).not.toHaveBeenCalled();
  });
});

describe("BareRuntimeClient", () => {
  it("HP-F2 starts one worklet only on initialize and tears it down deterministically", async () => {
    const worklet = createWorklet();
    const protocol = createProtocol();
    const client = new BareRuntimeClient(() => worklet, () => protocol, "file:/tmp/p2p-product");

    expect(client.currentState).toBe("stopped");
    await expect(client.initialize()).resolves.toEqual(handshake);
    await expect(client.ping()).resolves.toBe("pong");
    await client.shutdown();

    expect(worklet.start).toHaveBeenCalledWith(
      "/dubbridge-p2p-runtime.worklet",
      expect.anything(),
      ["file:/tmp/p2p-product"],
    );
    expect(protocol.handshake).toHaveBeenCalledTimes(1);
    expect(protocol.shutdown).toHaveBeenCalledTimes(1);
    expect(worklet.terminate).toHaveBeenCalledTimes(1);
    expect(client.currentState).toBe("stopped");
  });

  it("P4 delegates package RPC and blocks account cleanup until the package closes", async () => {
    const worklet = createWorklet();
    const protocol = createProtocol();
    const cleaner = jest.fn(async () => undefined);
    const client = new BareRuntimeClient(
      () => worklet,
      () => protocol,
      "file:/tmp/p2p-product",
      cleaner,
    );
    const bytes = new Uint8Array([1, 2, 3]);

    await expect(client.openProductPackage("viewer-1", "a".repeat(64))).rejects.toMatchObject({ code: "INVALID_STATE" });
    await client.initialize();
    await client.openProductPackage("viewer-1", "a".repeat(64));
    await expect(client.clearProductAccount("viewer-1")).rejects.toMatchObject({ code: "INVALID_STATE" });
    await expect(client.readProductFile("manifest.json")).resolves.toEqual(bytes);
    await expect(client.hashProductBytes(bytes)).resolves.toBe("d".repeat(64));
    await client.cancelProductPackage();
    await client.clearProductAccount("viewer-1");

    expect(cleaner).toHaveBeenCalledWith("file:/tmp/p2p-product", "viewer-1");
  });

  it("P5 stops active playback before clearing the signed-out account", async () => {
    const worklet = createWorklet();
    const protocol = createProtocol();
    const cleaner = jest.fn(async () => undefined);
    const client = new BareRuntimeClient(() => worklet, () => protocol, "file:/tmp/p2p-product", cleaner);
    await client.initialize();

    await expect(client.startProductPlayback(playbackInput)).resolves.toEqual(playbackReceipt);
    await client.clearProductAccount("viewer-1");

    expect(protocol.stopProductPlayback).toHaveBeenCalledTimes(1);
    expect(cleaner).toHaveBeenCalledWith("file:/tmp/p2p-product", "viewer-1");
  });

  it("EC-F2 rejects duplicate initialization with a typed error", async () => {
    const worklet = createWorklet();
    const client = new BareRuntimeClient(
      () => worklet,
      () => createProtocol(),
      "file:/tmp/p2p-product",
    );

    await client.initialize();
    await expect(client.initialize()).rejects.toMatchObject({ code: "INVALID_STATE" });
  });

  it("EC-F2 exposes typed stopped and failed runtime states", async () => {
    const worklet = createWorklet();
    const client = new BareRuntimeClient(
      () => worklet,
      () => createProtocol({
        handshake: jest.fn(async () => {
          throw new Error("handshake failed");
        }),
      }),
      "file:/tmp/p2p-product",
    );

    await expect(client.ping()).rejects.toMatchObject({ code: "INVALID_STATE" });
    await expect(client.initialize()).rejects.toMatchObject({ code: "START_FAILED", message: "handshake failed" });
    expect(client.currentState).toBe("failed");
    expect(worklet.terminate).toHaveBeenCalledTimes(1);
  });

  it("EC-F2 keeps a released startup stopped when its handshake resolves later", async () => {
    const worklet = createWorklet();
    let resolveHandshake: (value: RuntimeHandshake) => void = () => undefined;
    const handshakePromise = new Promise<RuntimeHandshake>((resolve) => {
      resolveHandshake = resolve;
    });
    const protocol = createProtocol({ handshake: jest.fn(() => handshakePromise) });
    const client = new BareRuntimeClient(() => worklet, () => protocol, "file:/tmp/p2p-product");

    const starting = client.initialize();
    await client.shutdown();
    resolveHandshake(handshake);

    await expect(starting).rejects.toMatchObject({ code: "INVALID_STATE" });
    expect(client.currentState).toBe("stopped");
    expect(worklet.terminate).toHaveBeenCalledTimes(1);
  });
});
