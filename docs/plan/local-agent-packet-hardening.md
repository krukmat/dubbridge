---
type: Plan
title: "Plan: local-agent delegation packets fail on prose acceptance tests, contradictory scope, and long JSON anchors"
description: "Three concrete, evidence-backed defects found while forensically reviewing the 2026-09-13 P2.T3c-S1b Low-band decomposition attempts: a hand-authored packet passed prose as an acceptance-test command, a packet's allowed_paths contradicted its own spec text, and Devstral broke JSON-schema-constrained output on a long Rust anchor. Fixes are additive hardening to the shared local-agent pipeline used by every band from Low through Med-high."
status: proposed
slice: local-agent-packet-hardening
---

# Plan: Local-Agent Delegation Packet Hardening

> **Status:** Proposed — surfaced 2026-09-14 during a forensic review of
> `P2.T3c-S1b`'s (closed 2026-09-13) Low-band decomposition attempts,
> requested after the owner asked whether the Moderate/Med-high segment has
> real local-execution problems.
> **Tasks ledger:** `docs/tasks/local-agent-packet-hardening.md`

## Origin and evidence

Reviewing `.agent/p2-t3c/s1b-*-repair-*.json` (the real dispatch attempts
behind `P2.T3c-S1b`'s ADR-038 Amendment 4 decomposition pass) found three
distinct, independently confirmed defects — not one generic "local models
are unreliable" finding:

1. **`s1b-validation-repair-card.json`** — `acceptance_tests` contains
   English prose (`"Validation code uses only real existing fields and
   dependencies"`) instead of a runnable command. The local implementer
   (Devstral) actually succeeded: `apply_patch` applied, `finish` called,
   `scope_check: in_scope: true`, `rustfmt` clean. The session only failed
   because the test runner tried to execute the prose as `argv` (`"argv":
   ["Validation", "code", "uses", ...]`) and got
   `command failed to start: [Errno 2] No such file or directory:
   'Validation'`. Confirmed root cause: the card was hand-authored outside
   the standard `scripts/delegate-low-rri.py` generator, with no validation
   catching a non-command string before dispatch.
2. **`s1b-source-repair-card.json`** — `allowed_paths` includes
   `crates/p2p/src/lib.rs`, but the card's own `spec` text says "Repair
   **only** crates/p2p/src/package_writer.rs". `scope_check` then flagged
   `lib.rs` as `offending_paths` despite it being present in `allowed_paths`
   — i.e. the enforced scope did not match the declared one, for reasons
   not yet root-caused (see T2).
3. **`s1b-tree-compare-repair-transcript.json`** — Devstral (confirmed by
   the sibling file `s1b-source-repair-devstral-transcript.json.ckg-context.json`)
   returned `non-JSON model response: Unterminated string...` when asked to
   emit an `apply_patch` anchor containing a multi-line Rust function
   (`walk_tree`) as a JSON string value. `scripts/local-agent/cli.py:148-153`
   requests Ollama's JSON-schema-constrained decoding
   (`format: TOOL_CALL_JSON_SCHEMA`), which still broke on a long,
   quote-heavy string payload. `session_loop.py`'s existing bounce-and-retry
   (`MAX_MALFORMED_BOUNCES=3`, `run_local_task.py:113`) consumes a turn per
   bounce; the leaf's own budget was `max_total_turns=3`, leaving little
   room to recover.

Two of the three findings are **packet-construction bugs with no runtime
guard**, not model-capability failures — worth fixing regardless of which
local model is bound to any band. The third is a genuine, model-agnostic
fragility in the `apply_patch` JSON contract on long anchor strings.

A fourth, separate gap: there is no consolidated track record of Devstral's
outcomes across the RRI 26-45 tasks it has run since the ADR-045 migration
(2026-09-07) — `grep` across `docs/plan/roadmap.md` for `devstral` finds
only the migration note itself. Any future decision to extend local-first
implementation into Med-high 46-55 or Complex 56-70 (raised in a prior
conversation, out of scope here) would need that evidence base; this plan
does not build it, but T4 assembles it as a read-only prerequisite.

## Objective

Close the three confirmed defects in the shared local-agent delegation
pipeline (`scripts/local-agent/cli.py`, `scope_check.py`, `session_loop.py`)
so that (a) a malformed `acceptance_tests` entry is rejected before dispatch
instead of burning a session on an unrunnable command, (b) the
`allowed_paths` vs. enforced-scope contradiction is either explained or
fixed, and (c) a JSON-anchor decode failure on a long Rust body degrades to
a `write_file` fallback instead of exhausting the turn budget. Separately,
assemble a factual Devstral RRI 26-45 track record (T4) as a prerequisite
for any later governance decision about expanding local-first bands — that
decision itself is out of scope for this plan.

## Non-goals

- No ADR change. None of these fixes alter RRI, band routing, review
  chains, or the ADR-038 46-55 cloud-only rule.
- No expansion of local-first implementation into 46-55 or 56-70. T4
  produces evidence for that future decision; it does not make it.
- No change to which model is bound to which band (ADR-045/046 bindings
  unchanged).

## Affected files

- `scripts/local-agent/cli.py` — card loading / test-runner invocation (T1).
- `scripts/local-agent/scope_check.py` — diff-scope enforcement (T2).
- `scripts/local-agent/session_loop.py` — turn loop, `MalformedToolCall`
  handling (T3a, T3b).
- `scripts/local-agent/run_local_task_test.py`,
  `scripts/local-agent/scope_check_test.py` — new/extended test coverage.
- `docs/audit/devstral-26-45-track-record-2026-09.md` — new, T4's output.

## Design decisions

- **Fail closed, don't guess.** T1 rejects an unrunnable `acceptance_tests`
  entry with a clear error at card-load time rather than trying to be
  clever about repairing it — a bad packet should stop before it wastes a
  session, matching the repo's existing fail-closed posture elsewhere in
  this pipeline (`BoundaryViolation`, `MalformedToolCall`).
- **Investigate before fixing (T2).** The `allowed_paths` contradiction
  might be a real bug in `_is_allowed`/`_git_paths`, or an artifact of a
  shared disposable worktree picking up another leaf's uncommitted change.
  T2's acceptance criteria require root-causing it first; the fix itself is
  conditional on what's found.
- **Narrow, cause-specific fallback (T3a/T3b).** The `write_file` fallback
  triggers only for `MalformedToolCall` raised from a `json.JSONDecodeError`
  specifically (long/malformed anchor content), never for other
  `MalformedToolCall` causes (unknown tool name, missing argument) — those
  indicate a genuinely confused model turn, not a JSON-transport limit, and
  should keep bouncing back for a normal retry.
- **RRI decomposition, not score management.** `scripts/rri.py` scored the
  combined "malformed-JSON fallback" change at RRI 70 (Complex), which
  mandates decomposition before implementation per
  `docs/policies/RRI_POLICY.md`. Rather than adjusting inputs to dodge that
  gate, it is split into T3a (classify the failure cause) and T3b (the
  fallback behavior itself), each independently scored at RRI 55 (Med-high).
  See each task's RRI block for the exact command and inputs.

## Routing note (read before approving)

All four development tasks (T1, T2, T3a, T3b) score **RRI 55, Med-high**
under the real `scripts/rri.py` v2 formula — driven by the ICI
bottleneck from touching 4-5 files each (implementation + test + this
plan/tasks pair), not by cyclomatic complexity, which is low in every case.
Under **ADR-038 Amendment 1**, Med-high 46-55 has no whole-task local repair
budget: each task routes through the Architect-refined single-attempt gate
(Qwen3.6 27B advisory → primary route receipt, `GO_LOCAL` or
`CLOUD_REQUIRED`), and per Amendment 4, a `GO_LOCAL`/`CLOUD_REQUIRED` result
first attempts Low-band decomposition before any cloud takeover. This is
worth flagging plainly: fixes meant to harden the *local* pipeline are
themselves not guaranteed a local-first implementation path under current
policy. T4 is RRI 25 (Low) and follows the ordinary Low-band route.

## Related

- `docs/tasks/local-agent-packet-hardening.md` — task ledger
- `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md`
- `docs/adr/ADR-045-devstral-local-implementer-binding.md`
- `docs/policies/RRI_POLICY.md`
- `.agent/p2-t3c/s1b-validation-repair-card.json`,
  `s1b-validation-repair-transcript.json`,
  `s1b-source-repair-card.json`, `s1b-source-repair-transcript.json`,
  `s1b-tree-compare-repair-transcript.json` — primary evidence
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md` §
  "P2.T3c-S1b — Rust P2P package materializer (Leaf A)" — the closure
  record whose stated decomposition rationale (shared control-flow graph)
  does not fully match what these artifacts show happened (mechanical
  dispatch failures); not contested here, only noted as a documentation-
  completeness gap.
