import b4a from "b4a";

import { hashProductBytes } from "./product-hash";
import { ProductPackageRuntime, validatePackagePath } from "./product-package-runtime";
import { RuntimeProtocolError } from "./protocol";
import type { WorkletRuntime } from "./transient-drive";

const LOOPBACK_HOST = "127.0.0.1";
const AUTH_TAG_BYTES = 16;
const MAX_HEADER_BYTES = 16_384;

type PlaybackManifestFile = {
  ciphertext_sha256: string;
  ciphertext_size: number;
  nonce_b64u: string;
  path: string;
  plaintext_size: number;
};

type PlaybackManifest = {
  asset_id: string;
  cipher: string;
  digest: string;
  files: PlaybackManifestFile[];
  lineage_id: string;
  manifest_version: string;
  publication_id: string;
};

export type StartPlaybackInput = {
  accountScope: string;
  assetId: string;
  externalPublicationId: string;
  lineageId: string;
  manifestDigestSha256: string;
  publicationId: string;
  ckBase64: string;
};

export type ProductPlaybackReceipt = {
  capability: "product-playback";
  schema_version: 1;
  playback_url: string;
};

type PlaybackSocket = {
  on(event: string, listener: (...args: unknown[]) => void): PlaybackSocket;
  write(data: Uint8Array | string): boolean;
  end(data?: Uint8Array | string): void;
  destroy(): void;
};

type PlaybackServer = {
  on(event: string, listener: (...args: unknown[]) => void): PlaybackServer;
  listen(options: { port: number; host: string }, listener: () => void): PlaybackServer;
  address(): { port: number } | null;
  close(listener?: () => void): PlaybackServer;
};

type ActivePlayback = {
  ck: Uint8Array;
  manifest: PlaybackManifest;
  server: PlaybackServer;
  sockets: Set<PlaybackSocket>;
  token: string;
};

type HttpRequest = {
  method: "GET" | "HEAD";
  path: string;
  range: string | null;
};

type ByteRange = { start: number; end: number };

type ProductDecipher = {
  setAAD(data: Uint8Array): unknown;
  setAuthTag(data: Uint8Array): unknown;
  update(data: Uint8Array): Uint8Array;
  final(): Uint8Array;
};

type ProductDecipherFactory = (
  algorithm: string,
  key: Uint8Array,
  iv: Uint8Array,
) => ProductDecipher;

function fail(message: string): never {
  throw new RuntimeProtocolError("PRODUCT_PLAYBACK_FAILED", message);
}

function decodeBase64(value: string): Uint8Array {
  try {
    return b4a.from(value, "base64");
  } catch {
    return fail("Playback content key is invalid");
  }
}

function decodeBase64Url(value: string): Uint8Array {
  const normalized = value.replace(/-/g, "+").replace(/_/g, "/");
  const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, "=");
  return decodeBase64(padded);
}

function canonicalAad(manifest: PlaybackManifest, path: string): string {
  return JSON.stringify({
    aad_version: "p2p-aad-v1",
    asset_id: manifest.asset_id,
    lineage_id: manifest.lineage_id,
    manifest_version: manifest.manifest_version,
    path,
    publication_id: manifest.publication_id,
  });
}

function loadDecipherFactory(): ProductDecipherFactory {
  try {
    const crypto = require("bare-crypto") as { createDecipheriv?: ProductDecipherFactory };
    if (typeof crypto.createDecipheriv !== "function") return fail("Playback crypto is unavailable");
    return crypto.createDecipheriv;
  } catch {
    return fail("Playback crypto is unavailable");
  }
}

export function decryptProductFile(
  ck: Uint8Array,
  manifest: PlaybackManifest,
  file: PlaybackManifestFile,
  ciphertext: Uint8Array,
): Uint8Array {
  if (ciphertext.byteLength !== file.ciphertext_size || hashProductBytes(ciphertext) !== file.ciphertext_sha256) {
    return fail("Playback ciphertext integrity check failed");
  }
  if (ciphertext.byteLength < AUTH_TAG_BYTES) return fail("Playback ciphertext is truncated");
  const nonce = decodeBase64Url(file.nonce_b64u);
  if (nonce.byteLength !== 12) return fail("Playback nonce is invalid");
  const encrypted = ciphertext.subarray(0, ciphertext.byteLength - AUTH_TAG_BYTES);
  const tag = ciphertext.subarray(ciphertext.byteLength - AUTH_TAG_BYTES);
  try {
    const decipher = loadDecipherFactory()("aes-256-gcm", ck, nonce);
    decipher.setAAD(b4a.from(canonicalAad(manifest, file.path)));
    decipher.setAuthTag(tag);
    const plaintext = b4a.concat([decipher.update(encrypted), decipher.final()]);
    if (plaintext.byteLength !== file.plaintext_size) return fail("Playback plaintext size is invalid");
    return plaintext;
  } catch (error) {
    if (error instanceof RuntimeProtocolError) throw error;
    return fail("Playback decryption failed");
  }
}

