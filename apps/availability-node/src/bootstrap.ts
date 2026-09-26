import { readFile } from "node:fs/promises";
import type { Server as HttpsServer } from "node:https";

import { closeSharedStore } from "./hyperdrive_store.js";
import {
  DEFAULT_HYPERSWARM_JOIN_TIMEOUT_MS,
  createPublicationExecutor,
} from "./publication_executor.js";
import { createPrivatePublicationServer } from "./server.js";

const DEFAULT_BIND_HOST = "127.0.0.1";
const DEFAULT_PORT = 8443;

export interface AvailabilityNodeRuntimeConfig {
  readonly bindHost: string;
  readonly port: number;
  readonly packageRoot: string;
  readonly driveStorageRoot: string;
  readonly indexRoot: string;
  readonly hyperswarmJoinTimeoutMs: number;
  readonly serverKeyPath: string;
  readonly serverCertPath: string;
  readonly caPath: string;
  readonly allowedClientFingerprints: readonly string[];
}

export interface AvailabilityNodeRuntime {
  readonly server: HttpsServer;
  readonly close: () => Promise<void>;
}

export function loadAvailabilityNodeConfig(
  env: NodeJS.ProcessEnv = process.env
): AvailabilityNodeRuntimeConfig {
  return {
    bindHost: optionalNonempty(env, "DUBBRIDGE_P2P_AVAILABILITY_BIND_HOST") ?? DEFAULT_BIND_HOST,
    port: positiveInteger(env, "DUBBRIDGE_P2P_AVAILABILITY_PORT", DEFAULT_PORT, 65_535),
    packageRoot: requiredNonempty(env, "DUBBRIDGE_P2P_CIPHERTEXT_ROOT"),
    driveStorageRoot: requiredNonempty(env, "DUBBRIDGE_P2P_AVAILABILITY_DRIVE_ROOT"),
    indexRoot: requiredNonempty(env, "DUBBRIDGE_P2P_AVAILABILITY_INDEX_ROOT"),
    hyperswarmJoinTimeoutMs: positiveInteger(
      env,
      "DUBBRIDGE_P2P_HYPERSWARM_JOIN_TIMEOUT_MS",
      DEFAULT_HYPERSWARM_JOIN_TIMEOUT_MS,
      Number.MAX_SAFE_INTEGER
    ),
    serverKeyPath: requiredNonempty(env, "DUBBRIDGE_P2P_AVAILABILITY_SERVER_KEY_PEM"),
    serverCertPath: requiredNonempty(env, "DUBBRIDGE_P2P_AVAILABILITY_SERVER_CERT_PEM"),
    caPath: requiredNonempty(env, "DUBBRIDGE_P2P_AVAILABILITY_CA_PEM"),
    allowedClientFingerprints: requiredCsv(
      env,
      "DUBBRIDGE_P2P_AVAILABILITY_ALLOWED_CLIENT_FINGERPRINTS"
    ),
  };
}

export async function startAvailabilityNode(
  config: AvailabilityNodeRuntimeConfig
): Promise<AvailabilityNodeRuntime> {
  const [key, cert, ca] = await Promise.all([
    readFile(config.serverKeyPath),
    readFile(config.serverCertPath),
    readFile(config.caPath),
  ]);

  const publisher = createPublicationExecutor({
    packageRoot: config.packageRoot,
    driveStorageRoot: config.driveStorageRoot,
    indexRoot: config.indexRoot,
    hyperswarmJoinTimeoutMs: config.hyperswarmJoinTimeoutMs,
  });
  const server = createPrivatePublicationServer(
    { key, cert, ca },
    config.allowedClientFingerprints,
    publisher
  );

  await listen(server, config.port, config.bindHost);

  let closed = false;
  return {
    server,
    close: async () => {
      if (closed) {
        return;
      }
      closed = true;
      await closeServer(server);
      await closeSharedStore(config.driveStorageRoot);
    },
  };
}

function optionalNonempty(env: NodeJS.ProcessEnv, name: string): string | undefined {
  const value = env[name];
  if (value === undefined) {
    return undefined;
  }
  const trimmed = value.trim();
  if (trimmed === "") {
    throw new TypeError(`${name} must not be empty`);
  }
  return trimmed;
}

function requiredNonempty(env: NodeJS.ProcessEnv, name: string): string {
  const value = optionalNonempty(env, name);
  if (value === undefined) {
    throw new TypeError(`${name} is required`);
  }
  return value;
}

function positiveInteger(
  env: NodeJS.ProcessEnv,
  name: string,
  defaultValue: number,
  maxValue: number
): number {
  const raw = optionalNonempty(env, name);
  if (raw === undefined) {
    return defaultValue;
  }
  if (!/^\d+$/.test(raw)) {
    throw new TypeError(`${name} must be a positive integer`);
  }
  const parsed = Number(raw);
  if (!Number.isSafeInteger(parsed) || parsed <= 0 || parsed > maxValue) {
    throw new TypeError(`${name} is outside the supported range`);
  }
  return parsed;
}

function requiredCsv(env: NodeJS.ProcessEnv, name: string): readonly string[] {
  const raw = requiredNonempty(env, name);
  const values = raw
    .split(",")
    .map((value) => value.trim())
    .filter((value) => value !== "");
  if (values.length === 0) {
    throw new TypeError(`${name} must contain at least one fingerprint`);
  }
  return values;
}

function listen(server: HttpsServer, port: number, host: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const onError = (error: Error): void => {
      server.off("listening", onListening);
      reject(error);
    };
    const onListening = (): void => {
      server.off("error", onError);
      resolve();
    };
    server.once("error", onError);
    server.once("listening", onListening);
    server.listen(port, host);
  });
}

function closeServer(server: HttpsServer): Promise<void> {
  if (!server.listening) {
    return Promise.resolve();
  }
  return new Promise((resolve, reject) => {
    server.close((error) => {
      if (error !== undefined) {
        reject(error);
      } else {
        resolve();
      }
    });
  });
}
