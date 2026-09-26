import type { RequestListener } from "node:http";
import {
  createServer as createHttpsServer,
  type Server as HttpsServer,
  type ServerOptions as HttpsServerOptions,
} from "node:https";

const COMPACT_SHA256_FINGERPRINT = /^[0-9a-fA-F]{64}$/;
const NODE_SHA256_FINGERPRINT = /^(?:[0-9a-fA-F]{2}:){31}[0-9a-fA-F]{2}$/;

export type ClientFingerprintPolicy = (
  presentedFingerprint: string | undefined
) => boolean;

export interface PrivateMtlsCredentials {
  readonly key: NonNullable<HttpsServerOptions["key"]>;
  readonly cert: NonNullable<HttpsServerOptions["cert"]>;
  readonly ca: NonNullable<HttpsServerOptions["ca"]>;
}

export function normalizeSha256Fingerprint(value: string): string {
  if (typeof value !== "string") {
    throw new TypeError("invalid SHA-256 certificate fingerprint");
  }

  if (COMPACT_SHA256_FINGERPRINT.test(value)) {
    return value.toLowerCase();
  }

  if (NODE_SHA256_FINGERPRINT.test(value)) {
    return value.replaceAll(":", "").toLowerCase();
  }

  throw new TypeError("invalid SHA-256 certificate fingerprint");
}

export function createClientFingerprintPolicy(
  allowedFingerprints: readonly string[]
): ClientFingerprintPolicy {
  if (!Array.isArray(allowedFingerprints) || allowedFingerprints.length === 0) {
    throw new TypeError("at least one client certificate fingerprint is required");
  }

  const allowed = new Set<string>();
  for (const fingerprint of allowedFingerprints) {
    const normalized = normalizeSha256Fingerprint(fingerprint);
    if (allowed.has(normalized)) {
      throw new TypeError("duplicate client certificate fingerprint");
    }
    allowed.add(normalized);
  }

  return (presentedFingerprint: string | undefined): boolean => {
    if (typeof presentedFingerprint !== "string") {
      return false;
    }

    try {
      return allowed.has(normalizeSha256Fingerprint(presentedFingerprint));
    } catch {
      return false;
    }
  };
}

function hasCredentialMaterial(value: unknown): boolean {
  if (typeof value === "string") {
    return value.length > 0;
  }
  if (Buffer.isBuffer(value)) {
    return value.length > 0;
  }
  if (Array.isArray(value)) {
    return value.length > 0 && value.every(hasCredentialMaterial);
  }
  return typeof value === "object" && value !== null;
}

export function createPrivateMtlsServer(
  credentials: PrivateMtlsCredentials,
  requestListener: RequestListener
): HttpsServer {
  if (typeof credentials !== "object" || credentials === null) {
    throw new TypeError("mTLS credentials are required");
  }
  if (!hasCredentialMaterial(credentials.key)) {
    throw new TypeError("mTLS server key is required");
  }
  if (!hasCredentialMaterial(credentials.cert)) {
    throw new TypeError("mTLS server certificate is required");
  }
  if (!hasCredentialMaterial(credentials.ca)) {
    throw new TypeError("mTLS client trust anchor is required");
  }
  if (typeof requestListener !== "function") {
    throw new TypeError("mTLS request listener is required");
  }

  const { key, cert, ca } = credentials;
  return createHttpsServer(
    {
      key,
      cert,
      ca,
      minVersion: "TLSv1.3",
      requestCert: true,
      rejectUnauthorized: true,
    },
    requestListener
  );
}
