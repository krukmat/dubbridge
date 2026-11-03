---
type: ADR
status: Accepted
date: 2026-11-03
---

# ADR-048 — Human Approval Authority as the DubBridge workflow authority boundary

- **Status:** Accepted
- **Date:** 2026-11-03
- **Scope:** DubBridge software-factory / development workflow only

## Context

DubBridge already has an authoritative workflow for RRI scoring, task approval,
agent routing, independent review, fallback selection, and closure evidence. It
also has a deliberate human-in-the-loop boundary.

Human Approval Authority (HAA) already provides a frozen protocol-v1 approval
lifecycle, a physical macOS Touch ID ceremony, exact-action binding, atomic
single-use authorization, detached `ExecutionGrant` verification, and a proven
external-consumer integration.

The integration must add cryptographically bound human authority without turning
routine development into a sequence of biometric prompts or duplicating DubBridge's
workflow state machine inside HAA.

## Decision

### 1. Authority split

DubBridge remains authoritative for:

- plan/task lifecycle;
- RRI and autonomy policy;
- model/agent routing;
- reviewer requirements and verdicts;
- retry/fallback mechanics;
- task and plan completion semantics.

HAA is authoritative only for:

- trusted presentation of one exact authorization action;
- Touch ID-backed human ceremony;
- approval/rejection lifecycle;
- exact-action consumption;
- issuance of a short-lived `ExecutionGrant`.

HAA must not recompute RRI, choose agents, decide task status, or close plans by
itself.

### 2. Public HAA boundary only

DubBridge integrates through HAA's documented public HTTP/SDK contracts and
authority-key metadata. It must not import HAA private core, persistence, server,
or macOS implementation modules.

Requester, approver, and executor credentials/principals remain logically
separate as required by HAA.

### 3. Touch ID-only baseline

The first DubBridge integration enables only the existing macOS HAA approver using
the MacBook's Touch ID / Secure Enclave path.

External fingerprint hardware and WebAuthn are outside this baseline.

### 4. HAA grants remain single-use

`ApprovalReceipt` remains audit evidence only.

A HAA `ExecutionGrant` remains exact-action-bound and single-use. DubBridge must
not reinterpret it as a reusable bearer token.

To reduce interaction cost, the exact side effect authorized by
`dubbridge.task.start.v1` is **activation of a bounded DubBridge
`AuthorityLease`**. The executor:

1. reconstructs the exact task-start action from current state;
2. calls HAA `authorizeAndConsume`;
3. detached-verifies the resulting grant;
4. only then materializes the internal lease.

The lease is not an HAA object and cannot be used outside DubBridge's software
factory.

### 5. Internal AuthorityLease boundary

A lease is bound to exactly one:

- repository;
- plan ID;
- task ID;
- workflow session ID;
- branch;
- approved base commit;
- normalized allowed-path set/scope digest;
- allowed capability set;
- expiry.

It may cover repeated workflow-internal actions such as tests, bounded fixes,
reviews, and commits only when those capabilities were explicitly shown in the
task-start ceremony and remain inside the authorized task/session/scope.

It never authorizes:

- a different task or session;
- scope expansion;
- a different repository/branch;
- a protected/outward-facing operation not explicitly covered;
- secrets/credential access;
- release/deploy;
- plan closure.

A hard-boundary action requires its own HAA authorization and a specific
ActionProfile. Existing `git.merge.v1` should be reused when it exactly matches
the operation.

### 6. Mandatory plan closure

Every plan closure is a separate authority boundary.

DubBridge introduces a `dubbridge.plan.close.v1` HAA action bound to the exact
closure state. `PLAN_DONE` is allowed only after:

- the plan reaches `PLAN_READY_FOR_APPROVAL`;
- an exact closure manifest is built;
- Touch ID approval succeeds;
- the executor re-reads current closure state;
- HAA `authorizeAndConsume` succeeds;
- the `ExecutionGrant` verifies detached against trusted ACTIVE authority keys.

A material change to final commit, task disposition, accepted exceptions, or
evidence after approval makes the previous closure authorization stale.

### 7. Additive ActionProfiles, not protocol changes

The initial DubBridge profiles are:

- `dubbridge.task.start.v1`;
- `dubbridge.plan.close.v1`.

They are additive HAA protocol-v1 ActionProfiles. Their implementation may require
small profile-registration changes in HAA, but must not alter HAA's frozen signed
primitives, lifecycle, receipt semantics, or grant semantics.

Unknown profile versions or unknown authorization-significant fields fail closed.

## Consequences

### Positive

- physical human authority is cryptographically bound to exact workflow actions;
- routine work avoids repeated Touch ID through a narrow internal lease rather
  than weakened HAA grants;
- HAA remains product-agnostic and DubBridge remains workflow-authoritative;
- plan closure gets a strong, inspectable final human gate;
- existing HAA W9 external-consumer architecture can be reused.

### Trade-offs

- two DubBridge-specific ActionProfiles must be maintained and versioned;
- task-start approval becomes a real execution boundary, so its scope and
  capability display must remain concise enough for a human to evaluate;
- DubBridge must persist/recover internal lease state safely;
- a MacBook/HAA outage blocks HAA-required transitions rather than opening a
  fallback bypass.

## Alternatives considered

### Reusable HAA ExecutionGrant

Rejected. It conflicts with HAA exact-action/single-consumption semantics and
would turn a short-lived grant into an over-broad bearer capability.

### Touch ID for every test/fix/commit

Rejected. It is operationally tedious and would incentivize bypassing the
authority mechanism.

### HAA owns DubBridge workflow state

Rejected. It duplicates RRI/routing/review/closure authorities and couples HAA
to one consumer.

### Generic `dubbridge.privileged-operation.v1`

Rejected as the default. Authorization-significant operations should use narrow
typed profiles with complete trusted display and precondition rules.

### VEL in the critical authorization path

Rejected. VEL can receive evidence later, but HAA/DubBridge correctness must not
depend on an additional ledger service.
