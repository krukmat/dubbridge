---
type: Audit
title: "MVP0-P2P P2.T3c/T3d orchestration-readiness review"
status: complete
date: 2026-09-09
task: P2.T3c-P2.T3d
---

# P2.T3c/T3d orchestration-readiness review

> **Superseded current-state note (2026-09-12):** T3c orchestration began and
> its mandatory preflight stopped at D2. The missing T2-to-volume package
> materializer and incomplete filesystem handoff now block T3c before RRI and
> presentation. Current evidence:
> `docs/audit/mvp0-p2p-p2-t3c-preflight.md`. The findings below remain the
> dated 2026-09-09 readiness snapshot.

## Result

- `P2.T3c`: **PREPARED FOR ORCHESTRATION**, not frozen, scored, reviewed,
  approved, or executable. The ledger now bounds the analysis questions,
  conservative path envelope, behavior, evidence, verification, exclusions,
  status synchronization, and handoff needed to start orchestration safely.
- `P2.T3d`: **DEFINITION PREPARED; BLOCKED ON T3c**. Its certification-only
  boundary and executable Node ESM paths are explicit, but it must not be
  presented or run before T3c closes.
- No product source, dependency manifest, lockfile, fixture, test, migration,
  commit, push, PR, or external system was changed by this review.

## Baseline inspected

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md`
- `docs/plan/mvp0-p2p-p2-encrypted-publication.md`
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
- `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md`
- `docs/fixtures/mvp0-p2p-publication-contract-v1.json`
- `docs/adr/ADR-044-p2p-audience-delivery-boundary.md`
- `apps/availability-node/package.json`
- `apps/availability-node/package-lock.json`
- `apps/availability-node/tsconfig.json`
- `apps/availability-node/src/contract.ts`
- `apps/availability-node/src/server.ts`
- `apps/availability-node/src/mtls.ts`
- existing `apps/availability-node/test/*.test.js`
- current T2 package-builder and persistence source.

## Findings corrected in the task definition

1. **No direct Hyperdrive stack dependency exists in the Availability Node.**
   `package.json` currently contains only TypeScript/Node type development
   dependencies. T3c's old two-source-file path set could not legally add
   Corestore, Hyperdrive, or Hyperswarm, and relying on mobile/transitive
   dependencies would make the service non-reproducible.
2. **T3c had no writable executable-test path.** A development task cannot close
   its `behavior-v2` HP/EC obligations using only a future task's tests. Focused
   store/idempotency `.test.js` paths are now in its candidate envelope.
3. **T3d's planned `.ts` tests were not runnable by the established package
   pipeline.** `tsconfig.json` includes only `src/**/*.ts`; current package tests
   are Node ESM `.test.js` files importing compiled `dist/`. The definition now
   follows that working convention.
4. **The package materialization boundary is not yet visible in source.** T2's
   current package builder returns manifest bytes/digest and ciphertext files in
   memory, while T3c expects an opaque relative `package_ref` on a shared
   ciphertext volume. The manifest filename/layout and the component that writes
   it must be identified before T3c scope can be frozen. Inventing that layout in
   the Availability Node would silently cross the T2/T3 boundary.
5. **Stable replay evidence needs a durable contract.** C0 requires the same
   `external_publication_id`, `evidence_id`, and `confirmed_at` after replay, but
   the prior task row did not say where or when that mapping becomes durable, how
   concurrent same-ID requests serialize, or how a failed/ambiguous seed avoids a
   false success record.
6. **Persistent roots and lifecycle were unspecified.** The task now requires
   separate injected ciphertext and Availability-Node storage roots, containment
   including symlink handling, long-lived seed ownership, and deterministic
   close semantics without absorbing deployment/listener configuration.

## Readiness rule applied

The task is ready to **begin orchestration** because every unresolved item is now
explicit, bounded, and assigned to the analysis stage. It is not ready for RRI or
implementation until those five mandatory analysis decisions are recorded and
the candidate path envelope is frozen. If package materialization needs a new
Rust/storage path, the orchestrator must create and approve a predecessor rather
than expanding T3c implicitly.

The C0 behavior and authority contract remains unchanged: ciphertext only,
relative `package_ref`, same-identity stable replay, fail-closed conflict,
Availability Node evidence subordinate to PostgreSQL, and no business/key/
database authority. The forward-looking path inventory in the active task ledger
is now the preparation source for T3c/T3d; C0 remains the semantic source.

## Nearby-task inventory

This review prepared the current T3 lane, not the entire remaining P2 backlog.
The other dependency-satisfied rows must not be mistaken for executable task
definitions:

| Task | Dependency state | Readiness | Reason |
|---|---|---|---|
| `P2.T3c` | satisfied | PREPARED FOR ORCHESTRATION | full bounded analysis entry now exists; five preflight decisions remain before RRI |
| `P2.T3d` | blocked on T3c | DEFINITION PREPARED | full certification entry exists; do not activate early |
| `P2.T4b` | T4a satisfied | NO PREPARADA | summary row/group HP-EC only; no task-local acceptance, evidence, status set, or handoff |
| `P2.T5a` | C0 satisfied | NO PREPARADA | summary row/group HP-EC only; characterization proof and exact command are not frozen |
| `P2.T6a` | C0 satisfied | NO PREPARADA | summary row/group HP-EC only; migration-specific acceptance/evidence and explicit approval boundary are not frozen |

Continue the selected T3 sequence first. Preparing T4b/T5a/T6a later is safe
documentation work, but activating them concurrently requires an explicit owner
decision and their own readiness/RRI flow.

## Documentation-task RRI and review disposition

The documentation review itself scored **RRI 25 Low / Effort S** with the current
v2 calculator across five documentation paths (`C=0`, `F=2`, `D=0`, `T=0`,
`A=0`, `K=1`, `P=0`, `X=3`; no penalties). Structure-heavy documentation stays
with the primary agent under the Low-band rule.

Task-analysis review: n/a - docs/plan/task-ledger/audit-only exemption

Code-solution review: n/a - docs/plan/task-ledger/audit-only exemption
