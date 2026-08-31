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
| S-230-T4p-R2 | not-applicable | user waived Phase 2 Gemma review ("lo damos por bueno, cerremos la task as is"); untracked file produced no `git diff` for `make qa-gemma-review`; 2 Reflection passes completed with no findings | Scope-note: user-directed waiver after structural verification (bash -n passes) and 2 Reflection passes | 2026-08-24 |
| X26-T3c-b1 | urgency | owner directed direct completion and closure after prolonged local-model attempts exceeded the operational budget | Waiver-by: matias | 2026-08-31 |
| X26-T3c-b2 | urgency | owner directed direct completion without further local-model protocol delay | Waiver-by: matias | 2026-08-31 |
| X26-T3c-b2-superseded | not-applicable | correction of the row above: the local stack (Ollama/Muse Glimmer) was healthy at close time, evidenced by the sibling phase-1 artifact from the same session; a genuine phase-2 review was subsequently run and passed with 0 findings, replacing the urgency waiver with a real `Review artifact:` receipt | Scope-note: see `docs/audit/local-delegation/x26-t3c-b2-phase2-review.json`; the original row is retained per this ledger's append-only rule | 2026-08-31 |
