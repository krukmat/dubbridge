import RPC = require("bare-rpc");

import {
  RUNTIME_CAPABILITIES,
  RUNTIME_COMMAND,
  RUNTIME_PROTOCOL_VERSION,
  RuntimeProtocolError,
  TRANSIENT_DRIVE_RECEIPT,
  decodeRequestPayload,
  type RuntimeFatalCode, type RuntimeProtocolErrorCode,
} from "./protocol";

interface WorkletRuntime {
  readonly version?: string;
  on(event: "suspend" | "resume", listener: () => void): void;
  on(event: "uncaughtException", listener: (error: unknown) => void): void;
  on(event: "unhandledRejection", listener: (reason: unknown) => void): void;
  argv?: string[];
}

interface TransientDriveStore {
  close(): Promise<void>;
}

interface TransientDrive {
  ready(): Promise<void>;
  close(): Promise<void>;
}

interface TransientDriveDependencies {
  Corestore: new (storage: string) => TransientDriveStore;
  Hyperdrive: new (store: TransientDriveStore) => TransientDrive;
}

let proofStorageUri = (runtime: WorkletRuntime): string => {
  let uri = runtime.argv?.[0];
  if (typeof uri !== "string" || !uri.startsWith("file:") || uri.length <= "file:".length) {
    throw new RuntimeProtocolError("PROOF_STORAGE_CONFIG_INVALID", "Proof storage configuration is invalid");
  }
  return uri;
};

let transientDriveDependencies = (): TransientDriveDependencies => {
  let Corestore: unknown;
  let Hyperdrive: unknown;
  try {
    Corestore = require("corestore");
    Hyperdrive = require("hyperdrive");
  } catch {
    throw new RuntimeProtocolError(
      "TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED",
      "Transient drive dependency could not be loaded",
    );
  }
  return validateTransientDriveDependencies({
    Corestore: Corestore as TransientDriveDependencies["Corestore"],
    Hyperdrive: Hyperdrive as TransientDriveDependencies["Hyperdrive"],
  });
};

function validateTransientDriveDependencies(value: unknown): TransientDriveDependencies {
  if (
    value === null ||
    typeof value !== "object" ||
    typeof (value as Partial<TransientDriveDependencies>).Corestore !== "function" ||
    typeof (value as Partial<TransientDriveDependencies>).Hyperdrive !== "function"
  ) {
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_BUNDLE_INVALID", "Transient drive bundle is invalid");
  }
  return value as TransientDriveDependencies;
}

function loadTransientDriveDependencies(): TransientDriveDependencies {
  try {
    return validateTransientDriveDependencies(transientDriveDependencies());
  } catch (error) {
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError(
      "TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED",
      "Transient drive dependency could not be loaded",
    );
  }
}

export function configureTransientDriveDependenciesForTest(
  load: () => unknown,
): () => void {
  const previous = transientDriveDependencies;
  transientDriveDependencies = () => load() as TransientDriveDependencies;
  return () => {
    transientDriveDependencies = previous;
  };
}

let openCloseTransientDrive = async (runtime: WorkletRuntime): Promise<typeof TRANSIENT_DRIVE_RECEIPT> => {
  let storageUri = proofStorageUri(runtime);
  let store: TransientDriveStore | undefined;
  let drive: TransientDrive | undefined;
  try {
    const { Corestore, Hyperdrive } = loadTransientDriveDependencies();
    store = new Corestore(storageUri);
    drive = new Hyperdrive(store);
    await drive.ready();
  } catch (error) {
    try {
      if (drive) await drive.close();
      else if (store) await store.close();
    } catch {
      throw new RuntimeProtocolError("TRANSIENT_DRIVE_CLOSE_FAILED", "Transient drive could not be closed");
    }
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_OPEN_FAILED", "Transient drive could not be opened");
  }
  try {
    await drive.close();
  } catch {
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_CLOSE_FAILED", "Transient drive could not be closed");
  }
  return TRANSIENT_DRIVE_RECEIPT;
};

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

async function handleRequest(
  runtime: WorkletRuntime,
  request: IncomingRequest,
  closeOnce: () => void,
): Promise<void> {
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
    if (request.command === RUNTIME_COMMAND.OPEN_CLOSE_TRANSIENT_DRIVE) {
      safeReply(request, success(await openCloseTransientDrive(runtime)), closeOnce);
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
