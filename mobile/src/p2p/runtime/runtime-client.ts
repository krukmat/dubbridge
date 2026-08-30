import RPC from "bare-rpc";

import {
  RUNTIME_COMMAND,
  RUNTIME_PROTOCOL_VERSION,
  RuntimeCodec,
  RuntimeProtocolError,
  decodeHandshakeResult,
  decodeResponseEnvelope,
  decodeRuntimeEvent,
  type RuntimeEvent,
  type RuntimeHandshake,
  type RuntimeRpcPort,
  type SeedWriteHashDeleteReceipt,
} from "./protocol";

export class BareRpcPort implements RuntimeRpcPort {
  private readonly rpc: RPC;

  constructor(
    private readonly stream: ConstructorParameters<typeof RPC>[0],
    onEvent: (event: RuntimeEvent) => void = () => undefined,
    onProtocolError: (error: RuntimeProtocolError) => void = () => undefined,
  ) {
    this.rpc = new RPC(stream, (request) => {
      if (request.command !== RUNTIME_COMMAND.LIFECYCLE_EVENT && request.command !== RUNTIME_COMMAND.FATAL_EVENT) {
        return;
      }
      try {
        onEvent(decodeRuntimeEvent(request.data));
      } catch (error) {
        onProtocolError(
          error instanceof RuntimeProtocolError
            ? error
            : new RuntimeProtocolError("INVALID_LIFECYCLE", "Runtime event could not be decoded"),
        );
      }
    });
  }

  get idle(): boolean {
    return this.rpc.idle;
  }

  request(command: number, payload: string): Promise<Uint8Array | string | null> {
    return this.sendRequest(this.rpc.request(command), payload);
  }

  private sendRequest(
    request: ReturnType<RPC["request"]>,
    payload: string,
  ): Promise<Uint8Array | string | null> {
    request.send(payload);
    return request.reply("utf8") as Promise<Uint8Array | string | null>;
  }

  close(error: Error): void {
    this.stream.destroy(error);
  }
}

export class RuntimeProtocolClient {
  private pendingCount = 0;

  constructor(
    private readonly port: RuntimeRpcPort,
    private readonly timeoutMs = 5_000,
  ) {}

  get idle(): boolean {
    return this.pendingCount === 0 && this.port.idle;
  }

  async handshake(): Promise<RuntimeHandshake> {
    return decodeHandshakeResult(await this.call(RUNTIME_COMMAND.HANDSHAKE));
  }

  async ping(): Promise<"pong"> {
    if ((await this.call(RUNTIME_COMMAND.PING)) !== "pong") {
      throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime ping reply is invalid");
    }
    return "pong";
  }

  async shutdown(): Promise<void> {
    if ((await this.call(RUNTIME_COMMAND.SHUTDOWN)) !== "stopped") {
      throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime shutdown reply is invalid");
    }
  }

  async seedWriteHashDelete(): Promise<SeedWriteHashDeleteReceipt> {
    const result = await this.call(RUNTIME_COMMAND.SEED_WRITE_HASH_DELETE);
    if (
      result === null ||
      typeof result !== "object" ||
      (result as Partial<SeedWriteHashDeleteReceipt>).capability !== "seed-write-hash-delete" ||
      typeof (result as Partial<SeedWriteHashDeleteReceipt>).byte_count !== "number" ||
      typeof (result as Partial<SeedWriteHashDeleteReceipt>).sha256 !== "string"
    ) {
      throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime seed reply is invalid");
    }
    return result as SeedWriteHashDeleteReceipt;
  }

  private async call(command: number): Promise<unknown> {
    this.pendingCount += 1;
    let timeout: ReturnType<typeof setTimeout> | undefined;

    try {
      return RuntimeCodec.successResult(
        decodeResponseEnvelope(
          await Promise.race([
            this.port.request(command, JSON.stringify({ protocolVersion: RUNTIME_PROTOCOL_VERSION })),
            new Promise<never>((_, reject) => {
              timeout = setTimeout(() => {
                this.port.close(new RuntimeProtocolError("RPC_TIMEOUT", "Runtime request timed out"));
                reject(new RuntimeProtocolError("RPC_TIMEOUT", "Runtime request timed out"));
              }, this.timeoutMs);
            }),
          ]),
        ),
      );
    } catch (error) {
      if (error instanceof RuntimeProtocolError) throw error;
      throw new RuntimeProtocolError("CHANNEL_CLOSED", "Runtime channel closed before replying");
    } finally {
      if (timeout !== undefined) clearTimeout(timeout);
      this.pendingCount -= 1;
    }
  }
}
