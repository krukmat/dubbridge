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
| `T2c-iv` superseded surface | `--cc 11 --T 3 --A 2 --X 2 --D 3 --K 4 --P 4 --penalty refactor_and_behavior`; `crates/jobs/src/lib.rs`, `apps/worker-runner/src/subtitle_enqueue.rs`, `apps/worker-runner/src/subtitle_runtime.rs`, `apps/worker-runner/src/subtitle_runtime_tests.rs`, `docs/bdd/README.md` | 63 | Complex / L — mandatory decomposition |
| `T2c-iv-a0` | `--cc 2 --T 2 --A 1 --X 1 --D 2 --K 2 --P 2`; `crates/jobs/src/lib.rs`, new `crates/jobs/src/subtitle_job.rs` | 34 | Moderate / M — cloud-only preceding extraction: source is 693 lines |
| `T2c-iv-a` | `--cc 4 --T 3 --A 1 --X 2 --D 3 --K 3 --P 4`; `crates/jobs/src/subtitle_job.rs` after `T2c-iv-a0` | 38 | Moderate / M — local-first once the target is below 500 lines |
| `T2c-iv-b` | `--cc 3 --T 3 --A 1 --X 1 --D 2 --K 3 --P 3`; `apps/worker-runner/src/subtitle_enqueue.rs` | 32 | Moderate / M |
| `T2c-iv-c` | `--cc 8 --T 3 --A 2 --X 2 --D 3 --K 4 --P 4`; new `apps/worker-runner/src/translation_fanout.rs`, `apps/worker-runner/src/translation_fanout_tests.rs` | 49 | Med-high / L |
| `T2c-v` | Prior exact 50 retained provisionally; narrowed to `crates/jobs/src/lib.rs`, `apps/worker-runner/src/translation_enqueue.rs`, and focused tests; rerun before presentation | 50 provisional | Med-high / L |
| `T2c-vi-a` | `--cc 8 --T 3 --A 2 --X 2 --D 3 --K 4 --P 4`; `apps/worker-runner/src/subtitle_runtime.rs`, `apps/worker-runner/src/subtitle_runtime_tests.rs`, `apps/worker-runner/src/main.rs`, `apps/worker-runner/src/runner_topology_tests.rs` | 51 | Med-high / L |
| `T2c-vi-b` | `--cc 2 --T 2 --A 1 --X 1 --D 2 --K 2 --P 3`; deletion of `apps/worker-runner/src/review_enqueue.rs`, `apps/worker-runner/src/main.rs`, S-140 BDD/status docs | 31 | Moderate / M |

All commands use `--platform dubbridge` and were run via `scripts/rri.py`.
No penalties were emitted. No child is Complex, so the parent mandatory
decomposition gate is satisfied; every child still needs its own phase-1 review
and explicit HITL approval before implementation.

## Rationale

`T2c-i` originally introduced a compatibility route, but the owner confirmed on
2026-08-13 that no queued legacy jobs exist and requested its removal. Combining
contract removal, producer change, runtime behavior, and BDD correction scored 63,
so `T2c-iv` is now a parent. `T2c-iv-a0` first extracts the 693-line job contract
into a local-sized module. `T2c-iv-a` then removes only the obsolete payload contract;
`T2c-iv-b` updates only its active producer; `T2c-iv-c` builds durable fan-out
without transport. `T2c-v` is narrowed to the Redis adapter. `T2c-vi-a` performs
the runtime/topology cutover after both service and adapter exist, and `T2c-vi-b`
deletes the dead review module and synchronizes S-140 BDD/status wording. No child
starts provider execution or creates an S-150 review row.

The child rows added by the replan are task-presentation estimates from exact
current path sets. Each must be rerun immediately before its own approval card.
