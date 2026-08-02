---
type: Audit
title: "RRI evidence: T3a - Versioned CWE watchlist"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t3a---versioned-cwe-watchlist
date: 2026-08-01
---

# RRI evidence: T3a — Versioned CWE watchlist

Task: `docs/tasks/antares-security-specialist-advisor.md` § T3a
Depends on: T2e (`[x] Done (owner-waived, 2026-07-30)`), decomposition of T3 (proposed 2026-08-01)

## Presentation-time computation (2026-08-01, pre-implementation)

```bash
python3 scripts/rri.py \
  --touches scripts/antares/cwe_watchlist.py \
  --touches scripts/antares/cwe_watchlist_test.py \
  --auto-cc \
  --D 1 --K 1 --P 2 \
  --T 1 --A 1 --X 1 \
  --penalty no_verification
```

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | auto-cc fallback (score=0): no local .rs files in --touches; clippy skipped | Low |
| F files | 1 | `--touches` -> 2 files | High |
| D domain | 1 | agent-supplied (no rubric match) — a static, validated list of CWE entries with metadata; simpler than T2d's artifact-schema work (D=2) because there is no runtime trace/redaction contract, just data + validation | High |
| T coverage | 1 | agent-supplied — validator + fixture tests are the deliverable itself | High |
| A ambiguity | 1 | agent-supplied — scope is narrow (schema, entries, CWE-732 exclusion); only the concrete file/data location is an open interpretation | High |
| K coupling | 1 | agent-supplied (no rubric match) — isolated module, no filesystem traversal, no process/network side effects beyond loading its own static data | High |
| P impact | 2 | agent-supplied (no rubric match) — a malformed or unauthorized entry could let a downstream task treat an unjustified CWE as in-scope, but this task performs no execution and touches no secrets directly (that boundary belongs to T3b) | High |
| X context | 1 | agent-supplied — one self-contained module | High |

**Base value:** 100 x (weighted / 5) = 22
**Penalties applied:** `no_verification` (+15, manual flag — no diff exists yet)
**Final RRI: 37 -> band Moderate (26-40) -> Effort M. Codex Balanced. Claude Balanced. thinking Off**
**Gates for this band:** Confirm tests exist in the affected area. **Implementation route:** local-first via `scripts/local-agent/run_local_task.py` + `DUBBRIDGE_LOCAL_AGENT_MODEL` (default `qwen3.6:35b-a3b`); primary agent remains orchestrator, cloud implementation is escalation/fallback only.
**Decomposition:** not triggered — within split target (RRI <= 55, A=1).

## Notes

- This subtask is the narrowest of the T3 -> T3a..T3d split proposed in
  `docs/tasks/antares-security-specialist-advisor.md` § T3 decomposition
  record. It has no dependency on repository traversal or packet
  construction, so it can be implemented first and independently.
- Phase-1/phase-2 reviewer for this band (owner directive 2026-07-21):
  `qwen3.6:27b-q4_K_M`, falling back to Gemma then D14 if unavailable.
