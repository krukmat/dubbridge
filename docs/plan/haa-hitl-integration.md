---
type: Plan
title: "Plan: DubBridge ↔ Human Approval Authority (HAA) workflow integration"
status: active
slice: haa-hitl-integration
---

# Plan: DubBridge ↔ Human Approval Authority (HAA) workflow integration

> **Status:** Active. W1 DONE; W2 staged but not merged/physically certified. W3.T3.1 mapping completed; T3.2–T3.6 implementation staged on one W3 branch (classifier, durable approval checkpoints, live grant verification, sealed AuthorityLease, escalation guard). Full local test gate and independent review remain pending. No W3 enforcement active.
> **Tasks ledger:** `docs/tasks/haa-hitl-integration.md`
> **Architecture decision:** `docs/adr/ADR-048-human-approval-authority-workflow-boundary.md`
> **Policy:** `docs/policies/HAA_WORKFLOW_AUTHORITY_POLICY.md`

## Objective

Integrate `krukmat/human-approval-authority` into the DubBridge software-factory
workflow as the human authorization boundary while preserving the existing
RRI/HITL routing model and keeping daily development low-friction.

The first supported authenticator is the MacBook's Touch ID through HAA's existing
macOS approver. No external biometric hardware, WebAuthn path, or product-runtime
authorization is in scope for this plan.

## Design decisions

1. **HAA is authority, not workflow.** DubBridge continues to own task/plan
   lifecycle, RRI, routing, reviews, retries, and closure rules. HAA decides only
   whether a named human authorization ceremony succeeded for one exact action.
2. **Protocol v1 is reused unchanged.** DubBridge consumes HAA through public
   HTTP/SDK contracts. `ApprovalReceipt` is audit evidence only; a detached-valid
   `ExecutionGrant` is the only HAA execution authority.
3. **Touch ID is the only active authenticator.** The macOS approver is the only
   channel enabled by this integration baseline.
4. **No reusable HAA grant.** HAA grants remain exact-action-bound and single-use.
   To avoid repetitive Touch ID prompts, DubBridge may consume a
   `dubbridge.task.start.v1` grant once to activate a bounded internal
   `AuthorityLease` for one task/session/scope. The lease is a DubBridge workflow
   construct, not an HAA bearer capability.
5. **Hard boundaries always escape the lease.** Scope escalation, task/session
   change, protected/outward-facing operations not explicitly covered by the
   lease, and plan closure require a new exact HAA authorization.
6. **Plan closure always requires Touch ID.** A plan cannot transition to
   `PLAN_DONE` without a valid, consumed `dubbridge.plan.close.v1` authorization
   bound to the exact final closure manifest.
7. **Dedicated ActionProfiles only.** Prefer a specific existing HAA profile
   (for example `git.merge.v1`) when it exactly matches the side effect.
   Otherwise add a narrowly typed/versioned profile; never introduce a generic
   "privileged operation" escape hatch.

## Waves

| Wave | Goal | State | Depends on |
|---|---|---|---|
| W1 | Contract & Policy | **DONE** | — |
| W2 | Authority Adapter | Integration + MacBook Touch ID gate pending | W1 |
| W3 | HITL Workflow Integration | T3.1 docs DONE; T3.2–T3.6 code staged without runtime enforcement, local review/test and W2 gates pending | W2 |
| W4 | Mandatory Plan Closure | Planned | W2, W3 |
| W5 | Hardening & Evidence | Planned | W3, W4 |

## W1 — Contract & Policy — DONE

W1 reconciles HAA's frozen protocol and external-integration rules with
DubBridge's current HITL/RRI workflow. It freezes the authority boundary,
ActionProfile shapes, internal lease semantics, policy matrix, and failure
behavior without changing runtime code.

Outputs:

- `docs/adr/ADR-048-human-approval-authority-workflow-boundary.md`
- `docs/policies/HAA_WORKFLOW_AUTHORITY_POLICY.md`
- `docs/tasks/haa-hitl-integration.md`
- `docs/audit/haa-w1-contract-review-2026-11-03.md`

## W2 — Authority Adapter

Goal: prove the public integration path from DubBridge to HAA and back without
changing the DubBridge workflow state machine.

Planned outputs:

- HAA requester/executor client boundary;
- exact ActionProfile builders;
- detached `ExecutionGrant` verification;
- authority-key trust configuration;
- replay/idempotency handling;
- physical macOS Touch ID smoke from the DubBridge integration surface.

W2 may require additive HAA ActionProfile registrations for
`dubbridge.task.start.v1` and `dubbridge.plan.close.v1`. That is an allowed
protocol-v1 extension, not a protocol/core lifecycle change.

## W3 — HITL Workflow Integration

