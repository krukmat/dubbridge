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
