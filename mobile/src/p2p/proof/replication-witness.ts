import type { DigestCompareResult } from "../runtime/digest-compare";
import type { ReconnectBudget } from "../runtime/reconnect-budget";
import {
  replicateWithOneReconnect,
  type ReplicationTopic,
} from "./replication-reconnect";
import {
  teardownDualSessionReplication,
  type ReplicationSideTeardown,
} from "./replication-teardown";
import {
  assembleReplicationVerdict,
  type ReplicationVerdictResult,
} from "./replication-verdict";

// assembleReplicationVerdict checks digest mismatch before reconnect
// exhaustion, so any DigestCompareResult passed here when reconnect is
// exhausted must not itself signal a mismatch, or the verdict would
// misreport DIGEST_MISMATCH instead of the real RECONNECT_EXHAUSTED cause.
// verify() is never invoked on this path (replication never completed), and
// assembleReplicationVerdict's own reconnect-exhausted check independently
// guarantees VERIFIED cannot be reached from here.
const VERIFY_NOT_ATTEMPTED: DigestCompareResult = { matched: true };

export async function runReplicationProof(
  seed: ReplicationSideTeardown,
  client: ReplicationSideTeardown,
  topic: ReplicationTopic,
  budget: ReconnectBudget,
  verify: () => Promise<DigestCompareResult>,
): Promise<ReplicationVerdictResult> {
  const reconnect = await replicateWithOneReconnect(
    seed.session,
    client.session,
    topic,
    budget,
  );

  const replicated =
    reconnect.attempted === false || reconnect.decision === "may-retry";
  const verifyResult = replicated ? await verify() : VERIFY_NOT_ATTEMPTED;

  const teardown = await teardownDualSessionReplication(seed, client);

  return assembleReplicationVerdict(verifyResult, reconnect, teardown);
}
