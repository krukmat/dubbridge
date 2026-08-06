---
type: Audit
title: "RRI evidence: T3d - Integrate T3a+T3b+T3c-2 behind touchpoint packet construction"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t3d
date: 2026-08-06
---

# T3d RRI evidence

Task: T3d — Integrate T3a + T3b + T3c-2 behind touchpoint-facing packet construction
Mode: pre-execution; no implementation diff
Date: 2026-08-06

Command:

```bash
python3 scripts/rri.py \
  --touches scripts/antares/cwe_watchlist.py \
  --touches scripts/antares/packet_schema.py \
  --touches scripts/antares/context_closure.py \
  --touches scripts/antares/governing_boundary_closure.py \
  --touches scripts/antares/governing_boundary_map.py \
  --cc 14 \
  --D 3 --K 3 --P 4 \
  --T 3 --A 1 --X 3
```

Output:

```text
**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | raw CC 14 -> score 2 (policy CC table) | High |
| F files | 2 | --touches -> 5 files | High |
| D domain | 3 | agent-supplied (no rubric match) | High |
| T coverage | 3 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 3 | agent-supplied (no rubric match) | High |
| P impact | 4 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 51
**Penalties applied:** none
**Final RRI:** 51 -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered
```

## Judgment rationale (agent-supplied variables)

- **D=3** — same security-boundary-construction domain as T3b (27) and T3c-2
  (49); the packet these entrypoints build is the artifact Antares actually
  consumes, one layer more consequential than either predecessor alone.
- **K=3** — this task's entire purpose is coupling three already-independent
  modules (`cwe_watchlist.py`, `packet_schema.py`, `context_closure.py` +
  `governing_boundary_closure.py`) behind shared entrypoints; higher than any
  single predecessor's own K score by construction.
- **P=4** — matches T3b/T3c-2's impact judgment: a defect here changes what
  Antares can see (data-visibility risk boundary), same anchor as the
  hard-exclusion and boundary-closure predecessors.
- **T=3** — no existing test suite exercises the entrypoints themselves (they
  do not exist yet); `context_closure_test.py`,
  `governing_boundary_closure_test.py`, and `packet_schema_test.py` cover the
  three inputs in isolation but not their composition, which is the actual
  net-new surface here.
- **A=1** — low ambiguity: `packet_schema.py`'s own docstring already reserves
  the four `context_closure_*` omission reasons this task must emit, and
  `governing_boundary_closure.py`'s docstring explicitly names "T3d" as the
  consumer that "builds packets or decides size-budget policy." The
  integration contract is already pinned by the predecessors; T3d does not
  invent new schema.
- **X=3** — requires holding the Rust crate-boundary map
  (`governing_boundary_map.py`), the Python import/manifest closure algorithm
  (`context_closure.py`), and the packet size-budget/exclusion contract
  (`packet_schema.py`) simultaneously to wire them correctly — narrower than
  T3c's original X=4 (which also had to design the closure algorithm itself)
  but still multi-subsystem.

No penalties applied: this is not a refactor-with-behavior-change (all three
inputs are already implemented and tested), not a schema/architecture
decision (the schema was fixed by T3a/T3b/T3c-2's own contracts), and
verification (unit tests against the new entrypoints) is planned as part of
the task, not absent.
