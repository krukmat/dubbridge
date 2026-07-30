---
type: Audit
title: "RRI evidence: Antares T2 sandboxed agentic harness and artifact schema"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t2---sandboxed-agentic-harness-and-artifact-schema
date: 2026-07-29
---

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | conservative agent-supplied score for bounded parser/sandbox/schema branches | Medium |
| F files | 4 | `--touches` -> 13 planned files | High |
| D domain | 5 | security-sensitive sandbox/runtime boundary | High |
| T coverage | 4 | new area with no existing tests in a high-impact path | High |
| A ambiguity | 0 | task ledger defines objective, HP/EC cases, evidence, and status artifacts | High |
| K coupling | 4 | subprocess/filesystem/runtime/artifact interactions across multiple modules | High |
| P impact | 5 | security, containment, credential isolation, and trace-handling impact | High |
| X context | 4 | several modules plus workflow/policy constraints | High |

**Base value:** 100 x (weighted / 5) = 68
**Penalties applied:** many_files (+8, F=4 >= 4); no_tests_high_impact (+10, T=4 >= 4 and P=5 >= 4)
**Final RRI:** 86 -> band Very high (86-100) -> Effort XL . Codex Premium . Claude Premium . thinking On
**Gates for this band:** Do not implement directly. Produce an ADR + risk analysis + decompose into subtasks.
**Decomposition:** triggered by RRI >= 56, F >= 4 and K >= 3, T >= 4 and P >= 4 — split before implementing

Command run:

```bash
python3 scripts/rri.py \
  --touches scripts/antares/__init__.py \
  --touches scripts/antares/tool_call.py \
  --touches scripts/antares/sandbox.py \
  --touches scripts/antares/runner.py \
  --touches scripts/antares/artifact_schema.py \
  --touches scripts/antares/replay.py \
  --touches scripts/antares_cli.py \
  --touches scripts/antares_tool_call_test.py \
  --touches scripts/antares_sandbox_test.py \
  --touches scripts/antares_runner_test.py \
  --touches scripts/antares_artifact_schema_test.py \
  --touches docs/tasks/antares-security-specialist-advisor.md \
  --touches docs/plan/antares-security-specialist-advisor.md \
  --C 2 --D 5 --K 4 --P 5 --T 4 --A 0 --X 4
```

## Notes

- This is a conservative pre-execution score. A higher realized branch count or a
  larger touched-file set can only increase the execution RRI.
- The planned file set is representative of the current T2 acceptance contract:
  parser, sandbox, runner, schema, replay, and dedicated tests. If the task is
  decomposed, each subtask must recompute its own RRI from its narrower scope.
