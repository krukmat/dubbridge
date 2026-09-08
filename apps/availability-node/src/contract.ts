// Pure availability-publication-v1 vocabulary, types, and validation.

export const CONTRACT_VERSION = "availability-publication-v1" as const;
export const MANIFEST_VERSION = "p2p-manifest-v1" as const;
export const METHOD_PUT = "PUT" as const;
export const PATH_PREFIX = "/v1/publications/" as const;

export const REQUEST_FIELDS = ["contract_version", "publication_id", "lineage_id", "manifest_version", "manifest_digest_sha256", "package_ref"] as const;
export const EVIDENCE_FIELDS = ["contract_version", "publication_id", "lineage_id", "manifest_digest_sha256", "external_publication_id", "evidence_id", "confirmed_at"] as const;
export const ERROR_CODE_STATUS = { invalid_contract: 400, service_identity_rejected: 403, publication_conflict: 409, package_invalid: 422, publication_unavailable: 503 } as const;

export type RequestFieldName = (typeof REQUEST_FIELDS)[number];
export type EvidenceFieldName = (typeof EVIDENCE_FIELDS)[number];
export type ErrorCode = keyof typeof ERROR_CODE_STATUS;
export type ErrorStatus = (typeof ERROR_CODE_STATUS)[ErrorCode];
export interface PublicationRequest { contract_version: typeof CONTRACT_VERSION; publication_id: string; lineage_id: string; manifest_version: typeof MANIFEST_VERSION; manifest_digest_sha256: string; package_ref: string; }
export interface PublicationEvidence { contract_version: typeof CONTRACT_VERSION; publication_id: string; lineage_id: string; manifest_digest_sha256: string; external_publication_id: string; evidence_id: string; confirmed_at: string; }
export type SuccessStatus = 200 | 201;
export interface PublicationSuccessResponse { status: SuccessStatus; evidence: PublicationEvidence; }
export interface PublicationErrorBody { contract_version: typeof CONTRACT_VERSION; code: ErrorCode; }
export interface PublicationErrorResponse { status: ErrorStatus; body: PublicationErrorBody; }
export type PublicationHttpResponse = PublicationSuccessResponse | PublicationErrorResponse;

export class PublicationContractError extends Error {
  readonly code: ErrorCode;
  constructor(code: ErrorCode) {
    super(code);
    this.name = "PublicationContractError";
    this.code = code;
  }
}

const UUID_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const SHA256_REGEX = /^[0-9a-f]{64}$/;

