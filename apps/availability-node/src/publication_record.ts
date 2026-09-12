import type { PublicationEvidence } from "./contract.js";
import {
  EVIDENCE_FIELDS,
  parsePublicationResponse,
  PublicationContractError,
} from "./contract.js";

export type PublicationRecord = PublicationEvidence;

function asEvidence(parsed: ReturnType<typeof parsePublicationResponse>): PublicationEvidence {
  if (!("evidence" in parsed)) {
    throw new PublicationContractError("invalid_contract");
  }
  return parsed.evidence;
}

export function encodePublicationRecord(record: PublicationRecord): Uint8Array {
  const parsed = parsePublicationResponse(200, record);
  const evidence = asEvidence(parsed);

  const ordered: Record<string, string> = {};
  for (const field of EVIDENCE_FIELDS) {
    ordered[field] = evidence[field];
  }

  const jsonString = JSON.stringify(ordered);
  return new TextEncoder().encode(jsonString);
}

export function decodePublicationRecord(bytes: Uint8Array): PublicationRecord {
  let decoded: string;
  try {
    decoded = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new PublicationContractError("invalid_contract");
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(decoded);
  } catch {
    throw new PublicationContractError("invalid_contract");
  }

  const result = parsePublicationResponse(200, parsed);
  const evidence = asEvidence(result);

  const reEncoded = encodePublicationRecord(evidence);

  if (reEncoded.length !== bytes.length) {
    throw new PublicationContractError("invalid_contract");
  }

  for (let i = 0; i < reEncoded.length; i++) {
    if (reEncoded[i] !== bytes[i]) {
      throw new PublicationContractError("invalid_contract");
    }
  }

  return evidence;
}