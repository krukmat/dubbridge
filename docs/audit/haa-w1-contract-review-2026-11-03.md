# HAA W1 contract review — 2026-11-03

## Scope

Read-only reconciliation of the current HAA authority contract with DubBridge's
current workflow policies before freezing W1.

## Sources reviewed

DubBridge:

- `AGENTS.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md`
- `docs/plan/roadmap.md`
- `docs/architecture.md`

HAA:

- `AGENTS.md`
- `docs/DUBBRIDGE-INTEGRATION-BACKLOG.md`
- `docs/EXTERNAL-INTEGRATION.md`
- `docs/PROTOCOL-V1.md`
- `docs/SECURITY.md`
- `docs/HAA-ACTIVE-ROADMAP.md`
- `docs/W9-REFERENCE-INTEGRATION.md`
- `tasks/manifest.yaml`
- `packages/protocol/src/index.ts`
- `packages/core/src/actions.ts`

## Findings

### F1 — HAA core does not need a lifecycle redesign

HAA protocol v1 is frozen and already supports additive typed ActionProfiles.
A DubBridge integration belongs above the protocol boundary.

**Disposition:** preserve protocol v1 and universal lifecycle unchanged.

### F2 — prior "reusable scoped grant" wording was unsafe

HAA defines `ApprovalReceipt` as audit-only and `ExecutionGrant` as
exact-action-bound, short-lived, and single-use through
`authorizeAndConsume(actualAction, executionId)`.

**Disposition:** replace reusable-grant language with a single-use task-start HAA
grant whose authorized side effect is activation of a bounded DubBridge
`AuthorityLease`.

### F3 — physical MacBook path is already proven by HAA

The HAA macOS approver/Touch ID path is closed and W9 validated a real external
consumer with a detached-verified grant.

**Disposition:** W2 should consume the public boundary and add only the
DubBridge-specific ActionProfiles/adapters needed by the concrete consumer.

### F4 — HAA must not become the DubBridge workflow engine

DubBridge already owns RRI, task approval rules, model routing, reviewers,
fallback selection, and closure procedure.

**Disposition:** HAA is an authority oracle/execution gate only.

### F5 — low-friction requires a DubBridge-local lease, not weaker HAA semantics

Asking Touch ID for every test/fix/commit is operationally hostile. Allowing one
HAA bearer grant to authorize many arbitrary future actions would violate HAA.

**Disposition:** consume one exact task-start authorization to activate a
task/session/scope-bound lease. Hard boundaries and plan close remain separate.

### F6 — plan close is an independent exact action

A task-start approval cannot truthfully authorize an unknown future final result.

**Disposition:** `dubbridge.plan.close.v1` binds final HEAD, deterministic
closure manifest digest, validation/evidence digest, task counts, and accepted
exceptions. Touch ID is mandatory for every plan close once W4 is active.

## W1 RRI

W1 touches documentation/policy/ADR surfaces only. Inputs used with the current
RRI-v2 formula:

```text
C=0
F=3   # 6-10 documentation/status surfaces
D=1
T=0
A=1
K=1
P=1
X=3
penalty=arch_decision (+12)

technical profile: L=0 I=1 Q=1 V=0
ICI=25 -> ici_band=25
risk/domain input=19
final=max(25,19)=25
```

Result: **RRI 25 / Low / Effort S**.

Because W1 is docs/policy/ADR-only, phase-1 and phase-2 code review are not
applicable and no Ollama precheck is required.

## W1 outcome

W1 freezes:

- ADR-048 authority split;
- task-start and plan-close ActionProfile contracts;
- Touch ID-only baseline;
- internal AuthorityLease semantics;
- failure mapping;
- activation sequencing;
- W2 as the next executable wave.

No runtime enforcement is claimed by W1.
