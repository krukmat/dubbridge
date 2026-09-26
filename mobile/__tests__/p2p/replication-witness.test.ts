import { runReplicationProof } from "../../src/p2p/proof/replication-witness";
import type { ReplicationSession } from "../../src/p2p/proof/ReplicationProofRunner";
import type { ReplicationSideTeardown } from "../../src/p2p/proof/replication-teardown";
import { createReconnectBudget } from "../../src/p2p/runtime/reconnect-budget";
import type { DigestCompareResult } from "../../src/p2p/runtime/digest-compare";

jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

jest.mock("../../src/p2p/proof/transient-storage", () => ({
  deleteProofRunDirectory: jest.fn().mockResolvedValue(undefined),
}));

const topic = Buffer.from("a".repeat(64), "hex");

function fakeSession(discoverAndReplicate: jest.Mock): ReplicationSession {
  return {
    worklet: {} as never,
    port: { close: jest.fn() } as never,
    client: {
      handshake: jest.fn().mockResolvedValue(undefined),
      discoverAndReplicate,
      shutdown: jest.fn().mockResolvedValue(undefined),
    } as never,
  };
}

const okReceipt = {
  capability: "discover-and-replicate" as const,
  schema_version: 1 as const,
  role: "seed" as const,
  byte_count: 42,
};

const matched: DigestCompareResult = { matched: true };
const mismatched: DigestCompareResult = { matched: false };
const disconnected = new Error("disconnected");

interface Case {
  name: string;
  seedDiscover: jest.Mock;
  clientDiscover: jest.Mock;
  verify: DigestCompareResult;
  budget: number;
  deleteFails: "seed" | "none";
  runIdSuffix: string;
  expected: ReturnType<typeof runReplicationProof> extends Promise<infer R>
    ? R
    : never;
  extra?: (ctx: {
    seedSession: ReplicationSession;
    clientSession: ReplicationSession;
    verifyMock: jest.Mock;
    seedDiscoverMock: jest.Mock;
    clientDiscoverMock: jest.Mock;
  }) => void;
}

const cases: Case[] = [
  {
    name: "HP-B2.f no disconnect: emits VERIFIED only after digest match",
    seedDiscover: jest.fn().mockResolvedValue(okReceipt),
    clientDiscover: jest.fn().mockResolvedValue(okReceipt),
    verify: matched,
    budget: 1,
    deleteFails: "none",
    runIdSuffix: "1",
    expected: { status: "VERIFIED" },
    extra: ({ verifyMock, seedDiscoverMock, clientDiscoverMock }) => {
      expect(verifyMock).toHaveBeenCalledTimes(1);
      expect(seedDiscoverMock).toHaveBeenCalledTimes(1);
      expect(clientDiscoverMock).toHaveBeenCalledTimes(1);
    },
  },
  {
    name: "HP-B2.f successful bounded reconnect: retries once within budget, then emits VERIFIED",
    seedDiscover: jest
      .fn()
      .mockRejectedValueOnce(disconnected)
      .mockResolvedValueOnce(okReceipt),
    clientDiscover: jest
      .fn()
      .mockRejectedValueOnce(disconnected)
      .mockResolvedValueOnce(okReceipt),
    verify: matched,
    budget: 1,
    deleteFails: "none",
    runIdSuffix: "2",
    expected: { status: "VERIFIED" },
    extra: ({ verifyMock, seedDiscoverMock, clientDiscoverMock }) => {
      expect(seedDiscoverMock).toHaveBeenCalledTimes(2);
      expect(clientDiscoverMock).toHaveBeenCalledTimes(2);
      expect(verifyMock).toHaveBeenCalledTimes(1);
    },
  },
  {
    name: "EC-B2.f digest mismatch never transitions to VERIFIED, and teardown still runs",
    seedDiscover: jest.fn().mockResolvedValue(okReceipt),
    clientDiscover: jest.fn().mockResolvedValue(okReceipt),
    verify: mismatched,
    budget: 1,
    deleteFails: "none",
    runIdSuffix: "3",
    expected: { status: "DIGEST_MISMATCH" },
    extra: ({ seedSession, clientSession }) => {
      expect(seedSession.client.shutdown).toHaveBeenCalledTimes(1);
      expect(clientSession.client.shutdown).toHaveBeenCalledTimes(1);
    },
  },
  {
    name: "EC-B2.f reconnect-budget exhaustion never transitions to VERIFIED, skips verify, and teardown still runs",
    seedDiscover: jest.fn().mockRejectedValue(new Error("disconnected")),
    clientDiscover: jest.fn().mockRejectedValue(new Error("disconnected")),
    verify: matched,
    budget: 0,
    deleteFails: "none",
    runIdSuffix: "4",
    expected: { status: "RECONNECT_EXHAUSTED" },
    extra: ({ verifyMock }) => {
      expect(verifyMock).not.toHaveBeenCalled();
    },
  },
  {
    name: "EC-B2.f teardown failure never transitions to VERIFIED even when replication and verify succeed",
    seedDiscover: jest.fn().mockResolvedValue(okReceipt),
    clientDiscover: jest.fn().mockResolvedValue(okReceipt),
    verify: matched,
    budget: 1,
    deleteFails: "seed",
    runIdSuffix: "5",
    expected: { status: "TEARDOWN_FAILED", side: "seed" },
  },
  {
    name: "EC-B2.f priority ordering: digest mismatch wins over a concurrent teardown failure",
    seedDiscover: jest.fn().mockResolvedValue(okReceipt),
    clientDiscover: jest.fn().mockResolvedValue(okReceipt),
    verify: mismatched,
    budget: 1,
    deleteFails: "seed",
    runIdSuffix: "6",
    expected: { status: "DIGEST_MISMATCH" },
  },
];

describe("runReplicationProof", () => {
  it.each(cases)(
    "$name",
    async ({
      seedDiscover,
      clientDiscover,
      verify,
      budget,
      deleteFails,
      runIdSuffix,
      expected,
      extra,
    }) => {
      const seedSession = fakeSession(seedDiscover);
      const clientSession = fakeSession(clientDiscover);
      const verifyMock = jest.fn().mockResolvedValue(verify);
      const { deleteProofRunDirectory } = jest.requireMock(
        "../../src/p2p/proof/transient-storage",
      ) as { deleteProofRunDirectory: jest.Mock };
      const seedRunId = `seed-run-${runIdSuffix}`;
      const clientRunId = `client-run-${runIdSuffix}`;
      deleteProofRunDirectory.mockImplementation((runId: string) =>
        deleteFails === "seed" && runId === seedRunId
          ? Promise.reject(new Error("delete failed"))
          : Promise.resolve(),
      );

      const result = await runReplicationProof(
        { session: seedSession, runId: seedRunId },
        { session: clientSession, runId: clientRunId },
        topic,
        createReconnectBudget(budget),
        verifyMock,
      );

      expect(result).toEqual(expected);
      expect(deleteProofRunDirectory).toHaveBeenCalledWith(seedRunId);
      expect(deleteProofRunDirectory).toHaveBeenCalledWith(clientRunId);
      extra?.({
        seedSession,
        clientSession,
        verifyMock,
        seedDiscoverMock: seedDiscover,
        clientDiscoverMock: clientDiscover,
      });
    },
  );
});
