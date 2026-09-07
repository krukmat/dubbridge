import {
  BareRuntimeClient,
  type BareRuntimeState,
} from "./runtime/BareRuntimeClient";
import type { RuntimeHandshake } from "./runtime/protocol";

export type P2PRuntimeSnapshot = Readonly<{
  runtimeState: BareRuntimeState;
  lastError: string | null;
}>;

export type P2PServiceListener = () => void;
export type P2PRuntimeClient = Pick<BareRuntimeClient, "currentState" | "initialize" | "ping" | "shutdown">;

/** Framework-independent product façade. Construction is deliberately inert. */
export class P2PService {
  private initialization: Promise<RuntimeHandshake> | null = null;
  private readonly listeners = new Set<P2PServiceListener>();
  private shutdownOperation: Promise<void> | null = null;
  private snapshot: P2PRuntimeSnapshot = { runtimeState: "stopped", lastError: null };

  constructor(
    private readonly runtime: P2PRuntimeClient = new BareRuntimeClient(),
  ) {}

  getSnapshot = (): P2PRuntimeSnapshot => this.snapshot;

  subscribe = (listener: P2PServiceListener): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  initialize(): Promise<RuntimeHandshake> {
    if (this.initialization) return this.initialization;

    const operation = this.initializeRuntime();
    this.initialization = operation;
    void operation.then(
      () => this.clearInitialization(operation),
      () => this.clearInitialization(operation),
    );
    return operation;
  }

  shutdown(): Promise<void> {
    if (this.shutdownOperation) return this.shutdownOperation;

    const operation = this.shutdownRuntime();
    this.shutdownOperation = operation;
    void operation.then(
      () => this.clearShutdown(operation),
      () => this.clearShutdown(operation),
    );
    return operation;
  }

  private async initializeRuntime(): Promise<RuntimeHandshake> {
    this.publish({ runtimeState: "starting", lastError: null });
    try {
      const handshake = await this.runtime.initialize();
      this.publish({ runtimeState: this.runtime.currentState, lastError: null });
      return handshake;
    } catch (error) {
      this.publish({ runtimeState: this.runtime.currentState, lastError: this.errorMessage(error) });
      throw error;
    }
  }

  async ping(): Promise<"pong"> {
    try {
      return await this.runtime.ping();
    } catch (error) {
      this.publish({ runtimeState: this.runtime.currentState, lastError: this.errorMessage(error) });
      throw error;
    }
  }

  private async shutdownRuntime(): Promise<void> {
    try {
      await this.runtime.shutdown();
      this.publish({ runtimeState: this.runtime.currentState, lastError: null });
    } catch (error) {
      this.publish({ runtimeState: this.runtime.currentState, lastError: this.errorMessage(error) });
      throw error;
    }
  }

  private publish(snapshot: P2PRuntimeSnapshot): void {
    if (
      snapshot.runtimeState === this.snapshot.runtimeState &&
      snapshot.lastError === this.snapshot.lastError
    ) return;
    this.snapshot = snapshot;
    for (const listener of this.listeners) listener();
  }

  private errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : "P2P runtime operation failed";
  }

  private clearInitialization(operation: Promise<RuntimeHandshake>): void {
    if (this.initialization === operation) this.initialization = null;
  }

  private clearShutdown(operation: Promise<void>): void {
    if (this.shutdownOperation === operation) this.shutdownOperation = null;
  }
}
