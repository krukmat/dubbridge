---
type: TaskList
title: "Tasks: ADR-036 local-first pilot"
plan: docs/plan/adr036-local-first-pilot.md
status: in-progress
slice: adr036-local-first-pilot
governed_by: [ADR-036]
---

# Tasks: ADR-036 Local-First Pilot

## Objective

Validate ADR-036 on the real hardware: measure the local stack, build the
bounded agentic runner, benchmark it on real repo tasks, and produce the
go/no-go evidence for the promotion gate — with every task routed to the
cheapest executor tier that can complete it safely.

## Slice RRI

The slice creates a new local-model execution surface (agentic runner +
security boundary) and produces the evidence that will decide a workflow-policy
change (T10); an architecture decision already exists (ADR-036), so the slice
carries an `arch_decision` penalty at presentation.

**Score: 40 → Moderate (26–40) → Effort M → thinking Off → Gate: plan + tasks
presented, human approval before implementation.**

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | slice is docs/tasks scaffolding, not a function body | High |
| F files | 2 | 3 governing files touched at slice level (ADR, plan, ledger); per-task scope is much smaller | High |
| D domain | 2 | local Ollama HTTP API + agent-loop/security-boundary code (Python) | High |
| T coverage | 0 | no existing tests in this area to confirm against; all new | High |
| A ambiguity | 1 | ADR-036 fixes the design; the 5 open questions are the slice's own subject, not unresolved scope | Medium |
| K coupling | 2 | shares `gemma_local.py` audit schema (T6c), the packet-delegation lineage, and the workflow guide's band table (T10) | High |
| P impact | 3 | outcome may change how RRI 26–40 is routed for every future task in this repo | High |
| X context | 3 | spans HITL policy, RRI policy, ADR-034 audit contract, and a new execution boundary | High |

Penalty: `arch_decision` (+12) — the slice's purpose is to validate (or refute)
a workflow-governing architecture decision (ADR-036) and, on a GO verdict,
change the workflow guide, RRI policy, and HITL policy in T10. Per-task scores
in this ledger are computed independently of the slice-level penalty; most
individual tasks (T2, T3, T6c, T6d) land in Low band on their own because they
are isolated, pre-designed, mechanical files.

**Procedural note (recorded for transparency):** this Slice RRI table was
computed and added to the ledger **after** T0, T1, and T2 had already been
presented and executed with only ad hoc per-task RRI estimates and no slice-level
gate — a deviation from `AGENT_WORKFLOW_GUIDE.md` §"Mandatory workflow before
implementing" step 4, which requires computing RRI (via `scripts/rri.py`) before
presenting a plan/tasks pair, not only per task. The gap was identified from a
user challenge, not caught proactively. T3 onward follows the corrected
procedure: this table gates the slice, and each task additionally computes and
states its own RRI at presentation time per the existing per-task convention.
T0–T2 are not being retroactively re-presented; their completion evidence
stands as recorded, with this note as the disclosed deviation.

## Governing Documents

- `docs/plan/adr036-local-first-pilot.md`
- `docs/adr/ADR-036-local-first-agentic-implementation-band.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/RRI_POLICY.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`
- `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md`

## Executor-tier routing summary

Granularization goal: reserve token-expensive agents for judgment work.

| Task | Executor tier | Why |
|---|---|---|
| T0 | primary + human | ADR ratification gate |
| T1 | primary (ops) | commands on the physical machine; config-only |
| T2 | **economy / gemma-developer** | isolated new-file Python, pre-designed contract |
| T3 | **economy / gemma-developer** | isolated new-file Python, pre-designed contract |
| T4 | **economy** (script) + primary (execution) | script is mechanical; the soak run needs the machine |
| T5 | primary | editorial judgment over repo history; HITL keeps interpretation-heavy work with the primary agent |
| T6a | balanced | agent loop with protocol/state handling |
| T6b | balanced + mandatory primary review | security-critical boundary enforcement |
| T6c | **economy** | schema extension of existing audit emitter, mechanical |
| T6d | **economy / gemma-developer** | deterministic packet transformation, pure function |
| T7 | primary (orchestration) | runs the corpus through the runner; token-light |
| T8 | primary | synthesis + go/no-go against promotion gates |
| T9 | per-task routing under pilot rules | the pilot itself |
| T10 | primary + human approval | policy propagation (only on GO) |

