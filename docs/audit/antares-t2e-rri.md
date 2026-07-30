---
type: Audit
title: "RRI evidence: T2e - Replay fixtures and integrated harness verification"
status: active
---

# RRI evidence: T2e — Replay fixtures and integrated harness verification

Task: `docs/tasks/antares-security-specialist-advisor.md` § T2e
Depends on: T2e-pre (`[x] Done`)

## Presentation-time computation (2026-07-30, pre-implementation)

```bash
python3 scripts/rri.py \
  --touches scripts/antares/tool_call_parser.py \
  --touches scripts/antares/terminal_state.py \
  --touches scripts/antares/command_policy.py \
  --touches scripts/antares/path_containment.py \
  --touches scripts/antares/sandbox_runner.py \
  --touches scripts/antares/sandbox_budget.py \
  --touches scripts/antares/artifact_schema.py \
  --auto-cc \
  --D 2 --K 4 --P 3 \
  --T 1 --A 1 --X 2 \
  --penalty no_verification
```

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | auto-cc fallback (score=0): no local .rs files in --touches; clippy skipped | Low |
| F files | 3 | `--touches` -> 7 files | High |
| D domain | 2 | agent-supplied (no rubric match) — same as T2d; harness composes existing, already-governance-reviewed layers, does not add new invariants | High |
| T coverage | 1 | agent-supplied — regression/replay tests are the deliverable itself | High |
| A ambiguity | 1 | agent-supplied — acceptance criteria and HP/EC set are fully specified in the task text, phase-1 review already `PASS` | High |
| K coupling | 4 | agent-supplied — raised above T2d's 3: this is the first task that composes all four prior layers (parser/terminal-state, policy/containment, sandbox runner/budget, artifact schema) in one entrypoint, higher integration coupling than any single-layer prior task | High |
| P impact | 3 | agent-supplied (no rubric match) — same as T2d; incorrect composition could silently blur fail-closed boundaries across layers | High |
| X context | 2 | agent-supplied — same as T2d | High |

**Base value:** 100 x (weighted / 5) = 40
**Penalties applied:** `no_verification` (+15, manual flag — no diff exists yet)
**Final RRI: 55 -> band Med-high (41-55) -> Effort L. Codex Balanced->Premium. Claude Balanced->Premium. thinking On**
**Decomposition:** not triggered (RRI 55 is within Med-high; Complex-band mandatory decomposition triggers at 56+)
**Gates for this band:** Plan + explicit acceptance criteria required before approval; ADR-038 Architect-refined single-attempt gate for implementation routing.

## Notes

- K was raised one point above the T2d precedent (2 -> now 4) to reflect that
  T2e is explicitly the first subtask integrating all four T2 layers at once,
  per the task's own Objective text.
- RRI lands at the top edge of Med-high (55/55). If implementation-time
  evidence (actual diff, real coverage numbers) pushes this above 55, the task
  must be re-scored post-implementation per policy and, if it crosses into
  Complex (56+), decomposition becomes mandatory before proceeding.
