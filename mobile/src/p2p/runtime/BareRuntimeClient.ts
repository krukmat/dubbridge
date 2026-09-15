import { Directory, Paths } from "expo-file-system";
import { Worklet } from "react-native-bare-kit";

import RUNTIME_WORKLET_SOURCE from "./worklet.bundle.js";
import { RuntimeProtocolError, type RuntimeHandshake } from "./protocol";
import { BareRpcPort, RuntimeProtocolClient } from "./runtime-client";

export type BareRuntimeState = "stopped" | "starting" | "ready" | "failed";

export class BareRuntimeClientError extends Error {
  constructor(
    readonly code: "INVALID_STATE" | "START_FAILED",
    message: string,
  ) {
    super(message);
    this.name = "BareRuntimeClientError";
  }
}

export type BareRuntimeWorklet = Pick<Worklet, "IPC" | "start" | "terminate">;
export type BareRuntimeWorkletFactory = () => BareRuntimeWorklet;
export type BareRuntimeProtocol = Pick<
  RuntimeProtocolClient,
  | "handshake"
  | "ping"
  | "shutdown"
  | "openProductPackage"
  | "readProductFile"
  | "hashProductBytes"
  | "closeProductPackage"
  | "cancelProductPackage"
>;
export type BareRuntimeProtocolFactory = (worklet: BareRuntimeWorklet) => BareRuntimeProtocol;
type BareRpcStream = ConstructorParameters<typeof BareRpcPort>[0];

const PRODUCT_WORKLET_FILENAME = "/dubbridge-p2p-runtime.worklet";

function defaultProductStorageUri(): string {
  return new Directory(Paths.cache, "dubbridge-p2p", "product-runtime").uri;
}

/** One product Bare worklet with no implicit network or proof behavior. */
export class BareRuntimeClient {
  private protocol: BareRuntimeProtocol | null = null;
  private state: BareRuntimeState = "stopped";
  private worklet: BareRuntimeWorklet | null = null;

  constructor(
    private readonly createWorklet: BareRuntimeWorkletFactory = () => new Worklet(),
    private readonly createProtocol: BareRuntimeProtocolFactory = (worklet) =>
      new RuntimeProtocolClient(new BareRpcPort(worklet.IPC as unknown as BareRpcStream)),
    private readonly productStorageUri: string = defaultProductStorageUri(),
  ) {}

  get currentState(): BareRuntimeState {
    return this.state;
  }

  async initialize(): Promise<RuntimeHandshake> {
    if (this.state !== "stopped") {
      throw new BareRuntimeClientError("INVALID_STATE", `Cannot initialize while runtime is ${this.state}`);
    }

    this.state = "starting";
    const worklet = this.createWorklet();
    this.worklet = worklet;

    try {
      worklet.start(PRODUCT_WORKLET_FILENAME, RUNTIME_WORKLET_SOURCE, [this.productStorageUri]);
      const protocol = this.createProtocol(worklet);
      this.protocol = protocol;
      const handshake = await protocol.handshake();
      if (this.worklet !== worklet || this.protocol !== protocol || this.state !== "starting") {
        throw new BareRuntimeClientError("INVALID_STATE", "Bare runtime stopped while starting");
      }
      this.state = "ready";
      return handshake;
    } catch (error) {
      if (this.worklet === worklet) {
        this.protocol = null;
        this.worklet = null;
        this.state = "failed";
        worklet.terminate();
      }
      if (error instanceof RuntimeProtocolError || error instanceof BareRuntimeClientError) {
        throw error;
      }
      throw new BareRuntimeClientError(
        "START_FAILED",
        error instanceof Error ? error.message : "Bare runtime could not start",
      );
    }
  }

  async ping(): Promise<"pong"> {
    return this.requireReady("ping").ping();
  }

  async openProductPackage(accountScope: string, externalPublicationId: string): Promise<void> {
    return this.requireReady("open product package").openProductPackage(
      accountScope,
      externalPublicationId,
    );
  }

  async readProductFile(path: string): Promise<Uint8Array> {
    return this.requireReady("read product file").readProductFile(path);
  }

  async hashProductBytes(bytes: Uint8Array): Promise<string> {
    return this.requireReady("hash product bytes").hashProductBytes(bytes);
  }

  async closeProductPackage(): Promise<void> {
    return this.requireReady("close product package").closeProductPackage();
  }

  async cancelProductPackage(): Promise<void> {
    return this.requireReady("cancel product package").cancelProductPackage();
  }

  async shutdown(): Promise<void> {
    if (this.state === "stopped") return;

    const protocol = this.protocol;
    const worklet = this.worklet;
    this.protocol = null;
    this.worklet = null;
    this.state = "stopped";

    try {
      if (protocol) await protocol.shutdown();
    } finally {
      worklet?.terminate();
    }
  }

  private requireReady(operation: string): BareRuntimeProtocol {
    if (this.state !== "ready" || !this.protocol) {
      throw new BareRuntimeClientError("INVALID_STATE", `Cannot ${operation} while runtime is ${this.state}`);
    }
    return this.protocol;
  }
}
