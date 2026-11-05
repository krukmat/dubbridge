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
| W3 | HITL Workflow Integration | Development | W2 | [~] design only; enforcement blocked |
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

## W2 — Authority Adapter — in progress

**Parent RRI:** 100 — Very high / Effort XL. Security/authorization + multi-repo
integration requires decomposition before implementation.

**Owner approval:** 2026-11-03 — owner instructed "Trabaja" immediately after W1
closure and the W2 Authority Adapter objective was presented. This authorizes the
bounded W2 parent envelope below; it does not authorize W3 workflow enforcement,
merge, release, deploy, secrets access, or plan closure.

| Leaf | Scope | Repo | RRI | Depends on | Status |
|---|---|---|---:|---|---|
| W2.T1a | `dubbridge.task.start.v1` HAA ActionProfile + tests | HAA | 55 | W1 | [~] implemented, verification pending |
| W2.T1b | `dubbridge.plan.close.v1` HAA ActionProfile + tests | HAA | 55 | W1 | [~] implemented, verification pending |
| W2.T2a | Deterministic DubBridge action/manifest builders | DubBridge | 25 | W1 | [ ] |
| W2.T2b | Requester/status HTTP client + requester-only config | DubBridge | 55 | T2a | [ ] |
| W2.T2c | Executor HTTP client + idempotency contract | DubBridge | 55 | T2a | [ ] |
| W2.T2d | Detached ExecutionGrant verification | DubBridge | 55 | T2c | [ ] |
| W2.T2e | Side-effect-free authority adapter CLI | DubBridge | 55 | T2b,T2c,T2d | [ ] |
| W2.T2f | Automated contract/negative-path suite | DubBridge | 25 | T2a–T2e | [ ] |
| W2.T2g | Physical MacBook Touch ID smoke | both | 55 | T1a,T1b,T2e,T2f | [ ] |

### W2.T1a / W2.T1b — HAA ActionProfiles

**Allowed HAA paths:**
- `packages/core/src/actions.ts`
- `tests/dubbridge-profiles.test.ts`

**Acceptance criteria:**
- task-start profile rejects unknown fields, unsafe/unsorted paths, unknown
  capabilities, TTL outside 60–1800 seconds, and stale actual state;
- plan-close profile requires PASS, completed=required, exact
  `PLAN_READY_FOR_APPROVAL` actual state, and matching final closure fields;
- trusted display derives only from canonical action fields;
- HAA protocol v1 primitives/lifecycle remain unchanged.

**HP-1:** exact task-start action + matching actual state -> profile validation PASS.
**EC-1:** changed HEAD/scope or hidden capability -> `STALE_APPROVAL`/validation failure.
**HP-2:** exact plan-close action + matching ready state -> profile validation PASS.
**EC-2:** ACTIVE state, changed digest, incomplete tasks, or FAIL validation -> denied.

### W2.T2a — deterministic contracts

Create pure Python builders for:
- canonical JSON/digest compatible with HAA;
- `TaskStartManifest` -> `dubbridge.task.start.v1`;
- `PlanClosureManifest` -> `dubbridge.plan.close.v1`;
- normalized sorted scope/capability lists.

No network, credentials, HAA request, or workflow mutation.

**HP-1:** same normalized inputs -> byte/digest-identical ActionSpec.
**EC-1:** unsafe path, unsupported capability, invalid TTL/count/status -> fail before network.

### W2.T2b — requester client

Separate requester-only configuration:
`HAA_BASE_URL`, `HAA_REQUESTER_API_KEY`,
`HAA_APPROVER_PRINCIPAL_ID`, `HAA_EXECUTOR_AUDIENCE`.

Requester can create/read requests only and must never read executor credentials.

**HP-1:** valid ActionSpec -> PENDING HAA request.
**EC-1:** missing config, non-JSON response, HTTP/transport error -> typed fail-closed error.

### W2.T2c — executor client

Separate executor-only configuration:
`HAA_BASE_URL`, `HAA_EXECUTOR_API_KEY`, `HAA_EXECUTOR_AUDIENCE`.

Executor may read request, call `authorizeAndConsume`, and retrieve trusted
authority-key metadata. It performs no workflow side effect in W2.

**HP-1:** approved exact action + stable actual state -> returns grant.
**EC-1:** stale/mutated action, wrong audience, consumed request with new execution
ID -> deny and return no authority.

### W2.T2d — detached grant verification

Use the already-proven Python verification pattern from
`krukmat/verifiable-event-ledger` W9: `cryptography>=45,<46`, canonical signed
bytes, ACTIVE authority key, signature, time/TTL, request/execution/action/audience
bindings.

