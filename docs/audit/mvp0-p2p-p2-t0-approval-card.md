---
type: Audit
title: "Compact Approval Task Card v2 — P2.T0"
status: ready_for_approval
slice: MVP0-P2P
parent: P2.T0
---

# Compact Approval Task Card v2 — P2.T0

## 1. Decision header

**P2.T0 — Availability Node trust/operation contract**  
Status: READY FOR OWNER APPROVAL  
Planning RRI: **64 Complex / Effort L**  
Source implementation: none in T0

ADR-044 is Accepted. T0 is the only architecture/security checkpoint remaining
before the first P2 implementation parent (`P2.T1`) may be frozen/presented.

## 2. Scope and acceptance

Freeze:

- Availability Node runtime/process ownership;
- publication-control authentication;
- same-identity idempotency/replay behavior;
- observable confirmation/health surface for O4 reconciliation;
- service secret/config allow/deny list;
- concrete publication-state semantics;
- minimum ADR-018 P2 audit-event inventory.

Primary behaviors:

- HP-T0-1 authenticated same-identity publish -> stable idempotent evidence;
- HP-T0-2 reconciler confirms same logical publication without queue/DB authority;
- EC-T0-1 conflicting package/hash under same identity -> fail closed;
- EC-T0-2 invalid service identity -> no publish action;
- EC-T0-3 Availability Node success never independently sets `P2P_READY`.

## 3. Workflow / Reflection

T0 is docs/architecture-only, so local-stack and phase-1/phase-2 implementation
review actions are `n/a`. The parent keeps Complex-band HITL and integrated
reflection.

Required integrated passes after owner selection:

1. service-auth/replay/secret boundary;
2. O4 authority/idempotency/reconciliation consistency;
3. K1 ciphertext/key-deny-list consistency;
4. scope: no P2 source work, ADR-032 unchanged, no queue authority.

## 4. Options

### Runtime

**AN-R1 — Node.js/TypeScript Availability Node — RECOMMENDED**

Dedicated `apps/availability-node/` service using the Hyperdrive/Hyperswarm JS
stack already proven by P1, but operationally independent from mobile Bare.
Straightforward container/service health model; no need to embed another runtime.

**AN-R2 — standalone Bare service**

Closer runtime symmetry with the mobile worklet, but weaker conventional service
operations/tooling and little product benefit for a server-side seeder.

**AN-R3 — Rust controller + spawned/embedded JS/Bare seeder**

Strong control-plane process model but adds a cross-runtime failure boundary and
operational complexity not justified for MVP0.

### Publication-control authentication

All options use TLS server authentication and a private/non-public endpoint. No
option shares the backend HS256 JWT signing secret.

**AN-A1 — mTLS service identity — RECOMMENDED long-term**

Dedicated client certificate for dispatcher/reconciler; Availability Node validates
the configured CA/pin. Strong mutual service identity, no application bearer token,
and minimal custom request-signing logic. Cost: certificate lifecycle/rotation.

**AN-A2 — dedicated Ed25519 request signing**

Publisher holds a dedicated P2 service private key; node holds only the public key.
Request signature binds method/path/body digest/publication id/timestamp. Good key
separation but introduces canonicalization/replay-window logic that must itself be
implemented correctly.

**AN-A3 — dedicated bearer/HMAC secret**

Smallest MVP operational surface, but symmetric compromise affects both request
creation and verification. Acceptable only as an explicit simplicity tradeoff; not
the preferred long-term contract.

### Proposed state semantics

`building -> publish_pending -> publishing -> reconciling -> ready`

`failed` is reserved for explicit terminal/non-retryable failure. Unknown remote
outcome is `reconciling`, never `ready` or terminal `failed` merely because the ACK
was lost.

### Proposed minimum durable audit inventory

- publication intent created;
- K1 lineage sealed/server-wrapped;
- external publication confirmed;
- reconciliation entered due to unknown external result;
- `P2P_READY` transition;
- terminal publication failure after bounded policy exhaustion.

## 5. Recommendation

For the long-term architecture already chosen in D3/O4:

**AN-R1 + AN-A1** — Node.js/TypeScript Availability Node with mTLS service identity.

Reasoning:

- keeps Hyperdrive/Hyperswarm in its native server ecosystem;
- avoids making Bare itself an operations platform;
- preserves a very narrow trust boundary;
- mTLS avoids inventing request-signature canonicalization and prevents reuse of
  application JWT/shared backend signing secrets;
- queue/outbox/reconciler remain completely independent from the chosen service
  runtime and can replace/scale the dispatcher later.

If certificate operations are deliberately too heavy for MVP infrastructure, the
fallback preference is **AN-R1 + AN-A2**, not AN-A3.

## 6. Owner checkpoint

Approve one runtime and one auth option (or a reviewed variant), and accept/adjust
the proposed state + audit minimum. After approval, T0 is codified mechanically and
P2.T1 becomes the next execution parent.

Execution has not started. Approve this task to proceed.
