import b4a from "b4a";
import { RuntimeCodec } from "./protocol-codec";

export const RUNTIME_PROTOCOL_VERSION = 1 as const;
export const RUNTIME_CAPABILITIES = [
  "ping",
  "lifecycle:suspend",
  "lifecycle:resume",
  "fatal",
  "shutdown",
  "product-package:v1",
  "product-playback:v1",
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
  OPEN_PRODUCT_PACKAGE: 10,
  READ_PRODUCT_FILE: 11,
  CLOSE_PRODUCT_PACKAGE: 12,
  CANCEL_PRODUCT_PACKAGE: 13,
  HASH_PRODUCT_BYTES: 14,
  START_PRODUCT_PLAYBACK: 15,
  STOP_PRODUCT_PLAYBACK: 16,
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
  | "TRANSIENT_DRIVE_READ_FAILED"
  | "REPLICATION_DISCOVERY_FAILED"
  | "REPLICATION_CONNECT_FAILED"
  | "REPLICATION_TRANSFER_FAILED"
  | "REPLICATION_CANCELLED"
  | "PRODUCT_STORAGE_CONFIG_INVALID"
  | "PRODUCT_PACKAGE_OPEN_FAILED"
  | "PRODUCT_PACKAGE_NOT_OPEN"
  | "PRODUCT_PACKAGE_READ_FAILED"
  | "PRODUCT_PACKAGE_CLOSE_FAILED"
  | "PRODUCT_HASH_FAILED"
  | "PRODUCT_PLAYBACK_FAILED";

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

export interface OpenProductPackageRequest {
  protocolVersion: typeof RUNTIME_PROTOCOL_VERSION;
  accountScope: string;
  externalPublicationId: string;
}

export interface ReadProductFileRequest {
  protocolVersion: typeof RUNTIME_PROTOCOL_VERSION;
  path: string;
}

export interface HashProductBytesRequest {
  protocolVersion: typeof RUNTIME_PROTOCOL_VERSION;
  bytesBase64: string;
}

export interface StartProductPlaybackRequest {
  protocolVersion: typeof RUNTIME_PROTOCOL_VERSION;
  accountScope: string;
  assetId: string;
  externalPublicationId: string;
  lineageId: string;
  manifestDigestSha256: string;
  publicationId: string;
  ckBase64: string;
}

export interface ProductPlaybackReceipt {
  capability: "product-playback";
  schema_version: 1;
  playback_url: string;
}

export interface ProductPackageFileReceipt {
  capability: "product-package-file";
  schema_version: 1;
  path: string;
  byte_count: number;
  bytes_base64: string;
}

const ACCOUNT_SCOPE = /^[A-Za-z0-9._-]{1,128}$/;
const BASE64 = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/;
const SHA256_HEX = /^[0-9a-f]{64}$/;
const DRIVE_KEY = /^[0-9a-f]{64}$/;

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

export function decodeOpenProductPackageRequest(value: unknown): OpenProductPackageRequest {
  if (
    !RuntimeCodec.isRecord(value) ||
    value.protocolVersion !== RUNTIME_PROTOCOL_VERSION ||
    typeof value.accountScope !== "string" ||
    !ACCOUNT_SCOPE.test(value.accountScope) ||
    typeof value.externalPublicationId !== "string" ||
    !DRIVE_KEY.test(value.externalPublicationId)
  ) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product package identity is invalid");
  }
  return value as unknown as OpenProductPackageRequest;
}

export function decodeReadProductFileRequest(value: unknown): ReadProductFileRequest {
  if (
    !RuntimeCodec.isRecord(value) ||
    value.protocolVersion !== RUNTIME_PROTOCOL_VERSION ||
    typeof value.path !== "string" ||
    value.path.length === 0
  ) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product package path is invalid");
  }
  return value as unknown as ReadProductFileRequest;
}

export function decodeHashProductBytesRequest(value: unknown): HashProductBytesRequest {
  if (
    !RuntimeCodec.isRecord(value) ||
    value.protocolVersion !== RUNTIME_PROTOCOL_VERSION ||
    typeof value.bytesBase64 !== "string" ||
    !BASE64.test(value.bytesBase64)
  ) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product hash payload is invalid");
  }
  return value as unknown as HashProductBytesRequest;
}

function validPlaybackIdentity(value: Record<string, unknown>): boolean {
  return typeof value.accountScope === "string" && ACCOUNT_SCOPE.test(value.accountScope) &&
    typeof value.assetId === "string" && value.assetId.length > 0 && value.assetId.length <= 128 &&
    typeof value.publicationId === "string" && value.publicationId.length > 0 && value.publicationId.length <= 128 &&
    typeof value.lineageId === "string" && value.lineageId.length > 0 && value.lineageId.length <= 128 &&
    typeof value.externalPublicationId === "string" && DRIVE_KEY.test(value.externalPublicationId) &&
    typeof value.manifestDigestSha256 === "string" && SHA256_HEX.test(value.manifestDigestSha256);
}

export function decodeStartProductPlaybackRequest(value: unknown): StartProductPlaybackRequest {
  if (!RuntimeCodec.isRecord(value) || value.protocolVersion !== RUNTIME_PROTOCOL_VERSION ||
      !validPlaybackIdentity(value) || typeof value.ckBase64 !== "string" || !BASE64.test(value.ckBase64)) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product playback request is invalid");
  }
  let keyBytes: Uint8Array;
  try {
    keyBytes = b4a.from(value.ckBase64, "base64");
  } catch {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product playback content key is invalid");
  }
  if (keyBytes.byteLength !== 32) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Product playback content key is invalid");
  }
  return value as unknown as StartProductPlaybackRequest;
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

export function decodeProductFileReceipt(value: unknown, expectedPath: string): Uint8Array {
  if (!RuntimeCodec.isRecord(value)) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime product file reply is invalid");
  }
  const receipt = value as Partial<ProductPackageFileReceipt>;
  if (
    receipt.capability !== "product-package-file" ||
    receipt.schema_version !== 1 ||
    receipt.path !== expectedPath ||
    typeof receipt.byte_count !== "number" ||
    typeof receipt.bytes_base64 !== "string"
  ) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime product file reply is invalid");
  }
  const bytes = b4a.from(receipt.bytes_base64, "base64");
  if (bytes.byteLength !== receipt.byte_count) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime product file length is invalid");
  }
  return bytes;
}

export function decodeProductPlaybackReceipt(value: unknown): ProductPlaybackReceipt {
  if (!RuntimeCodec.isRecord(value)) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime playback reply is invalid");
  }
  const receipt = value as Partial<ProductPlaybackReceipt>;
  if (receipt.capability !== "product-playback" || receipt.schema_version !== 1 ||
      typeof receipt.playback_url !== "string" || !/^http:\/\/127\.0\.0\.1:\d+\/[0-9a-f]{32}\/index\.m3u8$/.test(receipt.playback_url)) {
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime playback reply is invalid");
  }
  return receipt as ProductPlaybackReceipt;
}

export { RuntimeCodec, encodeProtocolValue, decodeRequestPayload, decodeResponseEnvelope, decodeHandshakeResult, decodeRuntimeEvent } from "./protocol-codec";