**W3.T3.1/T3.2 preparatory checkpoint (2026-11-04):** the real
runner/orchestrator entry points are mapped in
`docs/playbooks/HAA_W3_INTEGRATION_POINTS.md`, and a pure classifier
(`scripts/haa/boundary_detector.py`) with isolated unit tests has been
written. It returns ALLOW / REQUIRE_HAA / BLOCK but never verifies grants,
issues leases or changes workflow state. Independent code-solution review
and full-repository verification remain pending, and W2's physical Touch ID
gate has not been demonstrated. `ALLOW` never waives existing HITL/RRI
rules. No CI jobs were added.

Goal: replace the existing logical approval checkpoint with HAA-backed authority
where policy requires it, while avoiding unnecessary prompts.

**Preparatory checkpoint (2026-11-05):** `scripts/haa/request_checkpoint.py`
and `scripts/haa/live_verify.py` with targeted local tests now exist on
`feature/haa-hitl-w3-request-verify`, based on W3's pure detector branch.
They implement durable PENDING/APPROVED-not-authority records and a separate
exact task-start verification over trusted task-card-v2, live Git origin/branch/
HEAD, the preserved execution ID and an operator-pinned ACTIVE authority key.
`APPROVED` or `ApprovalReceipt` never grants execution. No future
AuthorityLease is created by these helpers. Local full-branch testing and
band-routed independent review are still pending; the verified physical
MacBook HAA round-trip and both W2 merges remain gating dependencies.
See `docs/playbooks/HAA_W3_REQUEST_VERIFY.md`.
No GitHub Actions were added, and no runner state machine has changed.

**W3.T3.5/T3.6 checkpoint (2026-11-05):** `scripts/haa/authority_lease.py`
creates a short-lived, HMAC-sealed internal lease only after the trusted
T3.4 verification function is invoked in the same executor process.
`scripts/haa/lease_guard.py` re-checks the task, session, scope, RRI,
approved Git ancestry and touched paths before routine operations;
escalations durably revoke the lease. Exact HAA profiles remain mandatory
for merges and plan closure. `docs/playbooks/HAA_W3_AUTHORITY_LEASE.md`
defines the security and TOCTOU limitations. The owner's **single-branch
directive** is applied: all W3 changes continue on
`feature/haa-hitl-w3-request-verify` with separate commits, no new
task branches. Unit tests were added to the local runner, but no test
execution, independent review, physical Touch ID evidence, or live
enforcement is claimed.

Planned behavior:

- RRI 0–25 normal work remains low-band and does not gain a new Touch ID prompt;
- RRI 26+ task-start approval becomes HAA-backed;
- a consumed task-start grant may create one bounded `AuthorityLease`;
- tests, fixes, reviews, and explicitly authorized commits inside the lease do not
  trigger repeated Touch ID;
- scope/risk/session escalation invalidates or bypasses the lease and requires a
  new authority decision;
- waiting/rejection/expiry become explicit workflow states/outcomes, never
  auto-approval.

## W4 — Mandatory Plan Closure

Goal: make human acceptance of a completed plan a cryptographically bound workflow
invariant.

The closure path is:

```text
ACTIVE
  -> PLAN_READY_FOR_APPROVAL
  -> build exact PlanClosureManifest
  -> HAA Touch ID ceremony
  -> authorizeAndConsume(actual closure action)
  -> detached ExecutionGrant verification
  -> atomic PLAN_DONE transition + evidence
```

Any material change to final HEAD, task disposition, accepted exceptions, or
closure evidence after approval invalidates the closure authorization and returns
the plan to `PLAN_READY_FOR_APPROVAL`.

## W5 — Hardening & Evidence

Goal: certify fail-closed behavior and recovery.

Minimum matrix:

- explicit reject;
- window close;
- local timeout;
- challenge expiry;
- request expiry;
- invalid signature;
- inactive authority key;
- action digest mismatch;
- executor audience mismatch;
- replay/different execution ID after consume;
- stale task-start base state;
- stale plan-closure final state;
- restart/recovery while waiting for human.

VEL remains an optional evidence sink and is not part of the authorization
critical path.

## Dependency view

```text
W1 Contract & Policy
        |
        v
W2 Authority Adapter
        |
        v
W3 HITL Workflow ---------+
        |                  |
        +-------------> W4 Plan Closure
                           |
                           v
                      W5 Hardening
```

## Out of scope

- biometric templates or raw biometric data in DubBridge/HAA;
- external fingerprint hardware;
- WebAuthn as an active DubBridge channel;
- replacing DubBridge's RRI/reviewer/routing authorities;
- making VEL an authorization dependency;
- changing HAA protocol-v1 signed primitives;
- generic reusable HAA bearer tokens.

## W1 resource / RRI note

W1 is docs/policy/ADR-only. Using the repository's RRI-v2 formula with the
planned documentation surfaces and the architecture-decision penalty yields
RRI **25 / Low / Effort S**. No local-model role is needed for W1.
