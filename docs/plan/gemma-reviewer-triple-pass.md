---
type: Plan
title: "Plan: Gemma Reviewer Triple-Pass Reconciliation"
status: superseded
superseded_by: docs/plan/gemma-audit-and-triple-pass.md
---
# Plan: Gemma Reviewer Triple-Pass Reconciliation

> **Status:** SUPERSEDED by `docs/plan/gemma-audit-and-triple-pass.md`, which
> folds this triple-pass work into a single slice with the Gemma process-audit
> telemetry and resolves the gaps left open here (RRI, task ledger, aggregate
> schema, quorum/partial-failure policy, reconciliation algorithm, exit codes,
> artifact naming, latency budget, and the affected governance docs). Kept for
> history; do not implement from this document.
> **Related prior slice:** `docs/plan/low-medium-gemma-code-review-role.md`
> **Tasks ledger (superseded):** `docs/tasks/gemma-reviewer-triple-pass.md`

## Objective

Upgrade the local `Gemma Reviewer` wrapper from a single advisory review run to a
**three-pass review sequence** that produces:

1. three raw per-pass review artifacts;
2. one primary wrapper result that contrasts those three passes;
3. one explicit reflection/reconciliation step that detects disagreements,
   duplicate findings, likely false positives, and reporting inconsistencies.

The role remains review-only and advisory: it does not write files, approve
tasks, or replace the primary agent's Reflection cycle.

## Scope

### Included

- extend `scripts/gemma-code-review.py` to run three review passes per invocation;
- persist three per-pass artifacts plus one aggregated result artifact;
- add a reconciliation/reflection step that compares the three reports and flags:
  - consensus findings;
  - pass-specific findings;
  - inconsistent severities or line numbers for the same issue;
  - likely false positives caused by misunderstandings of packet structure;
- keep the wrapper's read-only contract and non-patch parser guarantees;
- update tests, local docs, workflow docs, and task evidence expectations.

### Excluded

- changing the approval boundary or RRI bands;
- allowing Gemma Reviewer to write or auto-fix code;
- making remote CI depend on Ollama;
- changing Gemma Developer delegation semantics.

## Design decisions

### D1 — Three independent passes, one invocation contract

The wrapper should make three separate review calls using the same packet and
the same review contract, rather than asking the model to self-simulate three
opinions inside one completion. Each pass produces an independent artifact.

### D2 — Reconciliation is wrapper-owned, not model-owned

The reconciliation/reflection step should be deterministic Python logic in the
wrapper so the contrast step is inspectable and testable. The model may disagree;
the wrapper owns comparison and summary.

### D3 — Preserve advisory authority

The aggregated result still returns advisory findings only. The primary agent
remains responsible for accepting, rejecting, or repairing based on the reports.

### D4 — Report shape expands, single-pass contract remains parseable

The existing single review response format (`PASS|FINDINGS|BLOCKED` + finding
blocks) stays valid per pass. The wrapper adds a higher-level aggregate JSON
shape rather than changing the model's tagged output contract.

### D5 — Reflection evidence must surface disagreements

When the three passes disagree, the aggregate artifact must make that explicit so
the agent can contrast the reports instead of silently collapsing them.

## Affected files

| Layer | Path | Change |
|---|---|---|
| Wrapper | `scripts/gemma-code-review.py` | three-pass execution + reconciliation |
| Tests | `scripts/gemma_code_review_test.py` | parser/aggregation/CLI tests |
| Make | `Makefile` | reviewer target docs/output expectations if needed |
| Workflow docs | `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` | reviewer evidence/reporting expectations |
| Local Gemma docs | `docs/gemma-local-improve.md` | active contract summary |
| Task ledger | `docs/tasks/gemma-reviewer-triple-pass.md` | progress/evidence ledger |

## Verification

- `python3 -m unittest scripts/gemma_code_review_test.py`
- `make qa-docs`
- one local dry-run or live reviewer invocation that proves:
  - three pass artifacts are emitted;
  - the aggregate artifact contains reconciliation/reflection fields;
  - inconsistent findings are surfaced, not hidden.
