---
type: TaskList
title: "Tasks: DubBridge ↔ HAA workflow integration"
status: active
plan: docs/plan/haa-hitl-integration.md
---

# Tasks: DubBridge ↔ HAA workflow integration

**Plan:** `docs/plan/haa-hitl-integration.md`
**Architecture:** `docs/adr/ADR-048-human-approval-authority-workflow-boundary.md`
**Policy:** `docs/policies/HAA_WORKFLOW_AUTHORITY_POLICY.md`

## Status legend

- [ ] Not started
- [~] In progress
- [x] Done

## Dependency order

```text
W1.T1 -> W1.T2 -> W1.T3 -> W1.T4 -> W1.T5
                                      |
                                      v
                                    W2.*
                                      |
                                      v
                                    W3.*
                                      |
                                      +----> W4.*
                                               |
                                               v
                                             W5.*
```

## Wave summary

| Task | Title | Type | Depends on | Status |
|---|---|---|---|---|
| W1.T1 | Reconcile HAA and DubBridge authority contracts | Analysis/docs | — | [x] |
| W1.T2 | Freeze HAA/DubBridge authority boundary | ADR/docs | W1.T1 | [x] |
| W1.T3 | Freeze task-start and plan-close ActionProfile contracts | Policy/docs | W1.T2 | [x] |
| W1.T4 | Freeze low-friction authority policy and failure semantics | Policy/docs | W1.T3 | [x] |
| W1.T5 | Synchronize plan, ledger, ADR index, HITL pointer, roadmap | Docs | W1.T4 | [x] |
| W2 | Authority Adapter | Development | W1 | [ ] |
| W3 | HITL Workflow Integration | Development | W2 | [ ] |
| W4 | Mandatory Plan Closure | Development | W2, W3 | [ ] |
| W5 | Hardening & Evidence | Development | W3, W4 | [ ] |

## W1.T1 — Reconcile HAA and DubBridge authority contracts

- **Status:** [x] Done — 2026-11-03
- **Effort:** S
- **Objective:** Compare current HAA protocol/security/external-integration
  contracts with DubBridge HITL/RRI rules before defining any integration.
- **Acceptance criteria:**
  - HAA public surfaces and role separation are identified;
  - exact-action, single-consumption, expiry/replay, and detached-verification
    rules are preserved;
  - the current DubBridge approval/autonomy boundary is preserved unless a
    deliberate later task changes it;
  - no HAA private package is required by the proposed integration.
- **Evidence to emit:** `docs/audit/haa-w1-contract-review-2026-11-03.md`.
- **Status artifacts affected:** this ledger and linked plan.

### Completion record

Confirmed:

- HAA protocol v1 is frozen and explicitly supports new typed ActionProfiles;
- requester, approver, and executor roles must remain distinct;
- `ApprovalReceipt` is audit evidence and is never execution authority;
- `ExecutionGrant` is exact-action-bound, detached-verifiable, short-lived,
  and produced only by atomic `authorizeAndConsume`;
- HAA W9 has already validated the external-consumer pattern;
- HAA's parked DubBridge note requires a fresh product-integration plan rather
  than reviving its former task numbering.

## W1.T2 — Freeze HAA/DubBridge authority boundary

- **Status:** [x] Done — 2026-11-03
- **Effort:** S
- **Objective:** Decide which system owns workflow versus human authority.
- **Acceptance criteria:**
  - DubBridge remains owner of plan/task lifecycle, RRI, routing, review, and
    retry policy;
  - HAA owns human ceremony and exact authorization only;
  - Touch ID/macOS is the only active authenticator;
  - HAA protocol primitives remain unchanged.
- **Output:** ADR-048.

## W1.T3 — Freeze task-start and plan-close ActionProfile contracts

- **Status:** [x] Done — 2026-11-03
- **Effort:** S
- **Objective:** Define the two initial DubBridge-specific HAA actions.
- **Acceptance criteria:**
  - `dubbridge.task.start.v1` is exact, typed, bounded, and trusted-displayable;
  - `dubbridge.plan.close.v1` binds the exact final closure state;
  - unknown fields/profile versions fail closed;
  - profile semantics do not turn receipts into bearer capabilities;
  - a task-start grant is consumed once before any internal lease activation.
