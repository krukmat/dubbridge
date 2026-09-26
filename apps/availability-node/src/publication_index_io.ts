import { readFile } from "node:fs/promises";
import { verifyContainedRealpath } from "./containment.js";
import {
  encodePublicationRecord,
  decodePublicationRecord,
  type PublicationRecord,
} from "./publication_record.js";
import { writeFileAtomic } from "./write_atomic.js";

export function indexEntryPath(indexRoot: string, publicationId: string): string {
  return verifyContainedRealpath(indexRoot, `${publicationId}.json`);
}

export async function readIndexEntry(entryPath: string): Promise<PublicationRecord | null> {
  try {
    const bytes = await readFile(entryPath);
    return decodePublicationRecord(bytes);
  } catch (err: any) {
    if (err.code === "ENOENT") {
      return null;
    }
    throw err;
  }
}

export async function writeIndexEntry(
  entryPath: string,
  record: PublicationRecord
): Promise<void> {
  const bytes = encodePublicationRecord(record);
  await writeFileAtomic(entryPath, bytes);
}