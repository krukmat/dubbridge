import { RuntimeProtocolError, type SeedWriteHashDeleteReceipt } from "../runtime/protocol";
import {
  deleteProofRunDirectory,
  isWithinProofRoot,
  listAbandonedProofRuns,
  TransientStorageError,
} from "./transient-storage";
import { proofStorageUri, createProofSessionParts } from "./ProofRuntimeFactory";

export type SeedProofOutcome = {
  receipt: SeedWriteHashDeleteReceipt;
  deleted: true;
};

export async function runSeedProof(
  runId: string,
  timeoutMs = 5_000,
  createWorklet?: Parameters<typeof createProofSessionParts>[2],
): Promise<SeedProofOutcome> {
  const runRootUri = proofStorageUri(runId);
  if (!isWithinProofRoot(runRootUri, runId)) {
    throw new RuntimeProtocolError("PROOF_STORAGE_CONFIG_INVALID", "Proof storage configuration is invalid");
  }
  const { port, client } = createProofSessionParts(runId, timeoutMs, createWorklet);

  let receipt: SeedWriteHashDeleteReceipt;
  try {
    await client.handshake();
    receipt = await client.seedWriteHashDelete();
    await client.shutdown();
  } catch (error) {
    port.close(error instanceof Error ? error : new Error("Seed proof worklet failed"));
    throw error;
  }

  try {
    await deleteProofRunDirectory(runId);
  } catch (error) {
    if (error instanceof TransientStorageError) {
      throw new RuntimeProtocolError(
        error.code === "TRANSIENT_STORAGE_VERIFY_FAILED" ? "SEED_VERIFY_FAILED" : "SEED_DELETE_FAILED",
        error.code === "TRANSIENT_STORAGE_VERIFY_FAILED"
          ? "Seed run directory deletion could not be verified"
          : "Seed run directory could not be deleted",
      );
    }
    throw error;
  }

  return { receipt, deleted: true };
}

export async function janitorAbandonedProofRuns(
  maxAgeMs: number,
  now?: () => number,
): Promise<string[]> {
  const abandoned = await listAbandonedProofRuns(maxAgeMs, now);
  const removed: string[] = [];
  for (const runId of abandoned) {
    try {
      await deleteProofRunDirectory(runId);
      removed.push(runId);
    } catch (error) {
      if (!(error instanceof TransientStorageError) || error.code !== "TRANSIENT_STORAGE_NOT_FOUND") {
        throw error;
      }
    }
  }
  return removed;
}
