---
type: Plan
title: "Plan: Gemma Process Validation — Dual-Concept Live Test"
status: active
supersedes: ""
---
# Plan: Gemma Process Validation — Dual-Concept Live Test

> **Status:** Active
> **Tasks ledger:** `docs/tasks/gemma-process-validation.md`
> **Governing ADR:** `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md`
> **Related playbook:** `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`

## Objective

Validate two implemented-but-untested-in-practice processes against a curated
ground-truth set of real bugs and improvements found in recently delivered code
(S-127, ADR-034 slice):

1. **Gemma Developer (patch delegation)** — can the Low-RRI handoff protocol
   produce correct, scoped patches for confirmed Low-band issues without human
   redesign?
2. **Gemma Reviewer (triple-pass review)** — does the three-pass reconciler
   surface the same issues that a human arbiter already identified? What is the
   recall rate? What noise does it introduce?

The primary agent acts as **arbiter**: it owns the ground truth, executes the
handoffs, evaluates each output, and records verdict + process improvement signals.
The goal is not just to fix the five bugs — it is to stress-test the process and
produce calibration data for ADR-034's audit loop.

## Roles

### Primary agent as Arbiter

The primary agent (Claude) owns the **Arbiter** role throughout the experiment.
This is not a neutral observer role — it is an active judge with defined
responsibilities:

| Responsibility | When | Output |
|---|---|---|
| Seal the ground truth before any Gemma run | T0 | Ground truth table in plan |
| Write the Gemma Developer packet for each bug | T1–T4 | Handoff packet per task |
| Validate every patch: scope, structure, correctness | T1–T4 | Accept / reject / retry decision |
| Run `py_compile` / `tsc --noEmit` after each patch | T1–T4 | Verification evidence in task |
| Write a verdict block after each patch task | T1–T5 | `### Arbiter verdict` per task |
| Run Gemma Reviewer and compute recall + precision | T6 | Metrics in T6 verdict |
| Adjudicate Gemma Reviewer findings: confirm, dismiss, or escalate | T6 | Disposition per finding |
| Record `disposition_divergence` if primary disagrees with Gemma consensus | T6 | Audit signal |

**Authority:** The primary agent is orchestrator of record. It may reject a
Gemma Developer patch and retry with a narrower packet. It may dismiss a Gemma
Reviewer finding. Its disposition is final; Gemma's output is advisory only.

**Constraint:** The arbiter must not retroactively add bugs to the ground truth
after Gemma runs. The ground truth is sealed at T0. If Gemma surfaces a real
issue not in the ground truth, it is recorded as a bonus finding — not as a
recall hit.

### Primary agent as Process Improver

After T6, the primary agent steps into a second role: **Process Improver**. This
is distinct from the Arbiter role and activates only in T7.

| Responsibility | Output |
|---|---|
| Assess each pre-identified process gap (PG-01 to PG-05) against what the experiment actually showed | Disposition per PG entry (Confirmed / Disproved / Partially confirmed) |
| Identify new gaps surfaced during execution that were not anticipated | New PG entries if warranted |
| Propose concrete improvement candidates as `O-xx` entries with effort and impact | O-xx table in T7 |
| Distinguish between improvements that are low-effort and eligible for a follow-on Low-RRI task vs. those that require a new slice | Per-candidate classification |
| Recommend whether ADR-034 needs a revision based on findings | ADR revision candidate or explicit "no change needed" |

**Scope constraint:** The Process Improver documents and proposes — it does not
implement in this slice. No playbook, wrapper script, or policy changes during
T7. Improvements become inputs to a follow-on slice.

---

## Why this slice

ADR-034 introduced audit telemetry and triple-pass reconciliation but was built
and reviewed by the same toolchain it instruments. There is no external validation
yet. The five bugs identified on 2026-06-25 give a concrete, human-verified
benchmark:

- **Ground truth exists before Gemma runs** — so recall and precision are
  measurable, not retroactively rationalized.
- **All five are Low band (RRI 8–22)** — Gemma Developer is the correct
  delegation path per policy.
- **Two bugs are in the scripts Gemma runs** (delegate-low-rri.py,
  gemma-code-review.py) — making it a self-referential test: Gemma patches
  the very tools that execute it.

## Ground truth — confirmed bugs (arbiter-verified 2026-06-25)

| ID | File | Line(s) | Description | RRI |
|----|------|---------|-------------|-----|
| B-01 | `mobile/src/components/VideoPlayer.tsx` | 76, 96 | `?? "loading"` is dead code — `showOverlay` guarantees `overlay.kind !== null`; fallback unreachable and misleading | 8 |
| B-02 | `scripts/delegate-low-rri.py` | 728–733 | Tempfile not cleaned up if `tmp.write()` or `tmp.close()` throws before the outer `try:` is entered | 18 |
| B-03 | `scripts/delegate-low-rri.py` | 949 | `done_reason` hardcoded to `"stop"` even when `delegation["status"] == "blocked"` — audit log misleading for blocked delegations | 20 |
| B-04 | `scripts/gemma-code-review.py` | 259 | Bare-word STATUS fallback (`line.strip() in STATUS_VALUES`) accepts malformed Gemma output silently — no warning in audit | 22 |
| B-05 | `mobile/src/screens/ReviewDetailScreen.tsx` | 86 | `as ReviewTaskSummary["state"]` cast applied to `string` from API without runtime guard | 15 |

