import b4a from "b4a";

import {
  RUNTIME_PROTOCOL_VERSION,
  RUNTIME_CAPABILITIES,
  RuntimeProtocolError,
  type RuntimeCapability,
  type RuntimeProtocolErrorCode,
  type RuntimeHandshake,
  type RuntimeEvent,
  type RuntimeResponseEnvelope,
} from "./protocol";

const REDACTED_ERROR_MESSAGE: Record<RuntimeProtocolErrorCode, string> = {
  INVALID_PAYLOAD: "Runtime returned an invalid payload",
  UNSUPPORTED_VERSION: "Runtime protocol version is not supported",
  REMOTE_FAILURE: "Runtime request failed",
  RPC_TIMEOUT: "Runtime request timed out",
  INVALID_LIFECYCLE: "Runtime lifecycle event is invalid",
  CHANNEL_CLOSED: "Runtime channel closed before replying",
  PROOF_STORAGE_CONFIG_INVALID: "Proof storage configuration is invalid",
  TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED: "Transient drive dependency could not be loaded",
  TRANSIENT_DRIVE_BUNDLE_INVALID: "Transient drive bundle is invalid",
  TRANSIENT_DRIVE_OPEN_FAILED: "Transient drive could not be opened",
  TRANSIENT_DRIVE_CLOSE_FAILED: "Transient drive could not be closed",
  SEED_WRITE_FAILED: "Seed fixture could not be written",
  SEED_HASH_FAILED: "Seed fixture could not be hashed",
  SEED_CLOSE_FAILED: "Seed drive could not be closed",
  SEED_DELETE_FAILED: "Seed run directory could not be deleted",
  SEED_VERIFY_FAILED: "Seed run directory deletion could not be verified",
  REPLICATION_DISCOVERY_FAILED: "Replication peer discovery failed",
  REPLICATION_CONNECT_FAILED: "Replication peer connection failed",
  REPLICATION_TRANSFER_FAILED: "Replication transfer failed",
  REPLICATION_CANCELLED: "Replication was cancelled",
};

export class RuntimeCodec {
  static isRecord(value: unknown): value is Record<string, unknown> {
    return value !== null && typeof value === "object" && !Array.isArray(value);
  }

  static parseJson(value: Uint8Array | string | null): unknown {
    if (value === null) throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime reply was empty");
    try {
      return JSON.parse(typeof value === "string" ? value : b4a.toString(value));
    } catch {
      throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime reply was not valid JSON");
    }
  }

  static hasCurrentVersion(value: Record<string, unknown>): boolean {
    return value.protocolVersion === RUNTIME_PROTOCOL_VERSION;
  }

  static isErrorCode(value: unknown): value is RuntimeProtocolErrorCode {
    return Object.hasOwn(REDACTED_ERROR_MESSAGE, String(value));
  }

  static encode(value: unknown): Uint8Array {
    return b4a.from(JSON.stringify(value));
  }

  static versionedRecord(
    raw: Uint8Array | string | null,
    malformedMessage: string,
    malformedCode: RuntimeProtocolErrorCode,
    versionCode: RuntimeProtocolErrorCode,
  ): Record<string, unknown> {
    const value = RuntimeCodec.parseJson(raw);
    if (!RuntimeCodec.isRecord(value)) {
      throw new RuntimeProtocolError(malformedCode, malformedMessage);
    }
    if (!RuntimeCodec.hasCurrentVersion(value)) {
      throw new RuntimeProtocolError(versionCode, "Runtime protocol version is not supported");
    }
    return value;
  }

  static decodeRequest(raw: Uint8Array | string | null): Record<string, unknown> {
    return RuntimeCodec.versionedRecord(
      raw,
      "Runtime request must be an object",
      "INVALID_PAYLOAD",
      "UNSUPPORTED_VERSION",
    );
  }

  static responseEnvelope(value: Record<string, unknown>): RuntimeResponseEnvelope {
    if (value.ok === true && "result" in value) return value as RuntimeResponseEnvelope;
    if (
      value.ok === false &&
      RuntimeCodec.isRecord(value.error) &&
      RuntimeCodec.isErrorCode(value.error.code) &&
      typeof value.error.message === "string"
    ) {
      return value as RuntimeResponseEnvelope;
    }
    throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime reply envelope is malformed");
  }

  static decodeResponse(raw: Uint8Array | string | null): RuntimeResponseEnvelope {
    return RuntimeCodec.responseEnvelope(
      RuntimeCodec.versionedRecord(
        raw,
        "Runtime reply must be an object",
        "INVALID_PAYLOAD",
        "UNSUPPORTED_VERSION",
      ),
    );
  }

  static hasExpectedCapabilities(value: unknown): value is RuntimeCapability[] {
    return (
      Array.isArray(value) &&
      value.length === RUNTIME_CAPABILITIES.length &&
      value.every(
        (capability) =>
          typeof capability === "string" &&
          (RUNTIME_CAPABILITIES as readonly string[]).includes(capability),
      ) &&
      RUNTIME_CAPABILITIES.every((capability) => value.includes(capability))
    );
  }

  static decodeHandshake(value: unknown): RuntimeHandshake {
    if (
      !RuntimeCodec.isRecord(value) ||
      value.protocolVersion !== RUNTIME_PROTOCOL_VERSION ||
      typeof value.runtimeVersion !== "string" ||
      !RuntimeCodec.hasExpectedCapabilities(value.capabilities)
    ) {
      throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime handshake is malformed");
    }
    return value as unknown as RuntimeHandshake;
  }

  static lifecycleEvent(value: Record<string, unknown>): RuntimeEvent | null {
    return value.type === "lifecycle" && (value.state === "suspended" || value.state === "resumed")
      ? (value as RuntimeEvent)
      : null;
  }

  static fatalEvent(value: Record<string, unknown>): RuntimeEvent | null {
    return value.type === "fatal" &&
      RuntimeCodec.isRecord(value.error) &&
      (value.error.code === "UNCAUGHT_EXCEPTION" || value.error.code === "UNHANDLED_REJECTION") &&
      typeof value.error.message === "string"
      ? (value as RuntimeEvent)
      : null;
  }

  static decodeEvent(raw: Uint8Array | string | null): RuntimeEvent {
    return RuntimeCodec.eventFromRecord(
      RuntimeCodec.versionedRecord(
        raw,
        "Runtime event envelope is invalid",
        "INVALID_LIFECYCLE",
        "INVALID_LIFECYCLE",
      ),
    );
  }

  static eventFromRecord(value: Record<string, unknown>): RuntimeEvent {
    return RuntimeCodec.lifecycleEvent(value) ?? RuntimeCodec.fatalEvent(value) ?? RuntimeCodec.invalidEvent();
  }

  static invalidEvent(): never {
    throw new RuntimeProtocolError("INVALID_LIFECYCLE", "Runtime event payload is invalid");
  }

  static successResult(envelope: RuntimeResponseEnvelope): unknown {
    if (envelope.ok) return envelope.result;
    throw new RuntimeProtocolError(envelope.error.code, REDACTED_ERROR_MESSAGE[envelope.error.code]);
  }
}

export const {
  encode: encodeProtocolValue,
  decodeRequest: decodeRequestPayload,
  decodeResponse: decodeResponseEnvelope,
  decodeHandshake: decodeHandshakeResult,
  decodeEvent: decodeRuntimeEvent,
} = RuntimeCodec;