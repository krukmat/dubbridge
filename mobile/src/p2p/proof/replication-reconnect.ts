import {
  recordDisconnect,
  type ReconnectBudget,
} from "../runtime/reconnect-budget";
import {
  runAndReconcileDualSessionReplication,
  type ReplicationSession,
} from "./ReplicationProofRunner";
import type { ReconnectOutcome } from "./replication-verdict";

export type ReplicationTopic = Buffer;

export async function replicateWithOneReconnect(
  seedSession: ReplicationSession,
  clientSession: ReplicationSession,
  topic: ReplicationTopic,
  budget: ReconnectBudget,
): Promise<ReconnectOutcome> {
  const first = await runAndReconcileDualSessionReplication(
    seedSession,
    clientSession,
    topic,
  );
  if (first.ok) {
    return { attempted: false };
  }

  const { decision } = recordDisconnect(budget);
  if (decision === "exhausted") {
    return { attempted: true, decision: "exhausted" };
  }

  const retry = await runAndReconcileDualSessionReplication(
    seedSession,
    clientSession,
    topic,
  );
  return {
    attempted: true,
    decision: retry.ok ? "may-retry" : "exhausted",
  };
}