Per-task `RRI (est.)` values are **preliminary**; recompute with
`scripts/rri.py` at presentation time before executing any task (workflow
guide requirement). Every development task below still passes the standard
gates for its final band: phase-1 review, Gemma Reviewer/D14 phase-2 review,
Reflection log for 26+, unit coverage certification, owner verification.

## Behavioral coverage contract: unit-v1

Tests for this slice are Python (`scripts/**/*_test.py`), not Rust. The
Rust-only `.rs::test_name` certification enforced by
`scripts/check-task-unit-coverage.sh` does not apply; completion evidence is
the `python3 -m unittest` runs per task (same exception as the
`gemma-audit-and-triple-pass` slice).

## Task order and dependencies

```text
T0 ──► T1 ──► T2 ──► T3 ──► T4 ─────────────────────┐
 │                                                   ├──► T7 ──► T8 ──► T9 ──► T10
 ├────► T5 (corpus, parallel) ───────────────────────┤          (GO gate)
 └────► T6a ──► T6b ──► T6c ──► T6d ────────────────┘
```

T0 ratifies ADR-036 before anything else. The measurement chain (T1–T4) and
the runner chain (T6a–T6d) proceed in parallel after T0; T5 is independent
editorial work. T7 needs both chains plus the corpus. T8 is the Stage 1 exit.
T9/T10 run only on a GO verdict. Same-directory serialization: T6a→T6b→T6c→T6d
share `scripts/local-agent/` and are ordered.

---

## T0 — Ratify ADR-036 (decision gate)

- **Status:** [x] Done
- **Effort:** S
- **RRI:** n/a (decision-ratification gate, not a code task)
- **Executor tier:** primary + human
- **Scope:** `docs/adr/ADR-036-local-first-agentic-implementation-band.md`,
  `docs/adr/README.md`
- **Depends on:** none

### Goal

On slice approval, flip ADR-036 from `Proposed` to `Accepted` and apply the
ADR change propagation contract (frontmatter/prose parity, index row). If the
owner amends the stack or gates during review, propagate the amendment before
any downstream task starts.

### Acceptance Criteria

- ADR-036 frontmatter `status:` and prose `- **Status:**` both read `Accepted`;
  index row matches.
- `make qa-docs` deterministic checks pass.
- No code changes in this task.

### Handoff Prompt

T0 — ratify ADR-036 to `Accepted`; propagate per the workflow guide (index +
frontmatter/prose parity). Do not touch scripts. Stop after docs gates pass.

### Completion evidence

- ADR-036 frontmatter `status: Accepted` and prose `- **Status:** Accepted`
  both updated; `docs/adr/README.md` index row updated to `Accepted`.
- `bash scripts/check-doc-consistency.sh`, `bash scripts/check-roadmap-drift.sh`,
  `python3 scripts/check_okf_frontmatter.py` all passed.

---

## T1 — Install and pin the local stack (ops)

- **Status:** [x] Done
- **Effort:** S
- **RRI:** n/a (config/ops-only; exempt from review gates)
- **Executor tier:** primary (ops on the physical machine)
- **Scope:** local Ollama registry; `docs/evaluations/adr036-stage1-report.md` (stub)
- **Depends on:** T0

### Goal

Resolve ADR-036 open question 5: confirm `qwen3.6:35b-a3b` exists as an MLX
4-bit (or best-available Q4) build in the Ollama library; pull it and
`gemma4:26b-a4b-it-qat`; record exact tags, digests, quantization, and on-disk
sizes in the report stub. If the 35B-A3B build does not exist or only exists
in an unusable quantization, record it and activate the ADR-036 §6 contingency
binding (Gemma 4 26B A4B as implementer) for the rest of the slice.

