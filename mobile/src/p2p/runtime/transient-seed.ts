import { RuntimeProtocolError, type SeedWriteHashDeleteReceipt } from "./protocol";
import { closeStoreOrDrive, openStoreAndDrive, proofStorageUri, type WorkletRuntime } from "./transient-drive";

const SEED_FIXTURE_PATH = "/dubbridge-p2p-seed.bin";
const SEED_FIXTURE_CONTENT = "dubbridge-p2p-seed-fixture-v1";

interface SeedDrive {
  ready(): Promise<void>;
  put(path: string, content: Uint8Array): Promise<void>;
  close(): Promise<void>;
}

interface SeedStore {
  close(): Promise<void>;
}

interface SeedHash {
  update(data: Uint8Array): SeedHash;
  digest(encoding: "hex"): string;
}

interface SeedDependencies {
  Corestore: new (storage: string) => SeedStore;
  Hyperdrive: new (store: SeedStore) => SeedDrive;
  createHash: (algorithm: "sha256") => SeedHash;
}

let seedDependencies = (): SeedDependencies => {
  let Corestore: unknown;
  let Hyperdrive: unknown;
  let bareCrypto: unknown;
  try {
    Corestore = require("corestore");
    Hyperdrive = require("hyperdrive");
    bareCrypto = require("bare-crypto");
  } catch {
    throw new RuntimeProtocolError("SEED_WRITE_FAILED", "Seed fixture could not be written");
  }
  return validateSeedDependencies({
    Corestore: Corestore as SeedDependencies["Corestore"],
    Hyperdrive: Hyperdrive as SeedDependencies["Hyperdrive"],
    createHash: (bareCrypto as { createHash: SeedDependencies["createHash"] }).createHash,
  });
};

function validateSeedDependencies(value: unknown): SeedDependencies {
  const candidate = value as Partial<SeedDependencies> | null;
  if (
    candidate === null ||
    typeof candidate !== "object" ||
    typeof candidate.Corestore !== "function" ||
    typeof candidate.Hyperdrive !== "function" ||
    typeof candidate.createHash !== "function"
  ) {
    throw new RuntimeProtocolError("SEED_WRITE_FAILED", "Seed fixture could not be written");
  }
  return candidate as SeedDependencies;
}

export function configureSeedDependenciesForTest(load: () => unknown): () => void {
  const previous = seedDependencies;
  seedDependencies = () => load() as SeedDependencies;
  return () => {
    seedDependencies = previous;
  };
}

function digestFixture(createHash: SeedDependencies["createHash"], content: Uint8Array): string {
  try {
    return createHash("sha256").update(content).digest("hex");
  } catch {
    throw new RuntimeProtocolError("SEED_HASH_FAILED", "Seed fixture could not be hashed");
  }
}

async function closeSeedHandles(drive: SeedDrive | undefined, store: SeedStore | undefined): Promise<void> {
  try {
    await closeStoreOrDrive(drive, store);
  } catch {
    throw new RuntimeProtocolError("SEED_CLOSE_FAILED", "Seed drive could not be closed");
  }
}

export async function writeHashSeed(runtime: WorkletRuntime): Promise<SeedWriteHashDeleteReceipt> {
  const storageUri = proofStorageUri(runtime);
  const fixture = new TextEncoder().encode(SEED_FIXTURE_CONTENT);

  const handles: { store?: SeedStore; drive?: SeedDrive } = {};
  let digest: string;
  try {
    const { Corestore, Hyperdrive, createHash } = seedDependencies();
    await openStoreAndDrive(Corestore, Hyperdrive, storageUri, handles);
    try {
      await handles.drive!.put(SEED_FIXTURE_PATH, fixture);
    } catch {
      throw new RuntimeProtocolError("SEED_WRITE_FAILED", "Seed fixture could not be written");
    }
    digest = digestFixture(createHash, fixture);
  } catch (error) {
    try {
      await closeSeedHandles(handles.drive, handles.store);
    } catch {
      throw new RuntimeProtocolError("SEED_CLOSE_FAILED", "Seed drive could not be closed");
    }
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError("SEED_WRITE_FAILED", "Seed fixture could not be written");
  }

  await closeSeedHandles(handles.drive, handles.store);

  return {
    capability: "seed-write-hash-delete",
    schema_version: 1,
    byte_count: fixture.byteLength,
    sha256: digest,
  };
}
