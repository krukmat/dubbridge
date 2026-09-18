import { execFileSync } from "node:child_process";
import { X509Certificate, randomBytes } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import https from "node:https";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { closeSharedStore } from "../dist/hyperdrive_store.js";

export const REPO_ROOT = join(fileURLToPath(new URL("../../..", import.meta.url)));
export const MAX_REQUEST_BODY_BYTES = 64 * 1024;

export function createPublicationRoots(prefix = "publication-contract-") {
  const root = mkdtempSync(join(tmpdir(), prefix));
  const packageRoot = join(root, "packages-root");
  const driveStorageRoot = join(root, "drive-root");
  const indexRoot = join(root, "index-root");
  mkdirSync(packageRoot);
  mkdirSync(driveStorageRoot);
  mkdirSync(indexRoot);
  return { root, packageRoot, driveStorageRoot, indexRoot };
}

export async function cleanupPublicationRoots(config) {
  let closeError;
  try {
    await closeSharedStore(config.driveStorageRoot);
  } catch (error) {
    closeError = error;
  } finally {
    rmSync(config.root, { recursive: true, force: true });
  }
  if (existsSync(config.root)) {
    throw new Error(`temporary publication root was not removed: ${config.root}`);
  }
  if (closeError !== undefined) throw closeError;
}

export function makeRequest(publicationId, lineageId, manifestDigest) {
  return {
    contract_version: "availability-publication-v1",
    publication_id: publicationId,
    lineage_id: lineageId,
    manifest_version: "p2p-manifest-v1",
    manifest_digest_sha256: manifestDigest,
    package_ref: `packages/${publicationId}/${lineageId}`,
  };
}

export function defaultFiles(segmentContent = "segment-bytes-1") {
  return [
    { path: "index.m3u8", content: "#EXTM3U" },
    { path: "segments/000001.ts", content: segmentContent },
  ];
}

export function randomCkHex() {
  return randomBytes(32).toString("hex");
}

export function runFixtureBinary({ root, assetId, publicationId, lineageId, files }) {
  const args = [
    "run",
    "--quiet",
    "-p",
    "dubbridge-p2p",
    "--bin",
    "package_build_and_materialize_fixture",
    "--",
    "--root",
    root,
    "--asset-id",
    assetId,
    "--publication-id",
    publicationId,
    "--lineage-id",
    lineageId,
    "--ck-hex",
    randomCkHex(),
  ];
  for (const file of files ?? defaultFiles()) {
    args.push("--file", `${file.path}=${file.content}`);
  }

  const stdout = execFileSync("cargo", args, {
    cwd: REPO_ROOT,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  const built = JSON.parse(stdout.trim());
  if (built.ok !== true) {
    throw new Error(`real Rust fixture build failed: ${stdout.trim()}`);
  }
  return built;
}

function createCertificate({ dir, name, commonName, caKey, caCert, serial, selfSigned = false }) {
  const keyPath = join(dir, `${name}.key`);
  const certPath = join(dir, `${name}.crt`);
  const configPath = join(dir, `${name}.cnf`);
  const csrPath = join(dir, `${name}.csr`);
  const isServer = name === "server";

  execFileSync("openssl", ["genrsa", "-out", keyPath, "2048"], { stdio: "pipe" });
  writeFileSync(configPath, `
[req]
distinguished_name = req_distinguished_name
[req_distinguished_name]
[certificate_ext]
${isServer ? "subjectAltName = DNS:localhost\nextendedKeyUsage = serverAuth" : "extendedKeyUsage = clientAuth"}
`);

  if (selfSigned) {
    execFileSync(
      "openssl",
      [
        "req",
        "-x509",
        "-new",
        "-nodes",
        "-key",
        keyPath,
        "-out",
        certPath,
        "-days",
        "1",
        "-config",
        configPath,
        "-extensions",
        "certificate_ext",
        "-subj",
        `/CN=${commonName}`,
      ],
      { stdio: "pipe" }
    );
  } else {
    execFileSync(
      "openssl",
      ["req", "-new", "-key", keyPath, "-out", csrPath, "-config", configPath, "-subj", `/CN=${commonName}`],
      { stdio: "pipe" }
    );
    execFileSync(
      "openssl",
      [
        "x509",
        "-req",
        "-in",
        csrPath,
        "-CA",
        caCert,
        "-CAkey",
        caKey,
        "-set_serial",
        String(serial),
        "-out",
        certPath,
        "-days",
        "1",
        "-extfile",
        configPath,
        "-extensions",
        "certificate_ext",
      ],
      { stdio: "pipe" }
    );
  }

  return { key: readFileSync(keyPath), cert: readFileSync(certPath) };
}

export function createMtlsFixture() {
  const root = mkdtempSync(join(tmpdir(), "publication-contract-mtls-"));
  const caKey = join(root, "ca.key");
  const caCert = join(root, "ca.crt");
  execFileSync("openssl", ["genrsa", "-out", caKey, "2048"], { stdio: "pipe" });
  execFileSync(
    "openssl",
    ["req", "-x509", "-new", "-nodes", "-key", caKey, "-out", caCert, "-days", "1", "-subj", "/CN=T3d Test CA"],
    { stdio: "pipe" }
  );

  const server = createCertificate({
    dir: root,
    name: "server",
    commonName: "localhost",
    caKey,
    caCert,
    serial: 3100,
  });
  const allowed = createCertificate({
    dir: root,
    name: "client-allowed",
    commonName: "Allowed Client",
    caKey,
    caCert,
    serial: 3101,
  });
  const unlisted = createCertificate({
    dir: root,
    name: "client-unlisted",
    commonName: "Unlisted Client",
    caKey,
    caCert,
    serial: 3102,
  });
  const rogue = createCertificate({
    dir: root,
    name: "client-rogue",
    commonName: "Rogue Client",
    caKey,
    caCert,
    serial: 3103,
    selfSigned: true,
  });
  const ca = readFileSync(caCert);

  return {
    root,
    credentials: { key: server.key, cert: server.cert, ca },
    ca,
    allowed,
    unlisted,
    rogue,
    allowedFingerprint: new X509Certificate(allowed.cert).fingerprint256,
    cleanup() {
      rmSync(root, { recursive: true, force: true });
      if (existsSync(root)) {
        throw new Error(`temporary mTLS root was not removed: ${root}`);
      }
    },
  };
}

export async function startServer(server) {
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      server.off("error", reject);
      resolve();
    });
  });
  return server.address().port;
}