### Acceptance Criteria

- Both models pulled and answering a smoke prompt through `OLLAMA_HOST`.
- Report stub records: tag, digest, quant, size, runtime backend (MLX/GGUF).
- Contingency decision (if any) recorded explicitly.

### Handoff Prompt

T1 — pull and verify the two ADR-036 model bindings; record tags/digests/
sizes in the report stub. No scripts. Stop after the smoke prompts succeed.

### Completion evidence

- Both bindings were already present on the target machine (`ollama list` via
  `/api/tags`): `qwen3.6:35b-a3b` (36.0B/Q4_K_M/GGUF, 23.9 GB) and
  `gemma4:26b-a4b-it-qat` (25.2B/Q4_0/GGUF, 15.6 GB). No pull needed.
- Smoke prompts succeeded on both: Qwen 41.44 tok/s decode, Gemma 45.36 tok/s
  decode (see `docs/evaluations/adr036-stage1-report.md § T1`).
- **Deviation recorded:** both bindings run on Ollama's llama.cpp/GGUF
  backend, not MLX — only the pre-existing `gemma4:12b-mlx` is a true MLX
  build. Recorded as a T1 finding in the Stage 1 report; does not block the
  pilot but changes the throughput baseline interpretation (see report).
- No contingency triggered — both models loaded and answered without error.
- `docs/evaluations/adr036-stage1-report.md` created with T1 results.

---

## T2 — Inference measurement script

- **Status:** [x] Done
- **Effort:** M
- **RRI (est.):** ~24 → Low (C1 F1 D2 T2 A0 K1 P1 X1; isolated new file)
- **Executor tier:** economy / gemma-developer candidate (packet per function)
- **Scope:** `scripts/local-bench/measure_inference.py`,
  `scripts/local-bench/measure_inference_test.py`, `.gitignore`
- **Depends on:** T1

### Goal

New isolated script that measures, per model binding: decode tok/s, prefill
tok/s at 8K/16K/32K synthetic prompts, time-to-first-token, and peak process
memory, via the Ollama HTTP API. Emits one JSON artifact per run under
`logs/local-bench/` (git-ignored). Resolves ADR-036 open questions 2 and 3
(prefill throughput, effective KV footprint at 32K).

### Acceptance Criteria

- `HP-1`: run against an available model → JSON artifact with all metric
  fields populated and non-zero.
- `HP-2`: three prompt sizes (8K/16K/32K) measured in one invocation with
  per-size prefill numbers.
- `EC-1`: model tag absent from Ollama → exit non-zero with a one-line error;
  no partial artifact left behind.
- `EC-2`: `OLLAMA_HOST` unreachable → fail closed within the timeout; exit
  non-zero.
- Unit tests cover metric computation and artifact schema with a mocked API;
  `python3 -m unittest scripts/local-bench/measure_inference_test.py` passes.

### Delegation note (economy executor)

The orchestrator pre-designs: the JSON schema, the Ollama API call shapes, and
the CLI contract (`--model`, `--sizes`, `--out`). The cheap executor
implements to that contract; it does not design it.

### Handoff Prompt

T2 — implement `measure_inference.py` to the pre-designed CLI/schema contract
in this entry. Allowed paths: the two new files + `.gitignore`. Mocked-API
unit tests required. Stop after unittest passes; do not run live benchmarks.

### Completion evidence