- **Output:** `docs/policies/HAA_WORKFLOW_AUTHORITY_POLICY.md`.

## W1.T4 — Freeze low-friction authority policy and failure semantics

- **Status:** [x] Done — 2026-11-03
- **Effort:** S
- **Objective:** Preserve daily autonomy without weakening explicit authority
  boundaries.
- **Acceptance criteria:**
  - Low-band ordinary work does not gain a new Touch ID prompt;
  - a task-start approval may activate one internal task/session lease;
  - lease scope/capability/expiry rules are explicit;
  - privileged actions and plan closure cannot be authorized by the lease;
  - reject/timeout/expiry/HAA-down paths fail closed and never auto-approve.
- **Output:** `docs/policies/HAA_WORKFLOW_AUTHORITY_POLICY.md`.

## W1.T5 — Synchronize status artifacts

- **Status:** [x] Done — 2026-11-03
- **Effort:** S
- **Objective:** Leave W1 discoverable and unambiguous without claiming runtime
  enforcement that does not exist yet.
- **Acceptance criteria:**
  - ADR index includes ADR-048;
  - HITL policy points to HAA policy as planned enforcement rather than current
    behavior;
  - roadmap identifies W1 DONE and W2 as the next wave;
  - plan/ledger state agree.
- **Evidence:** source consistency review in
  `docs/audit/haa-w1-contract-review-2026-11-03.md`.

## W1 RRI / review classification

- **RRI:** 25 — Low, Effort S.
- **Reasoning:** docs/policy/ADR-only; one architecture decision penalty; no
  runtime/authentication implementation in W1.
- **Task-analysis review:** n/a — docs/policy/ADR-only exemption.
- **Code-solution review:** n/a — docs/policy/ADR-only exemption.
- **Ollama precheck:** n/a — no local-model role is invoked.

## W2 — Authority Adapter — planned

Minimum task decomposition to freeze at activation:

1. W2.T1 — client/config/credential boundary;
2. W2.T2 — HAA ActionProfile registration for DubBridge profiles where needed;
3. W2.T3 — requester builder + canonical scope/closure manifests;
4. W2.T4 — executor `authorizeAndConsume` + detached verification;
5. W2.T5 — idempotency/replay/authority-key trust handling;
6. W2.T6 — physical MacBook Touch ID integration smoke.

**Dependency:** W1 complete.

## W3 — HITL Workflow Integration — planned

Minimum objectives:

- policy evaluator invokes HAA only where required;
- one consumed task-start authorization may create a bounded internal
  `AuthorityLease`;
- scope/session/risk escalation invalidates the lease;
- waiting/rejection/expiry are explicit states/outcomes;
- no HAA action is bypassed on transport/authenticator failure.

**Dependency:** W2 complete.

## W4 — Mandatory Plan Closure — planned

Minimum objectives:

- add `PLAN_READY_FOR_APPROVAL`;
- build deterministic `PlanClosureManifest`;
- require `dubbridge.plan.close.v1` Touch ID authorization;
- verify/consume exact final state;
- allow `PLAN_DONE` only after verified authority;
- invalidate approval on material post-approval mutation.

**Dependencies:** W2 + W3.

## W5 — Hardening & Evidence — planned

Minimum objectives:

- negative-path matrix;
- crash/restart recovery;
- audit receipt and correlation;
- complete end-to-end certification;
- optional VEL sink boundary only after the authority path is independently
  correct.

**Dependencies:** W3 + W4.

## Execution handoff

> W1 is complete. The next executable wave is W2. Before W2 implementation,
> score the parent and leaves with `scripts/rri.py`, present the required
> approval card for any RRI 26+ envelope, and preserve ADR-048 plus
> `HAA_WORKFLOW_AUTHORITY_POLICY.md` as frozen W1 inputs.
