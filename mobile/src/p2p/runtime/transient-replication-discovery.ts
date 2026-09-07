import { RuntimeProtocolError } from "./protocol";
import type { ReconnectBudget } from "./reconnect-budget";
import { recordDisconnect } from "./reconnect-budget";

export interface JoinedSwarm {
  swarm: { on: Function; join: Function };
  discovery: unknown;
}

export interface SwarmConnectionEvents {
  on: (
    event: "connection",
    listener: (socket: unknown, peerInfo: unknown) => void,
  ) => void;
  off: (
    event: "connection",
    listener: (socket: unknown, peerInfo: unknown) => void,
  ) => void;
}

export function createAndJoinSwarm(
  Hyperswarm: new () => { join: Function; on: Function },
  topic: Buffer,
  role: "seed" | "client",
): JoinedSwarm {
  const swarm = new Hyperswarm();
  const discovery = (
    swarm as unknown as {
      join: (
        topic: Buffer,
        opts: { server: boolean; client: boolean },
      ) => unknown;
    }
  ).join(topic, { server: role === "seed", client: role === "client" });
  return { swarm, discovery };
}

export function awaitFirstConnection(
  swarm: SwarmConnectionEvents,
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

export interface DisconnectableSocket {
  on: (event: "close" | "error", listener: (err?: unknown) => void) => void;
}

export type DisconnectOutcome = "retry" | "fail";

export function watchForDisconnect(
  socket: DisconnectableSocket,
  budget: ReconnectBudget,
  onDisconnect: (
    outcome: DisconnectOutcome,
    updatedBudget: ReconnectBudget,
  ) => void,
): void {
  let handled = false;

  const handler = () => {
    if (handled) return;
    handled = true;

    const { decision, budget: updatedBudget } = recordDisconnect(budget);
    onDisconnect(decision === "may-retry" ? "retry" : "fail", updatedBudget);
  };

  socket.on("close", handler);
  socket.on("error", handler);
}
