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
  capabilities: ["ping", "lifecycle:suspend", "lifecycle:resume", "fatal", "shutdown"],
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
    shutdown: jest.fn(async () => {
      state = "stopped";
    }),
  };
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
    const worklet = { IPC: {}, start: jest.fn(), terminate: jest.fn() } as unknown as BareRuntimeWorklet;
    const protocol: BareRuntimeProtocol = {
      handshake: jest.fn(async () => handshake),
      ping: jest.fn(async () => "pong" as const),
      shutdown: jest.fn(async () => undefined),
    };
    const client = new BareRuntimeClient(() => worklet, () => protocol);

    expect(client.currentState).toBe("stopped");
    await expect(client.initialize()).resolves.toEqual(handshake);
    await expect(client.ping()).resolves.toBe("pong");
    await client.shutdown();

    expect(worklet.start).toHaveBeenCalledTimes(1);
    expect(protocol.handshake).toHaveBeenCalledTimes(1);
    expect(protocol.shutdown).toHaveBeenCalledTimes(1);
    expect(worklet.terminate).toHaveBeenCalledTimes(1);
    expect(client.currentState).toBe("stopped");
  });

  it("EC-F2 rejects duplicate initialization with a typed error", async () => {
    const worklet = { IPC: {}, start: jest.fn(), terminate: jest.fn() } as unknown as BareRuntimeWorklet;
    const client = new BareRuntimeClient(() => worklet, () => ({
      handshake: jest.fn(async () => handshake),
      ping: jest.fn(async () => "pong" as const),
      shutdown: jest.fn(async () => undefined),
    }));

    await client.initialize();
    await expect(client.initialize()).rejects.toMatchObject({ code: "INVALID_STATE" });
  });

  it("EC-F2 exposes typed stopped and failed runtime states", async () => {
    const worklet = { IPC: {}, start: jest.fn(), terminate: jest.fn() } as unknown as BareRuntimeWorklet;
    const client = new BareRuntimeClient(() => worklet, () => ({
      handshake: jest.fn(async () => {
        throw new Error("handshake failed");
      }),
      ping: jest.fn(async () => "pong" as const),
      shutdown: jest.fn(async () => undefined),
    }));

    await expect(client.ping()).rejects.toMatchObject({ code: "INVALID_STATE" });
    await expect(client.initialize()).rejects.toMatchObject({ code: "START_FAILED", message: "handshake failed" });
    expect(client.currentState).toBe("failed");
    expect(worklet.terminate).toHaveBeenCalledTimes(1);
  });

  it("EC-F2 keeps a released startup stopped when its handshake resolves later", async () => {
    const worklet = { IPC: {}, start: jest.fn(), terminate: jest.fn() } as unknown as BareRuntimeWorklet;
    let resolveHandshake: (value: RuntimeHandshake) => void = () => undefined;
    const handshakePromise = new Promise<RuntimeHandshake>((resolve) => {
      resolveHandshake = resolve;
    });
    const protocol: BareRuntimeProtocol = {
      handshake: jest.fn(() => handshakePromise),
      ping: jest.fn(async () => "pong" as const),
      shutdown: jest.fn(async () => undefined),
    };
    const client = new BareRuntimeClient(() => worklet, () => protocol);

    const starting = client.initialize();
    await client.shutdown();
    resolveHandshake(handshake);

    await expect(starting).rejects.toMatchObject({ code: "INVALID_STATE" });
    expect(client.currentState).toBe("stopped");
    expect(worklet.terminate).toHaveBeenCalledTimes(1);
  });
});
