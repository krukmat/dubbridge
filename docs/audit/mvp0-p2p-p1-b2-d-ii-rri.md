---
type: Audit
title: "P1.B2.d-ii — RRI report"
date: 2026-08-31
task: MVP0-P2P-P1-B2-D-II
---

# P1.B2.d-ii — RRI report

## Command

```bash
python3 scripts/rri.py --platform rn --cc 5 --touches mobile/src/p2p/proof/ReplicationProofRunner.ts --touches mobile/__tests__/p2p/replication-cleanup.test.ts --D 3 --T 1 --A 0 --K 2 --P 1 --X 2
```

## Result

```text
**Platform:** rn

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 0 | raw CC 5 -> score 0 (policy CC table) | High |
| F files | 1 | --touches -> 2 files | High |
| D domain | 3 | agent-supplied (no rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 2 | agent-supplied (no rubric match) | High |
| P impact | 1 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 24
**Penalties applied:** none
**Final RRI:** 24 -> band Low (0-25) -> Effort S . Codex Local Qwen Developer via Ollama . Claude Local Qwen Developer via Ollama . thinking Off
**Gates for this band:** Local delegation: delegate to local Qwen Developer via Ollama; validate and apply only an in-scope diff; review against requirements; verify; report.
**Decomposition:** not triggered
```
