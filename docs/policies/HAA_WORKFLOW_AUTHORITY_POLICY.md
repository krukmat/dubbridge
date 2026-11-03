---
type: Policy
title: "HAA Workflow Authority Policy"
governs: "planned HAA-backed human authority for the DubBridge software-factory workflow"
---

# HAA Workflow Authority Policy

> **Status:** W1 contract frozen; runtime enforcement is **not active** until W3.
> `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` remains the highest authority for
> workflow procedure. `docs/policies/HITL_AUTONOMY_POLICY.md` remains the
> authority for when approval is required. This policy defines how an existing
> approval requirement is represented through HAA once the integration is active.
>
> Architecture boundary: ADR-048.

## 1. Baseline

- Authenticator: macOS HAA approver using MacBook Touch ID only.
- HAA wire contract: protocol v1, unchanged.
- Integration surfaces: public HAA HTTP/SDK and authority-key metadata only.
- HAA roles: requester, approver, executor remain separated.
- HAA `ApprovalReceipt`: audit evidence only.
- HAA `ExecutionGrant`: exact-action-bound, short-lived, single-use.
- Unknown profile/version/field: fail closed.

## 2. Prompt-minimization rule

HAA is invoked at **authority boundaries**, not at every workflow action.

| Event | Touch ID when HAA enforcement is active |
|---|---|
| Ordinary RRI 0–25 task execution | No |
| RRI 26+ task start already requiring HITL approval | Yes, once for the approved task/session envelope |
| Tests/fixes/review inside a valid task lease | No |
| Commit inside a lease that explicitly includes `commit` | No additional prompt |
| New task or new workflow session | Yes when the normal HITL gate requires it |
| Scope expansion / branch or repo change | Yes |
| Sensitive/privileged operation outside lease capabilities | Yes |
| Merge where `git.merge.v1` applies | Yes |
| Release/deploy/secrets operation | Yes, using a dedicated profile |
| Plan closure | **Always yes** |

This policy does not weaken the current RRI/HITL gate. It only defines the HAA
mechanism and the bounded authority that one successful task-start ceremony may
activate.

## 3. `dubbridge.task.start.v1`

Purpose: authorize activation of one bounded DubBridge `AuthorityLease`.

### ActionSpec

```text
schema: haa.action.v1
type: dubbridge.task.start.v1

payload exact keys:
  repository          string
  planId              string
  taskId              string
  sessionId           string
  branch              string
  baseCommitSha       string
  allowedPaths        string[]   # normalized, sorted, non-empty
  scopeDigest         string     # sha256 of canonical task scope/card inputs
  capabilities        string[]   # allow-listed enum values
  leaseTtlSeconds     integer

preconditions:
  none in the wire object; the executor reconstructs actual state and the
  ActionProfile validates it against payload fields at authorizeAndConsume time.
```

Initial capability allow-list:

- `edit`
- `test`
- `review`
- `commit`

`push`, `merge`, `release`, `deploy`, `secrets`, and
`plan-close` are intentionally not valid task-start capabilities.

### Preconditions at consumption

The executor must prove current state still matches:

- repository;
- task/plan/session identity;
- branch;
- current HEAD equals `baseCommitSha` before lease activation;
- canonical allowed-path set hashes to `scopeDigest`.

If any value is stale, authorization fails closed.

### Trusted display claims

The profile must derive display claims from the canonical action and show at
least:

- ACTION = START DUBBRIDGE TASK;
- PLAN;
- TASK;
- REPOSITORY;
- BRANCH;
- BASE SHA;
- ALLOWED PATHS;
- CAPABILITIES;
- LEASE TTL.

Authorization-significant fields must be displayed or intentionally represented
by the profile; arbitrary requester prose is not trusted display input.

### Result

A valid HAA grant authorizes exactly one external side effect:
`activate AuthorityLease`.

The grant is consumed once. It is never cached/reused as the lease.

## 4. AuthorityLease

The internal lease contains:

```text
leaseId
haaRequestId
haaExecutionId
haaGrantDigest
planId
taskId
sessionId
repository
branch
baseCommitSha
allowedPaths
scopeDigest
capabilities
issuedAt
expiresAt
status
```

### Lease validity

The lease is valid only while all of the following remain true:

- same plan/task/session;
- same repo/branch;
- operation path remains within `allowedPaths`;
- operation capability is allow-listed;
- no explicit workflow invalidation occurred;
- lease has not expired.

