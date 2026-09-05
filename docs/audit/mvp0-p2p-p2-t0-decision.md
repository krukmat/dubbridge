---
type: Audit
title: "P2.T0 Availability Node trust/operation decision"
status: complete
slice: MVP0-P2P
parent: P2.T0
---

# P2.T0 — Availability Node trust/operation decision

Date: 2026-09-05  
Branch: `feature/p2p-mvp-core`

## Owner decision

The repository owner approved the recommended P2.T0 contract:

- **Runtime: AN-R1 — dedicated Node.js/TypeScript Availability Node** under the P2 application boundary.
- **Publication-control authentication: AN-A1 — mTLS service identity**.
- **State semantics: accepted as proposed.**
- **Minimum ADR-018 audit inventory: accepted as proposed.**

This approval closes the remaining P2 phase-level architecture/security checkpoint. It does not authorize source execution for P2.T1-T6; each executable parent retains its own RRI/HITL gate.

## Frozen contract

### Runtime and ownership

The Availability Node is a dedicated Node.js/TypeScript service, operationally independent from mobile Bare. It owns only ciphertext seeding/opening and the narrow publication-control/evidence surface required by O4. It does not own PostgreSQL state, business authorization, audience authorization, invitations, viewer/device state, key release, or `P2P_READY`.

The implementation target is `apps/availability-node/` unless the executable T3 plan finds an existing repository packaging convention that requires an equivalent path. Any path change is mechanical packaging, not permission to change this trust boundary.

### Authentication

Publication-control calls use TLS with **mutual TLS service authentication**. Dispatcher/reconciler clients present a dedicated P2 service certificate; the Availability Node validates the configured trust anchor/pin according to the deployment profile.

The mTLS credential is distinct from all application/user credentials. In particular, the Availability Node must never receive or reuse:

- backend HS256 JWT signing material;
- user bearer tokens;
- PostgreSQL credentials;
- server KEK material;
- plaintext CK;
- invitation/viewer/device secrets;
- business authorization data beyond non-secret stable publication identity/metadata required for idempotency.

Certificate provisioning/rotation mechanics are implementation/deployment details for the executable T3/T6 work; they must remain environment-explicit and fail closed.

### Publication identity and idempotency

The control request binds to one stable logical publication/package identity and its current ciphertext manifest/hash lineage. Duplicate requests for the same identity and same package/hash are idempotent and return stable existing publication evidence. The same identity paired with conflicting package/hash metadata fails closed and must not overwrite or fork the logical publication.

Availability Node evidence is external-publication evidence only. It is never database authority and cannot directly transition `P2P_READY`.

### O4 observability/reconciliation surface

The node exposes only the narrow health/publication-evidence surface required for dispatcher and PostgreSQL reconciler operation. Reconciliation must be possible without queue visibility and without giving the node database access.

Unknown outcomes remain non-ready. If the remote side may have succeeded but the response is lost, PostgreSQL enters/re-enters reconciliation and verifies/re-drives the same logical publication lineage.

### Frozen semantic publication states

The implementation must preserve these semantic states:

`building -> publish_pending -> publishing -> reconciling -> ready`

`failed` is reserved for an explicitly terminal/non-retryable failure after the bounded policy is exhausted or a deterministic contract violation occurs.

Rules:

- timeout/unknown outcome -> `reconciling`, never `ready`;
- queue ACK/outbox dispatch completion -> never `ready`;
- node reachability/health -> never `ready`;
- durable same-lineage external confirmation + PostgreSQL transition -> `ready`;
- outbox work state remains separate from product publication readiness.

Exact SQL enum/text encoding and auxiliary work-state names are T1 implementation details, but they may not change these semantics.

### Minimum durable ADR-018 audit inventory

P2 must durably record at minimum:

1. publication intent created;
2. K1 package lineage sealed/server-wrapped;
3. external publication confirmed;
4. reconciliation entered because the external result is unknown;
5. `P2P_READY` transition;
6. terminal publication failure after bounded policy exhaustion.

Audit payloads must never contain plaintext CK, KEK, raw media plaintext/ciphertext bytes, service credentials, raw invitation tokens, or user bearer credentials.

## Accepted HP / EC contract

- **HP-T0-1:** authenticated same-identity publish using only stable publication identity + ciphertext metadata returns stable idempotent evidence.
- **HP-T0-2:** PostgreSQL reconciler can confirm/re-drive that same logical publication without queue authority and without database credentials on the Availability Node.
- **EC-T0-1:** same identity + same package/hash converges idempotently; same identity + conflicting package/hash fails closed.
- **EC-T0-2:** invalid/missing service identity performs no publication action and exposes no sensitive metadata.
- **EC-T0-3:** Availability Node success cannot itself create `P2P_READY`.

## Reflection passes

1. **Service-auth / replay / secret boundary — PASS.** mTLS provides dedicated service identity without sharing application JWT or symmetric backend signing material; conflicting identity/hash combinations fail closed.
2. **O4 authority / idempotency / reconciliation — PASS.** PostgreSQL + transactional outbox remain durable authority; queue is optional; node evidence is non-authoritative; unknown outcomes reconcile same-lineage.
3. **K1 confidentiality — PASS.** Availability Node remains ciphertext-only and receives neither plaintext CK nor KEK; no T0 choice weakens Android/device-envelope boundaries.
4. **Scope / compatibility — PASS.** ADR-032 review playback is unchanged; no P2 source implementation is performed in T0; T1-T6 keep independent execution gates.

## Disposition

**P2.T0: PASS / complete.**

`P2.T1 — durable publication identity + transactional outbox persistence` is the next execution parent. It must be re-scored with `scripts/rri.py` against frozen exact paths and presented/approved before source work.