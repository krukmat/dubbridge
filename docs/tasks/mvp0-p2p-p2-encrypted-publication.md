---
type: TaskList
title: "Tasks: MVP0-P2P P2 encrypted publication"
status: ready_for_approval
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p2-encrypted-publication.md
behavioral_coverage_contract: behavior-v2
---

# P2 — encrypted P2P publication

## Parent status

- ADR-044: **Accepted 2026-09-05** — dependency satisfied.
- P1: Done — dependency satisfied.
- Parent P2 RRI: **131 Excessive / Effort XL** — no direct implementation.
- Parent disposition: mandatory re-scope into T0-T6 below.
- Source authorization: **none**. `P2.T0` is the next explicit owner gate.
- Review exception: existing owner-directed MVP0-P2P P0-P7 phase-1/phase-2
  review override remains in force; it does not waive RRI/HITL/Reflection/tests.

## Task map

The child scores below are conservative planning scores used to choose the gate;
each child must run the repository `scripts/rri.py` against its frozen exact paths
before its execution presentation. No score may be silently lowered to obtain a
cheaper route.

| ID | Objective | Planning RRI | Effort | Status | Depends on |
|---|---|---:|---|---|---|
| P2.T0 | Freeze Availability Node trust/operation + concrete O4 implementation contract | 64 Complex | L | **NEXT OWNER GATE** | ADR-044 Accepted |
| P2.T1 | Durable publication identity, state machine, migration + transactional outbox repository | 78 High | XL | Pending | T0 PASS |
| P2.T2 | K1 encrypted package construction + server-wrapped CK custody | 82 High | XL | Pending | T1 PASS |
| P2.T3 | Availability Node ciphertext publication executor | 72 High | XL | Pending | T0 + T2 PASS |
| P2.T4 | O4 outbox dispatch, optional queue acceleration + reconciler | 84 High | XL | Pending | T1 + T3 PASS |
| P2.T5 | S-120 downstream integration + fail-closed P2P_READY transition | 74 High | XL | Pending | T2 + T4 PASS |
| P2.T6 | ADR-018 audit inventory, crash-window integration certification + P2 closure | 76 High | XL | Pending | T1-T5 PASS |

Every High/Complex implementation parent is decomposed again before source edits;
Low-band maximization is applied only at real pure/mechanical seams.

## P2.T0 — Availability Node / O4 implementation contract

- **Type:** architecture/security contract, docs only.
- **Planning RRI:** 64 Complex / Effort L.
- **Allowed paths:** this ledger; linked P2 plan; a new T0 audit; ADR-044 only if a
  clarification is needed without reopening D1-D3; canonical status docs.
- **No source paths.**

### Decisions T0 must freeze

1. Availability Node runtime/process ownership and repository path.
2. Authentication for the narrow publication-control surface.
3. Request idempotency/replay binding to stable logical publication identity.
4. Node observability/health evidence consumed by O4 reconciliation.
5. Secret/config allow-list and deny-list.
6. Concrete publication/outbox states and claim/recovery semantics needed by T1-T4.
7. ADR-018 P2 audit-event minimum set required before P2 can close.

### Runtime options

- **AN-R1 — Node.js/TypeScript service.** Dedicated `apps/availability-node/`
  process using the same Hyperdrive/Hyperswarm ecosystem already proven in P1.
  Operationally conventional, separate from mobile Bare ownership, and easy to
  containerize. Recommended baseline.
- **AN-R2 — standalone Bare service.** Minimizes JS runtime differences versus the
  mobile worklet but makes service operations/health/tooling less conventional.
- **AN-R3 — Rust control process + embedded/spawned JS/Bare seeder.** Strong process
  control but adds an unnecessary cross-runtime orchestration boundary for MVP0.

### Publication-control authentication options

All options use TLS server authentication and a private/non-public control endpoint.
None gives the Availability Node DB, backend JWT-signing, KEK, or business authority.

- **AN-A1 — mTLS service identity.** Worker/dispatcher presents a dedicated client
  certificate; Availability Node validates its CA/pin. Strongest service identity
  and replay resistance at the transport layer; requires certificate lifecycle.
  **Recommended for long-term operation.**
- **AN-A2 — dedicated Ed25519 request signing.** Publisher holds a dedicated P2
  service-signing private key; node holds only the public key. Request signature binds
  method/path/body digest/logical publication id/timestamp. Strong separation but
  adds canonical request/replay-window logic.
- **AN-A3 — dedicated bearer/HMAC service secret.** Operationally simplest for MVP
  but symmetric credential compromise affects both verification and signing sides;
  least attractive long-term choice.

No option may reuse the backend HS256 JWT signing secret.

### State model proposed for T0 owner review

Semantic state names; SQL encoding may be enum/text with constraints after T1 review:

`building -> publish_pending -> publishing -> reconciling -> ready`

with `failed` only for an explicitly terminal/non-retryable condition. A timeout or
unknown remote result transitions to/remaining `reconciling`, never `ready` or
terminal `failed` solely because the response was lost.

Outbox work state is separate from product readiness. A queue ACK or outbox dispatch
completion never changes publication state to `ready`.

### Minimum audit inventory proposed

Durable ADR-018 events at minimum:

- P2 publication intent created;
- K1 package lineage sealed/server-wrapped;
- external publication confirmed;
- publication entered reconciliation due to unknown result;
- `P2P_READY` transition;
- terminal publication failure after bounded policy exhaustion.

No audit event includes plaintext CK, KEK, raw ciphertext bytes, raw tokens, or
service credentials.

### HP / EC

- **HP-T0-1:** chosen runtime/auth contract lets an authenticated dispatcher request
  publication using only stable publication identity + ciphertext metadata and
  receive idempotent evidence.
