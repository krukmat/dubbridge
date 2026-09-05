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
- **Scope:** the authorization, confidentiality, and transport boundary for
  invited audience playback delivered over P2P instead of server-side HTTP
- **Does not decide:** the mobile Bare runtime boundary (ADR-043, accepted),
  review-time playback (ADR-032, unchanged), or the rights/consent gates
  (ADR-008, ADR-028, unchanged)

> **This ADR is not accepted.** `docs/plan/mvp0-p2p-first.md` guardrail 9
> requires an accepted audience-delivery ADR before P2P invite delivery may
> be implemented. Until the owner accepts a version of this decision, `P2`
> cannot be presented for approval. Its § Open questions are genuine open
> questions, not editorial placeholders.

## Context

MVP0-P2P delivers invited playback over a peer-to-peer media data plane
rather than the server-side HTTP delivery boundary. `P0` proved Bare/Expo
native compatibility and `P1` establishes the mobile runtime foundation and
an isolated replication proof (ADR-043). Neither touches audience
authorization, encryption, or media transport.

Today, playback authorization is owned by ADR-032: the backend issues a
scoped, expiring `PlaybackGrant`, rewrites the HLS manifest, and serves
short-lived scoped segment references. Clients never construct object-store
keys, and `apps/api` remains the single authorization point. That contract
assumes the backend is also the *transport*.

P2P inverts the transport assumption while keeping the authorization
assumption. The external design input for this slice
(`docs/plan/mvp0-p2p-design-inputs.md`) states the intent as a control-plane /
data-plane split: DubBridge keeps auth, authorization, assets, ownership,
invites, structured state, and audit; P2P carries only encrypted package
publication, discovery, replication, and local ciphertext availability. The
Hyperdrive key is explicitly **not** the authorization boundary.

That leaves a boundary that no accepted ADR currently defines: what a viewer
must present, what the backend must verify, and what may cross into an
untrusted seeding runtime, when the bytes no longer travel through
`apps/api`.

Two source documents in the design input disagree about the resulting
playback path. `INVITE_CONTRACT.md` keeps S-125 in the path
(`viewer identity → verify claimed non-expired invite → S-125 playback
boundary → HLS → VideoPlayer`), while `ARCHITECTURE_P2P_GUIDANCE.md` and
`MVP_SCOPE.md` require the P2P data plane to replace server media delivery
for the certified path and forbid any HTTP/S3 media fallback during
certification. The reconciliation is the substance of this decision.

## Proposed decision

The following are proposed as the invariant core of the boundary. They are
derived from constraints already canonical in
`docs/plan/mvp0-p2p-first.md` (guardrails 8–11) and `docs/architecture.md`,
so they are the low-uncertainty part of the decision.

1. **Authorization stays in the control plane.** `apps/api` remains the sole
   authority on whether a given viewer may play a given asset. Possession of
   a Hyperdrive key, a replicated package, or a ciphertext cache never
   constitutes authorization.
2. **The data plane transports ciphertext only.** Every media file published
   to Hyperdrive is encrypted before publication. No participant in the P2P
   data plane — including the Availability Node and any peer — can derive
   plaintext from replication alone.
3. **ADR-032 is not replaced.** It remains authoritative for authenticated
   review playback. P2P audience delivery is an additional, separately gated
   path, not a migration of the existing one. A future decision may unify
   them; this one does not.
4. **Secrets never cross the runtime boundary.** The Bare worklet never
   receives the user password, the device private key, the server
   key-encryption key, the JWT signing key, or PostgreSQL credentials. The
   Availability Node never owns database credentials, user or invite data,
   the plaintext content key, business authorization, or backend signing
   keys.
5. **No token or plaintext key at rest.** Raw invitation tokens and plaintext
   content keys are never persisted or logged in any component. Invitations
   persist a token hash; content keys exist server-wrapped at rest and
   transiently in memory during playback.
6. **Certification forbids fallback.** The end-to-end certified path must
   complete with legacy HTTP media delivery disabled. A run that succeeds
   only because an HTTP or S3 media route served bytes is
   `MVP0_P2P_NOT_CERTIFIED`.
7. **P2P audience authorization is a parallel control-plane concept.** The
   owner selected `O3 parallel` at the D1 checkpoint on 2026-09-05. After a
   valid invitation claim, a distinct backend-owned audience authorization —
   not the claim alone, an ADR-032 `PlaybackGrant`, or possession of a
   Hyperdrive key or ciphertext — gates release of the wrapped content key.
   This composition choice does not define the authorization's name, record,
   API, fields, token, lifetime, or key-envelope mechanics.

## Open questions

The unresolved questions below must be answered before this ADR can move to
`Accepted`. Each unresolved item is a real design decision with more than one
defensible answer; an item explicitly marked resolved remains here to preserve
the numbered decision trail.

1. **Grant composition — resolved for D1 on 2026-09-05.** The owner selected
   `O3 parallel`: a distinct backend-owned audience-authorization concept
   follows a valid invitation claim and gates wrapped-content-key release.
   ADR-032 remains unchanged and no HTTP manifest/segment semantics are
   inherited by local P2P delivery. The evidence, neutral option matrix, exact
   owner response, and non-selected alternatives are recorded in
   `docs/audit/mvp0-p2p-adr044-d1-grant-composition.md`.
