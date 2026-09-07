import RPC = require("bare-rpc");

import { RUNTIME_COMMAND, type RuntimeFatalCode } from "./protocol";
import type { WorkletRuntime } from "./transient-drive";
import { handleRequest, versioned, type IncomingRequest } from "./worklet-request-handler";

export { configureTransientDriveDependenciesForTest } from "./transient-drive";
export { configureSeedDependenciesForTest } from "./transient-seed";

interface WorkletIpc {
  end(): void;
}

interface WorkletRpc {
  event(command: number): { send(data: string): void };
}

type WorkletRpcFactory = (
  stream: WorkletIpc,
  onRequest: (request: IncomingRequest) => void | Promise<void>,
) => WorkletRpc;

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