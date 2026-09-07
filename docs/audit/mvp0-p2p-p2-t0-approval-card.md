---
type: Audit
title: "Compact Approval Task Card v2 — P2.T0"
status: complete
slice: MVP0-P2P
parent: P2.T0
---

# Compact Approval Task Card v2 — P2.T0

## 1. Decision header

**P2.T0 — Availability Node trust/operation contract**  
Status: **COMPLETE / PASS**  
Planning RRI: **64 Complex / Effort L**  
Source implementation: none in T0

ADR-044 is Accepted. The repository owner approved the recommended T0 contract on 2026-09-05. The full frozen decision and Reflection evidence are recorded in `docs/audit/mvp0-p2p-p2-t0-decision.md`.

## 2. Scope and acceptance

Frozen:

- Availability Node runtime/process ownership;
- publication-control authentication;
- same-identity idempotency/replay behavior;
- observable confirmation/health surface for O4 reconciliation;
- service secret/config allow/deny list;
- concrete semantic publication-state contract;
- minimum ADR-018 P2 audit-event inventory.

Primary behaviors:

- HP-T0-1 authenticated same-identity publish -> stable idempotent evidence;
- HP-T0-2 reconciler confirms same logical publication without queue/DB authority;
- EC-T0-1 conflicting package/hash under same identity -> fail closed;
- EC-T0-2 invalid service identity -> no publish action;
- EC-T0-3 Availability Node success never independently sets `P2P_READY`.

## 3. Workflow / Reflection

T0 is docs/architecture-only, so local-stack and phase-1/phase-2 implementation review actions are `n/a`. Complex-band HITL was satisfied by the owner selection.

Integrated passes:

1. service-auth/replay/secret boundary — **PASS**;
2. O4 authority/idempotency/reconciliation consistency — **PASS**;
3. K1 ciphertext/key-deny-list consistency — **PASS**;
4. scope: no P2 source work, ADR-032 unchanged, no queue authority — **PASS**.

## 4. Selected contract

### Runtime

**AN-R1 — Node.js/TypeScript Availability Node — SELECTED**

Dedicated `apps/availability-node/` service using the Hyperdrive/Hyperswarm JS stack already proven by P1, operationally independent from mobile Bare. It owns ciphertext publication/evidence only and never owns PostgreSQL/business authorization or `P2P_READY`.

### Publication-control authentication

**AN-A1 — mTLS service identity — SELECTED**

Publication-control calls use TLS with mutual service authentication. Dispatcher/reconciler clients present a dedicated P2 service certificate; the Availability Node validates the configured trust anchor/pin. No application JWT signing secret or user bearer credential is reused.

### State semantics — ACCEPTED

`building -> publish_pending -> publishing -> reconciling -> ready`

`failed` is reserved for explicit terminal/non-retryable failure after bounded policy exhaustion or deterministic contract violation. Unknown remote outcome is `reconciling`, never `ready`; queue ACK, dispatch completion, health, or reachability never establish readiness.

### Minimum durable audit inventory — ACCEPTED

- publication intent created;
- K1 lineage sealed/server-wrapped;
- external publication confirmed;
- reconciliation entered due to unknown external result;
- `P2P_READY` transition;
- terminal publication failure after bounded policy exhaustion.

## 5. Decision rationale

AN-R1 + AN-A1 was selected because it keeps Hyperdrive/Hyperswarm in its native server ecosystem, avoids turning Bare into an operations platform, preserves a narrow replaceable trust boundary, and provides strong service identity without custom request-signing canonicalization or reuse of application credentials.

The selected design preserves D3/O4: PostgreSQL + transactional outbox remain authoritative, queue usage is optional/non-authoritative, and reconciliation remains the recovery safety net.

## 6. Owner checkpoint

**Satisfied 2026-09-05.** Owner approved AN-R1 + AN-A1, including the proposed state model and minimum audit inventory.

P2.T0 is closed. `P2.T1` is the next execution parent and requires its own frozen-path `scripts/rri.py` result and explicit owner approval before source edits.