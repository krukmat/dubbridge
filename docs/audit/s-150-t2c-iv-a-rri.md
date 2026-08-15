---
type: Audit
title: "RRI evidence: S-150-T2c-iv-a"
task: S-150-T2c-iv-a
status: current
---

# S-150-T2c-iv-a — Presentation-time RRI

Recomputed on 2026-08-13 against the exact compile-safe crate-internal scope:
the extracted 286-line job-contract module plus the required crate-root reexport
and queue-test cleanup.

```text
python3 scripts/rri.py --platform dubbridge \
  --touches crates/jobs/src/subtitle_job.rs \
  --touches crates/jobs/src/lib.rs \
  --cc 4 --T 3 --A 1 --X 2 --D 3 --K 3 --P 4
```

| Variable | Score | Evidence |
|---|---:|---|
| C cyclomatic | 0 | Raw CC 4 |
| F files | 1 | Two touched files |
| D domain | 3 | `crates/jobs` anchor floor |
| T coverage | 3 | Focused serialization-contract tests |
| A ambiguity | 1 | Legacy payload retirement is explicitly approved |
| K coupling | 3 | `crates/jobs` anchor floor |
| P impact | 4 | Serialized job payload consumed by a future queue/runtime cutover |
| X context | 2 | S-150 plan, task ledger, and legacy-retirement decision |

**Final RRI:** 40 — Moderate (26–40), Effort M. No penalties were applied.

The task therefore requires phase-1 review and explicit HITL approval before
implementation. The approved default authoring route, if approved, is the
local `qwen3.6:35b-a3b` runner in a disposable worktree, with at most two
evidence-backed repairs. The bounded local run applied the correct contract diff
but exhausted its repair budget when it detected the separately sequenced worker
consumers; the primary agent completed the two-file crate-internal cutover.
Operational-only takeover is `gpt-5.6-terra` at medium; a capability/risk
takeover requires rerunning RRI before any promotion.