**False positive rate from initial Explore agent pass: 7/12 (58%).** This is
itself a calibration signal for the bug-hunt methodology (see § Process gaps).

## Scope

### Included

- Patches for B-01 through B-05, each delegated to Gemma Developer
  individually with the standard Low-RRI handoff packet protocol.
- One Gemma Reviewer triple-pass run on the aggregate diff produced by the
  four patches (B-01–B-04); a second run on the B-05 diff separately.
- Arbiter evaluation for each run: recall (did Gemma find the known bug?),
  precision (did it raise false findings?), process signal (what does the audit
  log show?).
- Process improvement notes: written findings about gaps in the Low-RRI and
  Reviewer workflows surfaced by the experiment.
- Daily update to `docs/daily/2026-06-25.md` on close.

### Excluded

- Changes to ADR-034, the audit schema, or playbooks during the experiment
  (those come after, as improvement tasks, in a separate slice if warranted).
- Gemma Developer delegation of B-05 (the TypeScript cast) — TypeScript/React
  Native is outside Gemma's reliable delegation target; B-05 is fixed directly
  by the primary agent and used only for the Reviewer recall test.
- New features, refactors, or scope beyond the five confirmed bugs.

## Design decisions

### D1 — Ground truth precedes Gemma invocation

The arbiter identifies and documents all five bugs before any Gemma run. This
prevents post-hoc rationalization of what Gemma "should have found." Recall is
computed as `bugs_found_by_gemma / 5`; precision as `confirmed_findings /
total_gemma_findings`.

### D2 — One Gemma Developer packet per bug

Each of B-01–B-04 gets its own handoff packet, executed sequentially. Mixed
packets are an anti-pattern per the playbook. Sequence: B-01 → B-02 → B-03 →
B-04.

### D3 — Reviewer runs after all patches are applied

Gemma Reviewer is invoked on the diff produced by the full patch sequence
(B-01–B-04 applied), then separately on the B-05 fix. Running on post-patch
code is realistic — it mirrors actual workflow where Reviewer sees the candidate
commit, not the pre-patch state.

### D4 — Arbiter evaluation is written, not inferred

Each task ends with a written `### Arbiter verdict` block: recall, precision,
notable audit signals, process notes. The verdict is part of the task's
completion evidence and feeds the process improvement section.

### D5 — Process improvement notes are deferred, not implemented

Improvements to playbooks, wrapper scripts, or policies identified during the
experiment are **documented** in T6 but not implemented in this slice. They
become candidates for a follow-on slice after the arbiter's synthesis.

## Execution strategy

- B-05 (TypeScript): fixed directly by primary agent (not delegated).
- B-01–B-04: delegated via `scripts/delegate-low-rri.py` with `--mode full-file`
  (all target files are under 400 lines).
- Gemma Reviewer: `scripts/gemma-code-review.py` with default `--passes 3`.
- Primary agent validates every patch before applying; acts as orchestrator of
  record throughout.
- No human approval gate between patch tasks (all Low band; autonomy permitted
  per `HITL_AUTONOMY_POLICY.md`).

## Process gaps identified (pre-experiment)

These gaps motivated this slice and will be tested or refined during execution:

| ID | Gap | Source |
|----|-----|--------|
| PG-01 | Explore agent had 58% false positive rate on bug hunt — no instruction to verify API contracts before reporting | Bug hunt, 2026-06-25 |
| PG-02 | `done_reason` always `"stop"` in Developer audit log — blocked delegations are indistinguishable from stopped ones by this field | B-03 discovery |
| PG-03 | Bare-word STATUS fallback in Reviewer produces no audit signal — malformed Gemma output accepted silently | B-04 discovery |
| PG-04 | No ground-truth recall benchmark for Gemma Reviewer has ever been run — finding quality is unvalidated | ADR-034 gap |
| PG-05 | Bug searches start from full codebase rather than recent diffs — increases false positive risk from stable, intentional API contracts | Bug hunt methodology |
| PG-06 | `delegate-low-rri.py` does not validate/normalize OLLAMA_HOST — bare host without scheme causes URLError | T1 execution |
| PG-07 | `apply_before_after` includes `old mode → new mode` artifact in unified_diff — tempfile has no exec bit; display diff is misleading | T2/T3/T4 execution |
| PG-08 | Reviewer occasionally emits STATUS PASS with findings (logical contradiction) — parser rejects; no retry loop in ADR-034 protocol | T6 Diff-B attempt 1 |

## Task order and dependencies

```
T0 (ground truth) ──► T1 (B-01 patch) ──► T2 (B-02 patch) ──► T3 (B-03 patch)
                                                                       │
                   T5 (B-05 fix) ──► T4 (B-04 patch) ◄────────────────┘
                        │                    │
                        │                    ▼
                        └──────────► T6 (Reviewer runs + arbiter eval)
                                             │
                                             ▼
                                     T7 (process improvement notes + close)
```

T0 must complete before any patch task. T1–T4 are sequential (same file pair for
B-02/B-03; order matters for diff coherence). T5 is independent of T1–T4 but
feeds T6 as a separate diff. T6 requires all patches applied. T7 synthesizes.
