import b4a from "b4a";
import { RuntimeCodec } from "./protocol-codec";

export const RUNTIME_PROTOCOL_VERSION = 1 as const;
export const RUNTIME_CAPABILITIES = [
  "ping",
  "lifecycle:suspend",
  "lifecycle:resume",
  "fatal",
  "shutdown",
] as const;

export const RUNTIME_COMMAND = {
  HANDSHAKE: 1,
  PING: 2,
  SHUTDOWN: 3,
  LIFECYCLE_EVENT: 4,
  FATAL_EVENT: 5,
  OPEN_CLOSE_TRANSIENT_DRIVE: 6,
  SEED_WRITE_HASH_DELETE: 7,
  DISCOVER_AND_REPLICATE: 8,
  CANCEL_REPLICATION: 9,
} as const;

export type RuntimeCapability = (typeof RUNTIME_CAPABILITIES)[number];
export type RuntimeFatalCode = "UNCAUGHT_EXCEPTION" | "UNHANDLED_REJECTION";
export type RuntimeProtocolErrorCode =
  | "INVALID_PAYLOAD"
  | "UNSUPPORTED_VERSION"
  | "REMOTE_FAILURE"
  | "RPC_TIMEOUT"
  | "INVALID_LIFECYCLE"
  | "CHANNEL_CLOSED"
  | "PROOF_STORAGE_CONFIG_INVALID"
  | "TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED"
  | "TRANSIENT_DRIVE_BUNDLE_INVALID"
  | "TRANSIENT_DRIVE_OPEN_FAILED"
  | "TRANSIENT_DRIVE_CLOSE_FAILED"
  | "SEED_WRITE_FAILED"
  | "SEED_HASH_FAILED"
  | "SEED_CLOSE_FAILED"
  | "SEED_DELETE_FAILED"
  | "SEED_VERIFY_FAILED"
  | "DIGEST_COMPARE_FAILED"
  | "REPLICATION_DISCOVERY_FAILED"
  | "REPLICATION_CONNECT_FAILED"
  | "REPLICATION_TRANSFER_FAILED"
  | "REPLICATION_CANCELLED";

export const TRANSIENT_DRIVE_RECEIPT = {
  capability: "transient-hyperdrive-corestore",
  schema_version: 1,
} as const;

export interface SeedWriteHashDeleteReceipt {
  capability: "seed-write-hash-delete";
  schema_version: 1;
  byte_count: number;
  sha256: string;
}

export interface DiscoverAndReplicateReceipt {
  capability: "discover-and-replicate";
  schema_version: 1;
  role: "seed" | "client";
  byte_count: number;
}

export interface DiscoverAndReplicateRequest {
  protocolVersion: typeof RUNTIME_PROTOCOL_VERSION;
  topic: string;
  role: "seed" | "client";
}

export function decodeDiscoverAndReplicateRequest(value: unknown): DiscoverAndReplicateRequest {
  if (
    !RuntimeCodec.isRecord(value) ||
    value.protocolVersion !== RUNTIME_PROTOCOL_VERSION ||
    typeof value.topic !== "string" ||
    value.topic.length !== 64 ||
    !/^[0-9a-fA-F]+$/.test(value.topic) ||
    (value.role !== "seed" && value.role !== "client")
  ) {
    throw new RuntimeProtocolError("REPLICATION_DISCOVERY_FAILED", "Replication peer discovery failed");
  }
  return value as unknown as DiscoverAndReplicateRequest;
}

export interface RuntimeHandshake {
  protocolVersion: typeof RUNTIME_PROTOCOL_VERSION;
  runtimeVersion: string;
  capabilities: RuntimeCapability[];
}

export type RuntimeEvent =
  | { protocolVersion: typeof RUNTIME_PROTOCOL_VERSION; type: "lifecycle"; state: "suspended" | "resumed" }
  | { protocolVersion: typeof RUNTIME_PROTOCOL_VERSION; type: "fatal"; error: { code: RuntimeFatalCode; message: string } };

export type RuntimeResponseEnvelope =
  | { ok: true; protocolVersion: typeof RUNTIME_PROTOCOL_VERSION; result: unknown }
  | {
      ok: false;
      protocolVersion: typeof RUNTIME_PROTOCOL_VERSION;
      error: { code: RuntimeProtocolErrorCode; message: string };
    };

export interface RuntimeRpcPort {
  readonly idle: boolean;
  close(error: Error): void;
  request(command: number, payload: string): Promise<Uint8Array | string | null>;
}

export class RuntimeProtocolError extends Error {
  constructor(
    readonly code: RuntimeProtocolErrorCode,
    message: string,
  ) {
    super(message);
    this.name = "RuntimeProtocolError";
  }
}

export { RuntimeCodec, encodeProtocolValue, decodeRequestPayload, decodeResponseEnvelope, decodeHandshakeResult, decodeRuntimeEvent } from "./protocol-codec";