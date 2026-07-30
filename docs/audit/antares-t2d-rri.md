---
type: Audit
title: "RRI evidence: T2d - Versioned artifact schema and redacted trace contract"
status: active
---

# RRI evidence: T2d — Versioned artifact schema and redacted trace contract

Task: `docs/tasks/antares-security-specialist-advisor.md` § T2d
Depends on: T2c-2 (`[x] Done (owner-waived, 2026-07-30)`)

## Presentation-time computation (2026-07-30, pre-implementation)

```bash
python3 scripts/rri.py \
  --touches scripts/antares/terminal_state.py \
  --touches scripts/antares/sandbox_budget.py \
  --touches scripts/antares/sandbox_runner.py \
  --auto-cc \
  --D 2 --K 3 --P 3 \
  --T 1 --A 1 --X 2 \
  --penalty no_verification
```

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | auto-cc fallback (score=0): no local .rs files in --touches; clippy skipped | Low |
| F files | 2 | `--touches` -> 3 files | High |
| D domain | 2 | agent-supplied (no rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 3 | agent-supplied (no rubric match) | High |
| P impact | 3 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 35
**Penalties applied:** `no_verification` (+15, manual flag — no diff exists yet)
**Final RRI: 50 -> band Med-high (41-55) -> Effort L. Codex Balanced->Premium. Claude Balanced->Premium. thinking On**
**Decomposition:** not triggered

## D/K/P judgment rationale (relative to T2c-2's 51/53)

- **D=2 (domain, -1 vs T2c-2's D=3):** T2c-2 supervises live OS process
  resource limits and teardown — kernel-boundary enforcement. T2d is schema
  normalization and validation: it consumes already-produced `TerminalState`
  values and serializes them into a versioned, redacted artifact. No process
  spawning, no privilege/network boundary, no live resource accounting. This
  is a materially lower-risk domain than T2c-1/T2c-2, closer to T2a's
  parsing/validation shape (T2a scored D=2 per `docs/audit/antares-t2a-rri.md`)
  than to T2c's sandbox-execution shape.
- **K=3 (coupling, same as T2c-2):** the schema is the durable contract every
  other component and future consumer (T3 packet construction, T4 pilot
  evidence, human disposition audit) depends on. It must cover every terminal
  state enumerated across T2a/T2b/T2c-1/T2c-2 in `terminal_state.py` — that is
  broad downstream/upstream coupling even though the code itself is simple
  serialization logic.
- **P=3 (impact, same as T2c-2):** a defect here does not escape a sandbox, but
  it can silently leak redacted content (raw trace/prompt/credential material
  into a committed artifact) or make an undisposed finding appear closed by
  omission (EC-3 in the task definition) — both are governance-significant
  failures for an advisory-only, human-disposition-gated tool, even though the
  blast radius is documents/audit trail rather than a running process.

D=2 (lower than T2c-2) offsets K=3/P=3 (same as T2c-2), landing one band lower
in the raw base (35 vs T2c-2's post-recomputation ~38) but the `no_verification`
penalty still places it in Med-high at 50, just below T2c-2's 51/53. This is
consistent with T2d being real security-sensitive tooling (redaction/retention
contract, human-disposition mandatory fields) but not itself a process-isolation
boundary.

## Gates

- RRI 41-55 Med-high: present task card, wait for explicit human approval.
- Implementation routes through ADR-038 Architect-refined single-attempt gate
  (Qwen27 advisory refinement -> primary hash-bound route receipt -> GO_LOCAL
  bounded qwen3.6:35b-a3b session or CLOUD_REQUIRED escalation).
- 3 Reflection passes required at closure.
- Band-routed review (phases 1 and 2): `qwen3.6:27b-q4_K_M` -> Gemma -> D14.
