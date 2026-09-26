import {
  closeReplicationSession,
  type ReplicationSession,
} from "./ReplicationProofRunner";
import { deleteProofRunDirectory } from "./transient-storage";

export interface ReplicationSideTeardown {
  session: ReplicationSession;
  runId: string;
}

export type TeardownSideResult = { ok: true } | { ok: false; reason: unknown };

async function teardownOneSide(
  side: ReplicationSideTeardown,
): Promise<TeardownSideResult> {
  await closeReplicationSession(side.session);
  try {
    await deleteProofRunDirectory(side.runId);
  } catch (reason) {
    return { ok: false, reason };
  }
  return { ok: true };
}

export interface DualSessionTeardownResult {
  seed: TeardownSideResult;
  client: TeardownSideResult;
}

export async function teardownDualSessionReplication(
  seed: ReplicationSideTeardown,
  client: ReplicationSideTeardown,
): Promise<DualSessionTeardownResult> {
  const [seedResult, clientResult] = await Promise.all([
    teardownOneSide(seed),
    teardownOneSide(client),
  ]);
  return { seed: seedResult, client: clientResult };
}
