---
type: Audit
title: "RRI evidence: S-150-T2c decomposition"
task: S-150-T2c
status: current
---

# S-150-T2c — Decomposition RRI

Computed on 2026-08-13 against the current post-`T2b-ii-c` repository state.
These child scores replace the parent execution route; each child must be rerun
against its exact final paths immediately before its own approval card.

| Child | Command inputs | Final RRI | Route |
|---|---|---:|---|
| `T2c-i` | `--cc 8 --T 3 --A 2 --X 1 --D 3 --K 3 --P 3`; `crates/jobs/src/lib.rs` | 41 | Med-high / L |
| `T2c-ii` | `--cc 5 --T 3 --A 1 --X 1 --D 2 --K 2 --P 2`; `crates/db/src/subtitle_repo.rs`, `apps/api/tests/subtitle_repo_test.rs` | 37 | Moderate / M |
| `T2c-iii` | `--cc 6 --T 3 --A 2 --X 1 --D 3 --K 3 --P 3`; `crates/db/src/translation_delivery_repo.rs`, `apps/api/tests/translation_delivery_repo_test.rs` | 43 | Med-high / L |
| `T2c-iv` | `--cc 9 --T 3 --A 2 --X 1 --D 3 --K 4 --P 4`; `apps/worker-runner/src/subtitle_runtime.rs`, `apps/worker-runner/src/subtitle_runtime_tests.rs` | 48 | Med-high / L |
| `T2c-v` | `--cc 8 --T 3 --A 2 --X 1 --D 3 --K 4 --P 4`; `crates/jobs/src/lib.rs`, `apps/worker-runner/src/translation_enqueue.rs`, `apps/worker-runner/src/main.rs`, `apps/worker-runner/src/runner_topology_tests.rs` | 50 | Med-high / L |

All commands use `--platform dubbridge` and were run via `scripts/rri.py`.
No penalties were emitted. No child is Complex, so the parent mandatory
decomposition gate is satisfied; every child still needs its own phase-1 review
and explicit HITL approval before implementation.

## Rationale

`T2c-i` isolates the durable payload contract and deterministic identity. `T2c-ii`
isolates exact source-artifact recovery. `T2c-iii` owns only the outbox
acknowledgement lifecycle. `T2c-iv` is the route decision and per-target durable
fan-out, deliberately without Redis wiring. `T2c-v` supplies the queue adapter and
worker topology after the preceding contracts are stable. No child starts provider
execution or creates an S-150 review row.
