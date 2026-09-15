import {
  RUNTIME_COMMAND,
  RUNTIME_PROTOCOL_VERSION,
  decodeRequestPayload,
  encodeProtocolValue,
  type RuntimeRpcPort,
} from "../../src/p2p/runtime/protocol";
import { RuntimeProtocolClient } from "../../src/p2p/runtime/runtime-client";
import { versioned, workletHarness } from "../../test-utils/worklet-harness";

class FakePort implements RuntimeRpcPort {
  idle = true;
  closedWith: Error | null = null;

  constructor(
    private readonly respond: (
      command: number,
      payload: string,
    ) => Promise<Uint8Array | string | null>,
  ) {}

  close(error: Error): void {
    this.closedWith = error;
  }

  request(command: number, payload: string): Promise<Uint8Array | string | null> {
    return this.respond(command, payload);
  }
}

function response(result: unknown): Uint8Array {
  return encodeProtocolValue(versioned({ ok: true, result }));
}

describe("P4 product runtime protocol", () => {
  it("carries open/read/close/cancel over the versioned RPC boundary", async () => {
    const bytes = new TextEncoder().encode("ciphertext");
    const observed: number[] = [];
    const port = new FakePort(async (command, payload) => {
      observed.push(command);
      const request = decodeRequestPayload(payload);
      if (command === RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE) {
        expect(request).toEqual({
          protocolVersion: RUNTIME_PROTOCOL_VERSION,
          externalPublicationId: "a".repeat(64),
        });
        return response("opened");
      }
      if (command === RUNTIME_COMMAND.READ_PRODUCT_FILE) {
        expect(request).toEqual({ protocolVersion: RUNTIME_PROTOCOL_VERSION, path: "manifest.json" });
        return response({
          capability: "product-package-file",
          schema_version: 1,
          path: "manifest.json",
          byte_count: bytes.byteLength,
          bytes_base64: Buffer.from(bytes).toString("base64"),
        });
      }
      if (command === RUNTIME_COMMAND.CLOSE_PRODUCT_PACKAGE) return response("closed");
      return response("cancelled");
    });
    const client = new RuntimeProtocolClient(port, 100);

    await client.openProductPackage("a".repeat(64));
    await expect(client.readProductFile("manifest.json")).resolves.toEqual(bytes);
    await client.closeProductPackage();
    await client.cancelProductPackage();

    expect(observed).toEqual([
      RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE,
      RUNTIME_COMMAND.READ_PRODUCT_FILE,
      RUNTIME_COMMAND.CLOSE_PRODUCT_PACKAGE,
      RUNTIME_COMMAND.CANCEL_PRODUCT_PACKAGE,
    ]);
  });

  it("rejects malformed product file receipts before bytes reach sync", async () => {
    const port = new FakePort(async () =>
      response({
        capability: "product-package-file",
        schema_version: 1,
        path: "manifest.json",
        byte_count: 999,
        bytes_base64: Buffer.from("short").toString("base64"),
      }),
    );

    await expect(new RuntimeProtocolClient(port, 100).readProductFile("manifest.json")).rejects.toMatchObject({
      code: "INVALID_PAYLOAD",
    });
  });

  it("fails closed before opening storage for invalid product identities", async () => {
    const harness = workletHarness(["file:/tmp/p2p-product"]);
    harness.request(RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE, {
      protocolVersion: RUNTIME_PROTOCOL_VERSION,
      externalPublicationId: "not-a-drive-key",
    });
    await Promise.resolve();

    expect(harness.replies).toEqual([
      expect.objectContaining({
        ok: false,
        error: expect.objectContaining({ code: "INVALID_PAYLOAD" }),
      }),
    ]);
  });

  it("does not permit reads when no product package is open", async () => {
    const harness = workletHarness(["file:/tmp/p2p-product"]);
    harness.request(RUNTIME_COMMAND.READ_PRODUCT_FILE, {
      protocolVersion: RUNTIME_PROTOCOL_VERSION,
      path: "manifest.json",
    });
    await Promise.resolve();
    await Promise.resolve();

    expect(harness.replies).toEqual([
      expect.objectContaining({
        ok: false,
        error: expect.objectContaining({ code: "PRODUCT_PACKAGE_NOT_OPEN" }),
      }),
    ]);
  });
});
