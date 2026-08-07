---
type: Plan
title: "Plan: local-agent system prompt is blind to cwd and its own turn budget"
description: "TOOL_CALLING_SYSTEM_PROMPT never tells the local implementer that run_command executes relative to the worktree root, or what its actual per-band turn budget is — the model hallucinates absolute paths and burns the budget on failed reconnaissance before ever calling write_file/finish, a failure mode already observed live (T2a: 8/8 turns consumed on run_command reconnaissance, including calls against a non-existent /home/user/... path)."
status: implemented
slice: med-high-turn-budget-blind-prompt
---

# Plan: Local-Agent System Prompt Is Blind to `cwd` and Its Own Turn Budget

> **Status:** Proposed — surfaced 2026-08-07, during a review of why Med-high
> local sessions were failing to complete tasks within their turn budget.
> **Tasks ledger:** `docs/tasks/med-high-turn-budget-blind-prompt.md`
> **Discovered by:** live inspection of `.agent/s-140-t3c-iii-local-transcript.json`
> (Moderate band, 4/30 turns spent on pure `read_file` reconnaissance before
> this session was interrupted) plus the already-recorded T2a incident in
> `docs/tasks/antares-security-specialist-advisor.md` (Med-high band, 8/8
> turns consumed on `run_command` reconnaissance — including two calls
> against a hallucinated `/home/user/repos/antares/scripts/antares/` path —
> never reaching `write_file` or `finish`, forcing cloud escalation).

## Objective

Make `TOOL_CALLING_SYSTEM_PROMPT` (`scripts/local-agent/run_local_task.py:111`)
state two facts the local implementer currently has no way to know from its
own context: (1) `run_command` and all file-tool paths are resolved relative
to the worktree root it is already in — never an absolute host path — and (2)
the exact numeric turn budget for this session, so the model can prioritize
reaching `write_file`/`finish` over open-ended exploration. Both facts are
already computed by the runner (`worktree_dir` passed to `_run_command_with_timeout`,
`effective_limits.max_total_turns` from `resolve_effective_limits`) but never
surface in the one channel the model actually reads — the system prompt.

This does not change turn-budget values (`MAX_TOTAL_TURNS`,
`MED_HIGH_MAX_TOTAL_TURNS`), the loop's counting logic, or any ADR-038
routing rule. It closes an information gap in the contract given to the
model, not a code defect in the loop itself.

## Evidence

- **T2a (Med-high, RRI 45, `docs/tasks/antares-security-specialist-advisor.md`
  lines 709-716):** "The model spent all 8 turns on `run_command`
  reconnaissance (including two calls against a hallucinated, non-existent
  path `/home/user/repos/antares/scripts/antares/`), never called
  `write_file` or `finish`... Per ADR-038, Med-high has zero repair
  attempts — this correctly triggered immediate escalation rather than a
  local retry."