function validManifestFile(value: unknown): value is PlaybackManifestFile {
  if (value === null || typeof value !== "object") return false;
  const file = value as Partial<PlaybackManifestFile>;
  return typeof file.path === "string" && typeof file.nonce_b64u === "string" &&
    typeof file.ciphertext_sha256 === "string" && /^[0-9a-f]{64}$/.test(file.ciphertext_sha256) &&
    Number.isSafeInteger(file.ciphertext_size) && Number.isSafeInteger(file.plaintext_size);
}

function parseManifest(bytes: Uint8Array, input: StartPlaybackInput): PlaybackManifest {
  let value: unknown;
  try {
    value = JSON.parse(b4a.toString(bytes));
  } catch {
    return fail("Playback manifest is invalid JSON");
  }
  if (value === null || typeof value !== "object") return fail("Playback manifest is invalid");
  const manifest = value as Partial<PlaybackManifest>;
  if (manifest.asset_id !== input.assetId || manifest.publication_id !== input.publicationId ||
      manifest.lineage_id !== input.lineageId || manifest.manifest_version !== "p2p-manifest-v1" ||
      manifest.cipher !== "AES-256-GCM" || manifest.digest !== "SHA-256" || !Array.isArray(manifest.files) ||
      !manifest.files.every(validManifestFile)) return fail("Playback manifest identity is invalid");
  const typed = manifest as PlaybackManifest;
  for (const file of typed.files) validatePackagePath(file.path);
  if (!typed.files.some((file) => file.path === "index.m3u8")) return fail("Playback manifest has no HLS index");
  return typed;
}

function basename(path: string): string {
  const clean = path.split(/[?#]/, 1)[0] ?? path;
  const parts = clean.split("/");
  return parts[parts.length - 1] ?? clean;
}

function encodedPath(path: string): string {
  return path.split("/").map((segment) => encodeURIComponent(segment)).join("/");
}

export function rewriteHlsManifest(plaintext: Uint8Array, manifest: PlaybackManifest, token: string): Uint8Array {
  const byName = new Map<string, string>();
  for (const file of manifest.files) {
    if (file.path === "index.m3u8") continue;
    const name = basename(file.path);
    if (byName.has(name)) return fail("Playback manifest contains ambiguous segment names");
    byName.set(name, file.path);
  }
  const lines = b4a.toString(plaintext).split(/\r?\n/);
  const rewritten = lines.map((line) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) return line;
    const path = byName.get(basename(trimmed));
    if (!path) return fail("Playback HLS manifest references an unknown segment");
    return `/${token}/${encodedPath(path)}`;
  });
  return b4a.from(rewritten.join("\n"));
}

export function parseRangeHeader(value: string | null, size: number): ByteRange | null {
  if (value === null) return null;
  const match = /^bytes=(\d+)-(\d*)$/.exec(value.trim());
  if (!match) return fail("Playback byte range is invalid");
  const start = Number(match[1]);
  const requestedEnd = match[2] ? Number(match[2]) : size - 1;
  if (!Number.isSafeInteger(start) || !Number.isSafeInteger(requestedEnd) ||
      start < 0 || start >= size || requestedEnd < start) return fail("Playback byte range is invalid");
  return { start, end: Math.min(requestedEnd, size - 1) };
}

function contentType(path: string): string {
  if (path.endsWith(".m3u8")) return "application/vnd.apple.mpegurl";
  if (path.endsWith(".ts")) return "video/mp2t";
  if (path.endsWith(".m4s") || path.endsWith(".mp4")) return "video/mp4";
  if (path.endsWith(".aac")) return "audio/aac";
  return "application/octet-stream";
}

function parseHttpRequest(header: string): HttpRequest {
  const lines = header.split("\r\n");
  const match = /^(GET|HEAD) ([^ ]+) HTTP\/1\.[01]$/.exec(lines[0] ?? "");
  if (!match) return fail("Playback HTTP request is invalid");
  const range = lines.find((line) => line.toLowerCase().startsWith("range:"));
  return { method: match[1] as "GET" | "HEAD", path: match[2], range: range?.slice(6).trim() ?? null };
}

function socketReply(
  socket: PlaybackSocket,
  status: number,
  type: string,
  body: Uint8Array,
  range: ByteRange | null,
  headOnly = false,
): void {
  const payload = range ? body.subarray(range.start, range.end + 1) : body;
  const statusText = status === 206 ? "Partial Content" : status === 200 ? "OK" : "Error";
  const headers = [
    `HTTP/1.1 ${status} ${statusText}`,
    `Content-Type: ${type}`,
    `Content-Length: ${payload.byteLength}`,
    "Accept-Ranges: bytes",
    "Cache-Control: no-store",
    "Connection: close",
  ];
  if (range) headers.push(`Content-Range: bytes ${range.start}-${range.end}/${body.byteLength}`);
  socket.write(`${headers.join("\r\n")}\r\n\r\n`);
  if (headOnly) socket.end();
  else socket.end(payload);
}

function errorReply(socket: PlaybackSocket, status = 404): void {
  socketReply(socket, status, "text/plain", b4a.from("Not Found"), null);
}

