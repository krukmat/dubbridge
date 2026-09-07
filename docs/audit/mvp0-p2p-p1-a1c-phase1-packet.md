---
type: Audit
title: "P1.A1c task-analysis packet"
task: P1.A1c
phase: 1
date: 2026-08-30
---

# P1.A1c — Task-analysis review packet

## Current RRI and route

`python3 scripts/rri.py --platform rn --cc 8 --touches mobile/src/p2p/runtime/worklet.ts --touches mobile/src/p2p/runtime/protocol.ts --touches mobile/src/p2p/runtime/worklet.bundle.js --D 3 --K 2 --P 1 --T 1 --A 0 --X 1`

Final RRI: **28, Moderate**. The task's historical `23 Low` score cannot
cover the required shared protocol and generated bundle. Existing focused
tests in `mobile/__tests__/p2p/runtime-protocol.test.ts` satisfy the Moderate
pre-presentation check; P1.A1d remains responsible for the new EC-A1 tests
and closure evidence.

## Proposed scope

Only these paths may change:

- `mobile/src/p2p/runtime/worklet.ts` — classify load, bundle, invalid
  bootstrap path, open/ready, and close/cleanup failures before replying.
- `mobile/src/p2p/runtime/protocol.ts` — add only the redacted, versioned
  error-code vocabulary needed for client-side typed decoding.
- `mobile/src/p2p/runtime/worklet.bundle.js` — deterministic generated output
  of the worklet source.

No test, dependency, factory, product service, network, Hyperswarm, storage
location, or Android-native change belongs to this task.

## Acceptance and failure boundary

- Each dependency-load, bundle-resolution, invalid-path, open/ready, or
  close/cleanup failure emits a redacted recognized protocol error code and
  never returns `TRANSIENT_DRIVE_RECEIPT`.
- The valid bootstrap URI remains host-provided only through `Bare.argv[0]`;
  invalid-path proof uses a local-only/stubbed filesystem driver and must not
  execute a network or Hyperswarm code path.
- If a result is attributable to the X28 upstream transport/worklet execution
  defect, record `Environment/Blocked`; it is not evidence of a source-test
  failure.
- Preserve normal behavior: successful `drive.ready()` followed by
  `drive.close()` returns the existing two-field receipt, and partial
  construction closes only the Corestore when no drive exists.

## Constraints from P1.A1b.0 / ADR-043

The receipt remains exactly `{ capability: "transient-hyperdrive-corestore",
schema_version: 1 }`; it never contains a URI, directory, key, raw exception,
or network detail. No direct `bare-fs` dependency may be added. The worklet
must not use `P2PService`, discovery, replication, persistence, or product API.

## Review request

Review this task boundary for omissions, protocol compatibility, fail-closed
behavior, and accidental network/product scope. Return PASS only if the task
is implementable as stated; otherwise identify concrete blocking findings.
