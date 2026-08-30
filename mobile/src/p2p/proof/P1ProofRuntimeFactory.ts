import { Directory, Paths } from "expo-file-system";
import { Worklet } from "react-native-bare-kit";

import RUNTIME_WORKLET_SOURCE from "../runtime/worklet.bundle.js";

const RUN_ID = /^[a-z0-9]{8,64}$/;
const PROOF_WORKLET_FILENAME = "/dubbridge-p2p-proof.worklet";

export class ProofStorageConfigError extends Error {
  readonly code = "PROOF_STORAGE_CONFIG_INVALID" as const;
  constructor() {
    super("Proof storage configuration is invalid");
    this.name = "ProofStorageConfigError";
  }
}

export function createProofRunId(random: () => string = () => Math.random().toString(36).slice(2)): string {
  const runId = random();
  if (!RUN_ID.test(runId)) throw new ProofStorageConfigError();
  return runId;
}

export function proofStorageUri(runId: string): string {
  if (!RUN_ID.test(runId)) throw new ProofStorageConfigError();
  return new Directory(Paths.cache, "dubbridge-p2p", "proofs", runId).uri;
}

export function startProofWorklet(runId: string, createWorklet: () => Pick<Worklet, "start"> = () => new Worklet()): Pick<Worklet, "start"> {
  const uri = proofStorageUri(runId);
  if (!uri.startsWith("file:")) throw new ProofStorageConfigError();
  const worklet = createWorklet();
  worklet.start(PROOF_WORKLET_FILENAME, RUNTIME_WORKLET_SOURCE, [uri]);
  return worklet;
}
