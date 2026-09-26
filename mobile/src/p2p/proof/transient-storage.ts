import { Directory, Paths } from "expo-file-system";

const RUN_ID = /^[a-z0-9]{8,64}$/;

export class TransientStorageError extends Error {
  readonly code: "TRANSIENT_STORAGE_PATH_INVALID" | "TRANSIENT_STORAGE_NOT_FOUND" | "TRANSIENT_STORAGE_DELETE_FAILED" | "TRANSIENT_STORAGE_VERIFY_FAILED";

  constructor(code: "TRANSIENT_STORAGE_PATH_INVALID" | "TRANSIENT_STORAGE_NOT_FOUND" | "TRANSIENT_STORAGE_DELETE_FAILED" | "TRANSIENT_STORAGE_VERIFY_FAILED") {
    let message = "Transient storage error";
    if (code === "TRANSIENT_STORAGE_PATH_INVALID") {
      message = "Transient storage path is invalid";
    } else if (code === "TRANSIENT_STORAGE_NOT_FOUND") {
      message = "Transient storage not found";
    } else if (code === "TRANSIENT_STORAGE_DELETE_FAILED") {
      message = "Transient storage delete failed";
    } else if (code === "TRANSIENT_STORAGE_VERIFY_FAILED") {
      message = "Transient storage verify failed";
    }

    super(message);
    this.name = "TransientStorageError";
    this.code = code;
  }
}

export function isWithinProofRoot(uri: string, runId: string): boolean {
  if (!RUN_ID.test(runId)) {
    return false;
  }

  if (uri.includes("..")) {
    return false;
  }

  const proofRoot = new Directory(Paths.cache, "dubbridge-p2p", "proofs", runId).uri;

  if (uri === proofRoot) {
    return true;
  }

  if (uri.startsWith(proofRoot + "/")) {
    return true;
  }

  return false;
}

export async function deleteProofRunDirectory(runId: string): Promise<void> {
  if (!RUN_ID.test(runId)) {
    throw new TransientStorageError("TRANSIENT_STORAGE_PATH_INVALID");
  }

  const dir = new Directory(Paths.cache, "dubbridge-p2p", "proofs", runId);

  if (!dir.exists) {
    throw new TransientStorageError("TRANSIENT_STORAGE_NOT_FOUND");
  }

  try {
    dir.delete();
  } catch (e) {
    throw new TransientStorageError("TRANSIENT_STORAGE_DELETE_FAILED");
  }

  if (dir.exists) {
    throw new TransientStorageError("TRANSIENT_STORAGE_VERIFY_FAILED");
  }
}

export async function listAbandonedProofRuns(maxAgeMs: number, now: () => number = () => Date.now()): Promise<string[]> {
  const parentDir = new Directory(Paths.cache, "dubbridge-p2p", "proofs");

  if (!parentDir.exists) {
    return [];
  }

  const entries = parentDir.list();
  const abandoned: string[] = [];

  for (const entry of entries) {
    if (entry instanceof Directory) {
      if (RUN_ID.test(entry.name)) {
        const info = entry.info();
        if (info.modificationTime !== undefined) {
          if (info.modificationTime < now() - maxAgeMs) {
            abandoned.push(entry.name);
          }
        }
      }
    }
  }

  return abandoned;
}
