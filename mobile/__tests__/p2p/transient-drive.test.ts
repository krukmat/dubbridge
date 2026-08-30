import { readFileSync } from "node:fs";
import path from "node:path";

import { RUNTIME_COMMAND, RUNTIME_PROTOCOL_VERSION } from "../../src/p2p/runtime/protocol";
import { configureTransientDriveDependenciesForTest } from "../../src/p2p/runtime/worklet";
import { workletHarness } from "../../test-utils/worklet-harness";
let { ProofStorageConfigError, proofStorageUri, startProofWorklet } =
  require("../../src/p2p/proof/P1ProofRuntimeFactory");

jest.mock("expo-file-system", () => ({
  Paths: { cache: "file:///cache" },
  Directory: class Directory {
    uri: string;
    constructor(...parts: string[]) { this.uri = `${parts.join("/")}/`; }
  },
}));

jest.mock("react-native-bare-kit", () => ({ Worklet: class Worklet {} }));

let mobileRoot = path.resolve(__dirname, "../..");

async function requestTransientDrive(
  harness: ReturnType<typeof workletHarness>,
): Promise<void> {
  harness.request(RUNTIME_COMMAND.OPEN_CLOSE_TRANSIENT_DRIVE);
  await Promise.resolve();
  await Promise.resolve();
}

describe("P1.A1 transient drive lifecycle", () => {
  it("HP-A1 passes only the host-derived proof URI as the worklet argument", () => {
    let start = jest.fn();
    const worklet = startProofWorklet("proofrun1", () => ({ start }));

    expect(worklet).toEqual({ start });
    expect(proofStorageUri("proofrun1")).toBe("file:///cache/dubbridge-p2p/proofs/proofrun1/");
    expect(start).toHaveBeenCalledWith(
      "/dubbridge-p2p-proof.worklet",
      expect.any(String),
      ["file:///cache/dubbridge-p2p/proofs/proofrun1/"],
    );
  });

  it("EC-A1b rejects invalid proof configuration before storage is required", async () => {
    expect(() => proofStorageUri("UPPERCASE")).toThrow(ProofStorageConfigError);
    const load = jest.fn();
    const restore = configureTransientDriveDependenciesForTest(load);
    const harness = workletHarness();
    harness.request(RUNTIME_COMMAND.OPEN_CLOSE_TRANSIENT_DRIVE);
    await Promise.resolve();
    await Promise.resolve();
    expect(harness.replies).toEqual([expect.objectContaining({
      ok: false,
      error: expect.objectContaining({ code: "PROOF_STORAGE_CONFIG_INVALID" }),
    })]);
    expect(load).not.toHaveBeenCalled();
    const workletSource = readFileSync(
      path.join(mobileRoot, "src/p2p/runtime/worklet.ts"),
      "utf8",
    ).toLowerCase();
    expect(workletSource).not.toContain("hyperswarm");
    restore();
  });

  it.each([
    [
      "dependency load",
      () => { throw new Error("missing native module"); },
      "TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED",
    ],
    [
      "bundle validation",
      () => ({ Corestore: null, Hyperdrive: null }),
      "TRANSIENT_DRIVE_BUNDLE_INVALID",
    ],
    [
      "open",
      () => ({
        Corestore: class { async close() {} },
        Hyperdrive: class { async ready() { throw new Error("raw storage path"); } async close() {} },
      }),
      "TRANSIENT_DRIVE_OPEN_FAILED",
    ],
    [
      "partial close",
      () => ({
        Corestore: class { async close() { throw new Error("raw storage path"); } },
        Hyperdrive: class { constructor() { throw new Error("open failed"); } },
      }),
      "TRANSIENT_DRIVE_CLOSE_FAILED",
    ],
  ] as const)("EC-A1 returns a redacted typed error for %s failure", async (_caseName, load, code) => {
    const restore = configureTransientDriveDependenciesForTest(load);
    const harness = workletHarness(["file:///cache/dubbridge-p2p/proofs/proofrun1/"]);

    await requestTransientDrive(harness);

    expect(harness.replies).toEqual([expect.objectContaining({
      ok: false,
      error: { code, message: expect.any(String) },
    })]);
    expect(harness.replies[0].error).toEqual({ code, message: expect.any(String) });
    expect(JSON.stringify(harness.replies)).not.toContain("raw storage path");
    restore();
  });

  it("EC-A1 returns a close error without directly closing a drive-owned Corestore", async () => {
    const storeClose = jest.fn();
    const restore = configureTransientDriveDependenciesForTest(() => ({
      Corestore: class { async close() { storeClose(); } },
      Hyperdrive: class { async ready() {} async close() { throw new Error("raw storage path"); } },
    }));
    const harness = workletHarness(["file:///cache/dubbridge-p2p/proofs/proofrun1/"]);

    await requestTransientDrive(harness);

    expect(harness.replies).toEqual([expect.objectContaining({
      ok: false,
      error: { code: "TRANSIENT_DRIVE_CLOSE_FAILED", message: expect.any(String) },
    })]);
    expect(storeClose).not.toHaveBeenCalled();
    expect(JSON.stringify(harness.replies)).not.toContain("raw storage path");
    restore();
  });

  it("HP-A1 preserves the two-field receipt after ready then close", async () => {
    const restore = configureTransientDriveDependenciesForTest(() => ({
      Corestore: class { async close() {} },
      Hyperdrive: class { async ready() {} async close() {} },
    }));
    const harness = workletHarness(["file:///cache/dubbridge-p2p/proofs/proofrun1/"]);

    await requestTransientDrive(harness);

    expect(harness.replies).toEqual([expect.objectContaining({
      ok: true,
      result: { capability: "transient-hyperdrive-corestore", schema_version: 1 },
    })]);
    restore();
  });
});
