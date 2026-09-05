---
type: ADR
title: "ADR-044: P2P audience delivery boundary"
status: Proposed
supersedes: ""
superseded_by: ""
---

# ADR-044: P2P audience delivery boundary

- **Status:** Proposed
- **Date:** 2026-08-28
- **Deciders:** DubBridge owner and backend/mobile maintainers (pending)
- **Scope:** the authorization, confidentiality, publication, and transport boundary for invited audience playback delivered over P2P instead of server-side HTTP
- **Does not decide:** the mobile Bare runtime boundary (ADR-043, accepted), review-time playback (ADR-032, unchanged), or the rights/consent gates (ADR-008, ADR-028, unchanged)

> **This ADR is not accepted.** `docs/plan/mvp0-p2p-first.md` guardrail 9 requires an accepted audience-delivery ADR before P2P invite delivery may be implemented. D1-D3 are now resolved, but ADR acceptance remains an explicit owner gate and `P2` is still not authorized.

## Context

MVP0-P2P delivers invited playback over a peer-to-peer media data plane rather than the server-side HTTP delivery boundary. `P0` proved Bare/Expo native compatibility and `P1` establishes the mobile runtime foundation and an isolated replication proof (ADR-043). Neither touches audience authorization, encryption, or product P2P publication.

Today, playback authorization is owned by ADR-032: the backend issues a scoped, expiring `PlaybackGrant`, rewrites the HLS manifest, and serves short-lived scoped segment references. Clients never construct object-store keys, and `apps/api` remains the single authorization point. That contract assumes the backend is also the transport.

P2P inverts the transport assumption while keeping the authorization assumption. The external design input for this slice (`docs/plan/mvp0-p2p-design-inputs.md`) states the intent as a control-plane / data-plane split: DubBridge keeps auth, authorization, assets, ownership, invites, structured state, and audit; P2P carries only encrypted package publication, discovery, replication, and local ciphertext availability. The Hyperdrive key is explicitly **not** the authorization boundary.

That leaves a boundary that no accepted ADR currently defines: what a viewer must present, what the backend must verify, how content-key custody works, how durable P2P publication reaches readiness, and what may cross into an untrusted seeding runtime when media bytes no longer travel through `apps/api`.

Two source documents in the design input disagree about the resulting playback path. `INVITE_CONTRACT.md` keeps S-125 in the path (`viewer identity → verify claimed non-expired invite → S-125 playback boundary → HLS → VideoPlayer`), while `ARCHITECTURE_P2P_GUIDANCE.md` and `MVP_SCOPE.md` require the P2P data plane to replace server media delivery for the certified path and forbid HTTP/S3 media fallback during certification. The reconciliation is the substance of this decision.

## Proposed decision

The following are proposed as the invariant core of the boundary.

1. **Authorization stays in the control plane.** `apps/api` remains the sole authority on whether a given viewer may play a given asset. Possession of a Hyperdrive key, replicated package, ciphertext cache, queue message, or Availability Node publication result never constitutes authorization.
2. **The data plane transports ciphertext only.** Every media file published to Hyperdrive is encrypted before publication. No participant in the P2P data plane — including the Availability Node and any peer — can derive plaintext from replication alone.
3. **ADR-032 is not replaced.** It remains authoritative for authenticated review playback. P2P audience delivery is an additional, separately gated path, not a migration of the existing one.
4. **Secrets never cross the runtime boundary.** The Bare worklet never receives the user password, device private key, server key-encryption key, JWT signing key, or PostgreSQL credentials. The Availability Node never owns database credentials, user/invite data, plaintext content keys, business authorization, or backend signing keys.
5. **No token or plaintext key at rest.** Raw invitation tokens and plaintext content keys are never persisted or logged. Invitations persist a token hash; content keys exist server-wrapped at rest and transiently in memory during playback.
6. **Certification forbids fallback.** The end-to-end certified path must complete with legacy HTTP media delivery disabled. A run that succeeds only because an HTTP or S3 media route served bytes is `MVP0_P2P_NOT_CERTIFIED`.
7. **P2P audience authorization is a parallel control-plane concept.** D1 selected `O3 parallel`: after a valid invitation claim, a distinct backend-owned audience authorization — not the claim alone, an ADR-032 `PlaybackGrant`, or possession of Hyperdrive/ciphertext — gates wrapped-content-key release.
8. **D2 selects the K1 key/envelope profile.** Each P2P package uses a fresh 256-bit content key and AES-256-GCM media encryption. The CK is persisted only server-wrapped with AES-256-GCM under a versioned server KEK. The device envelope uses HPKE Base with `DHKEM(P-256, HKDF-SHA256)` / `HKDF-SHA256` / `AES-256-GCM`, bound to invitation, viewer, active device key, asset/package, O3 audience authorization, and expiry. Android uses a P-256 ECDH key in Android Keystore; the private key is non-exportable by contract, StrongBox and external hardware are not required, missing required capability fails closed, and there is no silent software-key fallback. Revocation prevents new envelope releases; MVP-0 does not claim remote erasure of a CK already legitimately released to volatile memory. Bare may receive only a transient CK for the authorized playback session.
9. **D3 selects O4 for durable publication.** P2P publication has its own durable readiness boundary separate from S-120 `PreparationStatus::Ready`. PostgreSQL is authoritative for logical publication identity, current publication state, durable outbox intent, and the semantic `P2P_READY` transition. A transactional outbox is the durable consistency authority; an existing/future queue may be used only as a replaceable delivery accelerator; a PostgreSQL-driven reconciler is the recovery safety net. Queue enqueue/ack, dispatch, Availability Node reachability, or a transport timeout never establishes publication success. Delivery is at-least-once and idempotent under one stable logical publication identity and K1 lineage. Unknown external outcome remains non-ready until deterministic same-lineage reconciliation confirms the result or safely re-drives it. Only durable confirmation of the same logical publication may transition PostgreSQL to `P2P_READY`.
10. **S-120 readiness remains independent.** P2P publication must not delay S-120 `PreparationStatus::Ready` or downstream transcription enqueue. P2P readiness is an additional state/predicate downstream of existing preparation.