- **T2e-pre rationale (same file, lines 2576-2579):** cites T2a's failure
  explicitly as precedent when deciding to downgrade a *different*,
  larger task straight to cloud without attempting a local session at all —
  i.e. the blind-prompt defect had already started shaping routing decisions
  away from local delegation, not just costing one session.
  ("T2a — a materially simpler single-file (169-line) parser task — already
  exhausted its full 8-turn local budget on `run_command` reconnaissance
  alone without ever reaching `write_file` or `finish`.")
- **Live transcript, Moderate band
  (`.agent/s-140-t3c-iii-local-transcript.json`):** 4 of 30 turns consumed
  reading `crates/jobs/src/lib.rs`, `.github/workflows/ci.yml`, `Makefile`,
  and `docs/tasks/s-140-subtitle-generation.md` in sequence before any write
  attempt — the same exploration-heavy pattern, just inside Moderate's much
  larger 30-turn/2-repair budget, where it costs proportionally less. Under
  Med-high's 8-turn/0-repair budget, an identical four-turn preamble would
  already consume half the session.
- **Source confirmation
  (`scripts/local-agent/run_local_task.py:338-355`):** `_run_command_with_timeout`
  launches `subprocess.Popen(argv, cwd=worktree_dir, ...)` — `cwd` is fixed
  to the worktree root — but `TOOL_CALLING_SYSTEM_PROMPT`
  (`run_local_task.py:111-149`, read in full) contains no mention of `cwd`,
  the worktree root, or relative-vs-absolute paths anywhere in its tool
  descriptions.
- **Source confirmation (turn budget):** `MED_HIGH_MAX_TOTAL_TURNS = 8`
  (`run_local_task.py:87`) and `MAX_TOTAL_TURNS = 30` (`:68`) are
  Python-level constants threaded into `EffectiveLimits` by
  `resolve_effective_limits()` (`:271-287`) and consumed by `run_loop`'s own
  turn-counting (`:460-483`) — but `run_loop`'s system message
  (`:468-473`) is built as `TOOL_CALLING_SYSTEM_PROMPT + "\n\nTask:\n" +
  card.spec`, a literal string concatenation with no interpolation of
  `limits.max_total_turns` anywhere. The model that must operate inside an
  8-turn budget is never told the number 8.

## Design decisions

1. **Fix the prompt, not the loop.** The turn-counting, malformed-bounce
   budget, and repair-attempt logic in `run_loop` are unaffected by this
   plan — they already work exactly as specified (confirmed by re-reading
   `:460-656` end to end). The defect is purely informational: the contract
   the model receives is incomplete relative to what the runner actually
   enforces.
2. **Interpolate `max_total_turns` at the `run_loop` call site, not inside
   the module-level `TOOL_CALLING_SYSTEM_PROMPT` constant.** The constant is
   band-agnostic (the same string serves Moderate's 30-turn and Med-high's
   8-turn sessions); the concrete number is only known once
   `resolve_effective_limits(card)` runs inside `run_loop` (`:460`). Adding a
   `{max_total_turns}` placeholder to the constant and `.format()`-ing it at
   the existing `messages = [...]` construction (`:468-473`) keeps the
   change local to the one place that already has both pieces of
   information in scope, rather than threading a new parameter through
   `build_live_chat_fn` or any other function that does not need it.
3. **State `cwd` as an absolute rule, not a hint.** The T2a incident's
   hallucinated path was not a near-miss the model self-corrected from — it
   consumed two of the eight total turns before the session moved on. The
   added sentence names the failure mode directly ("never use an absolute
   path like `/home/...` or `/Users/...`") rather than a softer phrasing,
   because the live evidence shows the model needs an explicit negative
   example, not just a positive statement of the rule.
4. **Do not add a turn-by-turn countdown or a live "N turns remaining"
   mechanism.** That would require threading state through `chat_fn` on
   every call, a materially larger change to `build_live_chat_fn`'s
   stateless-per-request design (see its own docstring at `:660-671`) for a
   problem the evidence does not show is needed — both real failures (T2a,
   the Moderate transcript) show the model burning turns on *unbounded*
   exploration with no sense of any budget at all, not on miscounting a
   budget it was tracking. A single upfront number plus an explicit
   "prioritize reaching `finish`" instruction addresses the observed failure
   mode without the added complexity.
5. **Implementation route: primary agent implements directly, not via
   `scripts/local-agent/run_local_task.py` itself.** This plan's RRI (33,
   Moderate) would ordinarily route through the local-first path this exact
   script implements. Per the precedent set in
   `docs/plan/med-high-escalation-bundle-crash.md`'s equivalent circularity
   argument (peer review rejected "the gate cannot repair itself" as a
   rationalization and required the ordinary downgrade mechanism instead),
   this is not treated as a blanket exemption — it is a normal, explainable
   downgrade decision: a session governed by the very prompt this change
   edits cannot be trusted as the sole validator of that edit's correctness
   until the fix is in place. Recorded here rather than left implicit.

## Objective boundary

**In scope:** `TOOL_CALLING_SYSTEM_PROMPT`'s content and the one call site
that builds `messages[0]` in `run_loop`.

**Out of scope:**
- `MAX_TOTAL_TURNS`, `MED_HIGH_MAX_TOTAL_TURNS`, or any other numeric budget
  value — this plan changes what the model is *told*, not what the runner
  *enforces*.
- `resolve_effective_limits`, `EffectiveLimits`, `_is_med_high` — unchanged.
- `build_live_chat_fn`, the per-turn progress line, or any live-countdown
  mechanism (Design decision 4).
- `run_med_high_task.py` / the ADR-038 supervisor — this plan's scope is the
  shared prompt both Moderate and Med-high sessions receive via
  `run_local_task.py`, not the Med-high-specific process-group supervisor.

## Affected files

- `scripts/local-agent/run_local_task.py` — `TOOL_CALLING_SYSTEM_PROMPT`
  (`:111-149`), `run_loop`'s `messages = [...]` construction (`:468-473`)
- `scripts/local-agent/run_local_task_test.py` — new coverage for the
  interpolated prompt

## Related

- `docs/tasks/med-high-turn-budget-blind-prompt.md` (task ledger)
- `docs/tasks/antares-security-specialist-advisor.md` § T2a (primary
  evidence), § T2e-pre (secondary evidence — routing decision shaped by the
  same defect)
- `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md`
- `docs/policies/RRI_POLICY.md` § anchor rubric (`D=4`, "agent orchestration")
- `docs/plan/med-high-escalation-bundle-crash.md` (precedent for the
  self-repair circularity argument in Design decision 5)
