import {
  createProofSessionParts,
  type ProofSessionParts,
} from "./ProofRuntimeFactory";

export type ReplicationSession = ProofSessionParts;

export const startReplicationSession = createProofSessionParts;

export async function closeReplicationSession(
  session: ReplicationSession,
): Promise<void> {
  try {
    await session.client.shutdown();
  } catch {
    // best-effort: fall through to closing the port regardless
  }
  try {
    session.port.close(new Error("Replication session closed"));
  } catch {
    // best-effort: this helper must never throw
  }
}

async function runOneSessionReplication(
  session: ReplicationSession,
  topic: Buffer,
  role: "seed" | "client",
) {
  await session.client.handshake();
  return session.client.discoverAndReplicate(topic, role);
}

export async function runDualSessionReplication(
  seedSession: ReplicationSession,
  clientSession: ReplicationSession,
  topic: Buffer,
): Promise<
  [
    PromiseSettledResult<Awaited<ReturnType<typeof runOneSessionReplication>>>,
    PromiseSettledResult<Awaited<ReturnType<typeof runOneSessionReplication>>>,
  ]
> {
  return Promise.allSettled([
    runOneSessionReplication(seedSession, topic, "seed"),
    runOneSessionReplication(clientSession, topic, "client"),
  ]);
}

export type DualSessionReplicationResult = Awaited<
  ReturnType<typeof runDualSessionReplication>
>;

export type ReplicationVerdict =
  | {
      ok: true;
      seed: Awaited<ReturnType<typeof runOneSessionReplication>>;
      client: Awaited<ReturnType<typeof runOneSessionReplication>>;
    }
  | { ok: false; reason: unknown };

export function interpretDualSessionResult(
  results: DualSessionReplicationResult,
): ReplicationVerdict {
  const [seedResult, clientResult] = results;
  if (
    seedResult.status === "fulfilled" &&
    clientResult.status === "fulfilled"
  ) {
    return { ok: true, seed: seedResult.value, client: clientResult.value };
  }
  const reason =
    seedResult.status === "rejected"
      ? seedResult.reason
      : (clientResult as PromiseRejectedResult).reason;
  return { ok: false, reason };
}

export async function runAndReconcileDualSessionReplication(
  seedSession: ReplicationSession,
  clientSession: ReplicationSession,
  topic: Buffer,
): Promise<ReplicationVerdict> {
  const results = await runDualSessionReplication(
    seedSession,
    clientSession,
    topic,
  );
  const verdict = interpretDualSessionResult(results);
  if (!verdict.ok) {
    await Promise.allSettled([
      closeReplicationSession(seedSession),
      closeReplicationSession(clientSession),
    ]);
  }
  return verdict;
}
