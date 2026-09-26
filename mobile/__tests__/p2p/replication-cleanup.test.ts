import {
  teardownDualSessionReplication,
  type ReplicationSideTeardown,
} from "../../src/p2p/proof/replication-teardown";
import { deleteProofRunDirectory } from "../../src/p2p/proof/transient-storage";

jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

jest.mock("../../src/p2p/proof/transient-storage", () => ({
  deleteProofRunDirectory: jest.fn(),
}));

function fakeSession(): any {
  return {
    worklet: {},
    port: { close: jest.fn() },
    client: { shutdown: jest.fn().mockResolvedValue(undefined) },
  };
}

describe("teardownDualSessionReplication", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it("HP-B2.d-ii both sessions close and both run directories are deleted and verified absent", async () => {
    const mockDelete = deleteProofRunDirectory as jest.MockedFunction<
      typeof deleteProofRunDirectory
    >;
    mockDelete.mockResolvedValue(undefined);

    const seedSession = fakeSession();
    const clientSession = fakeSession();

    const seed: ReplicationSideTeardown = {
      session: seedSession,
      runId: "seedrunid1",
    };
    const client: ReplicationSideTeardown = {
      session: clientSession,
      runId: "clientrunid1",
    };

    const result = await teardownDualSessionReplication(seed, client);

    expect(result).toEqual({
      seed: { ok: true },
      client: { ok: true },
    });

    expect(mockDelete).toHaveBeenCalledTimes(2);
    expect(mockDelete).toHaveBeenCalledWith("seedrunid1");
    expect(mockDelete).toHaveBeenCalledWith("clientrunid1");

    expect(seedSession.client.shutdown).toHaveBeenCalledTimes(1);
    expect(seedSession.port.close).toHaveBeenCalledTimes(1);
    expect(clientSession.client.shutdown).toHaveBeenCalledTimes(1);
    expect(clientSession.port.close).toHaveBeenCalledTimes(1);
  });

  it("EC-B2.d-ii a failure closing/deleting one side does not skip attempting the other; both failures surface, none are swallowed", async () => {
    const mockDelete = deleteProofRunDirectory as jest.MockedFunction<
      typeof deleteProofRunDirectory
    >;
    const seedRunId = "seedrunid2";
    const clientRunId = "clientrunid2";

    mockDelete.mockImplementation((runId: string) => {
      if (runId === seedRunId) {
        return Promise.reject(new Error("seed delete failed"));
      }
      if (runId === clientRunId) {
        return Promise.reject(new Error("client delete failed"));
      }
      return Promise.resolve();
    });

    const seedSession = fakeSession();
    const clientSession = fakeSession();

    const seed: ReplicationSideTeardown = {
      session: seedSession,
      runId: seedRunId,
    };
    const client: ReplicationSideTeardown = {
      session: clientSession,
      runId: clientRunId,
    };

    const result = await teardownDualSessionReplication(seed, client);

    expect(result.seed.ok).toBe(false);
    expect(result.client.ok).toBe(false);

    if (!result.seed.ok) {
      expect(result.seed.reason).toEqual(new Error("seed delete failed"));
    }
    if (!result.client.ok) {
      expect(result.client.reason).toEqual(new Error("client delete failed"));
    }

    expect(mockDelete).toHaveBeenCalledTimes(2);
    expect(mockDelete).toHaveBeenCalledWith(seedRunId);
    expect(mockDelete).toHaveBeenCalledWith(clientRunId);

    expect(seedSession.client.shutdown).toHaveBeenCalledTimes(1);
    expect(seedSession.port.close).toHaveBeenCalledTimes(1);
    expect(clientSession.client.shutdown).toHaveBeenCalledTimes(1);
    expect(clientSession.port.close).toHaveBeenCalledTimes(1);
  });
});
