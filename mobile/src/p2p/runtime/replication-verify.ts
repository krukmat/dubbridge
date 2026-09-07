import { readTransientDriveFile } from "./transient-drive";
import { compareDigest, DigestHash, DigestCompareResult } from "./digest-compare";

export async function verifyReplicatedFile(
  drive: { get(path: string): Promise<Uint8Array | null> },
  createHash: (algorithm: "sha256") => DigestHash,
  path: string,
  expectedHexDigest: string,
): Promise<DigestCompareResult> {
  const bytes = await readTransientDriveFile(drive, path);
  return compareDigest(createHash, bytes, expectedHexDigest);
}