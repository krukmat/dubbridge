import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

import RPC = require("bare-rpc");

import {
  BareRpcPort,
  RUNTIME_CAPABILITIES,
  RUNTIME_COMMAND,
  RUNTIME_PROTOCOL_VERSION,
  RuntimeProtocolClient,
  RuntimeProtocolError,
  decodeRequestPayload,
  decodeRuntimeEvent,
  encodeProtocolValue, type RuntimeRpcPort,
} from "../../src/p2p/runtime/protocol";
import { installRuntimeWorklet } from "../../src/p2p/runtime/worklet";

const mobileRoot = path.resolve(__dirname, "../..");

function versioned(payload: Record<string, unknown>): Record<string, unknown> {
  return { protocolVersion: RUNTIME_PROTOCOL_VERSION, ...payload };
}

function response(result: unknown): Uint8Array {
  return encodeProtocolValue(versioned({ ok: true, result }));
}

class FakePort implements RuntimeRpcPort {
  closedWith: Error | null = null;
  idle = true;

  constructor(
    private readonly respond: (
      command: number,
      payload: string,
    ) => Promise<Uint8Array | string | null>,
  ) {}

  close(error: Error): void {
    this.closedWith = error;
  }

  request(command: number, payload: string): Promise<Uint8Array | string | null> {
    return this.respond(command, payload);
  }
}

type RuntimeEventName = "suspend" | "resume" | "uncaughtException" | "unhandledRejection";

class MemoryDuplex {
  destroying = false;
  peer: MemoryDuplex | null = null;
  private readonly listeners = new Map<string, Array<(...args: unknown[]) => void>>();

  on(event: string, listener: (...args: unknown[]) => void): this {
    const listeners = this.listeners.get(event) ?? [];
    listeners.push(listener);
    this.listeners.set(event, listeners);
    return this;
  }

  write(data: Uint8Array): boolean {
    this.peer?.emit("data", data);
    return !this.destroying;
  }

  destroy(error?: Error): this {
    if (this.destroying) return this;
    this.destroying = true;
    if (error) this.emit("error", error);
    this.emit("close");
    return this;
  }

  private emit(event: string, ...args: unknown[]): void {
    for (const listener of this.listeners.get(event) ?? []) listener(...args);
  }
}

function duplexPair(): [MemoryDuplex, MemoryDuplex] {
  const left = new MemoryDuplex();
  const right = new MemoryDuplex();
  left.peer = right;
  right.peer = left;
  return [left, right];
}

function workletHarness() {
  const listeners = new Map<RuntimeEventName, (...args: unknown[]) => void>();
  const replies: Array<Record<string, unknown>> = [];
  const events: Array<{ command: number; value: Record<string, unknown> }> = [];
  let requestHandler: ((request: { command: number; data: Uint8Array; reply(data: string): void }) => void) | undefined;
  const ipc = { end: jest.fn() };
  const runtime = {
    version: "1.2.3-test",
    on: (event: RuntimeEventName, listener: (...args: unknown[]) => void) => listeners.set(event, listener),
  };
  const rpc = {
    event: (command: number) => ({
      send: (data: string) => events.push({ command, value: JSON.parse(data) }),
    }),
  };

  installRuntimeWorklet(runtime, ipc, (_stream, onRequest) => {
    requestHandler = onRequest as typeof requestHandler;
    return rpc;
  });

  const request = (command: number, payload: unknown = { protocolVersion: RUNTIME_PROTOCOL_VERSION }) => {
    requestHandler?.({
      command,
      data: encodeProtocolValue(payload),
      reply: (data) => replies.push(JSON.parse(data)),
    });
  };

  return { events, ipc, listeners, replies, request };
}

