import b4a from "b4a";

import {
  RUNTIME_CAPABILITIES,
  RUNTIME_COMMAND,
  RUNTIME_PROTOCOL_VERSION,
  RuntimeProtocolError,
  decodeDiscoverAndReplicateRequest,
  decodeOpenProductPackageRequest,
  decodeReadProductFileRequest,
  decodeRequestPayload,
  type RuntimeProtocolErrorCode,
} from "./protocol";
import { ProductPackageRuntime } from "./product-package-runtime";
import { discoverAndReplicate } from "./transient-replication";
import { openCloseTransientDrive, openHeldTransientDrive, type WorkletRuntime } from "./transient-drive";
import { writeHashSeed } from "./transient-seed";

export interface IncomingRequest {
  readonly command: number;
  readonly data: Uint8Array | null;
  reply(data: string): void;
}

const productPackages = new ProductPackageRuntime();

export function versioned(payload: Record<string, unknown>): Record<string, unknown> {
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

export async function handleRequest(
  runtime: WorkletRuntime,
  request: IncomingRequest,
  closeOnce: () => void,
): Promise<void> {
  try {
    const payload = decodeRequestPayload(request.data);
    if (await handleLifecycleCommand(runtime, request, closeOnce)) return;
    if (await handleProofCommand(runtime, request, payload, closeOnce)) return;
    if (await handleProductCommand(runtime, request, payload, closeOnce)) return;
    safeReply(request, failure("INVALID_PAYLOAD", "Runtime command is not supported"), closeOnce);
  } catch (error) {
    const protocolError = toProtocolError(error);
    safeReply(request, failure(protocolError.code, protocolError.message), closeOnce);
  }
}

async function handleLifecycleCommand(
  runtime: WorkletRuntime,
  request: IncomingRequest,
  closeOnce: () => void,
): Promise<boolean> {
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
    return true;
  }
  if (request.command === RUNTIME_COMMAND.PING) {
    safeReply(request, success("pong"), closeOnce);
    return true;
  }
  if (request.command === RUNTIME_COMMAND.SHUTDOWN) {
    await productPackages.close();
    safeReply(request, success("stopped"), closeOnce);
    closeOnce();
    return true;
  }
  return false;
}

async function handleProofCommand(
  runtime: WorkletRuntime,
  request: IncomingRequest,
  payload: Record<string, unknown>,
  closeOnce: () => void,
): Promise<boolean> {
  if (request.command === RUNTIME_COMMAND.OPEN_CLOSE_TRANSIENT_DRIVE) {
    safeReply(request, success(await openCloseTransientDrive(runtime)), closeOnce);
    return true;
  }
  if (request.command === RUNTIME_COMMAND.SEED_WRITE_HASH_DELETE) {
    safeReply(request, success(await writeHashSeed(runtime)), closeOnce);
    return true;
  }
  if (request.command !== RUNTIME_COMMAND.DISCOVER_AND_REPLICATE) return false;

  const { topic, role } = decodeDiscoverAndReplicateRequest(payload);
  const drive = await openHeldTransientDrive(runtime);
  const receipt = await discoverAndReplicate(Buffer.from(topic, "hex"), role, drive);
  safeReply(request, success(receipt), closeOnce);
  return true;
}

async function handleProductCommand(
  runtime: WorkletRuntime,
  request: IncomingRequest,
  payload: Record<string, unknown>,
  closeOnce: () => void,
): Promise<boolean> {
  if (request.command === RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE) {
    const { externalPublicationId } = decodeOpenProductPackageRequest(payload);
    await productPackages.open(runtime, externalPublicationId);
    safeReply(request, success("opened"), closeOnce);
    return true;
  }
  if (request.command === RUNTIME_COMMAND.READ_PRODUCT_FILE) {
    const { path } = decodeReadProductFileRequest(payload);
    const bytes = await productPackages.read(path);
    safeReply(request, success(productFileReceipt(path, bytes)), closeOnce);
    return true;
  }
  if (request.command === RUNTIME_COMMAND.CLOSE_PRODUCT_PACKAGE) {
    await productPackages.close();
    safeReply(request, success("closed"), closeOnce);
    return true;
  }
  if (request.command === RUNTIME_COMMAND.CANCEL_PRODUCT_PACKAGE) {
    await productPackages.cancel();
    safeReply(request, success("cancelled"), closeOnce);
    return true;
  }
  return false;
}

function productFileReceipt(path: string, bytes: Uint8Array): Record<string, unknown> {
  return {
    capability: "product-package-file",
    schema_version: 1,
    path,
    byte_count: bytes.byteLength,
    bytes_base64: b4a.toString(bytes, "base64"),
  };
}

function toProtocolError(error: unknown): RuntimeProtocolError {
  return error instanceof RuntimeProtocolError
    ? error
    : new RuntimeProtocolError("REMOTE_FAILURE", "Runtime request failed");
}