Commits created during the authorized task may advance HEAD; the lease binds the
approved starting ancestor and task/session/scope, not a permanently fixed
post-start HEAD.

### Lease invalidation

Invalidate or bypass the lease on:

- task/session change;
- repo/branch change;
- scope expansion or changed `scopeDigest`;
- requested capability not present in the lease;
- expiry;
- owner revocation/cancellation;
- workflow state leaving the authorized task;
- an operation whose policy requires its own exact HAA profile.

No invalid lease may fall back to implicit approval.

## 5. `dubbridge.plan.close.v1`

Purpose: authorize the exact transition from
`PLAN_READY_FOR_APPROVAL` to `PLAN_DONE`.

### ActionSpec

```text
schema: haa.action.v1
type: dubbridge.plan.close.v1

payload exact keys:
  repository             string
  planId                 string
  branch                 string
  finalCommitSha         string
  closureManifestDigest  string
  requiredTaskCount      integer
  completedTaskCount     integer
  acceptedExceptionCount integer
  validationStatus       string  # must equal PASS
  evidenceDigest         string

preconditions:
  none in the wire object; current closure state is reconstructed at consume time.
```

### PlanClosureManifest

Before creating the HAA request, DubBridge builds a deterministic manifest
containing at least:

- plan ID/version;
- final repository/branch/HEAD;
- required task dispositions;
- accepted exceptions;
- unresolved non-blockers;
- validation summary;
- evidence references/digests.

The payload binds the full manifest through `closureManifestDigest` and exposes
the human-relevant summary separately.

### Preconditions at consumption

Immediately before `authorizeAndConsume`, the executor re-reads workflow state
and proves:

- state is `PLAN_READY_FOR_APPROVAL`;
- plan/repository/branch still match;
- HEAD equals `finalCommitSha`;
- recomputed closure manifest digest matches;
- validation remains PASS;
- evidence digest/counts still match.

Any change makes the approval stale.

### Trusted display claims

Show at least:

- ACTION = CLOSE DUBBRIDGE PLAN;
- PLAN;
- REPOSITORY;
- BRANCH;
- FINAL SHA;
- COMPLETED / REQUIRED TASKS;
- ACCEPTED EXCEPTIONS;
- VALIDATION;
- CLOSURE MANIFEST DIGEST.

## 6. Privileged operations

Use an existing exact profile when possible. `git.merge.v1` is preferred for a
matching merge side effect.

Do not route distinct security-sensitive operations through one generic profile.
Release, deploy, secrets, destructive migration, or other hard boundaries require
their own typed profile and executor precondition contract before activation.

## 7. Failure mapping

HAA outcomes are control-flow outcomes, never bypass signals.

| HAA condition | DubBridge behavior |
|---|---|
| PENDING / waiting for human | remain waiting; no side effect |
| APPROVE but not yet consumed | still no side effect |
| `USER_ESCAPE` | record explicit rejection; stop current authority attempt |
| window close / timeout / challenge expiry / interaction error | fail closed; remain/re-enter waiting state with typed reason |
| request TTL EXPIRED | old request cannot be reused; new authority attempt required |
| invalid signature / digest / audience / key status | security failure; block operation |
| stale precondition | discard stale approval and rebuild from current state |
| HAA unavailable | block HAA-required transition; never auto-approve |
| request already CONSUMED with same executionId | idempotent recovery path only |
| request CONSUMED with different executionId | deny |

## 8. Plan-closure invariant

Once W4 enforcement is active:

```text
PLAN_DONE => verified + consumed dubbridge.plan.close.v1 authority exists
```

No direct `ACTIVE -> PLAN_DONE` or equivalent bypass is permitted.

A closure approval cannot be satisfied by:

- chat text alone;
- a task-start lease;
- an `ApprovalReceipt`;
- a stale prior plan-close grant;
- a different plan/session/action profile.

## 9. Activation sequencing

- **W1:** contract/policy only; no runtime behavior changes.
- **W2:** adapter/profile support and physical Touch ID smoke.
- **W3:** task-start HAA enforcement + AuthorityLease.
- **W4:** mandatory plan-close HAA enforcement.
- **W5:** recovery/negative-path/E2E certification.

Until W3 is complete, existing DubBridge HITL behavior remains authoritative and
unchanged.