## Open questions

Resolved items remain numbered to preserve the decision trail.

1. **Grant composition — resolved for D1 on 2026-09-05.** Owner selected `O3 parallel`. Full evidence: `docs/audit/mvp0-p2p-adr044-d1-grant-composition.md`.
2. **Key envelope — resolved for D2 on 2026-09-05.** Owner selected `K1`. Full contract and four integrated Reflection passes: `docs/audit/mvp0-p2p-adr044-d2-key-envelope.md`.
3. **Publication state — resolved for D3 on 2026-09-05.** Owner selected `O4`: transactional outbox as durable authority, optional queue accelerator, PostgreSQL reconciler safety net, at-least-once/idempotent same-lineage delivery, fail-closed unknown outcome, and separate `P2P_READY` that never delays S-120 Ready/ASR. Full evidence and four integrated Reflection passes: `docs/audit/mvp0-p2p-adr044-d3-publication.md`.
4. **Availability Node trust and operation.** Deployment, authentication to the publication-control surface, observability, and operational ownership of a component deliberately denied existing DB/business/key credentials.
5. **Certification profile.** How legacy HTTP media routes are disabled for the certified path without disabling control-plane APIs, and whether that profile is runtime configuration, build profile, or test harness.
6. **Audit obligations.** Which P2P events are governance-significant under ADR-018 and therefore require a durable audit row.
7. **Device lifecycle.** Persistent product cache, device identity, sign-out wipe, and background execution beyond P1's transient foreground proof.

## D3 publication contract details

### Authority hierarchy

```text
PostgreSQL publication state + transactional outbox
        = durable authority

queue
        = optional/replayable delivery accelerator

PostgreSQL reconciler
        = recovery safety net

Availability Node
        = ciphertext publication executor / evidence source
```

No queue or Availability Node state independently establishes product readiness.

### Durable intent and identity

Before an external publication obligation exists, one PostgreSQL transaction durably establishes the stable logical publication identity, non-ready publication state, K1 lineage reference, and outbox publication intent. The Availability Node side effect is outside that transaction.

Outbox retry, queue duplicate/redelivery, reconciler re-drive, and Availability Node confirmation use the same logical publication identity. The encrypted package identity, manifest/hash evidence, and K1 wrapped-key lineage remain stable across retries. A future explicit package replacement is a new lineage and requires a separately approved transition.

### Crash recovery

The selected contract must close these windows:

- before durable commit: no publication obligation;
- after durable commit but before dispatch: outbox/reconciler recover the same work;
- crash/timeout during external publication: non-ready/unknown until same-identity reconciliation;
- remote success with lost ACK: no second logical package; retry/query converges on the existing result;
- ACK received before local Ready commit: PostgreSQL remains non-ready until reconciliation reconfirms and commits Ready;
- duplicate after Ready: idempotent no-op, with no CK/package rotation or readiness regression.

### Fail-closed `P2P_READY`

The semantic predicate becomes true only when authoritative PostgreSQL state durably proves the current lineage is valid, K1 package construction completed, external publication of that same lineage is confirmed, persisted publication identifiers/evidence correspond to the lineage, and no unresolved unknown-outcome condition remains.

If any fact is missing, stale, conflicting, or unknown, the package is **not P2P-ready** and D2 device-envelope release fails closed.

## Risk analysis

