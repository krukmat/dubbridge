jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

import b4a from "b4a";

import { BareBridge, BareBridgeError } from "../../src/p2p/bare-bridge";

type DataListener = (data: Uint8Array) => void;

class MockIpc {
  readonly writes: Array<Record<string, unknown>> = [];
  private readonly listeners = new Set<DataListener>();

  constructor(private readonly onWrite: (request: Record<string, unknown>, ipc: MockIpc) => void) {}

  on(event: "data", listener: DataListener) {
    if (event === "data") this.listeners.add(listener);
  }

  removeListener(event: "data", listener: DataListener) {
    if (event === "data") this.listeners.delete(listener);
  }

  write(data: Uint8Array) {
    const request = JSON.parse(b4a.toString(data)) as Record<string, unknown>;
    this.writes.push(request);
    this.onWrite(request, this);
  }

  reply(value: Record<string, unknown> | string) {
    const payload = typeof value === "string" ? b4a.from(value) : b4a.from(JSON.stringify(value));
    for (const listener of this.listeners) listener(payload);
  }
}

class MockWorklet {
  readonly IPC: MockIpc;
  starts: Array<[string, string]> = [];
  terminateCalls = 0;

  constructor(onWrite: (request: Record<string, unknown>, ipc: MockIpc) => void) {
    this.IPC = new MockIpc(onWrite);
  }

  start(filename: string, source: string) {
    this.starts.push([filename, source]);
  }

  terminate() {
    this.terminateCalls += 1;
  }
}

function replyFor(command: string, request: Record<string, unknown>) {
  const value = command === "initialize" ? "ready" : command === "ping" ? "pong" : "stopped";
  return { type: "result", id: request.id, value };
}

describe("BareBridge", () => {
  it("runs initialize, ping, and shutdown through the typed IPC lifecycle", async () => {
    const worklet = new MockWorklet((request, ipc) => ipc.reply(replyFor(String(request.command), request)));
    const bridge = new BareBridge(() => worklet as never);

    await expect(bridge.initialize()).resolves.toBe("ready");
    await expect(bridge.ping()).resolves.toBe("pong");
    await expect(bridge.shutdown()).resolves.toBeUndefined();

    expect(worklet.starts).toHaveLength(1);
    expect(worklet.IPC.writes.map((request) => request.command)).toEqual(["initialize", "ping", "shutdown"]);
    expect(worklet.terminateCalls).toBe(1);
    expect(bridge.currentState).toBe("stopped");
  });

  it("contains a typed worklet failure without retaining the worklet", async () => {
    const worklet = new MockWorklet((request, ipc) =>
      ipc.reply({ type: "error", id: request.id, code: "WORKLET_FAILURE", message: "startup failed" }),
    );
    const bridge = new BareBridge(() => worklet as never);

    await expect(bridge.initialize()).rejects.toMatchObject({
      code: "WORKLET_FAILURE",
    });
    expect(worklet.terminateCalls).toBe(1);
    expect(bridge.currentState).toBe("stopped");
  });

  it("rejects a malformed reply and still allows deterministic cleanup", async () => {
    const worklet = new MockWorklet((request, ipc) => {
      if (request.command === "initialize") return ipc.reply(replyFor("initialize", request));
      if (request.command === "ping") return ipc.reply("not-json");
      return ipc.reply(replyFor("shutdown", request));
    });
    const bridge = new BareBridge(() => worklet as never);

    await bridge.initialize();
    await expect(bridge.ping()).rejects.toMatchObject({ code: "MALFORMED_REPLY" });
    await bridge.shutdown();

    expect(worklet.terminateCalls).toBe(1);
    expect(bridge.currentState).toBe("stopped");
  });

  it("releases a starting worklet when shutdown arrives before ready", async () => {
    const worklet = new MockWorklet(() => {});
    const bridge = new BareBridge(() => worklet as never);

    const initialization = bridge.initialize();
    await bridge.shutdown();

    await expect(initialization).rejects.toMatchObject({ code: "SHUTDOWN_BEFORE_READY" });
    expect(worklet.terminateCalls).toBe(1);
    expect(bridge.currentState).toBe("stopped");
  });

  it("drops a late ping reply after shutdown without a stale pending handle", async () => {
    let pendingPing: Record<string, unknown> | undefined;
    const worklet = new MockWorklet((request, ipc) => {
      if (request.command === "initialize") return ipc.reply(replyFor("initialize", request));
      if (request.command === "ping") {
        pendingPing = request;
        return;
      }
      return ipc.reply(replyFor("shutdown", request));
    });
    const bridge = new BareBridge(() => worklet as never);

    await bridge.initialize();
    const ping = bridge.ping();
    await bridge.shutdown();
    worklet.IPC.reply({ type: "result", id: pendingPing?.id, value: "pong" });

    await expect(ping).rejects.toMatchObject({ code: "WORKLET_FAILURE" });
    expect(worklet.terminateCalls).toBe(1);
  });
});
