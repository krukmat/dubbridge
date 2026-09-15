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
    if (handleImmediateLifecycleCommand(runtime, request, closeOnce)) return;
    if (request.command === RUNTIME_COMMAND.SHUTDOWN) {
      await productPackages.close();
      safeReply(request, success("stopped"), closeOnce);
      closeOnce();
      return;
    }
    if (isProofCommand(request.command)) {
      await executeProofCommand(runtime, request, payload, closeOnce);
      return;
    }
    if (isProductCommand(request.command)) {
      await executeProductCommand(runtime, request, payload, closeOnce);
      return;
    }
    safeReply(request, failure("INVALID_PAYLOAD", "Runtime command is not supported"), closeOnce);
  } catch (error) {
    const protocolError = toProtocolError(error);
    safeReply(request, failure(protocolError.code, protocolError.message), closeOnce);
  }
}

function handleImmediateLifecycleCommand(
  runtime: WorkletRuntime,
  request: IncomingRequest,
  closeOnce: () => void,
): boolean {
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
  if (request.command !== RUNTIME_COMMAND.PING) return false;
  safeReply(request, success("pong"), closeOnce);
  return true;
}

function isProofCommand(command: number): boolean {
  return (
    command === RUNTIME_COMMAND.OPEN_CLOSE_TRANSIENT_DRIVE ||
    command === RUNTIME_COMMAND.SEED_WRITE_HASH_DELETE ||
    command === RUNTIME_COMMAND.DISCOVER_AND_REPLICATE
  );
}

async function executeProofCommand(
  runtime: WorkletRuntime,
  request: IncomingRequest,
  payload: Record<string, unknown>,
  closeOnce: () => void,
): Promise<void> {
  if (request.command === RUNTIME_COMMAND.OPEN_CLOSE_TRANSIENT_DRIVE) {
    safeReply(request, success(await openCloseTransientDrive(runtime)), closeOnce);
    return;
  }
  if (request.command === RUNTIME_COMMAND.SEED_WRITE_HASH_DELETE) {
    safeReply(request, success(await writeHashSeed(runtime)), closeOnce);
    return;
  }
  const { topic, role } = decodeDiscoverAndReplicateRequest(payload);
  const drive = await openHeldTransientDrive(runtime);
  const receipt = await discoverAndReplicate(Buffer.from(topic, "hex"), role, drive);
  safeReply(request, success(receipt), closeOnce);
}

function isProductCommand(command: number): boolean {
  return (
    command === RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE ||
    command === RUNTIME_COMMAND.READ_PRODUCT_FILE ||
    command === RUNTIME_COMMAND.CLOSE_PRODUCT_PACKAGE ||
    command === RUNTIME_COMMAND.CANCEL_PRODUCT_PACKAGE
  );
}

async function executeProductCommand(
  runtime: WorkletRuntime,
  request: IncomingRequest,
  payload: Record<string, unknown>,
  closeOnce: () => void,
): Promise<void> {
  if (request.command === RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE) {
    const { externalPublicationId } = decodeOpenProductPackageRequest(payload);
    await productPackages.open(runtime, externalPublicationId);
    safeReply(request, success("opened"), closeOnce);
    return;
  }
  if (request.command === RUNTIME_COMMAND.READ_PRODUCT_FILE) {
    const { path } = decodeReadProductFileRequest(payload);
    const bytes = await productPackages.read(path);
    safeReply(request, success(productFileReceipt(path, bytes)), closeOnce);
    return;
  }
  if (request.command === RUNTIME_COMMAND.CLOSE_PRODUCT_PACKAGE) {
    await productPackages.close();
    safeReply(request, success("closed"), closeOnce);
    return;
  }
  await productPackages.cancel();
  safeReply(request, success("cancelled"), closeOnce);
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
