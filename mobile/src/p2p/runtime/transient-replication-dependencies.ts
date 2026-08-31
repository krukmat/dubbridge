import { RuntimeProtocolError } from "./protocol";
import { rethrowAsProtocolError } from "./rethrow-as-protocol-error";

export interface TransientReplicationDependencies {
  Hyperswarm: new () => { join: Function; on: Function };
}

let transientReplicationDependencies = (): TransientReplicationDependencies => {
  let Hyperswarm: unknown;
  try {
    Hyperswarm = require("hyperswarm");
  } catch {
    throw new RuntimeProtocolError(
      "REPLICATION_DISCOVERY_FAILED",
      "Replication peer discovery failed",
    );
  }
  return validateTransientReplicationDependencies({
    Hyperswarm: Hyperswarm as TransientReplicationDependencies["Hyperswarm"],
  });
};

function validateTransientReplicationDependencies(value: unknown): TransientReplicationDependencies {
  if (
    value === null ||
    typeof value !== "object" ||
    typeof (value as Partial<TransientReplicationDependencies>).Hyperswarm !== "function"
  ) {
    throw new RuntimeProtocolError("REPLICATION_DISCOVERY_FAILED", "Replication peer discovery failed");
  }
  return value as TransientReplicationDependencies;
}

export function loadTransientReplicationDependencies(): TransientReplicationDependencies {
  try {
    return validateTransientReplicationDependencies(transientReplicationDependencies());
  } catch (error) {
    rethrowAsProtocolError(error, "REPLICATION_DISCOVERY_FAILED", "Replication peer discovery failed");
  }
}

export function configureTransientReplicationDependenciesForTest(
  load: () => unknown,
): () => void {
  const previous = transientReplicationDependencies;
  transientReplicationDependencies = () => load() as TransientReplicationDependencies;
  return () => {
    transientReplicationDependencies = previous;
  };
}
