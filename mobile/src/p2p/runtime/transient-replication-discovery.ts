import { RuntimeProtocolError } from "./protocol";

export interface JoinedSwarm {
  swarm: { on: Function; join: Function };
  discovery: unknown;
}

export function createAndJoinSwarm(
  Hyperswarm: new () => { join: Function; on: Function },
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
