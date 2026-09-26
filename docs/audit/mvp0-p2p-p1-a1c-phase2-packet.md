---
type: Audit
title: "P1.A1c code-solution review packet"
task: P1.A1c
phase: 2
date: 2026-08-30
---

# P1.A1c — Code-solution review packet

## Acceptance

- A valid transient drive still returns exactly
  `{ capability: "transient-hyperdrive-corestore", schema_version: 1 }` only
  after `ready()` then `close()`.
- Invalid bootstrap, dependency loading, malformed bundled exports, open/ready,
  direct close, and partial-construction cleanup failures each return a
  recognized, redacted error code and never return the receipt.
- The invalid-path test proves its loader is not called and the worklet contains
  no Hyperswarm reference. No dependency, factory, product API, or network
  change is present.

## Implemented change

1. `RuntimeProtocolErrorCode` and its redaction map add:
   `PROOF_STORAGE_CONFIG_INVALID`,
   `TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED`,
   `TRANSIENT_DRIVE_BUNDLE_INVALID`,
   `TRANSIENT_DRIVE_OPEN_FAILED`, and
   `TRANSIENT_DRIVE_CLOSE_FAILED`.
2. The worklet validates the dependency exports, classifies require failures,
   construction/ready failures, and both normal/partial cleanup failures. Raw
   exceptions do not cross the RPC boundary.
3. A test-only dependency-loader seam enables isolated, no-network tests of
   every branch. The worklet bundle was regenerated deterministically.

## Key source excerpt

```ts
function loadTransientDriveDependencies(): TransientDriveDependencies {
  try {
    return validateTransientDriveDependencies(transientDriveDependencies());
  } catch (error) {
    if (error instanceof RuntimeProtocolError) throw error;
    throw new RuntimeProtocolError(
      "TRANSIENT_DRIVE_DEPENDENCY_LOAD_FAILED",
      "Transient drive dependency could not be loaded",
    );
  }
}

try {
  const { Corestore, Hyperdrive } = loadTransientDriveDependencies();
  store = new Corestore(storageUri);
  drive = new Hyperdrive(store);
  await drive.ready();
} catch (error) {
  try {
    if (drive) await drive.close();
    else if (store) await store.close();
  } catch {
    throw new RuntimeProtocolError("TRANSIENT_DRIVE_CLOSE_FAILED", "Transient drive could not be closed");
  }
  if (error instanceof RuntimeProtocolError) throw error;
  throw new RuntimeProtocolError("TRANSIENT_DRIVE_OPEN_FAILED", "Transient drive could not be opened");
}
try {
  await drive.close();
} catch {
  throw new RuntimeProtocolError("TRANSIENT_DRIVE_CLOSE_FAILED", "Transient drive could not be closed");
}
return TRANSIENT_DRIVE_RECEIPT;
```

## Tests and checks already run

- `npm run build:bare-worklet` — passed; committed bundle digest
  `62eaeed431bf61ee5034a7b87c40dd7d55130aa168bcea972c49e1be01324d2a`.
- `npm run typecheck` — passed.
- `npm test -- --runInBand __tests__/p2p/runtime-protocol.test.ts` — passed,
  20 tests, including all P1.A1c branches.
- `npm run lint` and `npm run check:bare-worklet` — passed.

## Review request

Review the implementation against the stated acceptance. Look particularly for
fail-open success, raw-error leakage, incorrect error precedence during
partial cleanup, test-only seam leakage into product behavior, and accidental
network/product scope. Return PASS only if no code change is required.
