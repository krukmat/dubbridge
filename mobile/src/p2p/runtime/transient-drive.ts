import { RuntimeProtocolError, TRANSIENT_DRIVE_RECEIPT } from "./protocol";
import { loadTransientDriveDependencies } from "./transient-drive-dependencies";

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
  replicate(isInitiator: boolean): { destroy(): void };
  get(path: string): Promise<Uint8Array | null>;
}

export async function openStoreAndDrive<
  Store extends { close(): Promise<void> },
  Drive extends { ready(): Promise<void>; close(): Promise<void> },
>(
  Corestore: new (storage: string) => Store,
  Hyperdrive: new (store: Store) => Drive,
  storageUri: string,
  handles: { store?: Store; drive?: Drive },
): Promise<void> {
  handles.store = new Corestore(storageUri);
  handles.drive = new Hyperdrive(handles.store);
  await handles.drive.ready();
}

export async function closeStoreOrDrive(
  drive: { close(): Promise<void> } | undefined,
  store: { close(): Promise<void> } | undefined,
): Promise<void> {
  if (drive) await drive.close();
  else if (store) await store.close();
}

export function proofStorageUri(runtime: WorkletRuntime): string {
  const uri = runtime.argv?.[0];
  if (typeof uri !== "string" || !uri.startsWith("file:") || uri.length <= "file:".length) {
    throw new RuntimeProtocolError("PROOF_STORAGE_CONFIG_INVALID", "Proof storage configuration is invalid");
  }
  return uri;
}

export async function openCloseTransientDrive(runtime: WorkletRuntime): Promise<typeof TRANSIENT_DRIVE_RECEIPT> {
  const storageUri = proofStorageUri(runtime);
  const handles: { store?: TransientDriveStore; drive?: TransientDrive } = {};
  try {
    const { Corestore, Hyperdrive } = loadTransientDriveDependencies();
    await openStoreAndDrive(Corestore, Hyperdrive, storageUri, handles);
  } catch (error) {
    try {
      await closeStoreOrDrive(handles.drive, handles.store);
    } catch {
      throw new RuntimeProtocolError("TRANSIENT_DRIVE_CLOSE_FAILED", "Transient drive could not be closed");
    }
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_OPEN_FAILED", "Transient drive could not be opened");
  }
  try {
    await handles.drive!.close();
  } catch {
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_CLOSE_FAILED", "Transient drive could not be closed");
  }
  return TRANSIENT_DRIVE_RECEIPT;
}

export async function openHeldTransientDrive(runtime: WorkletRuntime): Promise<TransientDrive> {
  const storageUri = proofStorageUri(runtime);
  const handles: { store?: TransientDriveStore; drive?: TransientDrive } = {};
  try {
    const { Corestore, Hyperdrive } = loadTransientDriveDependencies();
    await openStoreAndDrive(Corestore, Hyperdrive, storageUri, handles);
  } catch (error) {
    try {
      await closeStoreOrDrive(handles.drive, handles.store);
    } catch {
      throw new RuntimeProtocolError("TRANSIENT_DRIVE_CLOSE_FAILED", "Transient drive could not be closed");
    }
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_OPEN_FAILED", "Transient drive could not be opened");
  }
  return handles.drive!;
}

export async function readTransientDriveFile(
  drive: { get(path: string): Promise<Uint8Array | null> },
  path: string,
): Promise<Uint8Array> {
  let bytes: Uint8Array | null;
  try {
    bytes = await drive.get(path);
  } catch {
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_READ_FAILED", "Transient drive read failed");
  }
  if (bytes === null) {
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_READ_FAILED", "Transient drive read failed");
  }
  return bytes;
}

export { configureTransientDriveDependenciesForTest } from "./transient-drive-dependencies";