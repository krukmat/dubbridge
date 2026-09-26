---
type: Audit
title: "P1.A1c remedial code-solution review packet"
task: P1.A1c
phase: 2
date: 2026-08-30
---

# P1.A1c — Remedial Phase-2 code-solution review packet

Review the P1.A1c implementation after the initial advisory findings and the
targeted regression test described in
`docs/audit/mvp0-p2p-p1-a1c-phase2-remediation.md`.

## Frozen lifecycle ownership

`docs/audit/mvp0-p2p-p1-a1b-storage-contract.md` is authoritative:

- after `Hyperdrive` exists, it owns the `Corestore`; call only
  `drive.close()`;
- after partial construction with no drive, call `store.close()`;
- a close failure returns the single redacted terminal code
  `TRANSIENT_DRIVE_CLOSE_FAILED`, never a receipt or raw cause.

## Acceptance and coverage

- Valid `ready()` then `close()` returns exactly the two-field receipt.
- Invalid bootstrap, dependency load, malformed exports, open/ready, direct
  close, and partial-construction cleanup failures each return a recognized
  redacted protocol error and no receipt.
- The focused suite has 20 passing tests. The added test invokes a failing
  `drive.close()`, asserts `TRANSIENT_DRIVE_CLOSE_FAILED`, checks that a
  drive-owned store is not directly closed, and checks that its raw error is
  absent from the reply.
- Build, typecheck, lint, and worklet-bundle checks pass; no product API,
  dependency, or network code changes are in scope.

## Review request

Return PASS if the implementation satisfies this lifecycle contract, is
fail-closed and redacted, and needs no code change. Treat direct `store.close`
after a drive exists as a contract violation, not a remediation.