- **HP-T0-2:** reconciler can query/confirm the same logical publication without
  queue state and without DB credentials on the Availability Node.
- **EC-T0-1:** replay/duplicate command for the same identity converges idempotently;
  a conflicting package/hash for the same identity fails closed.
- **EC-T0-2:** unauthenticated/expired/invalid service identity receives no publish
  action or sensitive metadata.
- **EC-T0-3:** Node cannot become Ready authority even if it reports success.

### Acceptance

Owner selects one runtime option and one auth option (or a reviewed variant), and
accepts/adjusts the proposed state/audit minimum. T0 then codifies the selected
contract mechanically and stops before source implementation.

## P2.T1 — durable publication + outbox persistence

### Scope

- Domain value/state types for stable publication identity and lineage.
- Migration for publication record and outbox intent.
- Atomic create/ensure publication + outbox transaction.
- Repository operations for claim, retry/reconcile, external confirmation, and Ready.
- Database constraints preventing duplicate active logical publication and invalid
  direct Ready transitions.

### Candidate allowed paths

`crates/domain/src/**`, `crates/db/src/**`, `infra/migrations/**`, focused DB tests,
T1 audit/status docs. Exact files freeze at presentation.

### HP / EC

- **HP-T1-1:** one transaction creates non-ready publication identity + durable
  outbox obligation.
- **HP-T1-2:** restart/reclaim sees the same identity and outstanding work.
- **EC-T1-1:** duplicate create for same logical lineage is idempotent/conflict-safe,
  not a second package.
- **EC-T1-2:** database cannot set Ready without persisted same-lineage confirmation.

### Low-band candidates

Pure publication-id/state parsing and deterministic validation may become Low after
exact scoring. Migration/state constraints and transaction/repository transitions
must retain their real above-Low bands.

## P2.T2 — K1 encrypted package builder

### Scope

- Read existing S-120 prepared package through approved storage seams.
- Generate new lineage CK once.
- AES-256-GCM encrypt each package file with unique nonce.
- Versioned canonical AAD/manifest and ciphertext hashes.
- Persist only server-wrapped CK + metadata.
- Reuse frozen lineage on dispatch/retry; package replacement is a new explicit
  lineage, never an implicit retry side effect.

### HP / EC

- **HP-T2-1:** valid HLS package becomes a complete ciphertext-only K1 package with
  reproducible manifest/hash evidence.
- **EC-T2-1:** nonce collision/reuse, missing file, manifest mismatch, wrap failure,
  or storage failure leaves publication non-ready and no plaintext CK persisted.
- **EC-T2-2:** logs/errors/redaction never reveal plaintext CK/KEK/media plaintext.

Crypto/key-custody code is never forced into Low-band delegation.

## P2.T3 — Availability Node executor

### Scope

Implement only the T0-selected runtime/auth contract. Seed/open ciphertext package
bytes and return deterministic idempotent publication evidence.

### HP / EC

- **HP-T3-1:** authenticated same-identity publish returns stable Hyperdrive/publication
  evidence.
- **EC-T3-1:** same identity + different package/hash fails conflict-safe.
- **EC-T3-2:** service has no DB/business/key authority and cannot decrypt package.

## P2.T4 — O4 dispatcher + queue accelerator + reconciler

### Scope

- Claim committed outbox work.
- Dispatch directly or through optional existing queue acceleration.
- Re-drive stale/unknown publication from PostgreSQL truth.
- Reconcile lost dispatch, remote-success/ACK-loss, local Ready-commit loss.
- Duplicate-after-Ready is idempotent.

### HP / EC

- **HP-T4-1:** committed-before-dispatch crash is recovered.
- **HP-T4-2:** queue unavailable -> reconciliation/direct dispatch still converges.
- **EC-T4-1:** timeout/unknown result never yields Ready.
- **EC-T4-2:** duplicate queue delivery never rotates lineage or regresses Ready.

## P2.T5 — S-120 integration + P2P_READY

### Scope

- P2 trigger/downstream seam after required prepared HLS exists.
- Preserve S-120 Ready/ASR behavior.
- Consume T1-T4 confirmation to transition Ready fail-closed.
- Produce minimal internal P2 descriptor for future P3; no viewer/device envelope API.

### HP / EC

- **HP-T5-1:** S-120 Ready remains observable and ASR can start while P2 publication
  proceeds independently.
- **HP-T5-2:** confirmed same-lineage publication becomes P2P_READY.
- **EC-T5-1:** P2 failure does not undo/delay S-120 Ready; only P2 remains unavailable.
- **EC-T5-2:** queue ACK/reachability without durable confirmation stays non-ready.

## P2.T6 — audit + crash-window certification + closure

### Scope

- Freeze and implement final ADR-018 P2 audit inventory.
- Cross-component integration tests for PostgreSQL, worker, queue-optional path, and
  Availability Node contract.
- Six D3 crash-window scenarios.
- Ciphertext-only and secret-deny-list evidence.
- Status synchronization and owner verification.

### HP / EC

- **HP-T6-1:** clean publication passes full path and durable audit evidence.
- **EC-T6-1:** each injected crash window converges without false Ready/second lineage.
- **EC-T6-2:** audit persistence failure on required success path fails closed where
  ADR-018 requires it.

## Parent closure criteria

P2 is PASS only when T0-T6 are individually closed, every behavior maps to passing
executable evidence, integrated Reflection passes cover confidentiality, crash
consistency, readiness separation, and scope, canonical docs are synchronized, and
the repository owner performs final verification.

P3 remains blocked until that P2 PASS is recorded.
