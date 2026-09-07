import { DigestCompareResult } from "../runtime/digest-compare";
import { ReconnectDecision } from "../runtime/reconnect-budget";
import { DualSessionTeardownResult } from "./replication-teardown";

export type ReconnectOutcome =
  { attempted: false } | { attempted: true; decision: ReconnectDecision };

export type ReplicationVerdictResult =
  | { status: "VERIFIED" }
  | { status: "DIGEST_MISMATCH" }
  | { status: "RECONNECT_EXHAUSTED" }
  | { status: "TEARDOWN_FAILED"; side: "seed" | "client" | "both" };

export function assembleReplicationVerdict(
  verify: DigestCompareResult,
  reconnect: ReconnectOutcome,
  teardown: DualSessionTeardownResult,
): ReplicationVerdictResult {
  if (verify.matched === false) {
    return { status: "DIGEST_MISMATCH" };
  }
  if (reconnect.attempted === true && reconnect.decision === "exhausted") {
    return { status: "RECONNECT_EXHAUSTED" };
  }
  if (teardown.seed.ok === false && teardown.client.ok === false) {
    return { status: "TEARDOWN_FAILED", side: "both" };
  }
  if (teardown.seed.ok === false) {
    return { status: "TEARDOWN_FAILED", side: "seed" };
  }
  if (teardown.client.ok === false) {
    return { status: "TEARDOWN_FAILED", side: "client" };
  }
  return { status: "VERIFIED" };
}