export function parsePublicationRequest(input: unknown, pathPublicationId?: string): PublicationRequest {
  if (input === null || typeof input !== "object" || Array.isArray(input)) {
    throw new PublicationContractError("invalid_contract");
  }
  const proto = Object.getPrototypeOf(input);
  if (proto !== null && proto !== Object.prototype) {
    throw new PublicationContractError("invalid_contract");
  }
  const obj = input as Record<string, unknown>;
  const keys = Object.keys(obj);
  if (keys.length !== REQUEST_FIELDS.length) {
    throw new PublicationContractError("invalid_contract");
  }
  const keySet = new Set(keys);
  for (const field of REQUEST_FIELDS) {
    if (!keySet.has(field)) throw new PublicationContractError("invalid_contract");
  }
  for (const field of REQUEST_FIELDS) {
    if (typeof obj[field] !== "string") throw new PublicationContractError("invalid_contract");
  }
  if (obj.contract_version !== CONTRACT_VERSION || obj.manifest_version !== MANIFEST_VERSION) {
    throw new PublicationContractError("invalid_contract");
  }

  const publicationId = obj.publication_id as string;
  const lineageId = obj.lineage_id as string;
  const manifestDigest = obj.manifest_digest_sha256 as string;

  if (!UUID_REGEX.test(publicationId)) {
    throw new PublicationContractError("invalid_contract");
  }
  if (!UUID_REGEX.test(lineageId)) {
    throw new PublicationContractError("invalid_contract");
  }
  if (!SHA256_REGEX.test(manifestDigest)) {
    throw new PublicationContractError("invalid_contract");
  }

  if (pathPublicationId !== undefined) {
    if (!UUID_REGEX.test(pathPublicationId)) {
      throw new PublicationContractError("invalid_contract");
    }
    if (pathPublicationId !== publicationId) {
      throw new PublicationContractError("invalid_contract");
    }
  }

  const rawPackageRef = obj.package_ref as string;
  if (typeof rawPackageRef !== 'string' || rawPackageRef.length === 0) {
    throw new PublicationContractError("package_invalid");
  }

  const normalizedPackageRef = rawPackageRef.normalize('NFC');

  if (normalizedPackageRef.length === 0) {
    throw new PublicationContractError("package_invalid");
  }

  if (normalizedPackageRef !== rawPackageRef) {
    throw new PublicationContractError("package_invalid");
  }

  if (normalizedPackageRef.includes('\\') || normalizedPackageRef.includes('\0')) {
    throw new PublicationContractError("package_invalid");
  }

  if (normalizedPackageRef.startsWith('/') || normalizedPackageRef.endsWith('/')) {
    throw new PublicationContractError("package_invalid");
  }

  if (normalizedPackageRef.includes('//')) {
    throw new PublicationContractError("package_invalid");
  }

  if (/^[A-Za-z]:/.test(normalizedPackageRef)) {
    throw new PublicationContractError("package_invalid");
  }

  for (let i = 0; i < normalizedPackageRef.length; i++) {
    const code = normalizedPackageRef.charCodeAt(i);
    if (code < 32 || code === 127) {
      throw new PublicationContractError("package_invalid");
    }
  }

  const segments = normalizedPackageRef.split('/');
  if (segments.length === 0) {
    throw new PublicationContractError("package_invalid");
  }

  for (const segment of segments) {
    if (segment === '.' || segment === '..' || segment.length === 0) {
      throw new PublicationContractError("package_invalid");
    }
  }

  const packageRef = normalizedPackageRef;

  return {
    contract_version: obj.contract_version as typeof CONTRACT_VERSION,
    publication_id: publicationId,
    lineage_id: lineageId,
    manifest_version: obj.manifest_version as typeof MANIFEST_VERSION,
    manifest_digest_sha256: manifestDigest,
    package_ref: packageRef,
  };
}

const RFC3339_REGEX = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/;

function isPlainObject(value: unknown): value is Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    return false;
  }
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
}

function hasControlChars(str: string): boolean {
  for (let i = 0; i < str.length; i++) {
    const code = str.charCodeAt(i);
    if (code < 32 || code === 127) return true;
  }
  return false;
}

function isValidRfc3339Utc(str: string): boolean {
  if (!RFC3339_REGEX.test(str)) return false;
  const date = new Date(str);
  if (isNaN(date.getTime())) return false;
  const year = parseInt(str.substring(0, 4), 10);
  const month = parseInt(str.substring(5, 7), 10);
  const day = parseInt(str.substring(8, 10), 10);
  const hour = parseInt(str.substring(11, 13), 10);
  const minute = parseInt(str.substring(14, 16), 10);
  const second = parseInt(str.substring(17, 19), 10);
  if (date.getUTCFullYear() !== year || date.getUTCMonth() + 1 !== month || date.getUTCDate() !== day ||
      date.getUTCHours() !== hour || date.getUTCMinutes() !== minute || date.getUTCSeconds() !== second) {
    return false;
  }
  return true;
}

