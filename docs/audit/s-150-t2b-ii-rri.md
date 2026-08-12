---
type: Audit
title: "RRI evidence: S-150-T2b-ii durable translation delivery repository"
status: recorded
task: S-150-T2b-ii
date: 2026-08-12
---

# S-150-T2b-ii — Presentation-time RRI

## Scoped presentation surface

- `crates/db/src/translation_repo.rs`
- `crates/db/src/target_language_repo.rs`
- `crates/db/src/lib.rs`
- `apps/api/tests/localization_repo_test.rs`

The task is limited to the durable translation-dispatch repository transaction,
exact project/asset/target binding, and its live-PostgreSQL tests. It excludes
job structs, Redis enqueue, provider calls, review-task creation, and all TTS
behavior.

## Evidence used for the score

- `localization_repo_test` has 13 passing integration tests for the adjacent
  claim/current-pointer behavior. The new outbox lifecycle and guarded
  `enqueue_failed` update have no dedicated coverage yet, so `T=2` (partial).
- The new transactional API is expected to contain a limited set of branch
  points (membership/no-target rejection, create-or-reuse, conflict, and
  dispatch-state result): raw CC 9, therefore `C=1`.
- Four production/test files are expected, so `F=2`.
- `crates/db` gives floors `D=3`, `P=3`, and `K=3`. This task raises `K` to 4
  because it atomically coordinates claim/status/outbox state, and raises `P`
  to 4 because it enforces the persisted project--asset--target ownership
  boundary fail-closed. That ownership condition requires the manual
  `auth_security` penalty.

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 1 | raw CC 9 -> score 1 | Medium |
| F files | 2 | four scoped files | High |
| D domain | 3 | `crates/db` anchor floor | High |
| T coverage | 2 | adjacent real-DB suite exists; outbox API is new | High |
| A ambiguity | 1 | defined transaction, HP/EC cases, and exclusions | High |
| K coupling | 4 | one transaction spans claims, status, targets, and outbox | High |
| P impact | 4 | persisted project--asset--target ownership boundary | High |
| X context | 3 | repository, target lookup, schema, and integration tests | High |

**Base value:** 47

**Penalties applied:** `auth_security` (+10) — the task's explicit
asset/project/target ownership validation is a fail-closed persisted-data
boundary.

**Final RRI:** **57** -> **Complex (56–70)** -> Effort **L**.

**Gates:** mandatory decomposition before implementation; cross-vendor phase-1
and phase-2 review; explicit human approval for each resulting implementation
task; four Reflection passes per executable child; unit coverage certification
and owner verification.

**Decomposition:** triggered and approved by Matias on 2026-08-12.
`S-150-T2b-ii` must not be implemented as its current parent card. Its child tasks
(`S-150-T2b-ii-a`, `S-150-T2b-ii-b`, and `S-150-T2b-ii-c`) must each receive a
fresh RRI and approval card before code changes begin.

## Phase-1 task-analysis review

`Task-analysis review: d14 .agent/peer-task-review-S-150-T2b-ii.json - PASS`

The initially resolved cross-vendor Claude peer could not run: the repository
wrapper still invokes the retired `claude review --stdin` form, and a compatible
direct invocation then reported the provider's weekly quota exhausted. The
packet-bound ADR-039 fallback receipt at
`.agent/peer-task-review-S-150-T2b-ii.fallback-selection.json` records the
human-selected, same-provider-degraded D14 route: `gpt-5.6-sol` / `high`,
selected by `Matias`.

D14's first review found a TOCTOU risk in validating scope outside the writer
transaction. The revised decomposition assigns only reusable read helpers to
T2b-ii-a and makes T2b-ii-b execute them inside its own transaction before any
claim/outbox write. It also assigns each HP/EC behavior to a child, adds a
full-identity/state predicate to the failure transition, and requires a composed
real-PostgreSQL acceptance run. D14 re-reviewed that revision and passed it.

## Documentary closure

Matias approved the decomposition on 2026-08-12. The approved work recorded here
is task-ledger/plan/roadmap synchronization only; it starts no implementation
child and changes no product code.

`Code-solution review: n/a — planning/task-ledger-only decomposition exemption`

## Command

```bash
python3 scripts/rri.py --cc 9 --T 2 --A 1 --X 3 --D 3 --K 4 --P 4 \
  --touches crates/db/src/translation_repo.rs \
  --touches crates/db/src/target_language_repo.rs \
  --touches crates/db/src/lib.rs \
  --touches apps/api/tests/localization_repo_test.rs \
  --penalty auth_security --platform dubbridge
```
