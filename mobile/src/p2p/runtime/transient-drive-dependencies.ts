import { RuntimeProtocolError } from "./protocol";
import { rethrowAsProtocolError } from "./rethrow-as-protocol-error";

interface MinimalCloseable {
  close(): Promise<void>;
}

interface MinimalDrive extends MinimalCloseable {
  ready(): Promise<void>;
}

export interface TransientDriveDependencies {
  Corestore: new (storage: string) => MinimalCloseable;
  Hyperdrive: new (store: MinimalCloseable) => MinimalDrive;
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

export function loadTransientDriveDependencies(): TransientDriveDependencies {
  try {
    return validateTransientDriveDependencies(transientDriveDependencies());
  } catch (error) {
    rethrowAsProtocolError(
      error,
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