describe("P1.F1 runtime protocol", () => {
  it("HP-F1 builds the committed worklet bundle deterministically", () => {
    execFileSync(process.execPath, ["scripts/build-bare-worklet.mjs", "--check"], {
      cwd: mobileRoot,
      stdio: "pipe",
    });
    execFileSync(process.execPath, ["scripts/build-bare-worklet.mjs", "--check"], {
      cwd: mobileRoot,
      stdio: "pipe",
    });
    expect(
      createHash("sha256")
        .update(readFileSync(path.join(mobileRoot, "src/p2p/runtime/worklet.bundle.js")))
        .digest("hex"),
    ).toBe(
      createHash("sha256")
        .update(readFileSync(path.join(mobileRoot, "src/p2p/runtime/worklet.bundle.js")))
        .digest("hex"),
    );
  });

  it("HP-F1 negotiates capabilities, pings, and shuts down without pending work", async () => {
    const port = new FakePort(async (command, payload) => {
      expect(decodeRequestPayload(payload)).toEqual({ protocolVersion: RUNTIME_PROTOCOL_VERSION });
      if (command === RUNTIME_COMMAND.HANDSHAKE) {
        return response(versioned({
          runtimeVersion: "1.2.3-test",
          capabilities: [...RUNTIME_CAPABILITIES],
        }));
      }
      if (command === RUNTIME_COMMAND.PING) return response("pong");
      return response("stopped");
    });
    const client = new RuntimeProtocolClient(port, 100);

    await expect(client.handshake()).resolves.toMatchObject({ runtimeVersion: "1.2.3-test" });
    await expect(client.ping()).resolves.toBe("pong");
    await expect(client.shutdown()).resolves.toBeUndefined();
    expect(client.idle).toBe(true);
  });

  it("HP-F1 carries requests and lifecycle events through bare-rpc", async () => {
    const [hostStream, workletStream] = duplexPair();
    const received: { events: unknown[]; errors: RuntimeProtocolError[] } = { events: [], errors: [] };
    const port = new BareRpcPort(
      hostStream as never,
      (event) => received.events.push(event),
      (error) => received.errors.push(error),
    );
    const workletRpc = new RPC(workletStream as never, (request) => {
      const payload = decodeRequestPayload(request.data);
      expect(payload).toEqual({ protocolVersion: RUNTIME_PROTOCOL_VERSION });
      request.reply(
        JSON.stringify(versioned({ ok: true, result: "pong" })),
      );
    });

    await expect(new RuntimeProtocolClient(port, 100).ping()).resolves.toBe("pong");
    workletRpc.event(RUNTIME_COMMAND.LIFECYCLE_EVENT).send(
      JSON.stringify(versioned({ type: "lifecycle", state: "resumed" })),
    );
    workletRpc.event(RUNTIME_COMMAND.LIFECYCLE_EVENT).send("not-json");

    expect(received.events).toEqual([expect.objectContaining({ state: "resumed" })]);
    expect(received.errors).toEqual([expect.objectContaining({ code: "INVALID_PAYLOAD" })]);
    expect(port.idle).toBe(true);
  });

  it("EC-F1 rejects unsupported versions and malformed payloads with typed errors", () => {
    expect(() => decodeRequestPayload(encodeProtocolValue({ protocolVersion: 99 }))).toThrow(
      expect.objectContaining({ code: "UNSUPPORTED_VERSION" }),
    );
    expect(() => decodeRequestPayload(new TextEncoder().encode("not-json"))).toThrow(
      expect.objectContaining({ code: "INVALID_PAYLOAD" }),
    );
    expect(() => decodeRequestPayload(null)).toThrow(
      expect.objectContaining({ code: "INVALID_PAYLOAD" }),
    );
  });

  it("EC-F1 rejects incomplete capabilities and redacts remote failure details", async () => {
    await expect(new RuntimeProtocolClient(new FakePort(async () =>
      response(versioned({
        runtimeVersion: "1.2.3-test",
        capabilities: ["ping"],
      })),
    )).handshake()).rejects.toMatchObject({
      code: "INVALID_PAYLOAD",
    });

    const remoteFailure = new FakePort(async () =>
      encodeProtocolValue(versioned({
        ok: false,
        error: { code: "REMOTE_FAILURE", message: "secret-token-must-not-leak" },
      })),
    );
    await expect(new RuntimeProtocolClient(remoteFailure).ping()).rejects.toMatchObject({
      code: "REMOTE_FAILURE",
      message: "Runtime request failed",
    });
  });

  it("EC-F1 translates channel closure and invalid acknowledgements", async () => {
    const closed = new FakePort(async () => {
      throw new Error("low-level secret");
    });
    await expect(new RuntimeProtocolClient(closed).ping()).rejects.toMatchObject({
      code: "CHANNEL_CLOSED",
      message: "Runtime channel closed before replying",
    });

    await expect(
      new RuntimeProtocolClient(new FakePort(async () => response("not-pong"))).ping(),
    ).rejects.toMatchObject({ code: "INVALID_PAYLOAD" });
    await expect(
      new RuntimeProtocolClient(new FakePort(async () => response("not-stopped"))).shutdown(),
    ).rejects.toMatchObject({ code: "INVALID_PAYLOAD" });
  });

  it("EC-F1 times out, closes the channel, and clears pending work", async () => {
    jest.useFakeTimers();
    const port = new FakePort(() => new Promise(() => undefined));
    const client = new RuntimeProtocolClient(port, 50);
    const rejection = expect(client.ping()).rejects.toMatchObject({ code: "RPC_TIMEOUT" });

    await jest.advanceTimersByTimeAsync(50);
    await rejection;
    expect(port.closedWith).toBeInstanceOf(RuntimeProtocolError);
    expect(client.idle).toBe(true);
    jest.useRealTimers();
  });

  it("EC-F1 validates lifecycle events and rejects invalid states", () => {
    expect(
      decodeRuntimeEvent(
        encodeProtocolValue(versioned({ type: "lifecycle", state: "suspended" })),
      ),
    ).toMatchObject({ state: "suspended" });
    expect(() =>
      decodeRuntimeEvent(
        encodeProtocolValue(versioned({ type: "lifecycle", state: "paused" })),
      ),
    ).toThrow(expect.objectContaining({ code: "INVALID_LIFECYCLE" }));
    expect(
      decodeRuntimeEvent(
        encodeProtocolValue(versioned({
          error: { code: "UNCAUGHT_EXCEPTION", message: "Bare runtime terminated unexpectedly" }, type: "fatal",
        })),
      ),
    ).toMatchObject({ type: "fatal" });
    expect(() =>
      decodeRuntimeEvent(encodeProtocolValue({ type: "lifecycle", protocolVersion: 99, state: "resumed" })),
    ).toThrow(expect.objectContaining({ code: "INVALID_LIFECYCLE" }));
  });

  it("HP-F1 worklet replies to handshake/ping/shutdown and emits suspend/resume", async () => {
    const harness = workletHarness();
    harness.request(RUNTIME_COMMAND.HANDSHAKE);
    harness.request(RUNTIME_COMMAND.PING);
    harness.listeners.get("suspend")?.();
    harness.listeners.get("resume")?.();
    harness.request(RUNTIME_COMMAND.SHUTDOWN);
    await Promise.resolve();

    expect(harness.replies).toEqual([
      expect.objectContaining({ ok: true, result: expect.objectContaining({ runtimeVersion: "1.2.3-test" }) }),
      expect.objectContaining({ ok: true, result: "pong" }),
      expect.objectContaining({ ok: true, result: "stopped" }),
    ]);
    expect(harness.events).toEqual([
      expect.objectContaining({ command: RUNTIME_COMMAND.LIFECYCLE_EVENT, value: expect.objectContaining({ state: "suspended" }) }),
      expect.objectContaining({ command: RUNTIME_COMMAND.LIFECYCLE_EVENT, value: expect.objectContaining({ state: "resumed" }) }),
    ]);
    expect(harness.ipc.end).toHaveBeenCalledTimes(1);
  });

  it.each([
    ["uncaughtException", "UNCAUGHT_EXCEPTION"],
    ["unhandledRejection", "UNHANDLED_REJECTION"],
  ] as const)("EC-F1 emits a redacted fatal receipt for %s and closes", async (event, code) => {
    const harness = workletHarness();
    harness.listeners.get(event)?.(new Error("secret-token-must-not-leak"));
    await Promise.resolve();

    expect(harness.events).toEqual([
      {
        command: RUNTIME_COMMAND.FATAL_EVENT,
        value: versioned({ error: { code, message: "Bare runtime terminated unexpectedly" }, type: "fatal" }),
      },
    ]);
    expect(JSON.stringify(harness.events)).not.toContain("secret-token-must-not-leak");
    expect(harness.ipc.end).toHaveBeenCalledTimes(1);
  });

  it("EC-F1 returns typed errors for unsupported worklet requests", () => {
    const harness = workletHarness();
    harness.request(999);
    harness.request(RUNTIME_COMMAND.PING, { protocolVersion: 99 });

    expect(harness.replies).toEqual([
      expect.objectContaining({ ok: false, error: expect.objectContaining({ code: "INVALID_PAYLOAD" }) }),
      expect.objectContaining({ ok: false, error: expect.objectContaining({ code: "UNSUPPORTED_VERSION" }) }),
    ]);
  });
});
