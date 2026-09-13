// Generic async mutex keyed by an arbitrary string. No P2P-specific
// semantics: callers decide what a "key" means (here, a publication_id).

const activeLocks = new Map<string, Promise<void>>();

/**
 * Runs `fn` exclusively for the given `key`: concurrent calls with the same
 * key are queued and run strictly one at a time, in call order. Calls with
 * different keys run fully in parallel. The lock is released (and the key
 * evicted from the internal map) once `fn` settles, whether it resolves or
 * rejects — a throwing `fn` never leaves the key permanently locked.
 */
export async function withPublicationLock<T>(
  key: string,
  fn: () => Promise<T>
): Promise<T> {
  const previous = activeLocks.get(key) ?? Promise.resolve();

  let releaseNext: () => void;
  const next = new Promise<void>((resolve) => {
    releaseNext = resolve;
  });
  const chained = previous.then(() => next);
  activeLocks.set(key, chained);

  await previous;

  try {
    return await fn();
  } finally {
    releaseNext!();
    if (activeLocks.get(key) === chained) {
      activeLocks.delete(key);
    }
  }
}