**HP-1:** exact active Ed25519/ES256 grant -> verified.
**EC-1:** retired key, bad signature, expired/overlong grant, or any binding mismatch -> deny.

### W2.T2e — side-effect-free CLI

CLI operations:
- build task-start action;
- build plan-close action;
- request;
- status;
- authorize-and-verify.

The W2 CLI never creates an `AuthorityLease`, changes workflow state, commits,
pushes, merges, releases, deploys, or closes a plan.

### W2.T2f — automated gate

Tests cover deterministic builders, credential separation, HTTP failure mapping,
grant verification, idempotency/replay expectations, and no-side-effect W2 CLI.

### W2.T2g — physical MacBook gate

Run real local:
```text
DubBridge action -> HAA request -> macOS trusted display -> Touch ID
-> APPROVED -> authorizeAndConsume -> detached grant verification
```

Run once for task-start and once for plan-close with a non-mutating W2 executor.
This is the only W2 step that requires the owner's physical MacBook.

**Dependency:** W1 complete.

## W3 — HITL Workflow Integration — preparation in progress

**Prerequisite for enforcement:** W2 profiles/adapter must be integrated in
both repositories and the physical MacBook Touch ID E2E must PASS. The
owner approved **W3.T3.1 + W3.T3.2** on 2026-11-04 and **W3.T3.3 +
W3.T3.4**, then **W3.T3.5 + W3.T3.6** on 2026-11-05. Work before W2 certification is preparatory
only: no runner enforcement, merge authorization, or review waiver.

**Detailed seam map:** `docs/playbooks/HAA_W3_INTEGRATION_POINTS.md`.

| Task | Goal | Depends on | Status |
|---|---|---|---|
| W3.T3.1 | Identify real entry points and freeze non-bypass placement | W1; read-only W2 contract | [x] Docs/design completed |
| W3.T3.2 | Pure ALLOW / REQUIRE_HAA / BLOCK detector | T3.1 | [~] Code + tests written; independent review and full-repo local gate pending |
| W3.T3.3 | Request persistence + WAITING_FOR_HUMAN | W2 physical PASS; T3.2 reviewed before activation | [~] Code/tests written; review/local gate pending |
| W3.T3.4 | Live state reconstruction + exact grant verification | T3.3; W2 physical PASS before activation | [~] Code/tests written; review/local gate pending |
| W3.T3.5 | Scoped, short-lived AuthorityLease | T3.4; physical W2 PASS before activation | [~] Code/tests authored; local gate and independent review pending |
| W3.T3.6 | Scope/session/risk escalation guard | T3.2, T3.5; W2 physical gate before activation | [~] Code/tests authored; local gate and independent review pending |
| W3.T3.7 | Fail-closed failure and restart recovery | T3.3–T3.5 | [ ] |
| W3.T3.8 | Runner integration + negative-path certification | T3.6, T3.7 | [ ] |

### W3.T3.1 — Workflow Integration Points

- **Status:** [x] Completed — documentation-only.
- **Type / effort:** Docs/analysis, S.
- **Subtasks:**
  - T3.1.1: identify operator launcher, task-card parser, local runner and
    normalization facade entry points;
  - T3.1.2: define strict read-only boundary classifier interface and
    explicit split between workflow owner and HAA;
  - T3.1.3: document where future guards must sit to prevent bypass.
- **HP-1:** card/RRI/Git/session -> pure classifier before model runner
  or any privileged side effect.
- **EC-1:** guarding only `human-orchestrator.py` does not protect a
  direct `run_local_task.py` or normalized-facade invocation.
- **Evidence to emit:** `docs/playbooks/HAA_W3_INTEGRATION_POINTS.md`.
- **Status artifacts affected:** this ledger and
  `docs/plan/haa-hitl-integration.md`.
- **Review:** task-analysis n/a; code-solution n/a (docs-only).

### W3.T3.2 — Authority Boundary Detector

- **Status:** [~] Implementation written, pending independent review and
  official full-repository local verification. **Not production-certified.**
- **Type / effort:** Development, authorization/governance risk: do not use
  low-band routing simply because the function is pure.
- **Scope:**
  - `scripts/haa/boundary_detector.py`
  - `scripts/haa/boundary_detector_test.py`
  - `scripts/haa/run-tests.sh`
- **Behavioral contract:**
  - `ALLOW` means no additional HAA prompt, **not** a license to
    bypass existing RRI/HITL/path/command gates;
  - `REQUIRE_HAA` binds a specific known HAA ActionProfile;
  - `BLOCK` rejects unsupported, malformed or out-of-scope actions;
  - supplied scope snapshots can detect drift but do not authorize a task.
