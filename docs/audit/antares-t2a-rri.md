---
type: Audit
title: "RRI evidence: T2a tool-call parser and terminal-state contract"
status: proposed
slice: antares-security-specialist-advisor
---

# RRI evidence — T2a (task-presentation time)

Computed via `scripts/rri.py`, pre-implementation (no diff exists yet).

```
python3 scripts/rri.py \
  --touches scripts/antares/parser.py \
  --touches scripts/antares/terminal_state.py \
  --auto-cc \
  --D 3 --K 2 --P 2 \
  --T 1 --A 1 --X 1 \
  --penalty no_verification
```

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | auto-cc fallback (score=0): no local .rs files in --touches; clippy skipped — **Low confidence, see note** | Low |
| F files | 1 | --touches -> 2 files | High |
| D domain | 3 | agent-supplied (no anchor-rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 2 | agent-supplied (no anchor-rubric match) | High |
| P impact | 2 | agent-supplied (no anchor-rubric match) | High |
| X context | 1 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 30
**Penalties applied:** `no_verification` (+15, manual — no test/verification evidence exists yet at presentation time)
**Final RRI:** 45 -> band **Med-high (41-55)** -> Effort L
**Model tiers:** Codex Balanced->Premium, Claude Balanced->Premium, thinking On
**Decomposition:** not triggered

## Note on C (cyclomatic) confidence

`--auto-cc` only measures Rust via clippy under this platform profile; `scripts/antares/` is planned as Python. No Python CC measurer is wired into `rri.py`'s `auto-cc` path for this repo, so C fell back to score 0 at Low confidence rather than a true measurement. This does not change the band: D=3 (agent-judged, no anchor-rubric match for a from-scratch untrusted-input parser feeding a security-sensitive pipeline) combined with the `no_verification` penalty already places the task at 45, comfortably inside Med-high with margin before the C variable could move it. Recorded here per the Socratic-doubt communication rule rather than silently treating the fallback as a real measurement.

## D/K/P judgment rationale

- **D=3 (domain):** parses untrusted model-emitted tool-call JSON that will later drive command execution (T2b/T2c); no anchor-rubric row matches a bespoke security-tool parser, so this is agent judgment, not a rubric floor.
- **K=2 (coupling):** the terminal-state envelope produced here is the contract every downstream T2b–T2e module consumes; moderate coupling to the rest of the harness.
- **P=2 (impact):** a parser defect propagates malformed/ambiguous state into the sandboxed execution layer, but T2a itself performs no execution and no filesystem mutation — bounded impact at this task's boundary.

## Advisory notes from script

- `scripts/antares/parser.py`: no anchor-rubric match — agent judgment governs D/P/K
- `scripts/antares/terminal_state.py`: no anchor-rubric match — agent judgment governs D/P/K
