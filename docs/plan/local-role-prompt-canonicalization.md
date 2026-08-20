---
type: Plan
title: "Plan: Local Role Prompt Canonicalization"
status: done
supersedes: ""
---

# Plan: Local Role Prompt Canonicalization

> **Status:** Done - LRPC-0b through LRPC-8 all complete as of 2026-08-20.
> `prompt_anchors.py` + `prompt_builder.py` are delivered and consumed by all
> three target scripts (`gemma-code-review.py`, `run_local_task.py`'s
> `cli.py`, `run_analysis.py`); the `check-review-budget.py` overhead
> cross-check (LRPC-7) and `AGENT_WORKFLOW_GUIDE.md` docs propagation
> (LRPC-8) close the sequence.
> **Tasks ledger:** `docs/tasks/local-role-prompt-canonicalization.md`

## Objective

Replace the hardcoded, independently-maintained `system_prompt` strings in the
three local-model orchestration scripts (`scripts/gemma-code-review.py`,
`scripts/local-agent/run_local_task.py`, `scripts/local-architect/run_analysis.py`)
with a single canonical, provenance-tagged anchor source plus a shared runtime
builder that assembles each role's authority-boundary clause on demand, enforces
a token budget derived from that invocation's `num_ctx`, and fails before the
Ollama call if the clause does not fit — structurally eliminating the drift class
of bug this plan was opened to fix, rather than only detecting it after the fact.

## Why this exists

`AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer / Muse Glimmer Reviewer § Authority
boundary` states the Gemma Reviewer role:

> "may not write files, apply patches, approve tasks, **certify coverage**, or
> **mark tasks complete**."

The hardcoded prompt actually sent to Ollama, `scripts/gemma-code-review.py:188-189`,
says:

> "Do not approve, **close tasks**, modify files, emit patches, emit unified
> diffs, or output file bodies."

Two divergences in one sentence: "certify coverage" is missing entirely, and
"mark tasks complete" became "close tasks" — a different phrase, introduced by
manual paraphrase, not a deliberate edit. This is exactly the failure mode a
copy-and-hope mechanism produces, and it sits in the text that governs the
review role every RRI 26-55 task in this repo depends on for its phase-1/phase-2
gate (`docs/policies/RRI_POLICY.md § Local pipeline phase-1/phase-2 reviewer
bindings`). By contrast, `run_local_task.py:129-133`'s boundary clause
("You may only edit the listed allowed_paths and then call finish. Any read
attempt, command attempt, or unlisted path terminates immediately as
boundary_violation.") already tracks its canonical source
(`AGENT_WORKFLOW_GUIDE.md § Handoff prompt format`) closely — confirming this is
a real, uneven drift risk across the three scripts, not a hypothetical one.

## Scope

**In scope:** the *authority-boundary clause* of each role's system prompt — the
sentences that constrain what the model itself may output or attempt. Not in
scope: each script's own output-format contract (e.g. `gemma-code-review.py`'s
`STATUS`/`FINDING` tagged-block shape, `run_local_task.py`'s tool-call JSON
shape) — those have no canonical-doc source to drift from; they are
implementation detail local to each script and stay hardcoded.

**Affected files (eventual, across the full task ledger):**

- `scripts/gemma-code-review.py` (LRPC-0, LRPC-3)
- `scripts/local-agent/run_local_task.py` (LRPC-4)
- `scripts/local-architect/run_analysis.py` (LRPC-5) — both `DEFAULT_PROFILE`
  and `MED_HIGH_REFINEMENT_PROFILE`
- `scripts/check-review-budget.py` (LRPC-7, cross-check only)
- New: `scripts/local-agent/prompt_anchors.py` + test (LRPC-1)
- New: `scripts/local-agent/prompt_builder.py` + test (LRPC-2)
- New: golden-set fixture module (LRPC-6)
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (LRPC-8, propagation)

## Design decisions

### 1. Extraction, not paraphrase