- **Subtasks:**
  - T3.2.1: classify RRI and named operation without I/O;
  - T3.2.2: detect repo/plan/task/session/branch/base/scope/capability/RRI drift;
  - T3.2.3: map unknown/unsafe operations to BLOCK and versioned exact
    actions to REQUIRE_HAA;
  - T3.2.4: automated positive/negative-path tests.
- **HP-1:** RRI 25 ordinary task -> ALLOW; RRI 26 task -> REQUIRE_HAA
  (`dubbridge.task.start.v1`).
- **HP-2:** plan-close -> REQUIRE_HAA
  (`dubbridge.plan.close.v1`), merge -> REQUIRE_HAA
  (`git.merge.v1`) even with low RRI.
- **EC-1:** unsupported privileged action, invalid RRI/path/capability,
  or missing context -> BLOCK.
- **EC-2:** changed session/branch/repo/scope/capabilities/RRI against a
  snapshot -> REQUIRE_HAA, and an unchanged snapshot cannot waive RRI 26+.
- **Evidence to emit:** `scripts/haa/boundary_detector_test.py` plus
  local unit-test transcript and reviewer artifact before closure.
- **Status artifacts affected:** this ledger, integration seam playbook,
  and `docs/plan/haa-hitl-integration.md`.
- **Verification:** `python3 -m unittest scripts.haa.boundary_detector_test`
  and `sh scripts/haa/run-tests.sh` from repository root, without CI.
- **Current evidence:** 9/9 isolated tests passed against a local mirror of
  the existing W2 `build_task_scope`; not yet the entire remote branch.
  The code-solution independent review has **not** run. Leave this task
  open until a real local checkout/review produces PASS.
- **RRI gate:** the policy-sensitive change requires scoring with the
  canonical `scripts/rri.py` in a real repository checkout before
  certification. No unverified numeric score is claimed.

### W3.T3.3 — Durable request checkpoints

- **Status:** [~] Implemented in branch; not independently reviewed or
  verified in the real repository checkout; no runtime activation.
- **Owner scope:** approved 2026-11-05; preparatory work only.
- **Subtasks:**
  - T3.3.1: persist a request attempt and stable execution ID before
    creating a network request;
  - T3.3.2: persist the exact action digest/audience/request ID and
    map HAA lifecycle to `WAITING_FOR_HUMAN`, `APPROVED_NOT_CONSUMED`,
    `REJECTED`, `EXPIRED`, `CONSUMED_UNVERIFIED`;
  - T3.3.3: record uncertain POST outcomes as terminal
    `REQUEST_UNCERTAIN` without automatic duplicate POST;
  - T3.3.4: local negative-path tests.
- **HP-1:** valid request is stored with fixed action digest and
  execution ID; remote PENDING→APPROVED only changes checkpoint state,
  never authority.
- **EC-1:** lost HTTP response, rejection, expiration, status digest/audience
  mismatch, or stale transition -> no authorization and no automatic retry.
- **Security:** requester-only credential use; checkpoints have no API keys;
  files private and changed with an exclusive lock + atomic replacement.
- **Evidence to emit:** `scripts/haa/request_checkpoint_test.py` and a
  recorded local test run with independent review.
- **Status artifacts affected:** this ledger, plan, W3 request playbook.
- **Verification gate:** full-repo local unit tests and band-routed
  independent review remain pending.

### W3.T3.4 — Live source-of-truth authorization

- **Status:** [~] Implemented in branch; not independently reviewed or
  verified in the real checkout; no runtime activation.
- **Owner scope:** approved 2026-11-05; preparatory work only.
- **Subtasks:**
  - T3.4.1: reconstruct task-start ActionSpec from validated task-card-v2
    and live Git origin/branch/HEAD, not requester-provided actual-state;
  - T3.4.2: bind current task/session/RRI/actionDigest/audience to the
    durable request;
  - T3.4.3: require APPROVED, operator-pinned ACTIVE authority key,
    `authorizeAndConsume` and detached grant verification;
  - T3.4.4: re-read real Git/card state after consumption and reject drift;
  - T3.4.5: local negative-path tests.
- **HP-1:** exact live state + APPROVED + matching pinned key + valid
  ExecutionGrant -> `VERIFIED_NO_SIDE_EFFECT`; execution ID remains
  stable and no lease/runner work occurs.
- **EC-1:** stale HEAD, wrong origin/session/card, pending/rejected request,
  mismatched pin/signature, or post-consume drift -> deny; after an
  uncertain consume, do not execute and do not automatically retry.
- **Evidence to emit:** `scripts/haa/live_verify_test.py` and
  `scripts/haa/request_checkpoint_test.py` plus recorded local test/review.
