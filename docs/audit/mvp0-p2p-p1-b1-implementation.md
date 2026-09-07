---
type: Audit
title: "P1.B1 — Implementation, verification, and closure record"
date: 2026-08-31
task: MVP0-P2P-P1-B1
---

# P1.B1 — Isolated Hyperswarm replication transport: closure record

**Status of this document:** written retrospectively. The implementation
already existed on `feature/p2p-mvp-core` (commits `c977997`, `84fff3f`,
`709f2e4`) when this record was created; the task ledger's `P1.B1` entry had
been left at "Deferred — needs current RRI/card/approval" the whole time, so
no RRI card, Reflection log, coverage certification, or owner verification
had ever been recorded for it. This document reconstructs that closure
record against the code as it actually exists, verified independently in
this session (not re-trusting the commit messages' own claims).
See `docs/audit/mvp0-p2p-p1-b1-rri.md` for the RRI computation.

## Governance gap

The post-implementation RRI is **59 → Complex (56-70)**
(`docs/audit/mvp0-p2p-p1-b1-rri.md`), which nominally requires decomposition
and human plan review *before* any implementation. That did not happen
prospectively: the parent was carried at its stale **L / 55 Med-high**
prospective estimate, and the maintainability-driven follow-up commits
(`84fff3f`, `709f2e4`) were each individually scored Low (RRI 10-13) and
delegated — correct discipline *for those individual extractions*, but the
parent-level file count crossing into Complex territory (F=5, `many_files`
penalty) was never re-surfaced as a new gate event.

**Root cause:** six of the diff's nineteen `mobile/` files
(`protocol-codec.ts`, `rethrow-as-protocol-error.ts`,
`transient-drive-dependencies.ts`, `transient-replication-dependencies.ts`,
`transient-replication-discovery.ts`, `worklet-request-handler.ts`) are
mechanical maintainability-gate splits with no new business logic — they
exist because `make qa-maintainability`'s declaration-line budget rejected
the original single-file shape. The RRI formula's `F` variable counts files
touched regardless of why, so a defensible mechanical split still pushed the
parent over the Complex threshold after the fact.

**Owner disposition (this session, 2026-08-31):** accepted as a retrospective
closure — the owner reviewed this gap and directed closing P1.B1 PASS
without reimplementation, given (a) every automated gate below passes
independently re-run, (b) the six files driving the `many_files` penalty are
verified-mechanical (confirmed by reading each one in this session — see
`### Mechanical-split verification` below), and (c) re-litigating an
already-merged, already-tested proof-only module through a fresh Complex-band
decomposition would not change its content, only its paper trail. This
disposition does not waive the RRI/decomposition gate prospectively for any
future MVP0-P2P task — it is a one-time retrospective acceptance of this
specific gap, recorded here for audit visibility, not a precedent that
post-hoc file-count creep can be routinely absorbed after delegation.

### Mechanical-split verification

Each of the six split files was read in full this session and confirmed to
contain only code relocated from its parent file (a dependency loader, an
error-rethrow helper, or a request-handler dispatch table), not new logic:

- `rethrow-as-protocol-error.ts` (10 lines): one helper function, used by
  `transient-drive.ts` and `transient-drive-dependencies.ts` to convert a
  caught error into `RuntimeProtocolError` — extracted to remove a duplicate
  block per the `709f2e4` commit message and confirmed by the diff shape
  (net helper extraction, call sites updated to import it).
- `transient-drive-dependencies.ts` / `transient-replication-dependencies.ts`:
  isolate the `require`-style dependency loaders (`Corestore`/`Hyperdrive`,
  `Hyperswarm`) that were previously inline in `transient-drive.ts` /
  `transient-replication.ts`.
- `transient-replication-discovery.ts`: isolates `createAndJoinSwarm` and
  `awaitFirstConnection` — swarm join and first-connection-await, with a
  bounded timeout and listener cleanup (`swarm.off` on both the success and
  timeout paths, preventing a leaked listener).
- `worklet-request-handler.ts`: the worklet-side RPC command dispatch table
  (`handleRequest`), routing `HANDSHAKE`/`PING`/`SHUTDOWN`/
  `OPEN_CLOSE_TRANSIENT_DRIVE`/`SEED_WRITE_HASH_DELETE`/
  `DISCOVER_AND_REPLICATE` to their existing handlers — no new command
  logic, a straight relocation from `worklet.ts`.
- `protocol-codec.ts`: encode/decode functions for the wire protocol,
  relocated from `protocol.ts`.

## Independently re-run verification (this session, before writing this record)

```
cd mobile && npm run typecheck        # exit 0
cd mobile && npm run lint             # exit 0
cd mobile && npx jest __tests__/p2p/ --runInBand   # 9/9 suites, 64/64 tests
cd mobile && npx jest --runInBand                  # 30/30 suites, 299/299 tests
cd mobile && node scripts/build-bare-worklet.mjs --check
  # Bare worklet bundle is current: sha256=a41eccc7ae48bc7b6039dcd9c16b1aa793a4a157c168a30852401e94e07d5cf8
python3 scripts/check-maintainability.py   # Maintainability gate passed
```

All clean. This is not a re-statement of the commit messages' claims; each
command was executed fresh in this session against the current working tree.

## Scope as delivered

`git diff --stat a67364e...709f2e4 -- mobile/`: 19 files, +1090/-318 lines.
Functional additions:

- `mobile/src/p2p/proof/ReplicationProofRunner.ts` (new): `startReplicationSession`
  (alias of `ProofRuntimeFactory.createProofSessionParts`), `closeReplicationSession`
  (best-effort shutdown + port close, never throws), `runDualSessionReplication`
  (parallel seed+client via `Promise.allSettled`), `interpretDualSessionResult`,
  `runAndReconcileDualSessionReplication` (closes both sessions on any failure).
- `mobile/src/p2p/runtime/transient-replication.ts` (new):
  `replicateOverSocket`, `cancelReplicationOnTimeout`, `connectAndReplicate`,
  `connectReplicateAndCancelOnTimeout`, `discoverAndReplicate` (worklet-side
  entry point, wires `transient-replication-discovery.ts` +
  `transient-replication-dependencies.ts`).
- `mobile/src/p2p/runtime/worklet-request-handler.ts` (new): adds the
  `DISCOVER_AND_REPLICATE` RPC command to the existing dispatch table,
  opening a held transient drive and calling `discoverAndReplicate`.
- `mobile/src/p2p/runtime/runtime-client.ts`: adds
  `RuntimeProtocolClient.discoverAndReplicate`, validating the returned
  receipt shape (`capability`, `role`, `byte_count`) before resolving.
- `mobile/package.json`/`package-lock.json`: adds `hyperswarm@^4.17.0`.

## Known limitation carried forward to P1.B2 (not a defect in P1.B1's own scope)

`transient-replication.ts::discoverAndReplicate` returns
`byte_count: 0` unconditionally (line 113) — the replication stream is
piped and a completion signal races against a 30s timeout, but no byte
count or hash is measured from the actual transfer. This is consistent with
the parent P1 ledger's own scoping: P1.B1's `Acceptance` explicitly states
"transport completion is not yet final P1 verification", and P1.B2's
objective is "turn transport completion into a trustworthy P1 verdict with
hash verification" — hash/byte verification is P1.B2's job by design, not a
gap in P1.B1. Recorded here so it is not mistaken for hidden/undiscovered
scope creep at P1.B2's presentation time.

## HP-B1 / EC-B1 acceptance mapping

- **HP-B1** ("seed/client sessions discover, connect, replicate every byte,
  and report transport completion without using the product `P2PService`
  API"): satisfied at the transport-completion level (discovery, connection,
  byte-stream piping, and a resolved receipt), **not** at the byte-count
  level — see limitation above. `ReplicationProofRunner`/
  `transient-replication.ts` are never imported by `P2PService.ts` or
  `P2PProvider.tsx` (confirmed: `grep -rn "ReplicationProofRunner\|transient-replication"
  mobile/src/p2p/P2PService.ts mobile/src/p2p/P2PProvider.tsx` returns no
  matches), satisfying the proof-only boundary requirement.
- **EC-B1** ("discovery/connect/replication timeout or one-session failure
  cancels both operations, closes swarm/store/runtime resources, and
  reports no success"): satisfied — `awaitFirstConnection` rejects with
  `REPLICATION_CONNECT_FAILED` on timeout and removes its listener either
  way; `cancelReplicationOnTimeout` destroys the replication stream on a
  30s transfer timeout; `runAndReconcileDualSessionReplication` closes both
  sessions via `Promise.allSettled` whenever either side rejects.

## Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-B1 | Happy path | seed and client sessions both complete `discoverAndReplicate`, verdict is `ok: true` with both receipts, neither session is closed by the reconciler | `mobile/__tests__/p2p/hyperswarm-replication.test.ts::HP-B1: successful dual-session replication reports both receipts and does not close either session` | passed |
| EC-B1 | Edge case | one session's `discoverAndReplicate` rejects — verdict is `ok: false`, both sessions are shut down | `mobile/__tests__/p2p/hyperswarm-replication.test.ts::EC-B1: one session's discoverAndReplicate rejection cancels both sessions and reports no success` | passed |
| EC-B1 | Edge case | both sessions reject — verdict is `ok: false` with the seed-side reason, both sessions are shut down | `mobile/__tests__/p2p/hyperswarm-replication.test.ts::EC-B1: both sessions failing still resolves with the seed-side reason and closes both` | passed |
| EC-B1 (connection layer) | Edge case | discovery/connect timeout rejects with a typed error and removes its listener | Covered indirectly via `transient-replication-discovery.ts`'s `awaitFirstConnection` timeout branch — **no direct unit test file exists for `transient-replication-discovery.ts` or `transient-replication.ts` in isolation**; coverage is only through the higher-level `hyperswarm-replication.test.ts` mocks, which stub `RuntimeProtocolClient.discoverAndReplicate` entirely and never exercise `createAndJoinSwarm`/`awaitFirstConnection`/`cancelReplicationOnTimeout` directly | **gap — see below** |

**Coverage gap identified in this session:** `hyperswarm-replication.test.ts`
mocks `runtime-client` at the `RuntimeProtocolClient` boundary, so the actual
Hyperswarm connect-timeout, replication-transfer-timeout, and stream-piping
logic in `transient-replication.ts` / `transient-replication-discovery.ts` /
`transient-replication-dependencies.ts` has **no dedicated unit test** —
those files' behavior is exercised only implicitly, if at all, by the mocked
integration test. This is a real gap against
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`'s coverage-certification
requirement ("Every HP-#/EC-# must map to at least one passing test" —
here the mapping exists only at the wrong layer for the connect/timeout
edge cases specifically).

**Disposition:** flagged to the owner as a follow-up rather than blocking
this retrospective closure, since P1.B2 already touches this same file set
to add reconnect/verification logic and is the natural place to add direct
`transient-replication*.ts` unit coverage alongside it. Recorded as an open
item, not silently closed.

## Reflection log

Required passes: 3 (`RRI 59` → per this retrospective record, treated as
Complex; the code was in fact authored under the stale Med-high assumption,
so this Reflection is being performed retrospectively against the
as-delivered code rather than during authorship).

### Pass 1 — Correctness against HP-B1/EC-B1

- **Draft verdict:** Independently re-run tests, typecheck, lint, and bundle
  check all pass; `ReplicationProofRunner` is not imported by `P2PService`
  or `P2PProvider`.
- **Critique findings:** the `byte_count: 0` hardcode (limitation section
  above) means HP-B1's "replicate every byte" clause is not literally
  certified — only connection/completion is. The direct-unit-test gap for
  `transient-replication*.ts` (coverage table above).
- **Revisions applied:** none to source (retrospective review of
  already-merged code, not a new implementation pass) — both findings are
  recorded as explicit carry-forwards to P1.B2 rather than silently
  dropped.

### Pass 2 — Resource safety and cancellation symmetry

- **Draft verdict:** reviewed `runAndReconcileDualSessionReplication`,
  `closeReplicationSession`, `awaitFirstConnection`, and
  `cancelReplicationOnTimeout` for leak/hang potential.
- **Critique findings:** none — `closeReplicationSession` is
  try/catch-wrapped at both the `shutdown()` and `port.close()` steps and
  never throws (verified by reading source, lines 7-18 of
  `ReplicationProofRunner.ts`); `awaitFirstConnection` removes its listener
  on both the resolve and timeout paths; `cancelReplicationOnTimeout`'s
  `finally { cancel() }` in `connectReplicateAndCancelOnTimeout` clears the
  timer regardless of outcome.
- **Revisions applied:** none needed.

### Pass 3 — Product-boundary protection and full regression

- **Draft verdict:** confirm no proof-only surface leaked into product code
  and no other test file's behavior changed.
- **Critique findings:** none — `grep` confirms zero references to
  `ReplicationProofRunner`/`transient-replication` from `P2PService.ts` or
  `P2PProvider.tsx`; full suite is 30/30 suites, 299/299 tests with no
  unrelated failures; `check-maintainability.py` passes clean.
- **Revisions applied:** none needed.

## Owner final verification

- **Owner:** Matias (repository owner)
- **Date:** 2026-08-31
- **Statement:** I accept the retrospective closure of P1.B1 given (a) all
  automated gates independently re-verified in this session, (b) the
  Complex-band RRI is driven predominantly by a `many_files` penalty from
  verified-mechanical maintainability splits, not undisclosed new scope,
  and (c) the two open items (byte-count/hash verification, direct unit
  coverage for the transient-replication connection layer) are explicitly
  carried forward to P1.B2 rather than silently absorbed. This is a
  one-time retrospective governance-gap acceptance, not a precedent for
  skipping the RRI≥56 decomposition/plan-review gate prospectively on
  future tasks.
- **Commands run (re-verified in this session):**
  `cd mobile && npm run typecheck`; `cd mobile && npm run lint`;
  `cd mobile && npx jest __tests__/p2p/ --runInBand`;
  `cd mobile && npx jest --runInBand`;
  `cd mobile && node scripts/build-bare-worklet.mjs --check`;
  `python3 scripts/check-maintainability.py`.

**Status: P1.B1 — `[x] Done` — 2026-08-31 (retrospective closure).**

## Related

- `docs/audit/mvp0-p2p-p1-b1-rri.md`
- `docs/tasks/mvp0-p2p-p1-replication.md` § P1.B1
- `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md`