The anchor source is a curated set of **verbatim substrings** lifted from the
governing docs (`AGENT_WORKFLOW_GUIDE.md`, `HITL_AUTONOMY_POLICY.md`,
ADR-037), never a rewrite. This is mechanical, matching the pattern
`scripts/generate-agents-override.py` already uses for `AGENTS.override.md`
(literal concatenation, `"".join(...)`, zero rewriting) — extended here from
whole-file granularity to sentence granularity.

### 2. Classify before extracting

Not every sentence in a role's canonical "Authority boundary" section belongs
in that role's own prompt. Three classes:

| Class | Example | Treatment |
|---|---|---|
| Constrains the role's own output | "may not write files, apply patches, approve tasks, certify coverage, or mark tasks complete" | **always extracted**, verbatim |
| Describes downstream consumption by the orchestrator | "A finding ... never fails the review gate by itself ... advisory evidence" | **omitted** — does not change what the model should produce |
| Explanatory / rationale | "Gemma-authored Low-RRI patches require an independent primary-agent review even when Gemma Reviewer also runs" | cut first under budget pressure |

### 3. One deterministic cut order under the token cap

Rules 1(c) and 3 compose into a single explicit priority, not two
independently-judged heuristics (phase-1 review, LRPC-1, flagged the
original wording as under-specified): when a role's anchor exceeds its
`num_ctx` budget, cut in this fixed order — (i) explanatory/rationale
clauses (class 1c) first, (ii) permission ("may") clauses next, (iii)
prohibition ("may not"/"must not"/"never") clauses last, cut only if
nothing else remains. Omitting a permission under-informs (safe direction);
omitting a prohibition is the actual failure mode, so it is the last thing
removed.

### 4. Never truncate mid-clause

A sentence carrying `except`/`unless`/`only when` is included whole or not at
all. A truncation that keeps the head and drops the exception inverts the
sentence's meaning, which is strictly worse than omitting it.

### 5. Per-sentence provenance

Every anchor entry carries its exact source pointer (file + section/line).
This is what lets the future drift/budget check (LRPC-2's `CAP` step)
distinguish "the extraction is wrong" from "the source moved since this
anchor was last verified" — two different remedies.

### 6. Verification is behavioral equivalence, not readability

"It looks short and clear" proves nothing about information loss. The actual
test (LRPC-6): run a fixed set of adversarial fixture packets — cases where
the correct verdict depends on the exact clause being compressed — against
(a) the full canonical prose as context and (b) the anchor/builder output,
and require identical verdicts from the model in both conditions.

### 7. The extractor is never the constrained role

Gemma Reviewer must not be the one summarizing its own authority-boundary
text, for the same reason ADR-037 bars the Local Architect from authoring the
canonical document that governs it. Extraction is done by the orchestrator
(a deterministic script, per decision 1) and reviewed by a human or the
band-resolved reviewer — never generated ad hoc by the role it constrains.

## Architecture

```text
prompt_anchors.py (LRPC-1)          <- canonical, provenance-tagged, per-role
        |                              verbatim clauses; no logic
        v
prompt_builder.py (LRPC-2)          <- build_system_prompt(role, num_ctx, num_predict)
        |                              measures tokens, enforces budget, raises
        |                              before any Ollama call if over budget
        v
gemma-code-review.py (LRPC-3)       <- consumes builder for the boundary clause;
run_local_task.py (LRPC-4)             keeps its own output-format contract local
run_analysis.py (LRPC-5)               (not sourced from canonical docs)
```

`check-review-budget.py`'s `PACKET_OVERHEAD_TOKENS` (LRPC-7) is a separate,
adjacent budget (the diff/packet Gemma reviews, not the system prompt itself)
that can now be informed by the builder's actual measured prompt size instead
of a fixed guess — closed as a cross-check, not merged into one constant.

## Judgment calls to flag for review

- **P (public API / permissions / data impact) scored 3, not 4**, for LRPC-1
  and LRPC-2. Rationale: the safety properties these prompts describe
  (read-only, path-bounded) are actually enforced by the calling script's own
  code (`run_local_task.py`'s `boundary_violation` check on every tool call;
  `gemma-code-review.py` never grants Gemma Reviewer file-write capability
  regardless of prompt wording) — not by the model's compliance with the
  prompt text. The prompt is an internal contract between orchestrator and
  model, closer to P=3 ("changes internal API") than P=4
  ("permissions/ownership/data visibility"), which in this rubric's other
  examples means production RBAC/data-visibility changes. A reviewer may
  disagree and push this back to 4 — recorded here so the call is visible,
  not buried in a script flag. Phase-1 review (LRPC-1) noted the low P-score
  should not read as "low stakes": this task is a prerequisite for the
  review-pipeline's own integrity (the mechanism every RRI 26-55 task's
  phase-1/phase-2 gate depends on), which is why it stays at Moderate/D=4
  rather than dropping further despite P=3.
