import {
  indexEntryPath,
  readIndexEntry,
  writeIndexEntry,
} from "./publication_index_io.js";
import type { PublicationRecord } from "./publication_record.js";

export type PersistOutcome =
  | { kind: "written" }
  | { kind: "replayed" }
  | { kind: "conflict"; existing: PublicationRecord };

export async function decideAndPersist(
  indexRoot: string,
  record: PublicationRecord
): Promise<PersistOutcome> {
  const entryPath = indexEntryPath(indexRoot, record.publication_id);
  const existing = await readIndexEntry(entryPath);

  if (existing === null) {
    await writeIndexEntry(entryPath, record);
    return { kind: "written" };
  }

  if (
    existing.lineage_id === record.lineage_id &&
    existing.manifest_digest_sha256 === record.manifest_digest_sha256
  ) {
    return { kind: "replayed" };
  }

  return { kind: "conflict", existing };
}
