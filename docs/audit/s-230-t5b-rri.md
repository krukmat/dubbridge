---
type: Audit
title: "S-230-T5b RRI at task-presentation time (recomputed, then corrected)"
date: 2026-08-27
task: S-230-T5b
---

# S-230-T5b RRI

## Status: superseded by the correction below

The first recompute below (RRI 59 Complex) was challenged and found
miscalibrated the same day. **The corrected run (RRI 27 Moderate, further
down this file) is the value the task card uses.** Both runs are kept for
the audit trail — retracting a wrong number without recording why it was
wrong would lose the evidence that the correction is itself grounded, not
just a second guess.

**What was wrong:** the first run manually applied the `auth_security`
penalty and raised D/K/P from the anchor-rubric floor of 1 to 3, reasoning
that the task "authors the production secret-boundary contract." That is
not the rubric's test. `auth_security`'s own auto-detection condition in
`scripts/rri.py` (`detect_penalties()`) requires an anchor-rubric **P floor
≥ 4** (the auth/audit/rights/secrets tier of `docs/policies/RRI_POLICY.md` §
DubBridge anchor rubric — `crates/auth`, `apps/gateway/src/auth/**`,
`crates/audit`, rights-ledger, migrations, or the "secrets/credential
storage" row). The row that actually matches these files —
`config/*.toml` with env-wiring logic, `config/README.md` — floors D/P/K at
**1**, not 4. The rubric instructs raising a score above its floor only when
"the specific change within the path" warrants it, not because the subject
matter is conceptually adjacent to authentication or secrets. Manually
forcing the penalty and the D/K/P bump substituted general importance of the
topic for the rubric's own file-anchored test.

## Whole-task recompute — RETRACTED (miscalibrated, kept for the record)

Command:

```bash
python3 scripts/rri.py \
  --touches .env.example \
  --touches config/production.toml \
  --touches config/README.md \
  --touches docs/tasks/s-230-poc-v1-digitalocean.md \
  --C 2 --D 3 --K 3 --P 3 --T 2 --A 2 --X 3 \
  --penalty auth_security
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | agent-supplied score | High |
| F files | 2 | --touches -> 4 files | High |
| D domain | 3 | anchor rubric: config/README.md (ADR-026) -> floor 1 (agent 3 kept) | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 2 | agent-supplied | High |
| K coupling | 3 | anchor rubric: config/README.md (ADR-026) -> floor 1 (agent 3 kept) | High |
| P impact | 3 | anchor rubric: config/README.md (ADR-026) -> floor 1 (agent 3 kept) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 49
**Penalties applied:** auth_security (+10, manual flag)
**Final RRI:** 59 -> band Complex (56-70) -> Effort L . Codex Premium . Claude Premium . thinking On
**Gates for this band:** Plan first. Human reviews the plan before any implementation.
**Decomposition:** triggered by RRI >= 56 — split before implementing (unconditional gate, `docs/policies/RRI_POLICY.md § Decomposition triggers`)

### Scoring notes

- Supersedes the parent-table presentation-time estimate of RRI 49 Med-high
  recorded in `docs/tasks/s-230-poc-v1-digitalocean.md` § "Mandatory child
  decomposition" — that table explicitly notes child scores are estimates
  only and the actual report controls.
- `D`/`K`/`P` are raised above the ADR-026 anchor floor (1) to 3 because this
  task does not merely reference the production secret boundary — it
  authors the contract itself: the five double-underscore auth variables,
  the dead single-underscore legacy reader trap
  (`crates/config/src/lib.rs:361`-`382`), the legacy gateway OAuth secret
  carryover, and translation credential placeholders, applied identically
  across three services (api, gateway, worker-runner) plus the migration
  job.
- `auth_security` applies for the same reason the T5 parent and T5a used it:
  the task defines how JWT/storage/gateway/translation credentials cross the
  production fail-closed boundary, even though it commits no secret value.
- `T=2` reflects that verification exists (config-load parity check,
  secret-pattern scan) but is not yet implemented.
- `A=2` reflects residual ambiguity in exact variable naming/placement even
  though T5a froze the underlying values.

## Split into subtasks (target: each ≤ 55, A ∈ {0,1})

Per `docs/policies/RRI_POLICY.md § Decomposition triggers`, split target is
divide until each subtask scores RRI ≤ 55 with A ∈ {0, 1}. All three land
well under the ceiling on the first split, one file each:

```bash
# T5b-i
python3 scripts/rri.py --touches .env.example \
  --C 2 --D 2 --K 2 --P 2 --T 1 --A 1 --X 2 --penalty auth_security
# Final RRI: 40 -> Moderate (26-40)

# T5b-ii
python3 scripts/rri.py --touches config/production.toml \
  --C 1 --D 2 --K 2 --P 2 --T 1 --A 1 --X 2 --penalty auth_security
# Final RRI: 36 -> Moderate (26-40)

# T5b-iii
python3 scripts/rri.py --touches config/README.md \
  --touches docs/tasks/s-230-poc-v1-digitalocean.md \
  --C 1 --D 1 --K 1 --P 1 --T 1 --A 1 --X 1
# Final RRI: 20 -> Low (0-25)
```

| Child | Final RRI | Band | A |
|---|---|---|---|
| S-230-T5b-i (`.env.example`) | 40 | Moderate | 1 |
| S-230-T5b-ii (`config/production.toml`) | 36 | Moderate | 1 |
| S-230-T5b-iii (`config/README.md` + verification) | 20 | Low | 1 |

Narrower per-file scope drops `D`/`K`/`P` closer to the ADR-026 anchor floor
(each child now touches only one leg of the contract instead of all three at
once) and `auth_security` no longer applies to T5b-iii, which is
documentation plus verification rather than authorship of a secret-adjacent
value.

## Owner routing instruction (2026-08-27)

The owner (`matias`) instructed that no local model is used for any T5b
child's implementation, citing the criticality of the production
auth/secret-boundary surface this task authors. Recorded as a task-local
routing override:

- T5b-i and T5b-ii recomputed into Moderate (26-40), whose default route is
  `scripts/local-agent/run_local_task.py` local-first implementation. The
  owner instruction overrides this default; implementation is Claude
  Sonnet 5 direct instead.
- T5b-iii recomputed into Low (0-25), whose default is primary-agent-direct
  or eligible Qwen Developer delegation. The owner instruction rules out the
  Qwen Developer delegation path; this child closes as primary-agent-direct
  regardless.
- This does not change the RRI, band, required reviewer chain, or Reflection
  pass count for any child — only who authors the code.
