import b4a from "b4a";
import { Worklet } from "react-native-bare-kit";

import { BARE_WORKLET_FILENAME, BARE_WORKLET_SOURCE } from "./bare-worklet";
import { parseBareReply } from "./bare-protocol";
import type { BareCommand, BareRequest, BareResultValue, BareState } from "./bare-protocol";

interface PendingRequest {
  reject: (error: BareBridgeError) => void;
  resolve: (value: BareResultValue) => void;
  timeout: ReturnType<typeof setTimeout>;
}

export class BareBridgeError extends Error {
  constructor(
    readonly code:
      | "INVALID_STATE"
      | "MALFORMED_REPLY"
      | "RPC_TIMEOUT"
      | "SHUTDOWN_BEFORE_READY"
      | "WORKLET_FAILURE"
      | "WORKLET_PROTOCOL",
    message: string,
  ) {
    super(message);
    this.name = "BareBridgeError";
  }
}

export type BareWorkletFactory = () => Worklet;

export class BareBridge {
  private readonly pending = new Map<string, PendingRequest>();
  private readonly onData = (data: unknown) => {
    if (!(data instanceof Uint8Array)) {
      this.rejectPending(new BareBridgeError("MALFORMED_REPLY", "Bare worklet returned a non-binary reply"));
      return;
    }
    this.handleData(data);
  };
  private nextRequestId = 0;
  private state: BareState = "idle";
  private worklet: Worklet | null = null;

  constructor(
    private readonly createWorklet: BareWorkletFactory = () => new Worklet(),
    private readonly timeoutMs = 5_000,
  ) {}

  get currentState(): BareState {
    return this.state;
  }

  async initialize(): Promise<"ready"> {
    if (this.state !== "idle") {
      throw new BareBridgeError("INVALID_STATE", "Bare worklet is already started or stopped");
    }

    this.state = "starting";
    this.worklet = this.createWorklet();
    this.worklet.IPC.on("data", this.onData);

    try {
      this.worklet.start(BARE_WORKLET_FILENAME, BARE_WORKLET_SOURCE);
      const value = await this.request("initialize");

      if (value !== "ready") {
        throw new BareBridgeError("WORKLET_PROTOCOL", "Bare worklet did not acknowledge initialization");
      }

      this.state = "ready";
      return value;
    } catch (error) {
      this.release(error instanceof BareBridgeError ? error : undefined);
      throw error;
    }
  }

  async ping(): Promise<"pong"> {
    this.requireReady("ping");
    const value = await this.request("ping");

    if (value !== "pong") {
      throw new BareBridgeError("WORKLET_PROTOCOL", "Bare worklet returned an invalid ping reply");
    }

    return value;
  }

  async shutdown(): Promise<void> {
    if (this.state === "idle" || this.state === "stopped") return;

    if (this.state === "starting") {
      this.release(new BareBridgeError("SHUTDOWN_BEFORE_READY", "Bare shutdown requested before ready"));
      return;
    }

    try {
      const value = await this.request("shutdown");
      if (value !== "stopped") {
        throw new BareBridgeError("WORKLET_PROTOCOL", "Bare worklet did not acknowledge shutdown");
      }
    } finally {
      this.release();
    }
  }

  private requireReady(operation: string): void {
    if (this.state !== "ready") {
      throw new BareBridgeError("INVALID_STATE", `Cannot ${operation} while Bare is ${this.state}`);
    }
  }

  private request(command: BareCommand): Promise<BareResultValue> {
    const worklet = this.worklet;
    if (!worklet) {
      return Promise.reject(new BareBridgeError("INVALID_STATE", "Bare worklet is unavailable"));
    }

    const request: BareRequest = {
      type: "request",
      id: String(++this.nextRequestId),
      command,
    };

    return new Promise<BareResultValue>((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(request.id);
        reject(new BareBridgeError("RPC_TIMEOUT", `Bare ${command} request timed out`));
      }, this.timeoutMs);

      this.pending.set(request.id, { resolve, reject, timeout });

      try {
        worklet.IPC.write(b4a.from(JSON.stringify(request)));
      } catch (error) {
        clearTimeout(timeout);
        this.pending.delete(request.id);
        reject(
          new BareBridgeError(
            "WORKLET_FAILURE",
            error instanceof Error ? error.message : "Bare IPC write failed",
          ),
        );
      }
    });
  }

  private handleData(data: Uint8Array): void {
    const reply = parseBareReply(b4a.toString(data));
    if (!reply) {
      this.rejectPending(new BareBridgeError("MALFORMED_REPLY", "Bare worklet returned malformed data"));
      return;
    }

    if (reply.type === "error" && reply.id === null) {
      this.rejectPending(new BareBridgeError("WORKLET_FAILURE", reply.message));
      return;
    }

    if (typeof reply.id !== "string") {
      this.rejectPending(new BareBridgeError("MALFORMED_REPLY", "Bare worklet reply has no request id"));
      return;
    }

    const pending = this.pending.get(reply.id);
    if (!pending) return;

    clearTimeout(pending.timeout);
    this.pending.delete(reply.id);

    if (reply.type === "error") {
      pending.reject(new BareBridgeError("WORKLET_FAILURE", `${reply.code}: ${reply.message}`));
      return;
    }

    pending.resolve(reply.value);
  }

  private rejectPending(error: BareBridgeError): void {
    for (const [id, pending] of this.pending) {
      clearTimeout(pending.timeout);
      this.pending.delete(id);
      pending.reject(error);
    }
  }

  private release(reason?: BareBridgeError): void {
    const worklet = this.worklet;
    this.worklet = null;
    this.state = "stopped";
    this.rejectPending(reason ?? new BareBridgeError("WORKLET_FAILURE", "Bare worklet was released"));

    if (!worklet) return;
    worklet.IPC.removeListener("data", this.onData);
    worklet.terminate();
  }
}