2. **Key envelope.** Content-key algorithm, envelope format, wrapping scheme,
   device key generation and storage, and revocation semantics. Global
   invariant 8 of the design input bounds this to one invitation, one viewer,
   one active device path for MVP-0 — which materially simplifies the
   envelope but must be recorded as a deliberate limitation, not an
   accidental one.
3. **Publication state.** The publication/outbox schema, its recovery
   semantics, and how P2P publication state relates to
   `PreparationStatus::Ready` without delaying S-120 readiness or its
   downstream transcription enqueue (`docs/plan/mvp0-p2p-first.md`
   guardrail 8).
4. **Availability Node trust and operation.** Deployment, authentication to
   the control plane's publication-control API, observability, and
   operational ownership of a component that is deliberately denied every
   existing credential.
5. **Certification profile.** How legacy HTTP media routes are disabled for
   the certified path without disabling control-plane APIs, and whether that
   profile is a runtime configuration, a build profile, or a test harness.
6. **Audit obligations.** Which P2P events are governance-significant under
   ADR-018 and therefore require a durable audit row — invite creation,
   claim, key unwrap, verification failure, and certification outcome are
   candidates.
7. **Device lifecycle.** Persistent product cache, device identity, sign-out
   wipe, and background execution requirements beyond P1's transient
   foreground proof.

## Risk analysis

| Risk | Failure mode | Mitigation |
|---|---|---|
| Key-as-authorization drift | An implementation treats possession of the Hyperdrive key or a synced package as permission to play | Proposed decision 1; certification must include a negative case where a synced package is unplayable without control-plane authorization |
| Premature acceptance | This ADR is accepted with § Open questions unresolved, and P2/P3 inherit undefined crypto | The status gate above: `Proposed` blocks P2 presentation by design |
| Non-binding hypothesis hardening into contract | The suggested invite routes and RPC surface in `docs/plan/mvp0-p2p-design-inputs.md` are implemented as if decided | That document labels every non-binding section; P3's task card must restate the chosen surface as its own decision |
| Availability Node scope creep | The node acquires a credential or authority to "make integration easier" | Proposed decision 4 is an explicit deny-list, testable at review time |
| Silent HTTP fallback | Certification passes because a legacy route served bytes | Proposed decision 6; G7 requires legacy media delivery disabled during the critical proof |

## Consequences

If accepted as proposed:

- `P2` gains a defined publication target: encrypted package plus durable
  publication state, with Ready gated on publication succeeding.
- `P3` gains a defined authorization target: an invite record persisting only
  a token hash, and a claim that binds one viewer and yields a minimal P2P
  access descriptor plus a wrapped content key.
- `P4`/`P5` gain a defined trust boundary: the worklet verifies and serves
  ciphertext it cannot independently decrypt without a key the control plane
  released to the app.
- ADR-032 and the S-125 implementation are untouched, so review playback
  carries no regression risk from this slice.
- A second decision will eventually be required if audience delivery and
  review delivery are to be unified; this ADR deliberately defers it.

## Alternatives considered

- **Extend ADR-032 in place.** Rejected for now: ADR-032's grant, manifest
  rewriting, and segment-reference semantics are built around server-side
  transport. Overloading them with a P2P meaning would make both paths harder
  to reason about, and would put an operational, already-shipped boundary at
  risk for an unproven one.
- **Make the Hyperdrive key the capability.** Rejected: it collapses
  authorization into transport, makes revocation impossible without
  re-publication, and contradicts the control-plane/data-plane split that is
  the premise of the slice.
- **Publish plaintext HLS and rely on swarm obscurity.** Rejected: it makes
  every peer and the Availability Node a plaintext holder, which no
  rights/consent posture in this repository tolerates (ADR-008, ADR-028).
- **Defer the ADR and let P2 decide implicitly.** Rejected: it is what
  `docs/plan/mvp0-p2p-first.md` guardrail 9 exists to prevent, and it would
  place a security boundary decision inside an implementation task card.

## Implementation sequence

This ADR gates work; it does not schedule it.

1. Owner reviews § Proposed decision and § Open questions.
2. Open questions 1–3 are resolved (they block `P2` and `P3` scope).
3. This ADR moves to `Accepted`, with the resolved answers written into
   § Proposed decision as the decision of record.
4. `docs/plan/mvp0-p2p-p2-*.md` is authored and the `P2` ledger entry is
   expanded to a full task per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
   steps 2–3.
5. `P2` is scored, presented, and approved separately. Acceptance of this ADR
   is not approval of any implementation task.

Open questions 4–7 may be resolved later, but each blocks the phase that
depends on it: 4 blocks `P2` deployment, 5 blocks `P7`, 6 blocks `P2`/`P3`
closure evidence, 7 blocks `P4`.

## References

- `docs/plan/mvp0-p2p-first.md` — guardrails 8–11 and § Deferred decisions
- `docs/plan/mvp0-p2p-design-inputs.md` — transcribed design inputs and their binding status
- `docs/tasks/mvp0-p2p-first.md` — MVP0-P2P task ledger
- `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md` — accepted mobile runtime boundary
- `docs/adr/ADR-032-hls-playback-delivery-boundary.md` — authoritative for review playback
- `docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md` — fail-closed rights precondition
- `docs/adr/ADR-018-structured-observability-traceable-events.md` — durable audit obligation
