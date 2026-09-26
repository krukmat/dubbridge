import type { DuplexSocket } from "./transient-replication";

export interface ByteCounter {
  getByteCount(): number;
}

export function attachByteCounter(socket: DuplexSocket): ByteCounter {
  let total = 0;
  if (typeof socket.on === "function") {
    socket.on("data", (chunk) => {
      total += chunk.length;
    });
  }
  return { getByteCount: () => total };
}
