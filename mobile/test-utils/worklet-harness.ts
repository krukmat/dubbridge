import { RUNTIME_PROTOCOL_VERSION, encodeProtocolValue } from "../src/p2p/runtime/protocol";
import { installRuntimeWorklet } from "../src/p2p/runtime/worklet";

export type RuntimeEventName = "suspend" | "resume" | "uncaughtException" | "unhandledRejection";

export function versioned(payload: Record<string, unknown>): Record<string, unknown> {
  return { protocolVersion: RUNTIME_PROTOCOL_VERSION, ...payload };
}

export function workletHarness(argv?: string[]) {
  const listeners = new Map<RuntimeEventName, (...args: unknown[]) => void>();
  const replies: Array<Record<string, unknown>> = [];
  const events: Array<{ command: number; value: Record<string, unknown> }> = [];
  let requestHandler:
    | ((request: { command: number; data: Uint8Array; reply(data: string): void }) => void)
    | undefined;
  const ipc = { end: jest.fn() };
  const runtime = {
    argv,
    version: "1.2.3-test",
    on: (event: RuntimeEventName, listener: (...args: unknown[]) => void) => listeners.set(event, listener),
  };
  const rpc = {
    event: (command: number) => ({
      send: (data: string) => events.push({ command, value: JSON.parse(data) }),
    }),
  };

  installRuntimeWorklet(runtime, ipc, (_stream, onRequest) => {
    requestHandler = onRequest as typeof requestHandler;
    return rpc;
  });

  const request = (command: number, payload: unknown = { protocolVersion: RUNTIME_PROTOCOL_VERSION }) => {
    requestHandler?.({
      command,
      data: encodeProtocolValue(payload),
      reply: (data) => replies.push(JSON.parse(data)),
    });
  };

  return { events, ipc, listeners, replies, request };
}
