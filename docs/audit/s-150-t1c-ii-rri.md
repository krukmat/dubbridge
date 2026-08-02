---
type: Audit
title: "RRI evidence: S-150-T1c-ii localization repositories and readiness evidence"
status: active
task: S-150-T1c-ii
date: 2026-08-02
---

# S-150-T1c-ii — Presentation-time RRI

## Scoped presentation surface

- `crates/db/src/translation_repo.rs`
- `crates/db/src/dubbing_repo.rs`
- `crates/db/src/artifact_repo.rs`
- `crates/db/src/lib.rs`
- `apps/api/tests/localization_repo_test.rs`

This presentation assumes one shared repository test file covers both the
translation and dubbing repository contracts. `S-150-T1c-ii` explicitly excludes
new migrations; it consumes the `0028` schema from `S-150-T1c-i`.

Two repository files and the shared test file do not exist yet, so the
cyclomatic input cannot be auto-measured on a real diff. The `C` score below is
therefore an estimate derived from the neighboring repository shapes in
`crates/db/src/{subtitle,transcription}_repo.rs` and is marked Low confidence
per the RRI policy.

If implementation expands to two test files or requires reading oversized
neighbor files (for example `apps/api/tests/subtitle_repo_test.rs` at 639 lines
or `crates/domain/src/artifact.rs` at 734 lines) as part of the local authoring
surface, rerun this RRI and re-check the Med-high local gate before execution.

## Command

```bash
python3 scripts/rri.py \
  --C 1 \
  --low-confidence C \
  --T 2 \
  --A 1 \
  --X 4 \
  --D 3 \
  --K 3 \
  --P 3 \
  --touches crates/db/src/translation_repo.rs \
  --touches crates/db/src/dubbing_repo.rs \
  --touches crates/db/src/artifact_repo.rs \
  --touches crates/db/src/lib.rs \
  --touches apps/api/tests/localization_repo_test.rs \
  --platform dubbridge
```

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | agent-supplied score | Low |
| F files | 2 | `--touches` -> 5 files | High |
| D domain | 3 | anchor rubric: `crates/db` (ADR-006, ADR-018) -> floor 3 (agent 3 kept) | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 3 | anchor rubric: `crates/db` (ADR-006, ADR-018) -> floor 3 (agent 3 kept) | High |
| P impact | 3 | anchor rubric: `crates/db` (ADR-006, ADR-018) -> floor 3 (agent 3 kept) | High |
| X context | 4 | agent-supplied | High |

- Base value: `47`
- Penalties applied: `none`
- Final RRI: `47`
- Band: `Med-high (41-55)`
- Effort: `L`
- Codex capability: `Balanced -> Premium`
- Claude capability: `Balanced -> Premium`
- Thinking mode: `On`

## Implication

`S-150-T1c-ii` requires the full approval presentation before implementation and
keeps the Med-high closure gates:

- explicit human approval before any editing
- phase-1 and phase-2 review via `qwen3.6:27b-q4_K_M`, falling back to Gemma and
  then D14 if unavailable
- 3 Reflection passes at closure
- ADR-038 Med-high implementation routing after approval
