import { RuntimeProtocolError } from "./protocol";

export interface DigestHash {
  update(data: Uint8Array): DigestHash;
  digest(encoding: "hex"): string;
}

export interface DigestCompareResult {
  matched: boolean;
}

export function compareDigest(
  createHash: (algorithm: "sha256") => DigestHash,
  content: Uint8Array,
  expectedHexDigest: string,
): DigestCompareResult {
  let actual: string;
  try {
    actual = createHash("sha256").update(content).digest("hex");
  } catch {
    throw new RuntimeProtocolError("DIGEST_COMPARE_FAILED", "Digest could not be computed for comparison");
  }
  return { matched: actual === expectedHexDigest };
}