- **Status artifacts affected:** ledger, plan, request verification playbook.
- **Verification gate:** full-repo local unit tests, canonical RRI calculation,
  independent review and actual HAA Touch ID smoke pending.
- **No new GitHub Actions:** committed with `[skip ci]` and added only
  test entries to the existing local `scripts/haa/run-tests.sh`.

**W3.T3.3/T3.4 artifacts:**

- `scripts/haa/request_checkpoint.py`
- `scripts/haa/live_verify.py`
- `scripts/haa/request_checkpoint_test.py`
- `scripts/haa/live_verify_test.py`
- `docs/playbooks/HAA_W3_REQUEST_VERIFY.md`

### W3.T3.5 — Scoped, sealed AuthorityLease

- **Status:** [~] Implementation and unit tests authored on the **existing**
  `feature/haa-hitl-w3-request-verify` branch; **not certified** and
  not attached to any runner.
- **Subtasks:**
  - T3.5.1: immutable lease contract binds exact HAA request/execution/action,
    repository, plan, task, session, branch, base SHA, scope digest, paths,
    capabilities, RRI and TTL;
  - T3.5.2: trusted `activate_verified_task_lease()` invokes
    T3.4's `verify_live_task_authorization()` itself before issuing a lease;
  - T3.5.3: HMAC-sealed local persistence with 0700 directory, 0600 files,
    fsync/atomic write and exclusive lock; executor-only signing key;
  - T3.5.4: lease state and capability model covers repeated
    `edit/test/review/commit` without reusing HAA grants;
  - T3.5.5: unit tests for sealed restart/reload, tampering, invalid TTL,
    duplicate creation, revocation and verification dependency.
- **HP-1:** one verified consumed HAA grant -> one sealed scoped lease.
- **HP-2:** a valid lease survives a local trusted restart with the same
  executor-held signing key and remains bounded by the approved TTL.
- **EC-1:** forged/mutated record, wrong signing key, unsafe scope/TTL,
  revoked lease or duplicate issue -> fail closed.
- **EC-2:** grant consumed but lease storage fails -> **no** automatic
  grant replay or implicit approval.
- **Evidence:** `scripts/haa/authority_lease_test.py` (tests authored;
  local full checkout PASS not yet recorded).
- **Status artifacts:** this ledger, plan, roadmap, and
  `docs/playbooks/HAA_W3_AUTHORITY_LEASE.md`.
- **Review:** independent band-routed code-solution review pending.

### W3.T3.6 — Live Escalation Guard

- **Status:** [~] Implementation and local-Git integration tests authored;
  no runtime enforcement.
- **Subtasks:**
  - T3.6.1: check trusted live repo origin, branch, HEAD ancestry, plan/task/
    session and normalized scope;
  - T3.6.2: compare RRI and capability set; request fresh authority after
    escalation rather than silently broaden a lease;
  - T3.6.3: durably revoke invalidated leases, block unsupported privileged
    operations and route merge/plan-close to their separate exact HAA profiles;
  - T3.6.4: test repeated authorized operations, HEAD advancement, drift,
    expiry, tampering, traversal, symlinks and protected operations.
- **HP-1:** valid task lease + same session/scope + permitted operation ->
  `ALLOW` without Touch ID repetition.
- **HP-2:** HEAD advances by a task commit descended from the approved
  base -> lease remains valid subject to live path/operation checks.
- **EC-1:** branch, repo, session, task, RRI, scope, capability, history
  rewrite or TTL drift -> `REQUIRE_HAA` and/or fail-closed revocation.
- **EC-2:** mutation of sealed record or unsafe path -> no `ALLOW`.
- **EC-3:** `merge` and `plan_close` demand independent exact HAA
  profiles; `push`, `deploy`, `secrets` and destructive operations
  cannot be delegated to a task lease.
- **Evidence:** `scripts/haa/lease_guard_test.py` (local-Git tests
  authored; full checkout test and independent review pending).
- **Status artifacts:** this ledger, plan, roadmap, W3 lease playbook.
- **Review:** independent band-routed code-solution review pending.

**Branch discipline (owner instruction, 2026-11-05):**
Continue all W3 blocks in the **same**
`feature/haa-hitl-w3-request-verify` branch; separate commits preserve
traceability. Do not create a new branch per block or add GitHub Actions.

### W3 freeze line

W3.T3.1–T3.6 are prepared but **no W3 runtime enforcement** exists.
T3.3/T3.4 may make real HAA requests only when the trusted orchestrator
calls their library functions; neither is wired to the runner. Live
activation, W3.T3.7/T3.8, and W3 completion require both W2 integrations,
MacBook Touch ID smoke PASS, full local tests, and independent review.
The next planned development block is W3.T3.7 — Recovery & Failure Handling.

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
