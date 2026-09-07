import {
  RUNTIME_CAPABILITIES,
  RUNTIME_COMMAND,
  RUNTIME_PROTOCOL_VERSION,
  RuntimeProtocolError,
  decodeDiscoverAndReplicateRequest,
  decodeRequestPayload,
  type RuntimeProtocolErrorCode,
} from "./protocol";
import { discoverAndReplicate } from "./transient-replication";
import { openCloseTransientDrive, openHeldTransientDrive, type WorkletRuntime } from "./transient-drive";
import { writeHashSeed } from "./transient-seed";

export interface IncomingRequest {
  readonly command: number;
  readonly data: Uint8Array | null;
  reply(data: string): void;
}

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
    if (request.command === RUNTIME_COMMAND.SEED_WRITE_HASH_DELETE) {
      safeReply(request, success(await writeHashSeed(runtime)), closeOnce);
      return;
    }
    if (request.command === RUNTIME_COMMAND.DISCOVER_AND_REPLICATE) {
      const { topic, role } = decodeDiscoverAndReplicateRequest(request.data);
      const drive = await openHeldTransientDrive(runtime);
      const receipt = await discoverAndReplicate(Buffer.from(topic, "hex"), role, drive);
      safeReply(request, success(receipt), closeOnce);
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