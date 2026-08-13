---
type: Audit
title: "Gemma/Peer review evidence overrides ledger"
status: open
---
# Gemma/peer review evidence overrides ledger

Append-only. Every task-file section that closes with a `REVIEW-OVERRIDE:`
line (instead of a `Review artifact:` receipt reference) must have a
matching row here, keyed by task ID. This is the human-auditable trail for
skipping the Gemma Reviewer / cross-vendor peer review evidence gate — see
`docs/tasks/gemma-evidence-artifact-gate.md` (GEG-1c) and
`docs/policies/RRI_POLICY.md`.

Rows are never edited or removed after the fact; corrections are appended
as new rows with a note referencing the original.

## Ledger

| Task ID | Override type | Reason | Waiver-by / Failed-attempt / Scope-note | Date |
|---|---|---|---|---|
| GEG-TEST-URGENCY | urgency | synthetic validator test row | Waiver-by: matias | 2026-07-22 |
| GEG-TEST-PIPEFAIL | pipeline-failure | synthetic validator test row | Failed-attempt: local-agent malformed_tool_call_repeated | 2026-07-22 |
| GEG-TEST-NOTAPP | not-applicable | synthetic validator test row | Scope-note: synthetic fixture | 2026-07-22 |
| review-gate-diff-path-scoping | not-applicable | config-only Makefile wiring change (opt-in `REVIEW_PATHS` pathspec + existing tested `--files` flag; no new logic) — exempt from Step 1 mandatory code-solution review per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Development task closure checklist` config-only exemption | Scope-note: no domain/application code touched, task type declared `config` in the task ledger | 2026-07-30 |
| LASS-0 | not-applicable | ADR/policy/docs-only strict-scope contract ratification; exempt from phase reviews | Scope-note: no executable code changed in LASS-0 | 2026-08-13 |
