jest.mock("bare-crypto", () => require("node:crypto"));

const mockCreateServer = jest.fn();
jest.mock(
  "bare-tcp",
  () => ({ createServer: (...args: unknown[]) => mockCreateServer(...args) }),
  { virtual: true },
);

import { createCipheriv, createHash } from "node:crypto";
import { get as httpGet, type IncomingHttpHeaders } from "node:http";
import { createServer as createNodeServer, type Socket } from "node:net";

import type { ProductPackageRuntime } from "../../src/p2p/runtime/product-package-runtime";
import {
  ProductPlaybackRuntime,
  decryptProductFile,
  parseRangeHeader,
  rewriteHlsManifest,
} from "../../src/p2p/runtime/product-playback-runtime";
import type { WorkletRuntime } from "../../src/p2p/runtime/transient-drive";

function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function encryptFixture(path: string, plaintext: Uint8Array) {
  const key = Buffer.alloc(32, 7);
  const nonce = Buffer.from("000102030405060708090a0b", "hex");
  const manifestBase = {
    asset_id: "asset-1",
    cipher: "AES-256-GCM",
    digest: "SHA-256",
    lineage_id: "lineage-1",
    manifest_version: "p2p-manifest-v1",
    publication_id: "publication-1",
  };
  const aad = JSON.stringify({
    aad_version: "p2p-aad-v1",
    asset_id: manifestBase.asset_id,
    lineage_id: manifestBase.lineage_id,
    manifest_version: manifestBase.manifest_version,
    path,
    publication_id: manifestBase.publication_id,
  });
  const cipher = createCipheriv("aes-256-gcm", key, nonce);
  cipher.setAAD(Buffer.from(aad));
  const encrypted = Buffer.concat([cipher.update(plaintext), cipher.final(), cipher.getAuthTag()]);
  const file = {
    path,
    plaintext_size: plaintext.byteLength,
    ciphertext_size: encrypted.byteLength,
    nonce_b64u: nonce.toString("base64url"),
    ciphertext_sha256: sha256(encrypted),
  };
  return { key, encrypted, file, manifestBase };
}

type RuntimeFixture = ReturnType<typeof createRuntimeFixture>;

function createRuntimeFixture() {
  const indexPlaintext = Buffer.from("#EXTM3U\n#EXTINF:4,\n000001.ts\n#EXT-X-ENDLIST\n");
  const segmentPlaintext = Buffer.from("segment-payload");
  const index = encryptFixture("index.m3u8", indexPlaintext);
  const segment = encryptFixture("segments/000001.ts", segmentPlaintext);
  const manifest = {
    ...index.manifestBase,
    files: [index.file, segment.file],
  };
  const manifestBytes = Buffer.from(JSON.stringify(manifest));
  const objects = new Map<string, Uint8Array>([
    ["manifest.json", manifestBytes],
    ["index.m3u8", index.encrypted],
    ["segments/000001.ts", segment.encrypted],
  ]);
  const packages = {
    open: jest.fn(async () => undefined),
    read: jest.fn(async (path: string) => {
      const bytes = objects.get(path);
      if (!bytes) throw new Error(`missing fixture object: ${path}`);
      return bytes;
    }),
    close: jest.fn(async () => undefined),
  };
  const input = {
    accountScope: "viewer-1",
    assetId: "asset-1",
    externalPublicationId: "b".repeat(64),
    lineageId: "lineage-1",
    manifestDigestSha256: sha256(manifestBytes),
    publicationId: "publication-1",
    ckBase64: index.key.toString("base64"),
  };
  const runtimeArg = {
    argv: ["file:/tmp/dubbridge-p2p-test"],
  } as unknown as WorkletRuntime;

  return {
    index,
    input,
    manifest,
    objects,
    packages,
    runtimeArg,
    segment,
  };
}

type ServerHarness = {
  acceptSocket: ((socket: unknown) => void) | null;
  closed: boolean;
  listenOptions: { port: number; host: string } | null;
  server: {
    on(event: string, listener: (...args: unknown[]) => void): unknown;
    listen(
      options: { port: number; host: string },
      listener: () => void,
    ): unknown;
    address(): { port: number };
    close(listener?: () => void): unknown;
  };
};