- Implemented via background subagent per the pre-designed contract; changes
  independently reviewed by the primary agent (not accepted from the
  subagent's self-report alone).
- Files: `scripts/local-bench/measure_inference.py`,
  `scripts/local-bench/measure_inference_test.py`; `.gitignore` gained
  `logs/local-bench/` (artifact home named in the Goal, `--out` remains
  explicit/required).
- `python3 -m unittest scripts/local-bench/measure_inference_test.py -v`
  independently re-run by the primary agent: **10/10 passed**, 0 failures.
- HP-1/HP-2 covered by `SuccessfulRun` (3-size mocked run, full schema
  populated, non-zero fields); EC-1 by `ModelNotFound` (non-zero exit, no
  artifact written); EC-2 by `UnreachableHost` (both `URLError` and
  `TimeoutError` paths, non-zero exit, no artifact).
- Scope verified via `git status --porcelain`: only `.gitignore` and the two
  new files under `scripts/local-bench/` touched by this task.
- No live Ollama calls made, per the task constraint (T7 owns the live run).

---

## T3 — Load-cycle and residency measurement script

- **Status:** [x] Done
- **Effort:** S
- **RRI (est.):** ~20 → Low (isolated new file, same pattern as T2)
- **Executor tier:** economy / gemma-developer candidate
- **Scope:** `scripts/local-bench/measure_residency.py`,
  `scripts/local-bench/measure_residency_test.py`
- **Depends on:** T2

### Goal

Measure the ADR-036 §6 residency cycle: cold-load time per model, unload via
`keep_alive: 0`, reload time, and the implement→unload→verify→reload sequence
cost. Quantifies the repair-iteration latency risk named in the ADR
consequences.

### Acceptance Criteria

- `HP-1`: sequence run against two model tags → JSON with per-phase timings.
- `EC-1`: second model fails to load (memory) → recorded as a structured
  failure in the artifact, non-zero exit, first model unloaded on cleanup.
- Mocked-API unit tests pass.

### Handoff Prompt

T3 — implement `measure_residency.py` per this entry's contract; same
conventions as T2. Stop after unittest passes.

### Completion evidence

- Presented per the task-presentation contract (RRI computed at 24 → Low)
  before implementation; approved by owner.
- Implemented directly by the primary agent (not delegated), following T2's
  conventions: atomic JSON write, argparse CLI, mocked-API tests only.
- Files: `scripts/local-bench/measure_residency.py`,
  `scripts/local-bench/measure_residency_test.py`. No `.gitignore` change
  (artifact directory already covered by T2's `logs/local-bench/` entry).
- `python3 -m unittest scripts/local-bench/measure_residency_test.py -v`:
  **8/8 passed**, 0 failures.
- HP-1 covered by `CycleModel.test_hp1_cycle_produces_all_phase_timings` and
  `MultiModelRun.test_hp1_two_models_both_succeed` (per-phase timings for
  cold_load/unload/reload/total, for two model tags in one run).
- EC-1 covered by `test_ec1_second_model_fails_records_structured_failure_and_unloads_first`
  (failed model recorded with `failed: true` + `error`, non-zero-exit path via
  `test_main_writes_artifact_even_on_failure_and_exits_nonzero`, and a
  best-effort unload call is asserted for the failed model).
- Contract clarification versus the ledger's literal EC-1 wording: unlike T2,
  T3 **always writes the JSON artifact** (even on failure) because the
  failure is meant to be a structured field inside the artifact, not an
  absent-artifact case; `main()` returns non-zero exit whenever any model's
  cycle failed. This matches "recorded as a structured failure in the
  artifact" in the Goal/Acceptance Criteria.
- Scope verified via `git status --porcelain`: only the two new files
  touched.

---

## T4 — Dev-stack contention soak

- **Status:** [~] In progress (phase 1 of 2: script done; live 1-hour soak pending)
- **Effort:** M
- **RRI:** 32 → **Moderate** (recomputed with `scripts/rri.py` at presentation
  time; supersedes the ledger's ~22 → Low placeholder — see task presentation
  transcript for the full variable table)
- **Executor tier:** balanced (script authored/reviewed by primary agent,
  phase-1 implementation delegated to a background subagent under an explicit
  contract) + primary (live 1-hour soak execution)
- **Scope:** `scripts/local-bench/soak_contention.py`,
  `scripts/local-bench/soak_contention_test.py`;
  results into `docs/evaluations/adr036-stage1-report.md`
- **Depends on:** T3

### Goal

Answer ADR-036 open question 1 with data: run generation loops while the real
dev stack is active (`docker compose -f infra/local/docker-compose.yml up`
Postgres/Redis/MinIO + a `cargo build` loop), sampling swap activity, wired
memory, throughput degradation, and thermal throttling over a 1-hour soak.
The **contingency verdict** (35B-A3B stays primary vs demote to Gemma 26B
A4B) is recorded from this task's data.

### Acceptance Criteria

- `HP-1`: soak run completes → time-series artifact + summary (min/median
  throughput, peak swap, throttle events).
- `EC-1`: sampling continues and artifact remains valid if a generation call
  times out mid-soak (gap recorded, not crash).
- Contingency verdict written into the report with the supporting numbers.

### Handoff Prompt

T4 — script per contract (economy executor), then a supervised 1-hour soak on
the real machine (primary). Stop after the verdict paragraph is in the report.

### Phase 1 completion evidence (script only — soak still pending)

- Presented per the task-presentation contract with the recomputed RRI (32 →
  Moderate) before implementation; approved by owner.
- Phase 1 delegated to a background subagent under an explicit written
  contract (CLI shape, JSON schema, EC-1 failure-recording behavior, hard
  constraint: **no self-review, no `make qa-gemma-review`, no live
  Docker/cargo/soak** — those stay with the primary agent).
- Files: `scripts/local-bench/soak_contention.py`,
  `scripts/local-bench/soak_contention_test.py`. No other file touched by the
  subagent (`git status --porcelain` verified independently by the primary
  agent, not accepted from the subagent's self-report).
- `python3 -m unittest scripts/local-bench/soak_contention_test.py -v`:
  independently re-run by the primary agent — **9/9 passed**, 0 failures.
- HP-1 covered by `HP1FullRun` (mocked multi-tick run, fully populated
  `samples` + `summary`, `throttle_detected: null` placeholder as specified).
- EC-1 covered by `EC1MidSoakFailure` (a mid-run `URLError` is recorded as
  `sample_ok: false` + `error`, run completes without crashing,
  `failed_sample_count` reflects it, failed sample excluded from
  min/median/peak math).

#### Gemma Reviewer evidence

- Model: `qwen3.6:35b-a3b` (`DUBBRIDGE_REVIEW_MODEL` unset; wrapper default
  resolved to the installed local model)
- Command: `git diff -- scripts/local-bench/soak_contention.py
  scripts/local-bench/soak_contention_test.py | python3
  scripts/gemma-code-review.py --out /tmp/dubbridge-gemma-review-t4.json
  --passes 3 -` (run by the primary agent; `git add -N` used first so the
  untracked new files show as a diff against empty)
- Passes run / usable: `3/3`
- Aggregate status: `FINDINGS` (5 findings, all `pass-specific`, all `minor`;
  zero `consensus`, zero `blocking`/`major`)
- `python3 scripts/parse-review-findings.py` exit code checked: `0` (non-blocking
  per the classification, but every finding was still read and individually
  dispositioned per policy — a `0` exit does not mean "0 findings" here)
- Isolated adjudicator: not triggered (Gemma was available and produced a
  usable 3/3 aggregate)
- disposition_divergence: `none`
- Primary-agent disposition (all 5 rejected, with reasons):
  1. "No warning when `psutil` absent" — rejected, cosmetic; sibling scripts
     T2/T3 have the same soft-optional pattern without a warning.
  2. "No retry on `http_post_json`" — rejected by design: T4's purpose is to
     record contention-induced failures (EC-1), not mask them with retries;
     retrying would corrupt the contention signal the task exists to measure.
  3. "`socket.timeout` not explicitly caught" — **verified false positive**:
     confirmed via `socket.timeout.__mro__` on the repo's Python 3.9.6 that
     `socket.timeout` subclasses `OSError`, which is already in the except
     tuple; nothing escapes uncaught.
  4. "Redundant duration-elapsed check" — rejected; defensive double-check,
     not a correctness issue, left as-is.
  5. "`remaining` could go negative" — **verified non-issue**: the existing
     `if remaining > 0: time.sleep(remaining)` guard already prevents a
     negative-duration sleep; confirmed with a standalone repro.

### Reflection log

Required passes: 2 (`32` → `Moderate`)

#### Pass 1

- **Draft verdict:** the sampling loop correctly distinguishes a genuine
  per-sample failure (EC-1, caught in `take_sample`) from a real throughput
  collapse (which would show up as a low but present `decode_tok_s`, not a
  `sample_ok: false`) — the two are structurally separate fields
  (`sample_ok` vs. `decode_tok_s`), so a future throttle heuristic reading
  the time series cannot confuse a network blip with real contention.
- **Critique findings:** gap-recording does not mask a throttle event — a
  failed sample's `decode_tok_s` is `None` and excluded from aggregates, so
  it neither hides nor fakes a slowdown; the Gemma Reviewer's retry-logic
  suggestion (finding #2) would have *introduced* exactly this masking risk
  had it been accepted.
- **Revisions applied:** none — confirmed the design already separates
  connectivity failure from performance signal correctly.

#### Pass 2

- **Draft verdict:** the contingency verdict this task exists to produce is
  falsifiable in principle (the report will cite `min_decode_tok_s`,
  `peak_swap_used_bytes`, and the sample time series, not a subjective
  impression) — but that verdict itself is **not yet written**, because
  phase 2 (the live 1-hour soak against real Docker + cargo) has not run.
- **Critique findings:** the script has no explicit cleanup step for the
  probed model's residency (it neither force-unloads on exit nor on
  exception) — for a 1-hour soak this is likely fine (the point is to keep
  the model loaded throughout), but on an aborted/Ctrl-C run it could leave
  the model resident with no `keep_alive` correction. This is a real gap for
  phase 2's operational safety, not phase 1's acceptance criteria.
- **Revisions applied:** none to the script (out of phase-1 scope per the
  approved acceptance criteria, which do not require exit-cleanup handling);
  flagged explicitly here as an operational note the primary agent must
  handle manually when running phase 2 (unload the model afterward via a
  manual `keep_alive: 0` probe, the same mechanism `measure_residency.py`
  already uses).

### Phase 2 (live soak) — not yet started

Blocked on scheduling the 1-hour supervised run (Docker Postgres/Redis/MinIO +
`cargo build` loop + `soak_contention.py` against `qwen3.6:35b-a3b`). Status
stays `[~] In progress` until phase 2 completes and the contingency verdict is
written into `docs/evaluations/adr036-stage1-report.md`.

---

## T5 — Benchmark corpus (15–20 task cards from repo history)

- **Status:** [ ] Pending
- **Effort:** M
- **RRI:** n/a (editorial/docs task; interpretation-heavy → primary per HITL)
- **Executor tier:** primary
- **Scope:** `docs/evaluations/adr036-benchmark-corpus.md`
- **Depends on:** T0 (parallel to everything else)

### Goal

Select 15–20 completed tasks from git history across categories: small Rust
bug, mobile Jest/RN test task, small API feature, refactor, CI failure, docs
task. For each: task card with scope, pre-designed failing-test contract
(HP/EC per ADR-036 §4), verification commands, and the original solution
commit as reference answer. Cards must be executable by the runner without
further interpretation.

### Acceptance Criteria

- 15–20 cards; every card names allowed paths, acceptance tests, verify
  commands, and reference commit.
- Category coverage: ≥2 Rust, ≥2 mobile, ≥1 CI, ≥1 docs, ≥2 refactor.
- No card depends on production credentials or network beyond `OLLAMA_HOST`.

---

## T6a — Agentic runner skeleton

- **Status:** [ ] Pending
- **Effort:** M
- **RRI (est.):** ~38 → Moderate (agent loop, state handling, K with gemma_local)
- **Executor tier:** balanced
- **Scope:** `scripts/local-agent/run_local_task.py`,
  `scripts/local-agent/run_local_task_test.py`
- **Depends on:** T0

### Goal

Thin tool loop per plan design decision 1: ingest a task card, create the
isolated worktree, drive an OpenAI-compatible chat loop against `OLLAMA_HOST`
with file-read/file-write/run-command tools, capture the full transcript, and
stop on: acceptance tests green, repair budget exhausted (2), or boundary
violation. Boundary checks are delegated to the T6b module (stub interface in
this task).

### Acceptance Criteria

- `HP-1`: mocked model completes a toy card → worktree contains the diff,
  transcript artifact written, exit 0.
- `HP-2`: mocked failing tests twice then success → exactly 2 repair turns
  recorded, exit 0.
- `EC-1`: repair budget exhausted → runner stops, emits escalation trigger
  record, exit non-zero; no third attempt.
- `EC-2`: malformed tool call from the model → counted, bounced back once,
  aborted if repeated.
- All tests run against a mocked chat endpoint; no live model in unit tests.

---

## T6b — Boundary enforcement module (security-critical)

- **Status:** [ ] Pending
- **Effort:** M
- **RRI (est.):** ~42 → Med-high floor expected (P high: security boundary)
- **Executor tier:** balanced + **mandatory primary-agent review**
- **Scope:** `scripts/local-agent/boundary.py`,
  `scripts/local-agent/boundary_test.py`
- **Depends on:** T6a

### Goal

Implement ADR-036 §3 as code, fail-closed: allowed-path guard (worktree-jailed,
symlink-safe), command policy (denylist: `git push`, recursive delete outside
worktree, `docker`, migration commands against non-local DBs; allowlist:
`cargo test/build/check/fmt/clippy`, `npm test/run lint/typecheck`, `make qa-*`
local gates), environment stripping (only `DUBBRIDGE_ENV=local` bindings and
`OLLAMA_HOST` pass through), and no-push guarantee (credential-free
environment + hook check in the worktree).

### Acceptance Criteria

- `HP-1`: in-scope write + allowlisted command pass through unchanged.
- `EC-1`: path escape attempts (absolute, `..`, symlink out of worktree) →
  rejected, violation recorded, runner abort signaled.
- `EC-2`: `git push` and denylisted commands → rejected and recorded.
- `EC-3`: env leak probe (`env` output in transcript) contains no secret
  material and no production descriptor variables.
- Adversarial fixtures for every EC; `python3 -m unittest` passes.
- Primary-agent review recorded in addition to band-routed review.

---

## T6c — Runner audit records (ADR-034 schema extension)

- **Status:** [ ] Pending
- **Effort:** S
- **RRI (est.):** ~26 → Low/Moderate boundary (touches shared `gemma_local.py`)
- **Executor tier:** economy
- **Scope:** `scripts/gemma_local.py`, `scripts/gemma_local_test.py`,
  `scripts/local-agent/run_local_task.py` (emission call sites)
- **Depends on:** T6b

### Goal

Emit one JSONL audit record per runner session through the shared
`append_audit_log()` (ADR-034): role `local-implementer`, task id, RRI, band,
attempts, commands executed, test outcomes, boundary violations, escalation
flag, elapsed, model tag. Same redaction rules; no raw file bodies.

### Acceptance Criteria

- `HP-1`: completed session → one record with all fields; schema-compatible
  with existing consumers (`gemma-audit-report.py` does not break).
- `EC-1`: aborted session (boundary violation) → record written with the
  violation before exit.
- Existing `gemma_local` tests still pass; new fields covered by tests.

---

## T6d — Escalation packet builder

- **Status:** [ ] Pending
- **Effort:** S
- **RRI (est.):** ~20 → Low (pure deterministic transformation)
- **Executor tier:** economy / gemma-developer candidate
- **Scope:** `scripts/local-agent/escalation_packet.py`,
  `scripts/local-agent/escalation_packet_test.py`
- **Depends on:** T6c

### Goal

Build the ADR-036 §7 packet from runner artifacts: task spec + RRI table,
plan, allowed paths, full diff, commands with output, test results, per-attempt
summaries. Output is a single markdown file a cloud agent can start from
without re-exploring the repository.

### Acceptance Criteria

- `HP-1`: artifacts from a failed session → packet with all seven sections
  populated, diff verbatim.
- `EC-1`: missing artifact (e.g. no test output) → section rendered as
  explicit `MISSING`, never silently omitted; exit still 0.
- Golden-file test for the packet format.

---

## T7 — Run the Stage 1 benchmark

- **Status:** [ ] Pending
- **Effort:** M
- **RRI:** n/a (operational orchestration; no new code)
- **Executor tier:** primary (orchestration; local models do the token work)
- **Scope:** `logs/local-bench/` artifacts;
  `docs/evaluations/adr036-stage1-report.md` (raw results section)
- **Depends on:** T2, T3, T4, T5, T6d

### Goal

Run the T5 corpus through the runner with the active binding; collect per-task:
success, repairs, escalations, wall-clock, scope/boundary violations, peak
memory; run the cloud-baseline comparison on a 5-task subsample to anchor the
≤2× wall-clock and token-reduction gates.

### Acceptance Criteria

- Every corpus card attempted; per-card result row recorded.
- Metrics table complete for the ADR-036 §10 promotion-gate fields.
- Audit JSONL contains one record per session.

---

## T8 — Stage 1 report and go/no-go

- **Status:** [ ] Pending
- **Effort:** M
- **RRI:** n/a (analysis/synthesis docs task)
- **Executor tier:** primary
- **Scope:** `docs/evaluations/adr036-stage1-report.md`
- **Depends on:** T7

### Goal

Fill the promotion-gate table (≥75% success without escalation, ≤2 repairs
avg, zero scope/boundary violations, ≤2× wall-clock, measured cloud-token
reduction), answer the five ADR-036 open questions with data, state the
binding verdict (35B-A3B vs contingency), and issue **GO / NO-GO** for
Stage 2. On NO-GO, record which rollback state applies and what would need to
change for a retry.

### Acceptance Criteria

- Every promotion-gate row filled with measured numbers, not estimates.
- All five open questions answered with artifact references.
- Explicit GO/NO-GO with owner sign-off line.

---

## T9 — Stage 2 pilot: 5 real RRI 26–40 tasks (GO gate)

- **Status:** [ ] Pending (blocked on T8 = GO)
- **Effort:** L (aggregate of 5 individually gated tasks)
- **RRI:** computed per pilot task at presentation time
- **Executor tier:** per ADR-036 routing (local implementer under full gates)
- **Scope:** determined per pilot task
- **Depends on:** T8 (GO verdict)

### Goal

Run 5 real Moderate-band backlog tasks through the local path with the full
production discipline: HITL approval per task, phase-1 review, local
implementation in the boundary, Gemma Reviewer phase-2, orchestrator
Reflection (2 passes), coverage certification, owner verification. The pilot
measures the process, not toy tasks: escalations and failures are valid data,
not pilot failures.

### Acceptance Criteria

- 5 tasks executed under the unmodified gate set; each closure record complete.
- Rolling metrics recorded in the same report (escalation rate, repairs,
  wall-clock vs estimate).
- Any boundary violation aborts the pilot immediately and is reported.

---

## T10 — Policy propagation on promotion

- **Status:** [ ] Pending (blocked on T9 results + human approval)
- **Effort:** M
- **RRI:** n/a (policy/docs task; explicit human approval required)
- **Executor tier:** primary + human
- **Scope:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
  `docs/policies/RRI_POLICY.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`,
  `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md` (cross-reference),
  `docs/adr/ADR-036-...md` (implementation notes)
- **Depends on:** T9

### Goal

Only if the pilot sustains the gates: amend the workflow guide (band table,
reviewer pairing, residency rule, escalation packet), RRI policy crosswalk,
and HITL policy (local delegation section) to make the 26–40 local path the
Balanced-mode default; record rollback triggers (ADR-036 §10) as operative
policy. This is the single task in the slice that changes agent-facing policy,
and it requires explicit human approval of the exact diff.

### Acceptance Criteria

- All governing docs amended consistently in one change; `make qa-docs` passes.
- ADR-036 implementation-status note added; index untouched (status stays
  `Accepted`).
- Rollback trigger wording present in the workflow guide.