function randomToken(): string {
  try {
    const crypto = require("bare-crypto") as { randomBytes?: (size: number) => Uint8Array };
    if (typeof crypto.randomBytes !== "function") return fail("Playback random source is unavailable");
    return b4a.toString(crypto.randomBytes(16), "hex");
  } catch {
    return fail("Playback random source is unavailable");
  }
}

function createTcpServer(listener: (socket: PlaybackSocket) => void): PlaybackServer {
  try {
    const tcp = require("bare-tcp") as { createServer?: (listener: (socket: PlaybackSocket) => void) => PlaybackServer };
    if (typeof tcp.createServer !== "function") return fail("Playback loopback transport is unavailable");
    return tcp.createServer(listener);
  } catch {
    return fail("Playback loopback transport is unavailable");
  }
}

function listen(server: PlaybackServer): Promise<number> {
  return new Promise((resolve, reject) => {
    server.on("error", (error: unknown) => reject(error instanceof Error ? error : new Error("loopback listen failed")));
    server.listen({ port: 0, host: LOOPBACK_HOST }, () => {
      const address = server.address();
      if (!address || !Number.isInteger(address.port)) reject(new Error("loopback address missing"));
      else resolve(address.port);
    });
  });
}

async function closeServer(active: ActivePlayback): Promise<void> {
  for (const socket of active.sockets) socket.destroy();
  await new Promise<void>((resolve) => active.server.close(resolve));
}

export class ProductPlaybackRuntime {
  private active: ActivePlayback | null = null;

  constructor(private readonly packages: ProductPackageRuntime) {}

  get isActive(): boolean {
    return this.active !== null;
  }

  async start(runtime: WorkletRuntime, input: StartPlaybackInput): Promise<ProductPlaybackReceipt> {
    if (this.active) return fail("A playback session is already active");
    const ck = decodeBase64(input.ckBase64);
    if (ck.byteLength !== 32) return fail("Playback content key is invalid");
    await this.packages.open(runtime, input.accountScope, input.externalPublicationId);
    try {
      const manifestBytes = await this.packages.read("manifest.json");
      if (hashProductBytes(manifestBytes) !== input.manifestDigestSha256) return fail("Playback manifest digest mismatch");
      const manifest = parseManifest(manifestBytes, input);
      const token = randomToken();
      const sockets = new Set<PlaybackSocket>();
      const server = createTcpServer((socket) => this.acceptSocket(socket));
      this.active = { ck, manifest, server, sockets, token };
      const port = await listen(server);
      return { capability: "product-playback", schema_version: 1, playback_url: `http://${LOOPBACK_HOST}:${port}/${token}/index.m3u8` };
    } catch (error) {
      const active = this.active;
      this.active = null;
      ck.fill(0);
      if (active) await closeServer(active).catch(() => undefined);
      await this.packages.close().catch(() => undefined);
      if (error instanceof RuntimeProtocolError) throw error;
      return fail("Playback session could not start");
    }
  }

  async stop(): Promise<void> {
    const active = this.active;
    this.active = null;
    if (!active) return;
    active.ck.fill(0);
    await Promise.allSettled([closeServer(active), this.packages.close()]);
  }

  private acceptSocket(socket: PlaybackSocket): void {
    const active = this.active;
    if (!active) return socket.destroy();
    active.sockets.add(socket);
    let received = b4a.alloc(0);
    let handled = false;
    socket.on("close", () => active.sockets.delete(socket));
    socket.on("error", () => active.sockets.delete(socket));
    socket.on("data", (chunk: unknown) => {
      if (handled) return;
      if (!(chunk instanceof Uint8Array)) return socket.destroy();
      received = b4a.concat([received, chunk]);
      if (received.byteLength > MAX_HEADER_BYTES) { handled = true; return errorReply(socket, 431); }
      const marker = b4a.toString(received).indexOf("\r\n\r\n");
      if (marker >= 0) {
        handled = true;
        void this.handleHttp(socket, b4a.toString(received.subarray(0, marker + 4)));
      }
    });
  }

  private async handleHttp(socket: PlaybackSocket, raw: string): Promise<void> {
    try {
      const active = this.active;
      if (!active) return errorReply(socket);
      const request = parseHttpRequest(raw);
      const [rawPath] = request.path.split("?", 1);
      const decoded = decodeURIComponent(rawPath ?? "");
      const prefix = `/${active.token}/`;
      if (!decoded.startsWith(prefix)) return errorReply(socket);
      const path = decoded.slice(prefix.length);
      validatePackagePath(path);
      const file = active.manifest.files.find((candidate) => candidate.path === path);
      if (!file) return errorReply(socket);
      const ciphertext = await this.packages.read(path);
      let plaintext = decryptProductFile(active.ck, active.manifest, file, ciphertext);
      if (path === "index.m3u8") plaintext = rewriteHlsManifest(plaintext, active.manifest, active.token);
      const range = parseRangeHeader(request.range, plaintext.byteLength);
      socketReply(socket, range ? 206 : 200, contentType(path), plaintext, range, request.method === "HEAD");
    } catch {
      errorReply(socket);
    }
  }
}