function installServerHarness(options?: {
  failListen?: boolean;
  onListen?: () => void;
}): ServerHarness {
  const listeners = new Map<string, (...args: unknown[]) => void>();
  const harness: ServerHarness = {
    acceptSocket: null,
    closed: false,
    listenOptions: null,
    server: {
      on(event, listener) {
        listeners.set(event, listener);
        return harness.server;
      },
      listen(listenOptions, listener) {
        harness.listenOptions = listenOptions;
        options?.onListen?.();
        if (options?.failListen) {
          listeners.get("error")?.(new Error("listen failed"));
        } else {
          listener();
        }
        return harness.server;
      },
      address() {
        return { port: 32123 };
      },
      close(listener) {
        harness.closed = true;
        listener?.();
        return harness.server;
      },
    },
  };
  mockCreateServer.mockImplementationOnce(
    (listener: (socket: unknown) => void) => {
      harness.acceptSocket = listener;
      return harness.server;
    },
  );
  return harness;
}

type TestPlaybackSocket = {
  on(
    event: string,
    listener: (...args: unknown[]) => void,
  ): TestPlaybackSocket;
  write(data: string | Uint8Array): boolean;
  end(data?: string | Uint8Array): void;
  destroy(): void;
};

function createSocketHarness() {
  const listeners = new Map<string, Array<(...args: unknown[]) => void>>();
  const writes: Array<string | Uint8Array> = [];
  let resolveEnded: (() => void) | null = null;
  const ended = new Promise<void>((resolve) => {
    resolveEnded = resolve;
  });
  const socket: TestPlaybackSocket = {
    on: jest.fn(
      (
        event: string,
        listener: (...args: unknown[]) => void,
      ): TestPlaybackSocket => {
      const current = listeners.get(event) ?? [];
      current.push(listener);
      listeners.set(event, current);
        return socket;
      },
    ),
    write: jest.fn((data: string | Uint8Array): boolean => {
      writes.push(data);
      return true;
    }),
    end: jest.fn((data?: string | Uint8Array): void => {
      if (data !== undefined) writes.push(data);
      resolveEnded?.();
    }),
    destroy: jest.fn((): void => {
      for (const listener of listeners.get("close") ?? []) listener();
      resolveEnded?.();
    }),
  };
  return {
    ended,
    socket,
    writes,
    emitData(data: string) {
      for (const listener of listeners.get("data") ?? []) {
        listener(Buffer.from(data));
      }
    },
  };
}

function responseHeaders(writes: Array<string | Uint8Array>): string {
  return writes.filter((value): value is string => typeof value === "string").join("");
}

function responseBody(writes: Array<string | Uint8Array>): string {
  return writes
    .filter((value): value is Uint8Array => value instanceof Uint8Array)
    .map((value) => Buffer.from(value).toString())
    .join("");
}

function sessionToken(playbackUrl: string): string {
  return playbackUrl.split("/")[3] ?? "";
}

function requestLoopback(
  url: string,
): Promise<{
  statusCode: number;
  headers: IncomingHttpHeaders;
  body: Buffer;
}> {
  return new Promise((resolve, reject) => {
    const request = httpGet(url, (response) => {
      const chunks: Buffer[] = [];
      response.on("data", (chunk: Buffer) => chunks.push(chunk));
      response.on("end", () =>
        resolve({
          statusCode: response.statusCode ?? 0,
          headers: response.headers,
          body: Buffer.concat(chunks),
        }),
      );
    });
    request.on("error", reject);
  });
}

