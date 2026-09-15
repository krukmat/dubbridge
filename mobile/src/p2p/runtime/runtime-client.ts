import b4a from "b4a";
import RPC from "bare-rpc";

import {
  RUNTIME_COMMAND,
  RUNTIME_PROTOCOL_VERSION,
  RuntimeCodec,
  RuntimeProtocolError,
  decodeHandshakeResult,
  decodeProductFileReceipt,
  decodeProductPlaybackReceipt,
  decodeResponseEnvelope,
  decodeRuntimeEvent,
  type DiscoverAndReplicateReceipt,
  type ProductPlaybackReceipt,
  type RuntimeEvent,
  type RuntimeHandshake,
  type RuntimeRpcPort,
  type SeedWriteHashDeleteReceipt,
  type StartProductPlaybackRequest,
} from "./protocol";

const PRODUCT_RPC_TIMEOUT_MS = 35_000;
const SHA256_HEX = /^[0-9a-f]{64}$/;

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
    await this.expectExact(RUNTIME_COMMAND.PING, "pong", "Runtime ping reply is invalid");
    return "pong";
  }

  async shutdown(): Promise<void> {
    await this.expectExact(RUNTIME_COMMAND.SHUTDOWN, "stopped", "Runtime shutdown reply is invalid");
  }

  async openProductPackage(accountScope: string, externalPublicationId: string): Promise<void> {
    await this.expectExact(
      RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE,
      "opened",
      "Runtime product package open reply is invalid",
      { accountScope, externalPublicationId },
      PRODUCT_RPC_TIMEOUT_MS,
    );
  }

  async readProductFile(path: string): Promise<Uint8Array> {
    const result = await this.call(RUNTIME_COMMAND.READ_PRODUCT_FILE, { path }, PRODUCT_RPC_TIMEOUT_MS);
    return decodeProductFileReceipt(result, path);
  }

  async hashProductBytes(bytes: Uint8Array): Promise<string> {
    const result = await this.call(
      RUNTIME_COMMAND.HASH_PRODUCT_BYTES,
      { bytesBase64: b4a.toString(bytes, "base64") },
      PRODUCT_RPC_TIMEOUT_MS,
    );
    if (typeof result !== "string" || !SHA256_HEX.test(result)) {
      throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime product hash reply is invalid");
    }
    return result;
  }

  async closeProductPackage(): Promise<void> {
    await this.expectExact(
      RUNTIME_COMMAND.CLOSE_PRODUCT_PACKAGE,
      "closed",
      "Runtime product package close reply is invalid",
      undefined,
      PRODUCT_RPC_TIMEOUT_MS,
    );
  }

  async cancelProductPackage(): Promise<void> {
    await this.expectExact(
      RUNTIME_COMMAND.CANCEL_PRODUCT_PACKAGE,
      "cancelled",
      "Runtime product package cancel reply is invalid",
      undefined,
      PRODUCT_RPC_TIMEOUT_MS,
    );
  }

  async startProductPlayback(input: Omit<StartProductPlaybackRequest, "protocolVersion">): Promise<ProductPlaybackReceipt> {
    const result = await this.call(RUNTIME_COMMAND.START_PRODUCT_PLAYBACK, input, PRODUCT_RPC_TIMEOUT_MS);
    return decodeProductPlaybackReceipt(result);
  }

  async stopProductPlayback(): Promise<void> {
    await this.expectExact(
      RUNTIME_COMMAND.STOP_PRODUCT_PLAYBACK,
      "stopped",
      "Runtime product playback stop reply is invalid",
      undefined,
      PRODUCT_RPC_TIMEOUT_MS,
    );
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

  async discoverAndReplicate(topic: Buffer, role: "seed" | "client"): Promise<DiscoverAndReplicateReceipt> {
    const result = await this.call(RUNTIME_COMMAND.DISCOVER_AND_REPLICATE, {
      topic: topic.toString("hex"),
      role,
    });
    if (
      result === null ||
      typeof result !== "object" ||
      (result as Partial<DiscoverAndReplicateReceipt>).capability !== "discover-and-replicate" ||
      (result as Partial<DiscoverAndReplicateReceipt>).role !== role ||
      typeof (result as Partial<DiscoverAndReplicateReceipt>).byte_count !== "number"
    ) {
      throw new RuntimeProtocolError("INVALID_PAYLOAD", "Runtime discover-and-replicate reply is invalid");
    }
    return result as DiscoverAndReplicateReceipt;
  }

  private async expectExact(
    command: number,
    expected: string,
    errorMessage: string,
    extraPayload?: Record<string, unknown>,
    timeoutMs = this.timeoutMs,
  ): Promise<void> {
    if ((await this.call(command, extraPayload, timeoutMs)) !== expected) {
      throw new RuntimeProtocolError("INVALID_PAYLOAD", errorMessage);
    }
  }

  private async call(
    command: number,
    extraPayload?: Record<string, unknown>,
    timeoutMs = this.timeoutMs,
  ): Promise<unknown> {
    this.pendingCount += 1;
    let timeout: ReturnType<typeof setTimeout> | undefined;

    try {
      return RuntimeCodec.successResult(
        decodeResponseEnvelope(
          await Promise.race([
            this.port.request(
              command,
              JSON.stringify({ protocolVersion: RUNTIME_PROTOCOL_VERSION, ...extraPayload }),
            ),
            new Promise<never>((_, reject) => {
              timeout = setTimeout(() => {
                this.port.close(new RuntimeProtocolError("RPC_TIMEOUT", "Runtime request timed out"));
                reject(new RuntimeProtocolError("RPC_TIMEOUT", "Runtime request timed out"));
              }, timeoutMs);
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
