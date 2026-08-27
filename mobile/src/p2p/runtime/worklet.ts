import RPC = require("bare-rpc");

import {
  RUNTIME_CAPABILITIES,
  RUNTIME_COMMAND,
  RUNTIME_PROTOCOL_VERSION,
  RuntimeProtocolError,
  decodeRequestPayload,
  type RuntimeFatalCode, type RuntimeProtocolErrorCode,
} from "./protocol";

interface WorkletRuntime {
  readonly version?: string;
  on(event: "suspend" | "resume", listener: () => void): void;
  on(event: "uncaughtException", listener: (error: unknown) => void): void;
  on(event: "unhandledRejection", listener: (reason: unknown) => void): void;
}

interface WorkletIpc {
  end(): void;
}

interface IncomingRequest {
  readonly command: number;
  readonly data: Uint8Array | null;
  reply(data: string): void;
}

interface WorkletRpc {
  event(command: number): { send(data: string): void };
}

type WorkletRpcFactory = (
  stream: WorkletIpc,
  onRequest: (request: IncomingRequest) => void | Promise<void>,
) => WorkletRpc;

function versioned(payload: Record<string, unknown>): Record<string, unknown> {
  return { protocolVersion: RUNTIME_PROTOCOL_VERSION, ...payload };
}

function success(result: unknown): string {
  return JSON.stringify(versioned({ ok: true, result }));
}

function failure(code: RuntimeProtocolErrorCode, message: string): string {
  return JSON.stringify(versioned({ ok: false, error: { code, message } }));
}

function safeReply(request: IncomingRequest, payload: string, closeOnce: () => void): void {
  try {
    request.reply(payload);
  } catch {
    closeOnce();
  }
}

function handleRequest(
  runtime: WorkletRuntime,
  request: IncomingRequest,
  closeOnce: () => void,
): void {
  try {
    decodeRequestPayload(request.data);
    if (request.command === RUNTIME_COMMAND.HANDSHAKE) {
      safeReply(
        request,
        success({
          ...versioned({
            runtimeVersion: runtime.version ?? "unknown",
            capabilities: [...RUNTIME_CAPABILITIES],
          }),
        }),
        closeOnce,
      );
      return;
    }
    if (request.command === RUNTIME_COMMAND.PING) {
      safeReply(request, success("pong"), closeOnce);
      return;
    }
    if (request.command === RUNTIME_COMMAND.SHUTDOWN) {
      safeReply(request, success("stopped"), closeOnce);
      closeOnce();
      return;
    }
    safeReply(request, failure("INVALID_PAYLOAD", "Runtime command is not supported"), closeOnce);
  } catch (error) {
    const protocolError =
      error instanceof RuntimeProtocolError
        ? error
        : new RuntimeProtocolError("REMOTE_FAILURE", "Runtime request failed");
    safeReply(request, failure(protocolError.code, protocolError.message), closeOnce);
  }
}

function registerRuntimeHandlers(
  runtime: WorkletRuntime,
  sendEvent: (command: number, payload: unknown) => void,
  sendFatal: (code: RuntimeFatalCode) => void,
): void {
  runtime.on("suspend", () =>
    sendEvent(
      RUNTIME_COMMAND.LIFECYCLE_EVENT,
      versioned({ type: "lifecycle", state: "suspended" }),
    ),
  );
  runtime.on("resume", () =>
    sendEvent(
      RUNTIME_COMMAND.LIFECYCLE_EVENT,
      versioned({ type: "lifecycle", state: "resumed" }),
    ),
  );
  runtime.on("uncaughtException", () => sendFatal("UNCAUGHT_EXCEPTION"));
  runtime.on("unhandledRejection", () => sendFatal("UNHANDLED_REJECTION"));
}

export function installRuntimeWorklet(
  runtime: WorkletRuntime,
  ipc: WorkletIpc,
  createRpc: WorkletRpcFactory = (stream, onRequest) =>
    new RPC(stream as ConstructorParameters<typeof RPC>[0], onRequest as ConstructorParameters<typeof RPC>[1]),
): WorkletRpc {
  let closed = false;
  let rpc: WorkletRpc;

  const closeOnce = () => {
    if (closed) return;
    closed = true;
    queueMicrotask(() => ipc.end());
  };

  const sendEvent = (command: number, payload: unknown) => {
    if (closed) return;
    try {
      rpc.event(command).send(JSON.stringify(payload));
    } catch {
      closeOnce();
    }
  };

  const sendFatal = (code: RuntimeFatalCode) => {
    sendEvent(
      RUNTIME_COMMAND.FATAL_EVENT,
      versioned({
        error: { code, message: "Bare runtime terminated unexpectedly" }, type: "fatal",
      }),
    );
    closeOnce();
  };

  rpc = createRpc(ipc, (request) => handleRequest(runtime, request, closeOnce));
  registerRuntimeHandlers(runtime, sendEvent, sendFatal);

  return rpc;
}

const globals = globalThis as typeof globalThis & {
  Bare?: WorkletRuntime;
  BareKit?: { IPC?: WorkletIpc };
};

if (globals.Bare && globals.BareKit?.IPC) {
  installRuntimeWorklet(globals.Bare, globals.BareKit.IPC);
}
