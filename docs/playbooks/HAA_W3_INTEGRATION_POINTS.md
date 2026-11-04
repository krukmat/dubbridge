---
type: Playbook
title: "DubBridge HAA W3 task-start authorization integration seams"
governs: "read-only W3.T3.1 mapping; no runtime enforcement"
---

# W3.T3.1 — HITL integration points

> **Implementation status:** W3.T3.1 mapping completed; W3.T3.2
> pure classifier written but not integrated into any launcher or runner.
> **W2 physical Touch ID gate remains pending.** Do not treat this
> document or the classifier as an active authorization mechanism.

## Current ownership

| Surface | Current role | W3 placement |
|---|---|---|
| `scripts/rri.py` + `RRI_POLICY.md` | Authoritative RRI/band calculation | Upstream input; W3 must never guess or recompute RRI |
| `docs/policies/HITL_AUTONOMY_POLICY.md` | Approval/autonomy policy | Authoritative requirement; HAA cannot waive existing gates |
| `scripts/local-agent/task_card.py` | Strict task-card-v2 parser, allowed paths and RRI | Source of validated plan/task/scope fields, not proof of approval |
| `scripts/human-orchestrator.py` | Operator launcher; delegates to existing tools | Visible entry point; never the sole enforcement point |
| `scripts/local-agent/cli.py` | Local runner entry, loads card before model invocation | Mandatory future pre-execution guard (W3.T3.8), before model activity |
| `scripts/local-agent/run_local_task.py` | Delegates to `cli.main()` | Keep model execution unchanged; direct invocation cannot bypass W3 gate |
| `scripts/local-agent/run_normalized_task.py` | Normalization facade around same runner | Must inherit authoritative guard; not a second permission engine |
| `scripts/local-agent/boundary.py` | File/argv capability sandbox | Complementary guard; W3 cannot relax file/command scope |
| `scripts/local-agent/med_high_gate.py` | Model-route/refinement decision | Routing is advisory, never human authorization |
| `scripts/haa/contracts.py` | W2 typed, deterministic HAA action builders | Action profile construction only; not HAA approval evidence |
| `scripts/haa/authority.py` | W2 request/consume/verify authority boundary | W3.T3.3/T3.4 will call it after actual-state reconstruction |
| `scripts/haa/adapter.py` | Side-effect-free W2 smoke/CLI | **Not** a live state reader: synthetic action-derived `actual_state` is only for W2 smoke |

## Proposed flow (no runtime attachment in T3.1–T3.2)

```text
Trusted task card + authoritative RRI + Git repo/branch/HEAD + session
                          |
                          v
               W3.T3.2 pure classifier
                   /       |        \
                ALLOW  REQUIRE_HAA  BLOCK
                 |          |         |
          Existing HITL   W3.T3.3  Stop with
          and runner      request   typed reason
          guards          HAA
                            |
                         W3.T3.4
                  fresh actual state +
                  authorizeAndConsume +
                  detached grant verify
                            |
                         W3.T3.5
                  optional bounded local
                  AuthorityLease (only for
                  exact task-start approval)
```

`ALLOW` means *no additional HAA-specific prompt*. It is not a
permission slip and cannot bypass the repository's existing approvals,
RRI gate, command/path boundary, independent review, or owner controls.

## Exact boundaries classified by W3.T3.2

- `task_start`, `edit`, `test`, `review`, `commit`: on RRI
  0–25, no additional HAA prompt; on RRI 26+, request
  `dubbridge.task.start.v1` **until** W3.T3.5 supplies a separately
  validated in-scope lease. The pure classifier never accepts an
  `approved=true`/receipt/lease flag.
- A changed repository, plan, task, session, branch, base SHA, path scope,
  capability set, or RRI against a frozen scope snapshot requests new
  task-start authority. The snapshot is a drift detector, **not proof**
  of an authorized lease.
- `merge`: always routes to `git.merge.v1`.
- `plan_close`: always routes to `dubbridge.plan.close.v1`.
  Mandatory enforcement/state transition belongs to W4.
- `push`, `create_pr`, `deploy`, `release`, `secrets`, `delete`,
  `overwrite`, `migration`, `external_call`: `BLOCK` until a
  dedicated exact ActionProfile exists; never downgrade to generic HAA.
- Invalid RRI, scope, operation, or unsupported capability: `BLOCK`.

## Critical trust rules for W3.T3.3–T3.8

1. Task card, RRI and current Git state must be read **inside the trusted
   orchestrator/executor**, not accepted from free-form agent/model text.
2. The W2 `task_start_actual_state(action)` and
   `plan_close_actual_state(action)` helpers merely mirror an action:
   they are acceptable in synthetic smoke testing, but **forbidden
   as live executor precondition evidence**.
3. Requester credentials cannot be used for `authorizeAndConsume`.
   HAA APPROVED or ApprovalReceipt alone never means executor authority.
4. Reconstruct actual repository, task, branch, scope and validation state
   immediately before HAA consumption; verify the single-use
   ExecutionGrant detached against trusted ACTIVE authority keys.
5. A future `AuthorityLease` requires an independently verified grant,
   current task/session/scope, an explicit TTL, allowed capabilities,
   invalidation on escalation, and no protected actions.
6. Do not enforce only in `human-orchestrator.py`: direct runner invocation
   and normalized facades would bypass such a check.
7. Keep W3 disabled until **both W2 repositories are integrated**,
   physical Touch ID validation passes, and independent W3 review passes.
8. Do not create or modify GitHub Actions for W3. Use local unit tests.

## Acceptance / evidence

- **T3.1.HP-1:** given a real task card, identify the intended guard placement
  before the agent is run, while preserving existing CLI and runner behavior.
- **T3.1.EC-1:** a direct runner or normalized facade cannot be assumed safe
  solely because the operator launcher has an approval check.
- **T3.2.HP-1:** Low routine -> ALLOW; 26+ task -> REQUIRE_HAA;
  plan close and merge -> exact profiles.
- **T3.2.EC-1:** invalid/out-of-scope/unsupported -> BLOCK; any scope
  drift -> REQUIRE_HAA; a supplied snapshot never waives a high-RRI gate.

Automated unit source:
`scripts/haa/boundary_detector_test.py`.
Local runner: `python3 -m unittest scripts.haa.boundary_detector_test`.

No Touch ID session, live grant, workflow mutation, or W3 certification is
claimed by T3.1/T3.2.
