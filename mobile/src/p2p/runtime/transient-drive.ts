import { RuntimeProtocolError, TRANSIENT_DRIVE_RECEIPT } from "./protocol";

export interface WorkletRuntime {
  readonly version?: string;
  on(event: "suspend" | "resume", listener: () => void): void;
  on(event: "uncaughtException", listener: (error: unknown) => void): void;
  on(event: "unhandledRejection", listener: (reason: unknown) => void): void;
  argv?: string[];
}

interface TransientDriveStore {
  close(): Promise<void>;
}

interface TransientDrive {
  ready(): Promise<void>;
  close(): Promise<void>;
}

interface TransientDriveDependencies {
  Corestore: new (storage: string) => TransientDriveStore;
  Hyperdrive: new (store: TransientDriveStore) => TransientDrive;
}

export function proofStorageUri(runtime: WorkletRuntime): string {
  const uri = runtime.argv?.[0];
  if (typeof uri !== "string" || !uri.startsWith("file:") || uri.length <= "file:".length) {
    throw new RuntimeProtocolError("PROOF_STORAGE_CONFIG_INVALID", "Proof storage configuration is invalid");
  }
  return uri;
}

let transientDriveDependencies = (): TransientDriveDependencies => {
  let Corestore: unknown;
  let Hyperdrive: unknown;
  try {
    Corestore = require("corestore");
    Hyperdrive = require("hyperdrive");
  } catch {
    throw new RuntimeProtocolError(
      "TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED",
      "Transient drive dependency could not be loaded",
    );
  }
  return validateTransientDriveDependencies({
    Corestore: Corestore as TransientDriveDependencies["Corestore"],
    Hyperdrive: Hyperdrive as TransientDriveDependencies["Hyperdrive"],
  });
};

function validateTransientDriveDependencies(value: unknown): TransientDriveDependencies {
  if (
    value === null ||
    typeof value !== "object" ||
    typeof (value as Partial<TransientDriveDependencies>).Corestore !== "function" ||
    typeof (value as Partial<TransientDriveDependencies>).Hyperdrive !== "function"
  ) {
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_BUNDLE_INVALID", "Transient drive bundle is invalid");
  }
  return value as TransientDriveDependencies;
}

function loadTransientDriveDependencies(): TransientDriveDependencies {
  try {
    return validateTransientDriveDependencies(transientDriveDependencies());
  } catch (error) {
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError(
      "TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED",
      "Transient drive dependency could not be loaded",
    );
  }
}

export function configureTransientDriveDependenciesForTest(
  load: () => unknown,
): () => void {
  const previous = transientDriveDependencies;
  transientDriveDependencies = () => load() as TransientDriveDependencies;
  return () => {
    transientDriveDependencies = previous;
  };
}

export async function openCloseTransientDrive(runtime: WorkletRuntime): Promise<typeof TRANSIENT_DRIVE_RECEIPT> {
  const storageUri = proofStorageUri(runtime);
  let store: TransientDriveStore | undefined;
  let drive: TransientDrive | undefined;
  try {
    const { Corestore, Hyperdrive } = loadTransientDriveDependencies();
    store = new Corestore(storageUri);
    drive = new Hyperdrive(store);
    await drive.ready();
  } catch (error) {
    try {
      if (drive) await drive.close();
      else if (store) await store.close();
    } catch {
      throw new RuntimeProtocolError("TRANSIENT_DRIVE_CLOSE_FAILED", "Transient drive could not be closed");
    }
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_OPEN_FAILED", "Transient drive could not be opened");
  }
  try {
    await drive.close();
  } catch {
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_CLOSE_FAILED", "Transient drive could not be closed");
  }
  return TRANSIENT_DRIVE_RECEIPT;
}
