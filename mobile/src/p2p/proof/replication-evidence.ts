export interface RawReplicationEvidence {
  [key: string]: unknown;
}

export interface RedactedReplicationEvidence {
  [key: string]: unknown;
}

const SENSITIVE_KEYS = ["topic", "fixture", "fixtureContent", "discoveryKey"] as const;

export function redactReplicationEvidence(
  evidence: RawReplicationEvidence,
): RedactedReplicationEvidence {
  const redacted: RedactedReplicationEvidence = {};
  for (const [key, value] of Object.entries(evidence)) {
    if ((SENSITIVE_KEYS as readonly string[]).includes(key)) {
      continue;
    }
    redacted[key] = value;
  }
  return redacted;
}