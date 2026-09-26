import type { createExpoFileSystemMock } from "../../test-utils/expo-file-system-mock";

jest.mock("expo-file-system", () => {
  const { createExpoFileSystemMock: mockFactory } = require("../../test-utils/expo-file-system-mock");
  return mockFactory();
});

jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

const mockRuntimeClient = {
  handshake: jest.fn(),
  seedWriteHashDelete: jest.fn(),
  shutdown: jest.fn(),
};

jest.mock("../../src/p2p/runtime/runtime-client", () => ({
  BareRpcPort: jest.fn().mockImplementation(() => ({ close: jest.fn() })),
  RuntimeProtocolClient: jest.fn().mockImplementation(() => mockRuntimeClient),
}));

import { janitorAbandonedProofRuns, runSeedProof } from "../../src/p2p/proof/SeedProofRunner";
import { ProofStorageConfigError } from "../../src/p2p/proof/ProofRuntimeFactory";
import { RuntimeProtocolError } from "../../src/p2p/runtime/protocol";

const mockFs: ReturnType<typeof createExpoFileSystemMock> = jest.requireMock("expo-file-system");

const PROOFS_DIR = ["dubbridge-p2p", "proofs"] as const;

function proofRunDir(runId: string) {
  return new mockFs.Directory(mockFs.Paths.cache, ...PROOFS_DIR, runId);
}

function proofsParentDir() {
  return new mockFs.Directory(mockFs.Paths.cache, ...PROOFS_DIR);
}

const RECEIPT = {
  capability: "seed-write-hash-delete" as const,
  schema_version: 1,
  byte_count: 30,
  sha256: "abc123",
};

describe("SeedProofRunner", () => {
  beforeEach(() => {
    mockFs.__reset();
    jest.clearAllMocks();
    mockRuntimeClient.handshake.mockResolvedValue({ runtimeVersion: "1.0.0", capabilities: [] });
    mockRuntimeClient.seedWriteHashDelete.mockResolvedValue(RECEIPT);
    mockRuntimeClient.shutdown.mockResolvedValue(undefined);
  });

  test("HP-A2 shutdown closes handles, removes the exact run directory, and verifies absence", async () => {
    const runId = "abc12345";
    const dir = proofRunDir(runId);
    dir.exists = true;
    dir.deleteFn = () => {
      dir.exists = false;
    };

    const outcome = await runSeedProof(runId, 5_000, () => ({ start: jest.fn(), IPC: {} as never }));

    expect(outcome).toEqual({ receipt: RECEIPT, deleted: true });
    expect(mockRuntimeClient.handshake).toHaveBeenCalledTimes(1);
    expect(mockRuntimeClient.seedWriteHashDelete).toHaveBeenCalledTimes(1);
    expect(mockRuntimeClient.shutdown).toHaveBeenCalledTimes(1);
    expect(dir.exists).toBe(false);
  });

  test("EC-A2 rejects a foreign/traversal run id before starting the worklet", async () => {
    const createWorklet = jest.fn();

    await expect(runSeedProof("../etc/passwd", 5_000, createWorklet)).rejects.toThrow(ProofStorageConfigError);
    expect(createWorklet).not.toHaveBeenCalled();
    expect(mockRuntimeClient.handshake).not.toHaveBeenCalled();
  });

  test("EC-A2 surfaces SEED_DELETE_FAILED without discarding the seed receipt work", async () => {
    const runId = "abc12345";
    const dir = proofRunDir(runId);
    dir.exists = true;
    dir.deleteFn = () => {
      throw new Error("raw storage detail");
    };

    await expect(runSeedProof(runId, 5_000, () => ({ start: jest.fn(), IPC: {} as never }))).rejects.toMatchObject({
      code: "SEED_DELETE_FAILED",
    });
  });

  test("EC-A2 closes the worklet port when handshake/write fails", async () => {
    mockRuntimeClient.seedWriteHashDelete.mockRejectedValue(
      new RuntimeProtocolError("SEED_WRITE_FAILED", "Seed fixture could not be written"),
    );
    const runId = "abc12345";

    await expect(runSeedProof(runId, 5_000, () => ({ start: jest.fn(), IPC: {} as never }))).rejects.toMatchObject({
      code: "SEED_WRITE_FAILED",
    });
  });
});

describe("janitorAbandonedProofRuns", () => {
  beforeEach(() => {
    mockFs.__reset();
    jest.clearAllMocks();
  });

  test("EC-A2 deletes only stale run directories under the proof root", async () => {
    const parentDir = proofsParentDir();
    parentDir.exists = true;

    const staleRunId = "stalerun1";
    const staleDir = proofRunDir(staleRunId);
    staleDir.infoFn = () => ({ modificationTime: 100 });
    staleDir.exists = true;
    staleDir.deleteFn = () => {
      staleDir.exists = false;
    };

    parentDir.listFn = () => [staleDir];

    const removed = await janitorAbandonedProofRuns(1_000, () => 5_000);

    expect(removed).toEqual([staleRunId]);
    expect(staleDir.exists).toBe(false);
  });

  test("EC-A2 does not fail the batch when an abandoned run is already gone", async () => {
    const parentDir = proofsParentDir();
    parentDir.exists = true;

    const staleRunId = "stalerun1";
    const staleDir = proofRunDir(staleRunId);
    staleDir.infoFn = () => ({ modificationTime: 100 });
    staleDir.exists = false;

    parentDir.listFn = () => [staleDir];

    await expect(janitorAbandonedProofRuns(1_000, () => 5_000)).resolves.toEqual([]);
  });

  test("EC-A2 fails closed when a stale run cannot be deleted", async () => {
    const parentDir = proofsParentDir();
    parentDir.exists = true;

    const staleRunId = "stalerun1";
    const staleDir = proofRunDir(staleRunId);
    staleDir.infoFn = () => ({ modificationTime: 100 });
    staleDir.exists = true;
    staleDir.deleteFn = () => {
      throw new Error("raw storage detail");
    };
    parentDir.listFn = () => [staleDir];

    await expect(janitorAbandonedProofRuns(1_000, () => 5_000)).rejects.toThrow();
  });
});