export async function stopServer(server) {
  if (!server.listening) return;
  await new Promise((resolve, reject) => {
    server.close(error => (error ? reject(error) : resolve()));
  });
}

export function httpsPublicationRequest({
  port,
  ca,
  client,
  publicationId,
  body,
  contentType = "application/json; charset=utf-8",
  headers = {},
}) {
  const payload = Buffer.isBuffer(body)
    ? body
    : Buffer.from(typeof body === "string" ? body : JSON.stringify(body));
  const clientCredentials = client === undefined ? {} : { key: client.key, cert: client.cert };

  return new Promise((resolve, reject) => {
    const request = https.request(
      {
        host: "127.0.0.1",
        port,
        path: `/v1/publications/${publicationId}`,
        method: "PUT",
        ca: [ca],
        ...clientCredentials,
        servername: "localhost",
        minVersion: "TLSv1.3",
        agent: false,
        headers: {
          "Content-Type": contentType,
          "Content-Length": payload.length,
          ...headers,
        },
      },
      response => {
        const chunks = [];
        response.on("data", chunk => chunks.push(Buffer.from(chunk)));
        response.on("end", () => {
          const rawBody = Buffer.concat(chunks).toString("utf8");
          let parsedBody;
          try {
            parsedBody = JSON.parse(rawBody);
          } catch (error) {
            reject(error);
            return;
          }
          resolve({
            status: response.statusCode,
            body: parsedBody,
            rawBody,
            headers: response.headers,
          });
        });
      }
    );
    request.on("error", reject);
    request.end(payload);
  });
}

export function createConsoleCapture() {
  const methods = ["debug", "error", "info", "log", "warn"];
  const originals = new Map();
  const entries = [];
  for (const method of methods) {
    originals.set(method, console[method]);
    console[method] = (...args) => {
      entries.push(args.map(value => (typeof value === "string" ? value : JSON.stringify(value))).join(" "));
    };
  }
  return {
    entries,
    restore() {
      for (const method of methods) console[method] = originals.get(method);
    },
  };
}

export function loadSecretDenyList() {
  const fixturePath = fileURLToPath(
    new URL("../../../docs/fixtures/mvp0-p2p-publication-contract-v1.json", import.meta.url)
  );
  return JSON.parse(readFileSync(fixturePath, "utf8")).availability_node.secret_deny_list;
}