export function parsePublicationResponse(
  status: number,
  body: unknown,
  expectedRequest?: PublicationRequest
): PublicationHttpResponse {
  if (status === 200 || status === 201) {
    if (!isPlainObject(body)) {
      throw new PublicationContractError("invalid_contract");
    }
    const keys = Object.keys(body);
    if (keys.length !== EVIDENCE_FIELDS.length) {
      throw new PublicationContractError("invalid_contract");
    }
    const keySet = new Set(keys);
    for (const field of EVIDENCE_FIELDS) {
      if (!keySet.has(field)) {
        throw new PublicationContractError("invalid_contract");
      }
    }
    for (const field of EVIDENCE_FIELDS) {
      if (typeof body[field] !== 'string') {
        throw new PublicationContractError("invalid_contract");
      }
    }

    const evidence = body as Record<string, string>;
    if (evidence.contract_version !== CONTRACT_VERSION) {
      throw new PublicationContractError("invalid_contract");
    }
    const publicationId = evidence.publication_id!;
    const lineageId = evidence.lineage_id!;
    const manifestDigest = evidence.manifest_digest_sha256!;
    const externalPublicationId = evidence.external_publication_id!;
    const evidenceId = evidence.evidence_id!;
    const confirmedAt = evidence.confirmed_at!;

    if (!UUID_REGEX.test(publicationId)) {
      throw new PublicationContractError("invalid_contract");
    }
    if (!UUID_REGEX.test(lineageId)) {
      throw new PublicationContractError("invalid_contract");
    }
    if (!SHA256_REGEX.test(manifestDigest)) {
      throw new PublicationContractError("invalid_contract");
    }
    if (externalPublicationId.length === 0 || hasControlChars(externalPublicationId)) {
      throw new PublicationContractError("invalid_contract");
    }
    if (evidenceId.length === 0 || hasControlChars(evidenceId)) {
      throw new PublicationContractError("invalid_contract");
    }
    if (!isValidRfc3339Utc(confirmedAt)) {
      throw new PublicationContractError("invalid_contract");
    }

    if (expectedRequest) {
      if (publicationId !== expectedRequest.publication_id) {
        throw new PublicationContractError("invalid_contract");
      }
      if (lineageId !== expectedRequest.lineage_id) {
        throw new PublicationContractError("invalid_contract");
      }
      if (manifestDigest !== expectedRequest.manifest_digest_sha256) {
        throw new PublicationContractError("invalid_contract");
      }
    }

    return {
      status,
      evidence: {
        contract_version: CONTRACT_VERSION,
        publication_id: publicationId,
        lineage_id: lineageId,
        manifest_digest_sha256: manifestDigest,
        external_publication_id: externalPublicationId,
        evidence_id: evidenceId,
        confirmed_at: confirmedAt,
      }
    };
  }

  if (status === 400 || status === 403 || status === 409 || status === 422 || status === 503) {
    if (!isPlainObject(body)) {
      throw new PublicationContractError("invalid_contract");
    }
    const keys = Object.keys(body);
    if (keys.length !== 2 || !keys.includes('contract_version') || !keys.includes('code')) {
      throw new PublicationContractError("invalid_contract");
    }
    const errorBody = body as Record<string, unknown>;
    if (errorBody.contract_version !== CONTRACT_VERSION) {
      throw new PublicationContractError("invalid_contract");
    }
    const code = errorBody.code;
    if (typeof code !== 'string' || !Object.keys(ERROR_CODE_STATUS).includes(code)) {
      throw new PublicationContractError("invalid_contract");
    }
    const mappedStatus = ERROR_CODE_STATUS[code as ErrorCode];
    if (mappedStatus !== status) {
      throw new PublicationContractError("invalid_contract");
    }
    return {
      status,
      body: {
        contract_version: CONTRACT_VERSION,
        code: code as ErrorCode,
      }
    };
  }

  throw new PublicationContractError("invalid_contract");
}

export function createPublicationError(code: ErrorCode): PublicationErrorResponse {
  const status = ERROR_CODE_STATUS[code];
  return {
    status,
    body: {
      contract_version: CONTRACT_VERSION,
      code,
    }
  };
}

export function serializePublicationBody(
  response: PublicationHttpResponse,
  expectedRequest?: PublicationRequest
): string {
  const revalidated = parsePublicationResponse(response.status, 'evidence' in response ? response.evidence : response.body, expectedRequest);
  const wireBody = 'evidence' in revalidated ? revalidated.evidence : revalidated.body;
  return JSON.stringify(wireBody);
}