describe("P5 decrypt-on-read runtime", () => {
  beforeEach(() => {
    mockCreateServer.mockReset();
  });

  it("decrypts the exact P2 AES-256-GCM/AAD layout and rejects corruption", () => {
    const plaintext = Buffer.from("segment payload");
    const fixture = encryptFixture("segments/000001.ts", plaintext);
    const manifest = { ...fixture.manifestBase, files: [fixture.file] };

    expect(
      Buffer.from(
        decryptProductFile(
          fixture.key,
          manifest,
          fixture.file,
          fixture.encrypted,
        ),
      ),
    ).toEqual(plaintext);

    const corrupted = Buffer.from(fixture.encrypted);
    corrupted[0] ^= 1;
    expect(() =>
      decryptProductFile(fixture.key, manifest, fixture.file, corrupted),
    ).toThrow("integrity check failed");
  });

  it("rejects ciphertext when manifest path changes the authenticated AAD", () => {
    const plaintext = Buffer.from("segment payload");
    const fixture = encryptFixture("segments/000001.ts", plaintext);
    const alteredFile = {
      ...fixture.file,
      path: "segments/000002.ts",
    };
    const manifest = {
      ...fixture.manifestBase,
      files: [alteredFile],
    };

    expect(() =>
      decryptProductFile(
        fixture.key,
        manifest,
        alteredFile,
        fixture.encrypted,
      ),
    ).toThrow("Playback decryption failed");
  });

  it("rewrites HLS segment references to the randomized loopback session", () => {
    const token = "a".repeat(32);
    const manifest = {
      asset_id: "asset-1",
      cipher: "AES-256-GCM",
      digest: "SHA-256",
      lineage_id: "lineage-1",
      manifest_version: "p2p-manifest-v1",
      publication_id: "publication-1",
      files: [
        {
          path: "index.m3u8",
          plaintext_size: 1,
          ciphertext_size: 17,
          nonce_b64u: "AAECAwQFBgcICQoL",
          ciphertext_sha256: "a".repeat(64),
        },
        {
          path: "segments/000001.ts",
          plaintext_size: 1,
          ciphertext_size: 17,
          nonce_b64u: "AAECAwQFBgcICQoL",
          ciphertext_sha256: "b".repeat(64),
        },
      ],
    };
    const source = Buffer.from(
      "#EXTM3U\n#EXTINF:4,\nprepared/asset/000001.ts\n#EXT-X-ENDLIST\n",
    );
    const rewritten = Buffer.from(
      rewriteHlsManifest(source, manifest, token),
    ).toString();

    expect(rewritten).toContain(`/${token}/segments/000001.ts`);
    expect(rewritten).not.toContain("prepared/asset/000001.ts");
  });

  it("supports bounded single byte ranges for expo-video seek requests", () => {
    expect(parseRangeHeader(null, 100)).toBeNull();
    expect(parseRangeHeader("bytes=10-19", 100)).toEqual({
      start: 10,
      end: 19,
    });
    expect(parseRangeHeader("bytes=90-", 100)).toEqual({
      start: 90,
      end: 99,
    });
    expect(() => parseRangeHeader("bytes=100-120", 100)).toThrow(
      "byte range is invalid",
    );
  });

  it("serves the verified manifest over a real OS loopback TCP listener", async () => {
    const fixture = createRuntimeFixture();
    mockCreateServer.mockImplementationOnce(
      (listener: (socket: unknown) => void) =>
        createNodeServer((socket: Socket) => listener(socket)),
    );
    const runtime = new ProductPlaybackRuntime(
      fixture.packages as unknown as ProductPackageRuntime,
    );

    const receipt = await runtime.start(fixture.runtimeArg, fixture.input);
    const response = await requestLoopback(receipt.playback_url);

    expect(receipt.playback_url).toMatch(
      /^http:\/\/127\.0\.0\.1:\d+\/[0-9a-f]{32}\/index\.m3u8$/,
    );
    expect(response.statusCode).toBe(200);
    expect(response.headers["cache-control"]).toBe("no-store");
    expect(response.body.toString()).toContain(
      `/${sessionToken(receipt.playback_url)}/segments/000001.ts`,
    );

    await runtime.stop();
  });

  it("serves verified HLS only through randomized 127.0.0.1 session URLs and zeroizes CK on stop", async () => {
    const fixture = createRuntimeFixture();
    const serverHarness = installServerHarness();
    const runtime = new ProductPlaybackRuntime(
      fixture.packages as unknown as ProductPackageRuntime,
    );

    const receipt = await runtime.start(fixture.runtimeArg, fixture.input);

    expect(serverHarness.listenOptions).toEqual({
      port: 0,
      host: "127.0.0.1",
    });
    expect(receipt.playback_url).toMatch(
      /^http:\/\/127\.0\.0\.1:32123\/[0-9a-f]{32}\/index\.m3u8$/,
    );
    const token = sessionToken(receipt.playback_url);
    const active = (
      runtime as unknown as {
        active: { ck: Uint8Array } | null;
      }
    ).active;
    expect(active).not.toBeNull();
    const ckReference = active?.ck;
    expect(ckReference).toBeDefined();
    expect(Array.from(ckReference ?? [])).toEqual(
      Array.from(Buffer.alloc(32, 7)),
    );

    const socket = createSocketHarness();
    serverHarness.acceptSocket?.(socket.socket);
    socket.emitData(
      `GET /${token}/index.m3u8 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n`,
    );
    await socket.ended;

    const headers = responseHeaders(socket.writes);
    const body = responseBody(socket.writes);
    expect(headers).toContain("HTTP/1.1 200 OK");
    expect(headers).toContain("Cache-Control: no-store");
    expect(body).toContain(`/${token}/segments/000001.ts`);
    expect(body).not.toMatch(/https?:\/\/(?!127\.0\.0\.1)/);

    await runtime.stop();

    expect(serverHarness.closed).toBe(true);
    expect(socket.socket.destroy).toHaveBeenCalledTimes(1);
    expect(fixture.packages.close).toHaveBeenCalledTimes(1);
    expect(Array.from(ckReference ?? [])).toEqual(new Array(32).fill(0));
  });

  it("denies foreign session tokens, traversal, and tampered ciphertext without fallback", async () => {
    const fixture = createRuntimeFixture();
    const serverHarness = installServerHarness();
    const runtime = new ProductPlaybackRuntime(
      fixture.packages as unknown as ProductPackageRuntime,
    );
    const receipt = await runtime.start(fixture.runtimeArg, fixture.input);
    const token = sessionToken(receipt.playback_url);

    const foreign = createSocketHarness();
    serverHarness.acceptSocket?.(foreign.socket);
    foreign.emitData(
      `GET /${"f".repeat(32)}/index.m3u8 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n`,
    );
    await foreign.ended;
    expect(responseHeaders(foreign.writes)).toContain("HTTP/1.1 404 Error");

    const traversal = createSocketHarness();
    serverHarness.acceptSocket?.(traversal.socket);
    traversal.emitData(
      `GET /${token}/../secret HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n`,
    );
    await traversal.ended;
    expect(responseHeaders(traversal.writes)).toContain(
      "HTTP/1.1 404 Error",
    );

    const corrupted = Buffer.from(fixture.segment.encrypted);
    corrupted[0] ^= 1;
    fixture.objects.set("segments/000001.ts", corrupted);

    const tampered = createSocketHarness();
    serverHarness.acceptSocket?.(tampered.socket);
    tampered.emitData(
      `GET /${token}/segments/000001.ts HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n`,
    );
    await tampered.ended;
    expect(responseHeaders(tampered.writes)).toContain("HTTP/1.1 404 Error");

    expect(fixture.packages.open).toHaveBeenCalledTimes(1);
    expect(
      fixture.packages.read.mock.calls.filter(
        ([path]) => path === "segments/000001.ts",
      ),
    ).toHaveLength(1);

    await runtime.stop();
  });

  it("rejects a missing content key before opening package or transport state", async () => {
    const fixture = createRuntimeFixture();
    const runtime = new ProductPlaybackRuntime(
      fixture.packages as unknown as ProductPackageRuntime,
    );

    await expect(
      runtime.start(fixture.runtimeArg, {
        ...fixture.input,
        ckBase64: "",
      }),
    ).rejects.toThrow("Playback content key is invalid");

    expect(fixture.packages.open).not.toHaveBeenCalled();
    expect(mockCreateServer).not.toHaveBeenCalled();
    expect(runtime.isActive).toBe(false);
  });

  it("zeroizes the decoded CK and closes package state when loopback startup fails", async () => {
    const fixture = createRuntimeFixture();
    let ckReference: Uint8Array | undefined;
    let runtime: ProductPlaybackRuntime;
    installServerHarness({
      failListen: true,
      onListen: () => {
        const active = (
          runtime as unknown as {
            active: { ck: Uint8Array } | null;
          }
        ).active;
        ckReference = active?.ck;
      },
    });
    runtime = new ProductPlaybackRuntime(
      fixture.packages as unknown as ProductPackageRuntime,
    );

    await expect(runtime.start(fixture.runtimeArg, fixture.input)).rejects.toThrow(
      "Playback session could not start",
    );

    expect(ckReference).toBeDefined();
    expect(Array.from(ckReference ?? [])).toEqual(new Array(32).fill(0));
    expect(fixture.packages.close).toHaveBeenCalledTimes(1);
    expect(runtime.isActive).toBe(false);
  });
});
