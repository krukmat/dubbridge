import { RUNTIME_COMMAND } from "../../src/p2p/runtime/protocol";
import { configureSeedDependenciesForTest } from "../../src/p2p/runtime/worklet";
import { workletHarness } from "../../test-utils/worklet-harness";

jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

async function requestSeedWrite(
  harness: ReturnType<typeof workletHarness>,
): Promise<void> {
  harness.request(RUNTIME_COMMAND.SEED_WRITE_HASH_DELETE);
  for (let tick = 0; tick < 8; tick += 1) {
    await Promise.resolve();
  }
}

type SeedDependencyOverrides = {
  put?: () => Promise<void>;
  close?: () => Promise<void>;
  hashUpdate?: () => unknown;
};

function makeSeedDependencies(overrides: SeedDependencyOverrides = {}) {
  const close = overrides.close ?? (async () => {});
  const put = overrides.put ?? (async () => {});
  const hashUpdate = overrides.hashUpdate ?? function (this: unknown) { return this; };
  return {
    Corestore: class {
      close = close;
    },
    Hyperdrive: class {
      async ready() {}
      put = put;
      close = close;
    },
    createHash: () => ({
      update: hashUpdate,
      digest() {
        return "abc123";
      },
    }),
  };
}

describe("P1.A2 seed write/hash/close", () => {
  it("HP-A2 preserves the seed receipt after write, hash, and close", async () => {
    const restore = configureSeedDependenciesForTest(() => makeSeedDependencies());
    const harness = workletHarness(["file:///cache/dubbridge-p2p/proofs/proofrun1/"]);

    await requestSeedWrite(harness);

    expect(harness.replies).toEqual([
      expect.objectContaining({
        ok: true,
        result: {
          capability: "seed-write-hash-delete",
          schema_version: 1,
          byte_count: expect.any(Number),
          sha256: "abc123",
        },
      }),
    ]);
    restore();
  });

  it.each([
    [
      "dependency load",
      () => {
        throw new Error("raw-fixture-detail");
      },
      "SEED_WRITE_FAILED",
    ],
    [
      "put",
      () =>
        makeSeedDependencies({
          put: async () => {
            throw new Error("raw-fixture-detail");
          },
        }),
      "SEED_WRITE_FAILED",
    ],
    [
      "hash",
      () =>
        makeSeedDependencies({
          hashUpdate: () => {
            throw new Error("raw-fixture-detail");
          },
        }),
      "SEED_HASH_FAILED",
    ],
  ] as const)("EC-A2 returns a redacted typed error for %s failure", async (_caseName, load, code) => {
    const restore = configureSeedDependenciesForTest(load);
    const harness = workletHarness(["file:///cache/dubbridge-p2p/proofs/proofrun1/"]);

    await requestSeedWrite(harness);

    expect(harness.replies).toEqual([
      expect.objectContaining({
        ok: false,
        error: { code, message: expect.any(String) },
      }),
    ]);
    expect(JSON.stringify(harness.replies)).not.toContain("raw-fixture-detail");
    restore();
  });

  it("EC-A2 returns a close error without leaking raw details", async () => {
    const restore = configureSeedDependenciesForTest(() =>
      makeSeedDependencies({
        close: async () => {
          throw new Error("raw-fixture-detail");
        },
      }),
    );
    const harness = workletHarness(["file:///cache/dubbridge-p2p/proofs/proofrun1/"]);

    await requestSeedWrite(harness);

    expect(harness.replies).toEqual([
      expect.objectContaining({
        ok: false,
        error: { code: "SEED_CLOSE_FAILED", message: expect.any(String) },
      }),
    ]);
    expect(JSON.stringify(harness.replies)).not.toContain("raw-fixture-detail");
    restore();
  });
});
