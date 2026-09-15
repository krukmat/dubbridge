jest.mock("bare-crypto", () => ({
  createHash: require("node:crypto").createHash,
}));

import { productAccountStorageUri } from "../../src/p2p/runtime/product-package-runtime";
import {
  RUNTIME_COMMAND,
  RUNTIME_PROTOCOL_VERSION,
  decodeRequestPayload,
  encodeProtocolValue,
  type RuntimeRpcPort,
} from "../../src/p2p/runtime/protocol";
import { RuntimeProtocolClient } from "../../src/p2p/runtime/runtime-client";
import type { WorkletRuntime } from "../../src/p2p/runtime/transient-drive";
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
  it("carries account-scoped open/read/hash/close/cancel over the versioned RPC boundary", async () => {
    const bytes = new TextEncoder().encode("ciphertext");
    const digest = "305531dcc50ebca31cf1d5b31e9fc76ed51f66b3b6dd5a030c6539ae6532f979";
    const observed: number[] = [];
    const port = new FakePort(async (command, payload) => {
      observed.push(command);
      const request = decodeRequestPayload(payload);
      if (command === RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE) {
        expect(request).toEqual({
          protocolVersion: RUNTIME_PROTOCOL_VERSION,
          accountScope: "viewer-1",
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
      if (command === RUNTIME_COMMAND.HASH_PRODUCT_BYTES) {
        expect(request).toEqual({
          protocolVersion: RUNTIME_PROTOCOL_VERSION,
          bytesBase64: Buffer.from(bytes).toString("base64"),
        });
        return response(digest);
      }
      if (command === RUNTIME_COMMAND.CLOSE_PRODUCT_PACKAGE) return response("closed");
      return response("cancelled");
    });
    const client = new RuntimeProtocolClient(port, 100);

    await client.openProductPackage("viewer-1", "a".repeat(64));
    await expect(client.readProductFile("manifest.json")).resolves.toEqual(bytes);
    await expect(client.hashProductBytes(bytes)).resolves.toBe(digest);
    await client.closeProductPackage();
    await client.cancelProductPackage();

    expect(observed).toEqual([
      RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE,
      RUNTIME_COMMAND.READ_PRODUCT_FILE,
      RUNTIME_COMMAND.HASH_PRODUCT_BYTES,
      RUNTIME_COMMAND.CLOSE_PRODUCT_PACKAGE,
      RUNTIME_COMMAND.CANCEL_PRODUCT_PACKAGE,
    ]);
  });

  it("keeps Corestore namespaces distinct for separate signed-in accounts", () => {
    const runtime = {
      argv: ["file:/tmp/p2p-product"],
      on: jest.fn(),
    } as unknown as WorkletRuntime;

    expect(productAccountStorageUri(runtime, "viewer-a")).toBe(
      "file:/tmp/p2p-product/accounts/viewer-a",
    );
    expect(productAccountStorageUri(runtime, "viewer-b")).toBe(
      "file:/tmp/p2p-product/accounts/viewer-b",
    );
    expect(productAccountStorageUri(runtime, "viewer-a")).not.toBe(
      productAccountStorageUri(runtime, "viewer-b"),
    );
  });

  it("hashes ciphertext inside the Bare worklet with SHA-256", async () => {
    const harness = workletHarness(["file:/tmp/p2p-product"]);
    harness.request(RUNTIME_COMMAND.HASH_PRODUCT_BYTES, {
      protocolVersion: RUNTIME_PROTOCOL_VERSION,
      bytesBase64: Buffer.from("abc").toString("base64"),
    });
    await Promise.resolve();

    expect(harness.replies).toEqual([
      expect.objectContaining({
        ok: true,
        result: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
      }),
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

  it("rejects malformed product hash replies", async () => {
    const port = new FakePort(async () => response("not-a-sha256"));

    await expect(new RuntimeProtocolClient(port, 100).hashProductBytes(new Uint8Array([1]))).rejects.toMatchObject({
      code: "INVALID_PAYLOAD",
    });
  });

  it("fails closed before opening storage for invalid product identities", async () => {
    const harness = workletHarness(["file:/tmp/p2p-product"]);
    harness.request(RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE, {
      protocolVersion: RUNTIME_PROTOCOL_VERSION,
      accountScope: "viewer-1",
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

  it("rejects unsafe account scopes before opening product storage", async () => {
    const harness = workletHarness(["file:/tmp/p2p-product"]);
    harness.request(RUNTIME_COMMAND.OPEN_PRODUCT_PACKAGE, {
      protocolVersion: RUNTIME_PROTOCOL_VERSION,
      accountScope: "../viewer-1",
      externalPublicationId: "a".repeat(64),
    });
    await Promise.resolve();

    expect(harness.replies).toEqual([
      expect.objectContaining({
        ok: false,
        error: expect.objectContaining({ code: "INVALID_PAYLOAD" }),
      }),
    ]);
  });

  it("rejects malformed base64 before product hashing", async () => {
    const harness = workletHarness(["file:/tmp/p2p-product"]);
    harness.request(RUNTIME_COMMAND.HASH_PRODUCT_BYTES, {
      protocolVersion: RUNTIME_PROTOCOL_VERSION,
      bytesBase64: "%%not-base64%%",
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