| Risk | Failure mode | Mitigation |
|---|---|---|
| Key-as-authorization drift | Hyperdrive/ciphertext possession is treated as permission | Control-plane authorization remains authoritative; D2 release consumes current D3 readiness |
| Premature acceptance | ADR accepted or P2 started before explicit D4 | ADR remains `Proposed`; D4 and P2 require separate gates |
| Key-custody downgrade | K1 failure silently exports/software-stores private key | D2 STOP condition; no automatic K2 fallback |
| Publication dual authority | Queue/Availability Node status becomes product truth | D3 O4 authority hierarchy: PostgreSQL + outbox only; queue is optional accelerator |
| Lost publication obligation | DB commit succeeds but enqueue/dispatch is lost | Transactional outbox persists obligation before external side effect; reconciler recovers |
| Duplicate publication/key lineage | Retry creates new package/CK after timeout | Stable logical publication identity and K1 lineage across outbox/queue/reconciler retries |
| False readiness after unknown outcome | Timeout or queue ACK produces Ready | Unknown outcome remains non-ready until deterministic same-lineage reconciliation |
| Availability Node scope creep | Node acquires DB/business/key authority | Explicit ciphertext-only deny-list |
| Silent HTTP fallback | Certification passes via legacy media route | G7 certification forbids fallback |

## Consequences

If accepted as proposed:

- `P2` gains a defined encrypted-package target **and** a long-term durable publication contract: PostgreSQL authoritative state, transactional outbox, optional queue acceleration, reconciler recovery, same-lineage idempotency, and separate fail-closed `P2P_READY`.
- `P2` must remain downstream of S-120 Ready so P2P publication cannot delay existing readiness or transcription enqueue.
- `P3` gains the O3 authorization + K1 device-envelope target and may release an envelope only when current D3 readiness and every other fail-closed predicate succeeds.
- `P4`/`P5` keep the ciphertext trust boundary; Bare may receive only a transient CK after authorized host-side unwrap.
- ADR-032/S-125 remain untouched for review playback.
- Queue technology, SQL names, retry constants, Availability Node deployment/auth, certification profile, audit-event inventory, and persistent device lifecycle remain later decisions.

## Alternatives considered

- **Extend ADR-032 in place.** Rejected: its manifest/segment semantics are server-transport-specific and already operational.
- **Make Hyperdrive key the capability.** Rejected: collapses authorization into transport and breaks revocation/control-plane authority.
- **Publish plaintext HLS.** Rejected: every peer/seed becomes a plaintext holder.
- **K2 portable HPKE with software-held private key.** Not selected: weakens K1 non-exportability and is not an automatic fallback.
- **K3 hardware JWE.** Not selected: unnecessary JOSE/session complexity for MVP.
- **D3 O1 pure transactional outbox.** O1 remains O4's consistency core, but the owner chose to make queue acceleration and reconciliation explicit in the long-term contract.
- **D3 O2 durable-state-reconciler.** Not selected: product state and work leasing/retry mechanics become too tightly coupled as the system evolves.
- **D3 O3 queue-primary-with-reconciliation.** Not selected: queue-primary operation adds a larger dual-mechanism correctness burden. O4 keeps queue use optional and subordinate to PostgreSQL/outbox authority.
- **Defer the ADR and let P2 decide implicitly.** Rejected: security/publication boundaries must not be hidden inside implementation tasks.

## Implementation sequence

This ADR gates work; it does not schedule it.

1. D1-D3 decision questions are resolved.
2. `ADR044-D4` is the next explicit architecture gate: owner reviews the consolidated proposed decision and, if approved, changes ADR-044 to `Accepted` with canonical status propagation.
3. Only after ADR acceptance is the P2 plan/task authored or expanded, scored, presented, and approved separately.
4. Acceptance of ADR-044 is not approval of P2 implementation.

Open questions 4–7 remain phase-specific: question 4 blocks P2 deployment completion, question 5 blocks P7, question 6 blocks P2/P3 closure evidence, and question 7 blocks P4 product lifecycle closure.

## References

- `docs/plan/mvp0-p2p-first.md`
- `docs/plan/mvp0-p2p-design-inputs.md`
- `docs/tasks/mvp0-p2p-first.md`
- `docs/tasks/mvp0-p2p-adr044.md`
- `docs/tasks/mvp0-p2p-adr044-d3.md`
- `docs/audit/mvp0-p2p-adr044-d1-grant-composition.md`
- `docs/audit/mvp0-p2p-adr044-d2-key-envelope.md`
- `docs/audit/mvp0-p2p-adr044-d3-publication.md`
- `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md`
- `docs/adr/ADR-032-hls-playback-delivery-boundary.md`
- `docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md`
- `docs/adr/ADR-018-structured-observability-traceable-events.md`
