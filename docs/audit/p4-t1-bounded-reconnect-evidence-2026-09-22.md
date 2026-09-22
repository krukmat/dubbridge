---
type: Audit
title: "P4.T1 bounded reconnect implementation evidence"
date: 2026-09-22
task: P4.T1
status: implementation_verified_review_pending
---

# P4.T1 bounded reconnect — implementation evidence

## Scope

P4.T1 closes the previously named product-path gap where
`mobile/src/p2p/sync/P2pProductSync.ts` could persist a restartable state after
transport failure but did not perform a bounded automatic reconnect.

Execution RRI is recorded in:

- `docs/audit/p4-t1-bounded-reconnect-rri-2026-09-22.md`
- Final RRI: **55 / Med-high**

Primary implementation commit:

- `de199ff9f7cd8b9a1f207e5036b4cb6cb714d0e4` —
  `feat(p4): add bounded product reconnect and cache resume`

Follow-up coverage commits relevant to the accepted P4.T1 behavior:

- `45f133a1e63ff841618fbd3306db1943dc121b92` —
  reject invalid cached ciphertext on resume
- `0c3d484648565755348011b85a6a84acca41fb0c` —
  cover sign-out during reconnect

## Implemented behavior

The product sync path now:

1. creates an existing `ReconnectBudget` for the foreground sync run;
2. defaults to one automatic reconnect;
3. retries only failures classified at the P2P source transport boundary
   (`open`, manifest read, ciphertext read);
4. closes the failed source session before reopening;
5. resumes through the existing persistent cache so valid ciphertext is reused;
6. re-fetches cached ciphertext that does not match the manifest size/digest;
7. never retries `PackageVerificationError` as a peer outage;
8. lets cancellation/sign-out dominate reconnect and prevents READY promotion;
9. preserves the invariant that READY follows complete package verification.

No RPC version, DB schema, crypto/K1 contract, P2/P3 authorization contract,
proof topology, progressive streaming path, or remote media fallback changed.

## Acceptance evidence

### HP-P4.T1-1 — interrupted replication resumes without restarting valid work

Executable evidence in `mobile/__tests__/p2p.product-sync.test.ts`:

- transient source-open failure reconnects once and reaches READY;
- an interrupted package read reconnects and reuses the already verified first
  ciphertext file instead of downloading it again;
- invalid cached ciphertext is explicitly rejected and fetched again;
- each opened source session is closed.

Result on GitHub Actions run `35720515725`, HEAD
`0c3d484648565755348011b85a6a84acca41fb0c`:

- mobile job: **PASS**
- `__tests__/p2p.product-sync.test.ts`: **PASS**
- total mobile suites: **60/60 PASS**
- total mobile tests: **434/434 PASS**

### EC-P4.T1-1 — unavailable peers/cancellation terminate bounded work without READY

Executable evidence:

- reconnect budget exhaustion performs exactly two total opens with
  `maxReconnectRetries=1`, then throws and remains non-READY;
- digest/integrity verification failure performs only one source open and
  transitions to FAILED rather than consuming reconnect budget;
- cancellation during the second reconnect attempt stops further opens and
  persists CANCELLED;
- sign-out via `clearAccount` during the second reconnect attempt cancels the
  active source, closes it, wipes the account snapshot/cache and never promotes
  READY;
- account cache isolation remains covered by the existing product-sync suite.

## CI interpretation

At the exact evidence SHA, the mobile gate is green.

The overall workflow may still report red because of Rust-only `fmt` /
`clippy` failures in `crates/db/tests/p2p_audience_repo.rs`, a separate
concurrent coverage-remediation change. Those failures do not execute or compile
the mobile P4.T1 path and are not P4.T1 behavioral failures.

The previously diagnosed CI blockers are already green on the same run:

- `deny`: PASS
- `s3-integration`: PASS
- `coverage`: PASS

## Review/closure gate

Implementation verification is complete.

Required Med-high band independent code-solution review is **not yet recorded**
for P4.T1. Therefore this artifact does not claim formal `Done` status.

Current state:

`P4.T1 IMPLEMENTED + VERIFIED / CODE-SOLUTION REVIEW PENDING`

Task-analysis review was not recorded before the concurrent implementation;
owner execution approval was explicit in the active session. This process
deviation is named rather than retroactively fabricated.

## Downstream consequence

P4.T3 can use this executable evidence for interruption/resume and bounded-peer
behavior, but aggregate P4 closure still requires P4.T3 certification evidence
and the normal closure gates.
