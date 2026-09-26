import {
  startReplicationSession,
  runAndReconcileDualSessionReplication,
  ReplicationSession,
} from "../../src/p2p/proof/ReplicationProofRunner";

jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

const mockRuntimeClient = {
  handshake: jest.fn(),
  discoverAndReplicate: jest.fn(),
  shutdown: jest.fn(),
};

jest.mock("../../src/p2p/runtime/runtime-client", () => ({
  BareRpcPort: jest.fn().mockImplementation(() => ({ close: jest.fn() })),
  RuntimeProtocolClient: jest.fn().mockImplementation(() => mockRuntimeClient),
}));

describe("hyperswarm-replication", () => {
  let seedSession: ReplicationSession;
  let clientSession: ReplicationSession;

  beforeEach(() => {
    jest.clearAllMocks();

    mockRuntimeClient.handshake.mockResolvedValue({
      runtimeVersion: "1.0.0",
      capabilities: [],
    });

    seedSession = startReplicationSession("seedrun01", 5_000, () => ({
      start: jest.fn(),
      IPC: {} as never,
    }));

    clientSession = startReplicationSession("clientrun1", 5_000, () => ({
      start: jest.fn(),
      IPC: {} as never,
    }));
  });

  it("HP-B1: successful dual-session replication reports both receipts and does not close either session", async () => {
    mockRuntimeClient.discoverAndReplicate.mockImplementation(
      (topic: Buffer, role: "seed" | "client") =>
        Promise.resolve({
          capability: "discover-and-replicate",
          schema_version: 1,
          role,
          byte_count: 42,
        }),
    );

    const verdict = await runAndReconcileDualSessionReplication(
      seedSession,
      clientSession,
      Buffer.from("a".repeat(64), "hex"),
    );

    expect(verdict).toEqual({
      ok: true,
      seed: {
        capability: "discover-and-replicate",
        schema_version: 1,
        role: "seed",
        byte_count: 42,
      },
      client: {
        capability: "discover-and-replicate",
        schema_version: 1,
        role: "client",
        byte_count: 42,
      },
    });

    expect(mockRuntimeClient.shutdown).not.toHaveBeenCalled();
  });

  it("EC-B1: one session's discoverAndReplicate rejection cancels both sessions and reports no success", async () => {
    mockRuntimeClient.discoverAndReplicate.mockImplementation(
      (topic: Buffer, role: "seed" | "client") =>
        role === "client"
          ? Promise.reject(new Error("client discovery timed out"))
          : Promise.resolve({
              capability: "discover-and-replicate",
              schema_version: 1,
              role,
              byte_count: 0,
            }),
    );

    const verdict = await runAndReconcileDualSessionReplication(
      seedSession,
      clientSession,
      Buffer.from("a".repeat(64), "hex"),
    );

    expect(verdict).toEqual({
      ok: false,
      reason: expect.objectContaining({ message: "client discovery timed out" }),
    });

    expect(mockRuntimeClient.shutdown).toHaveBeenCalledTimes(2);
  });

  it("EC-B1: both sessions failing still resolves with the seed-side reason and closes both", async () => {
    mockRuntimeClient.discoverAndReplicate.mockImplementation(
      (topic: Buffer, role: "seed" | "client") =>
        role === "seed"
          ? Promise.reject(new Error("seed failed"))
          : Promise.reject(new Error("client failed")),
    );

    const verdict = await runAndReconcileDualSessionReplication(
      seedSession,
      clientSession,
      Buffer.from("a".repeat(64), "hex"),
    );

    expect(verdict).toEqual({
      ok: false,
      reason: expect.objectContaining({ message: "seed failed" }),
    });

    expect(mockRuntimeClient.shutdown).toHaveBeenCalledTimes(2);
  });
});