- **No anchor-rubric path match** for any of the new/touched files — none of
  them are `crates/**`. D/K/P are agent-judged, not rubric-floored;
  `scripts/rri.py`'s advisory line flags this on every touched path.
- **Antares Security-Specialist Advisor: skipped.** No watchlisted CWE
  (`scripts/antares/cwe_watchlist.py`: CWE-89, CWE-306, CWE-22, CWE-732) maps
  to prompt-boundary-text fidelity for local review models; this is a
  governance/fidelity concern, not a classic injection or access-control
  vulnerability class. Recorded as a typed skip, not invoked as a generic
  sweep.

## Sequencing

Full ordered task list, dependencies, RRI per task, and HP/EC cases live in
`docs/tasks/local-role-prompt-canonicalization.md`, starting at LRPC-1 — the
first task requiring the RRI 26+ approval gate. LRPC-2 through LRPC-8 are
sequenced but intentionally left at a lighter definition pending their own
pre-implementation analysis pass, per the incremental "present the next
task" discipline in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`. The confirmed
drift bug that motivates this plan (see "Why this exists" above) is tracked
as an independent, out-of-band fix at
`docs/tasks/gemma-push-reviewer-role.md § T9` — it has no dependency edge
into this ledger and is not part of the canonicalization mechanism itself.

**LRPC-0b inserted 2026-08-19, before LRPC-1's Implement phase — done
2026-08-19.** LRPC-1's approved implementation route delegates through
`scripts/local-agent/run_local_task.py`, which was 1491 lines, exceeding the
`AGENT_WORKFLOW_GUIDE.md § Handoff prompt format` "Target-file size gate"
(500-line ceiling on any file the local implementer must read in full).
LRPC-0b refactored that runner into five cohesive submodules (Extract
Module/Single Responsibility, behavior-preserving, existing test suite as
the regression oracle): `run_local_task.py` (415 lines, facade),
`session_loop.py` (475), `audit_record.py` (269), `rust_toolchain.py` (123),
`cli.py` (440). Implemented directly by the primary orchestrator rather than
through the band's normal Med-high cloud-only route — owner-directed
2026-08-19 deviation, no Codex/cloud tokens available this session; full
rationale in `docs/tasks/local-role-prompt-canonicalization.md § LRPC-0b
Implementation routing evidence`. Gemma phase-2 review (3 passes) found no
actionable findings. This did not reopen or alter LRPC-1's own scope,
acceptance criteria, or completed phase-1 review — it only gated when
LRPC-1's Implement phase could start; that phase is now unblocked. Full
definition and closure record:
`docs/tasks/local-role-prompt-canonicalization.md § LRPC-0b`.

## Related

- `docs/tasks/local-role-prompt-canonicalization.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Gemma Reviewer / Muse Glimmer
  Reviewer, § Handoff prompt format, § Local Architect / Complex Analyst (ADR-037)
- `docs/policies/RRI_POLICY.md` § Local pipeline phase-1/phase-2 reviewer bindings
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- ADR-037
