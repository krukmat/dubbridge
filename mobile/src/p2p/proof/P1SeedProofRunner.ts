import type { Worklet } from "react-native-bare-kit";

import { RuntimeProtocolError, type SeedWriteHashDeleteReceipt } from "../runtime/protocol";
import { BareRpcPort, RuntimeProtocolClient } from "../runtime/runtime-client";
import { deleteProofRunDirectory, TransientStorageError } from "./transient-storage";
import { startProofWorklet } from "./P1ProofRuntimeFactory";

export type SeedProofOutcome = {
  receipt: SeedWriteHashDeleteReceipt;
  deleted: true;
};

export async function runSeedProof(
  runId: string,
  timeoutMs = 5_000,
  createWorklet?: () => Pick<Worklet, "start" | "IPC">,
): Promise<SeedProofOutcome> {
  const worklet = startProofWorklet(runId, createWorklet) as Pick<Worklet, "start" | "IPC">;
  const port = new BareRpcPort(worklet.IPC as never);
  const client = new RuntimeProtocolClient(port, timeoutMs);

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
