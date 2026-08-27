export type BareCommand = "initialize" | "ping" | "shutdown";
export type BareState = "idle" | "starting" | "ready" | "stopped";
export type BareResultValue = "ready" | "pong" | "stopped";

export interface BareRequest {
  type: "request";
  id: string;
  command: BareCommand;
}

interface BareResult {
  type: "result";
  id: string;
  value: BareResultValue;
}

interface BareError {
  type: "error";
  id: string | null;
  code: string;
  message: string;
}

export type BareReply = BareResult | BareError;

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object";
}

function isBareResult(value: unknown): value is BareResult {
  return (
    isRecord(value) &&
    value.type === "result" &&
    typeof value.id === "string" &&
    (value.value === "ready" || value.value === "pong" || value.value === "stopped")
  );
}

function isBareError(value: unknown): value is BareError {
  return (
    isRecord(value) &&
    value.type === "error" &&
    (typeof value.id === "string" || value.id === null) &&
    typeof value.code === "string" &&
    typeof value.message === "string"
  );
}

export function parseBareReply(raw: string): BareReply | null {
  try {
    const value: unknown = JSON.parse(raw);
    return isBareResult(value) || isBareError(value) ? value : null;
  } catch {
    return null;
  }
}
