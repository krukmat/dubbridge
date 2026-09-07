---
type: Audit
title: "P1.B2.e — RRI report"
date: 2026-08-31
task: MVP0-P2P-P1-B2-E
---

# P1.B2.e — RRI report

## Command

```bash
python3 scripts/rri.py --platform rn --cc 4 --touches mobile/src/p2p/proof/replication-verdict.ts --touches mobile/__tests__/p2p/replication-verdict.test.ts --D 2 --T 2 --A 0 --K 0 --P 2 --X 2
```

## Result

```text
**Platform:** rn

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 0 | raw CC 4 -> score 0 (policy CC table) | High |
| F files | 1 | --touches -> 2 files | High |
| D domain | 2 | agent-supplied (no rubric match) | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 0 | agent-supplied (no rubric match) | High |
| P impact | 2 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 21
**Penalties applied:** none
**Final RRI:** 21 -> band Low (0-25) -> Effort S . Codex Local Qwen Developer via Ollama . Claude Local Qwen Developer via Ollama . thinking Off
**Gates for this band:** Local delegation: delegate to local Qwen Developer via Ollama; validate and apply only an in-scope diff; review against requirements; verify; report.
**Decomposition:** not triggered
```
