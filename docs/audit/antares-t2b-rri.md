---
type: Audit
title: "RRI evidence: T2b command allowlist and canonical path containment"
status: proposed
slice: antares-security-specialist-advisor
---

# RRI evidence — T2b (task-presentation time)

Computed via `scripts/rri.py`, pre-implementation (no diff exists yet).

```
python3 scripts/rri.py \
  --touches scripts/antares/command_policy.py \
  --touches scripts/antares/path_containment.py \
  --auto-cc \
  --D 3 --K 3 --P 3 \
  --T 1 --A 1 --X 1 \
  --penalty no_verification
```

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | auto-cc fallback (score=0): no local .rs files in --touches; clippy skipped — Low confidence, same limitation as T2a | Low |
| F files | 1 | --touches -> 2 files | High |
| D domain | 3 | agent-supplied (no anchor-rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 3 | agent-supplied (no anchor-rubric match) | High |
| P impact | 3 | agent-supplied (no anchor-rubric match) | High |
| X context | 1 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 35
**Penalties applied:** `no_verification` (+15, manual — no test/verification evidence exists yet at presentation time)
**Final RRI:** 50 -> band **Med-high (41-55)** -> Effort L
**Model tiers:** Codex Balanced->Premium, Claude Balanced->Premium, thinking On
**Decomposition:** not triggered

## D/K/P judgment rationale (relative to T2a's 45)

- **D=3 (domain):** same order as T2a — this module still operates on
  untrusted, model-derived data (candidate argv and paths) that has already
  passed T2a's parser, but has not yet been validated against execution
  policy. No anchor-rubric row matches a bespoke security-policy/containment
  module.
- **K=3 (coupling, +1 vs T2a's 2):** this module's output (`command plan` /
  `containment-valid` marker) is the single gate T2c's sandbox runner must
  trust before invoking anything; T2a's terminal-state envelope is consumed
  more passively downstream, whereas T2b's validated-plan output directly
  authorizes execution.
- **P=3 (impact, +1 vs T2a's 2):** a defect here (an allowlist gap, an
  option-level bypass, or a symlink/`..` containment miss) is the direct
  proximate cause of a sandbox escape or unauthorized command execution —
  unlike T2a, whose worst-case failure is malformed *state*, not unauthorized
  *action*.

## Note on C (cyclomatic) confidence

Same limitation as `docs/audit/antares-t2a-rri.md`: `--auto-cc` only measures
Rust via clippy under the `dubbridge` platform profile; `scripts/antares/` is
Python, so C fell back to score 0 at Low confidence. Does not change the band:
D=3 + K=3 + P=3 combined with the `no_verification` penalty places the task at
50, well inside Med-high with margin before C could move it.

## Advisory notes from script

- `scripts/antares/command_policy.py`: no anchor-rubric match — agent judgment governs D/P/K
- `scripts/antares/path_containment.py`: no anchor-rubric match — agent judgment governs D/P/K
