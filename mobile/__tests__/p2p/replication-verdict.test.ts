import {
  assembleReplicationVerdict,
  type ReconnectOutcome,
  type ReplicationVerdictResult,
} from "../../src/p2p/proof/replication-verdict";
import { type DigestCompareResult } from "../../src/p2p/runtime/digest-compare";
import {
  type DualSessionTeardownResult,
  type TeardownSideResult,
} from "../../src/p2p/proof/replication-teardown";

const ok: TeardownSideResult = { ok: true };
const failed: TeardownSideResult = { ok: false, reason: "error" };
const noReconnect: ReconnectOutcome = { attempted: false };

interface Case {
  name: string;
  verify: DigestCompareResult;
  reconnect: ReconnectOutcome;
  teardown: DualSessionTeardownResult;
  expected: ReplicationVerdictResult;
}

const cases: Case[] = [
  {
    name: "HP-B2.e: all-success inputs yield VERIFIED",
    verify: { matched: true },
    reconnect: noReconnect,
    teardown: { seed: ok, client: ok },
    expected: { status: "VERIFIED" },
  },
  {
    name: "HP-B2.e: successful reconnect variant yields VERIFIED",
    verify: { matched: true },
    reconnect: { attempted: true, decision: "may-retry" },
    teardown: { seed: ok, client: ok },
    expected: { status: "VERIFIED" },
  },
  {
    name: "EC-B2.e: verify mismatch yields DIGEST_MISMATCH",
    verify: { matched: false },
    reconnect: noReconnect,
    teardown: { seed: ok, client: ok },
    expected: { status: "DIGEST_MISMATCH" },
  },
  {
    name: "EC-B2.e: reconnect exhausted yields RECONNECT_EXHAUSTED",
    verify: { matched: true },
    reconnect: { attempted: true, decision: "exhausted" },
    teardown: { seed: ok, client: ok },
    expected: { status: "RECONNECT_EXHAUSTED" },
  },
  {
    name: "EC-B2.e: teardown seed fails only yields TEARDOWN_FAILED seed",
    verify: { matched: true },
    reconnect: noReconnect,
    teardown: { seed: failed, client: ok },
    expected: { status: "TEARDOWN_FAILED", side: "seed" },
  },
  {
    name: "EC-B2.e: teardown client fails only yields TEARDOWN_FAILED client",
    verify: { matched: true },
    reconnect: noReconnect,
    teardown: { seed: ok, client: failed },
    expected: { status: "TEARDOWN_FAILED", side: "client" },
  },
  {
    name: "EC-B2.e: teardown both fail yields TEARDOWN_FAILED both",
    verify: { matched: true },
    reconnect: noReconnect,
    teardown: { seed: failed, client: failed },
    expected: { status: "TEARDOWN_FAILED", side: "both" },
  },
  {
    name: "EC-B2.e: priority ordering - verify mismatch wins over teardown failure",
    verify: { matched: false },
    reconnect: noReconnect,
    teardown: { seed: failed, client: ok },
    expected: { status: "DIGEST_MISMATCH" },
  },
];

describe("assembleReplicationVerdict", () => {
  it.each(cases)("$name", ({ verify, reconnect, teardown, expected }) => {
    const result = assembleReplicationVerdict(verify, reconnect, teardown);
    expect(result).toEqual(expected);
  });
});
