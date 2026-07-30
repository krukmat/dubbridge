---
type: Audit
title: "RRI evidence: T2c ephemeral sandbox runner and resource enforcement"
status: proposed
slice: antares-security-specialist-advisor
---

# RRI evidence — T2c (pre-decomposition)

## Command

```bash
python3 scripts/rri.py \
  --touches scripts/antares/sandbox_runner.py \
  --auto-cc \
  --D 4 --K 3 --P 4 \
  --T 1 --A 2 --X 2 \
  --penalty no_verification
```

## Result

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | auto-cc fallback (score=0): no local .rs files in `--touches`; clippy skipped | Low |
| F files | 0 | `--touches` -> 1 file | High |
| D domain | 4 | agent-supplied (no rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 2 | agent-supplied | High |
| K coupling | 3 | agent-supplied (no rubric match) | High |
| P impact | 4 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 41
**Penalties applied:** `no_verification` (+15, manual flag)
**Final RRI: 56 -> band Complex (56-70) -> Effort L. Codex Premium. Claude Premium. thinking On**
**Decomposition:** triggered by RRI >= 56 (hard gate per `docs/policies/RRI_POLICY.md` § Decomposition triggers)

## D/K/P judgment rationale (relative to T2b's 50)

- **D=4 (domain, +1 vs T2b's 3):** T2b validated argv/paths as pure computation —
  no side effects, no process boundary. T2c actually **spawns and supervises OS
  subprocesses** under network isolation, dropped privileges, and resource limits.
  That is a materially different domain: kernel-level isolation guarantees, not
  string/path validation. This is the deciding variable between Med-high and
  Complex, so it is recorded explicitly rather than folded into the total.
- **K=3 (coupling, same as T2b):** the sandbox consumes T2b's `COMMAND_PLAN_VALID`
  output directly and its own `TerminalState` output will be consumed by T2d
  (artifact schema). Same order of downstream coupling as T2b.
- **P=4 (impact, +1 vs T2b's 3):** a defect here is not a malformed-state bug —
  it is a **sandbox escape or resource-exhaustion** class failure: network access
  from an isolated run, privilege retention, teardown that leaves a process
  running, or a resource cap that doesn't actually bound CPU/RAM/PIDs. This is
  the harness's actual security boundary per the plan's Non-negotiable harness
  properties list (`docs/plan/antares-security-specialist-advisor.md` lines
  227-242).

D=4 + K=3 + P=4 combined with the `no_verification` penalty places the task at
RRI 56, one point into the Complex band. Decomposition is mandatory and
unconditional at this threshold — see `docs/policies/RRI_POLICY.md` line 596:
"Final RRI >= 56. This is the default hard gate."

## Decomposition

Split per the policy target (`RRI <= 55` and `A in {0,1}` per subtask):

### T2c-1 — Sandbox process execution and isolation

```bash
python3 scripts/rri.py \
  --touches scripts/antares/sandbox_runner.py \
  --auto-cc \
  --D 4 --K 2 --P 3 \
  --T 1 --A 1 --X 2 \
  --penalty no_verification
```

**Final RRI: 49 -> band Med-high (41-55) -> Effort L. Codex Balanced->Premium. Claude Balanced->Premium. thinking On**

Scope: spawn a validated `COMMAND_PLAN_VALID` argv in an isolated subprocess
(network-disabled, read-only mounts, dropped privileges, credentials stripped),
per-command timeout, bounded stdout/stderr capture, `runtime_unavailable` on
bootstrap failure. Covers HP-1 and EC-2.

### T2c-2 — Resource budget, wall-timeout, and teardown enforcement

```bash
python3 scripts/rri.py \
  --touches scripts/antares/sandbox_runner.py \
  --touches scripts/antares/sandbox_budget.py \
  --auto-cc \
  --D 3 --K 3 --P 3 \
  --T 1 --A 1 --X 2 \
  --penalty no_verification
```

**Final RRI: 51 -> band Med-high (41-55) -> Effort L. Codex Balanced->Premium. Claude Balanced->Premium. thinking On**

Scope: CPU/RAM/PID caps, output-limit breach detection, wall-timeout across the
15-command budget, and guaranteed teardown on timeout, sandbox violation, or
early process exit. Depends on T2c-1's runner existing. Covers HP-2, EC-1, EC-3.

Both subtasks stay within Med-high (41-55): band-routed review (phases 1/2, per
the session's cloud-routing override for this session), 3 Reflection passes, and
the human approval gate apply to each independently. Neither reaches the
decomposition threshold again.
