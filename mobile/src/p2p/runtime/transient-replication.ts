import { RuntimeProtocolError, type DiscoverAndReplicateReceipt } from "./protocol";

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
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError(
      "REPLICATION_DISCOVERY_FAILED",
      "Replication peer discovery failed",
    );
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

export interface JoinedSwarm {
  swarm: { on: Function; join: Function };
  discovery: unknown;
}

export function createAndJoinSwarm(
  Hyperswarm: TransientReplicationDependencies["Hyperswarm"],
  topic: Buffer,
  role: "seed" | "client",
): JoinedSwarm {
  const swarm = new Hyperswarm();
  const discovery = (swarm as unknown as { join: (topic: Buffer, opts: { server: boolean; client: boolean }) => unknown }).join(
    topic,
      { server: role === "seed", client: role === "client" },
    );
  return { swarm, discovery };
}

export function awaitFirstConnection(
  swarm: {
    on: (event: "connection", listener: (socket: unknown, peerInfo: unknown) => void) => void;
    off: (event: "connection", listener: (socket: unknown, peerInfo: unknown) => void) => void;
    },
  timeoutMs: number,
): Promise<{ socket: unknown; peerInfo: unknown }> {
  return new Promise((resolve, reject) => {
    let settled = false;
    const onConnection = (socket: unknown, peerInfo: unknown) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      swarm.off("connection", onConnection);
      resolve({ socket, peerInfo });
      };
    const timer = setTimeout(() => {
      if (settled) return;
      settled = true;
      swarm.off("connection", onConnection);
      reject(
        new RuntimeProtocolError(
            "REPLICATION_CONNECT_FAILED",
            "Replication peer connection failed",
          ),
        );
      }, timeoutMs);
    swarm.on("connection", onConnection);
    });
}

export interface ReplicableDrive {
  replicate(isInitiator: boolean): { destroy(): void };
}

export interface DuplexSocket {
  pipe(destination: unknown): unknown;
}

export function replicateOverSocket(
  drive: ReplicableDrive,
  socket: DuplexSocket,
  isInitiator: boolean,
): { destroy(): void } {
  let replicationStream: { destroy(): void };
  try {
    replicationStream = drive.replicate(isInitiator);
    } catch {
    throw new RuntimeProtocolError(
        "REPLICATION_TRANSFER_FAILED",
        "Replication transfer failed",
      );
    }
    (socket.pipe(replicationStream) as { pipe(destination: unknown): unknown }).pipe(socket);
  return replicationStream;
}

export interface CancellableTimeout<T> {
  promise: Promise<T>;
  cancel(): void;
}

export function cancelReplicationOnTimeout(
  replicationStream: { destroy(): void },
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
  swarm: {
    on: (event: "connection", listener: (socket: unknown, peerInfo: unknown) => void) => void;
    off: (event: "connection", listener: (socket: unknown, peerInfo: unknown) => void) => void;
    },
  connectTimeoutMs: number,
  drive: ReplicableDrive,
  isInitiator: boolean,
): Promise<{ destroy(): void }> {
  const { socket } = await awaitFirstConnection(swarm, connectTimeoutMs);
  return replicateOverSocket(drive, socket as DuplexSocket, isInitiator);
}

export async function connectReplicateAndCancelOnTimeout(
  swarm: {
    on: (event: "connection", listener: (socket: unknown, peerInfo: unknown) => void) => void;
    off: (event: "connection", listener: (socket: unknown, peerInfo: unknown) => void) => void;
    },
  connectTimeoutMs: number,
  drive: ReplicableDrive,
  isInitiator: boolean,
  transferTimeoutMs: number,
  finishedSignal: Promise<void>,
): Promise<{ destroy(): void }> {
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
  await connectReplicateAndCancelOnTimeout(
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
    byte_count: 0,
   };
}