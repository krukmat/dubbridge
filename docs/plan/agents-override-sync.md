---
type: Plan
title: "Plan: AGENTS.override.md generation and drift guard"
status: proposed
---
# Plan: AGENTS.override.md generation and drift guard

**Roadmap position:** Cross-cutting governance tooling, sibling to the doc-consistency
guardrails work (`docs/plan/doc-consistency-guardrails.md`). Not on the media
pipeline; protects the integrity of Codex's native-instruction source.

## Problem

`scripts/agent-preflight.py` hardcodes two different native-instruction mechanisms
per agent:

- Claude Code: `native_instruction_mechanism = "@import"`, `native_instruction_path
  = "CLAUDE.md"` — `CLAUDE.md` natively imports `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
  and `docs/policies/HITL_AUTONOMY_POLICY.md` at load time (lines 524, 576-577).
- Codex: `native_instruction_mechanism = "generated-bundle"`, `native_instruction_path
  = "AGENTS.override.md"` (lines 525, 623-624) — because Codex has no equivalent
  `@import` capability, this file exists as a **manually maintained, full-text
  concatenation** of `AGENTS.md` + `AGENT_WORKFLOW_GUIDE.md` (confirmed by direct
  read: line 1 of `AGENTS.override.md` is `# AGENTS.md`, line 226 is `# Agent
  Workflow Guide` with its own frontmatter intact, and the file ends at the same
  `## Related` section as `AGENT_WORKFLOW_GUIDE.md`).

Two concrete failure modes follow from "manually maintained":

1. **Drift on edit** — editing `AGENT_WORKFLOW_GUIDE.md` (or `AGENTS.md`) does not
   propagate to `AGENTS.override.md`. This already happened once: the T6 task in
   `docs/tasks/antares-security-specialist-advisor.md` found `AGENTS.override.md`
   carrying stale pre-T4/T5 Antares text after `AGENT_WORKFLOW_GUIDE.md` had moved
   on, and had to hand-mirror four edits into it as an in-flight scope addition.
2. **Missing source** — `AGENTS.override.md` contains zero occurrences of "HITL
   Autonomy Policy" (`grep -c` = 0). Claude Code's `@import` chain pulls in
   `HITL_AUTONOMY_POLICY.md`; the Codex bundle never did. Codex sessions have never
   received that policy as native instruction.

Nothing today detects either failure. `make qa-docs` (`scripts/check-doc-consistency.sh`)
checks ADR index/reference parity but has no notion of `AGENTS.override.md` at all.

## Objective

Make `AGENTS.override.md` a **generated artifact with an enforced freshness gate**,
mirroring the existing pattern in `scripts/check-doc-consistency.sh` (deterministic,
report-and-fail, no auto-fix, wired into `make qa-docs`) rather than inventing a new
enforcement style.

1. **Generator** (`scripts/generate-agents-override.py`): deterministically
   concatenates `AGENTS.md` + `AGENT_WORKFLOW_GUIDE.md` + `HITL_AUTONOMY_POLICY.md`
   (closing the missing-source gap) with a fixed `---` separator, matching the
   existing seam format already present in the file. Fails closed if any source
   file is missing or unreadable.
2. **Drift check** (new function inside `scripts/check-doc-consistency.sh`, no new
   Makefile target per the repo's Makefile-simplicity convention — the script is
   already invoked directly by `make qa-docs`): regenerates the expected content in
   memory and compares it byte-for-byte against the committed `AGENTS.override.md`.
   Reports and fails; never auto-writes.
3. **One-time regeneration** of `AGENTS.override.md` itself, closing the current
   drift and the HITL-policy gap in the same change.

## Scope

### Included
- `scripts/generate-agents-override.py` — new, deterministic, no LLM.
- Drift-detection function added to `scripts/check-doc-consistency.sh`, invoked by
  the existing `qa-docs` Makefile target (no new target).
- Regenerated `AGENTS.override.md` (adds the missing `HITL_AUTONOMY_POLICY.md`
  section; content elsewhere unchanged).

### Excluded (deferred)
- Regenerating on pre-commit/pre-push automatically (the check fails closed; a
  human or agent runs the generator and commits the result, same discipline as
  `check-doc-consistency.sh`'s other checks).
- Any change to `AGENT_WORKFLOW_GUIDE.md`'s or `HITL_AUTONOMY_POLICY.md`'s own
  content — this task changes only how `AGENTS.override.md` is produced and
  verified.
- Extending the generator to any file beyond the three current sources.

## Design decisions

- **Separator format**: reuse the exact `---` line already present at the seam
  between `AGENTS.md` and `AGENT_WORKFLOW_GUIDE.md` content (confirmed at
  `AGENTS.override.md:225`), so the one-time regeneration is a pure superset diff
  (adds the HITL section) rather than a reformat of existing content.
- **Fail-closed generation**: the generator errors out (non-zero exit, no partial
  write) if `AGENTS.md`, `AGENT_WORKFLOW_GUIDE.md`, or `HITL_AUTONOMY_POLICY.md` is
  missing or empty — matching ADR-026's "compiled defaults forbidden" fail-closed
  spirit applied to doc tooling.
- **No Makefile target for the generator** — invoked directly
  (`python3 scripts/generate-agents-override.py`), consistent with prior guidance
  to keep the Makefile free of thin wrappers around directly-invocable scripts.
  Only the drift *check* rides inside the existing `qa-docs` chain.

## Module dependencies

`scripts/generate-agents-override.py` has no dependency on
`scripts/check-doc-consistency.sh`; the drift check imports/shells out to the
generator to get the expected content rather than re-implementing the
concatenation logic, so there is exactly one place that knows the seam format.

## Related

- `docs/tasks/agents-override-sync.md` (this plan's task ledger)
- `docs/plan/doc-consistency-guardrails.md` — precedent for the
  report-and-fail / no-auto-fix / `make qa-docs` wiring style
- `docs/tasks/antares-security-specialist-advisor.md` § T6 — the drift incident that
  motivated this plan
- `scripts/agent-preflight.py` — defines the two native-instruction mechanisms this
  plan keeps in sync
