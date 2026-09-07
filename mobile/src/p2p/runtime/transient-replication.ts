import { RuntimeProtocolError, type DiscoverAndReplicateReceipt } from "./protocol";
import { loadTransientReplicationDependencies } from "./transient-replication-dependencies";
import { createAndJoinSwarm, awaitFirstConnection, type SwarmConnectionEvents } from "./transient-replication-discovery";
import { attachByteCounter } from "./byte-counter";

export interface ReplicableDrive {
  replicate(isInitiator: boolean): { destroy(): void };
}

export interface DuplexSocket {
  pipe(destination: unknown): unknown;
  on?(event: "data", listener: (chunk: { length: number }) => void): unknown;
}

export function replicateOverSocket(
  drive: ReplicableDrive,
  socket: DuplexSocket,
  isInitiator: boolean,
): { destroy(): void; getByteCount(): number } {
  let replicationStream: { destroy(): void };
  try {
    replicationStream = drive.replicate(isInitiator);
  } catch {
    throw new RuntimeProtocolError(
      "REPLICATION_TRANSFER_FAILED",
      "Replication transfer failed",
    );
  }
  const counter = attachByteCounter(socket);
  (socket.pipe(replicationStream) as { pipe(destination: unknown): unknown }).pipe(socket);
  return { destroy: () => replicationStream.destroy(), getByteCount: counter.getByteCount };
}

export interface CancellableTimeout<T> {
  promise: Promise<T>;
  cancel(): void;
}

export function cancelReplicationOnTimeout(
  replicationStream: { destroy(): void; getByteCount(): number },
  timeoutMs: number,
): CancellableTimeout<never> {
  let timer: ReturnType<typeof setTimeout>;
  const promise = new Promise<never>((_resolve, reject) => {
    timer = setTimeout(() => {
      replicationStream.destroy();
      reject(
        new RuntimeProtocolError(
          "REPLICATION_TRANSFER_FAILED",
          "Replication transfer failed",
        ),
      );
    }, timeoutMs);
  });
  return {
    promise,
    cancel: () => clearTimeout(timer),
  };
}

export async function connectAndReplicate(
  swarm: SwarmConnectionEvents,
  connectTimeoutMs: number,
  drive: ReplicableDrive,
  isInitiator: boolean,
): Promise<{ destroy(): void; getByteCount(): number }> {
  const { socket } = await awaitFirstConnection(swarm, connectTimeoutMs);
  return replicateOverSocket(drive, socket as DuplexSocket, isInitiator);
}

export async function connectReplicateAndCancelOnTimeout(
  swarm: SwarmConnectionEvents,
  connectTimeoutMs: number,
  drive: ReplicableDrive,
  isInitiator: boolean,
  transferTimeoutMs: number,
  finishedSignal: Promise<void>,
): Promise<{ destroy(): void; getByteCount(): number }> {
  const replicationStream = await connectAndReplicate(
    swarm,
    connectTimeoutMs,
    drive,
    isInitiator,
  );
  const { promise: timeoutPromise, cancel } = cancelReplicationOnTimeout(
    replicationStream,
    transferTimeoutMs,
  );
  try {
    await Promise.race([finishedSignal, timeoutPromise]);
  } finally {
    cancel();
  }
  return replicationStream;
}

export async function discoverAndReplicate(
  topic: Buffer,
  role: "seed" | "client",
  drive: ReplicableDrive,
): Promise<DiscoverAndReplicateReceipt> {
  const { Hyperswarm } = loadTransientReplicationDependencies();
  const { swarm } = createAndJoinSwarm(Hyperswarm, topic, role);
  const result = await connectReplicateAndCancelOnTimeout(
    swarm as unknown as Parameters<typeof connectReplicateAndCancelOnTimeout>[0],
    30_000,
    drive,
    role === "client",
    30_000,
    Promise.resolve(),
  );
  return {
    capability: "discover-and-replicate",
    schema_version: 1,
    role,
    byte_count: result.getByteCount(),
  };
}
