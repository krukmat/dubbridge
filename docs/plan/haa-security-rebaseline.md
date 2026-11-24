---
type: Plan
title: HAA security rebaseline S1–S4
status: proposed
date: 2026-11-24
---

# HAA security rebaseline

**2026-11-24 S1–S3:** ADR-049 accepted; development RRI suspended. Protected executor, volatile registry, IPC and canonical closure candidate artifacts published. Provisioning, complete trusted runner/commit/verification integration and real plan ingestion/projection remain open. New regression tests NOT RUN. See `docs/audit/haa-protected-executor-checkpoint-2026-11-24.md`. Activation NO-GO.


**Follow-up owner decision (2026-11-24):** the RRI calculation requires reevaluation and is suspended until further notice for development of this HAA ↔ DubBridge feature. Do not recalculate or automatically reinstate it; preserve historical scores. Resumption requires an explicit owner decision after reevaluation. This does not suspend runtime HITL/authorization rules or technical security gates.


Owner approved preparation of S1–S4 after E1–E3. The concrete architecture
in ADR-049 and new task cards are proposed; do not infer blanket authorization
for host account changes or activation. Feature development RRI gates
are now explicitly waived by the owner; ADR-049 ratification remains separate.

Sources: `docs/audit/haa-e1-e3-local-validation-2026-11-24.md`;
`docs/adr/ADR-049-haa-protected-executor-and-volatile-authority.md`;
`docs/tasks/haa-security-rebaseline.md`.

## S1 — Identity and protected storage
Threat model and deployment contract first. Keep the existing Mac and
interactive Touch ID approver. Isolate HAA authority service and executor
from agent-controlled code, credentials, worktrees and commands. Use one
protected service identity and a separate worker boundary. Define trusted
installation, IPC, source-of-truth, Git/evidence projection and secret
provisioning. A service UID alone does not certify the whole workflow.

## S2 — Replay and rollback
Replace persisted ACTIVE authority with service-generation-bound volatile
leases. Keep durable terminal journal in protected storage. Explicitly
invalidate all old leases on restart, no auto-retry after uncertain POST
or consume. Do not treat prior HMAC lease files as migration input. Bind
all closure transactions to a fresh grant and protected canonical state.

## S3 — Bounded implementation and acceptance criteria
Preserve the original RRI 100 audit as historical evidence. The owner
explicitly removed RRI as a feature development gate on 2026-11-24.
No new RRI calculations are required. Use the existing decomposition to
keep changes reviewable, with explicit acceptance criteria, threat controls
and regression tests. The protected executor architecture must be ratified
before authority-changing code is implemented. Existing runtime task RRI
and HITL rules are unchanged.

## S4 — Certification
Prepare negative fixtures before host changes. Consolidate local execution
at the end: verify identity and service protection, key isolation, IPC
identity/scope, worker privilege boundaries, rollback, restart, race,
actual runner and canonical closure source, exact checkout and regression.
Do not test with real credentials or modify an existing production plan.
E4 Touch ID only after all these gates pass and owner authorizes activation
testing. E5 reports GO/NO-GO; actual activation/merge is separate.

## Dependencies
S1 design -> S2 design -> S3 acceptance criteria -> bounded implementation
-> S4 non-physical host tests -> E4 integrated physical test -> E5 decision.
No independent reviewer (existing bounded owner waiver); no new branches
or CI. Maximize ChatGPT; host provisioning/testing is last and requires
explicit owner authorization where privileged changes are necessary.

## Confidence semantics
Confidence >85% applies to completing an explicitly bounded deliverable,
not guaranteeing a positive test or safe activation. The design and test
protocol can be completed in cloud; actual service installation, real
non-bypass enforcement and final host outcomes are unmeasured. They
cannot honestly be assigned >85% execution confidence until the host
preconditions and architecture gates are resolved. Mark those conditional/BLOCKED;
do not inflate confidence to make the plan appear executable.

## Ratification

ADR-049 accepted by owner on 2026-11-24. Build candidate modules and
acceptance tests before privileged host provisioning; no active authority
from old disk leases. Runtime RRI/HITL rules unchanged.
