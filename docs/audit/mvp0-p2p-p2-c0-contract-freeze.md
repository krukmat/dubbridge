---
type: Audit
title: "P2.C0 shared contract, fixture, and ownership freeze"
status: complete
slice: MVP0-P2P
parent: P2.C0
---

# P2.C0 — shared contract freeze

## Status

**PASS — owner-approved and frozen 2026-09-06.**

C0 is a planning/contract task only. It does not modify or retrospectively review P2.T1 source/persistence, and it does not authorize P2.T2–T6 source execution. The accepted P2.T1 schema/repository are inputs to this contract.

Base at C0 presentation: `df20a4fdd857ff2783e35cdffa02c60f2e5cb218`.

Golden fixtures:

- `docs/fixtures/mvp0-p2p-manifest-v1.json`
- `docs/fixtures/mvp0-p2p-publication-contract-v1.json`

## 1. Manifest-v1 canonical contract

### Encoding and serialization

- Contract identifier: `p2p-manifest-v1`.
- Manifest bytes are UTF-8 JSON without BOM.
- Canonical bytes follow RFC 8785 JCS semantics, restricted to strings, integers, booleans, arrays, objects, and null; **floats are forbidden** in manifest/AAD contracts.
- Object keys are canonicalized by the JCS serializer. File entries are explicitly sorted by normalized path before serialization.
- Manifest digest is SHA-256 over the exact canonical UTF-8 manifest bytes and is encoded as **64 lowercase hexadecimal characters**.
- Ciphertext file digest is SHA-256 over the complete ciphertext object bytes (ciphertext including the AES-GCM authentication tag as produced by the selected API) and uses the same lowercase-hex encoding.

### Path normalization

Every package path is relative to the package root and must satisfy all of the following before encryption or hashing:

1. Unicode NFC normalization.
2. `/` is the only separator; `\\` is rejected rather than rewritten.
3. No leading `/` or drive/root prefix.
4. No empty segment, `.` segment, or `..` segment.
5. Case is preserved; no locale/case folding.
6. Ordering is ascending lexicographic order of the normalized UTF-8 byte sequence.
7. Two input names that normalize to the same path are a hard conflict; package construction fails before encryption.

The golden fixture includes valid, NFC-normalized, parent-traversal, absolute-path, and backslash cases.

### Required manifest fields

The v1 manifest contains:

- `manifest_version`
- `asset_id`
- `publication_id`
- `lineage_id`
- `cipher = "AES-256-GCM"`
- `digest = "SHA-256"`
- ordered `files[]`, each carrying:
  - normalized `path`
  - `plaintext_size`
  - `ciphertext_size`
  - `nonce_b64u`
  - `ciphertext_sha256`

Logical package identity is the PostgreSQL `publication_id + lineage_id` pair. A manifest digest is evidence for that identity, not a replacement identity.

## 2. K1 crypto and key-custody contract

### AES-256-GCM and canonical AAD

Each package file is encrypted independently under one lineage CK with AES-256-GCM.

The exact AAD object is:

```json
{
  "aad_version": "p2p-aad-v1",
  "asset_id": "<uuid>",
  "publication_id": "<uuid>",
  "lineage_id": "<uuid>",
  "manifest_version": "p2p-manifest-v1",
  "path": "<normalized-relative-path>"
}
```

AAD bytes use the same restricted JCS canonicalization as the manifest. The ciphertext digest is deliberately not part of AAD because it does not exist before encryption.

### Nonce allocation

- nonce length: exactly 96 bits / 12 bytes;
- source: CSPRNG;
- encoding in manifest: base64url without padding;
- uniqueness scope: every encryption under the same CK lineage;
- nonce allocation occurs once while building a new lineage;
- duplicate/collision detection during the build fails the lineage build closed;
- retry/reconciliation of an already sealed lineage **reuses the sealed ciphertext and manifest**, never re-encrypts the same logical lineage with newly allocated nonces.

### Generate-once CK

- A new logical lineage receives one fresh CSPRNG 256-bit CK.
- The CK is generated only when the lineage has no sealed K1 material.
- Before the lineage becomes seal-complete, the CK is wrapped under the currently active server KEK and the wrap metadata is durably persisted with the sealed package metadata.
- A retry, dispatch replay, queue redelivery, reconciler re-drive, or lost ACK cannot rotate CK or create a second package lineage.
- Explicit package replacement is a **new lineage** and is outside retry semantics.
- Plaintext CK is process-transient only, never stored/logged/audited/returned to the Availability Node; implementation must use best-effort memory zeroization after encryption/wrap use.

### Versioned KEK

- KEK material is injected through the backend secret boundary only.
- `kek_id + kek_version` identify a resolver entry; raw KEK bytes never enter PostgreSQL, logs, audit detail, Availability Node payloads, or mobile.
- The active KEK version is used only for **new** lineage wrapping.
- Historical versions remain resolvable for existing wrapped CK material during the beta lifetime.
- Retry uses the lineage's persisted KEK version; it never silently upgrades to the active version.
- Re-wrapping an existing lineage for rotation is explicit maintenance/deferred scope and cannot alter `publication_id`, `lineage_id`, package bytes, or manifest digest.

## 3. Availability Node contract v1

Runtime/trust remain the T0 owner choice: **AN-R1 Node.js/TypeScript + AN-A1 private mTLS**.

### Deployment handoff

For the October single-droplet beta, the builder and Availability Node share a dedicated persistent **ciphertext-only package volume**. The control contract carries an opaque normalized `package_ref` relative to the Availability Node's configured ciphertext root; it never carries an arbitrary absolute host path. This transport detail may later change without changing logical publication identity or the idempotency contract.

### Request

`PUT /v1/publications/{publication_id}` over private mTLS:

```json
{
  "contract_version": "availability-publication-v1",
  "publication_id": "<uuid>",
  "lineage_id": "<uuid>",
  "manifest_version": "p2p-manifest-v1",
  "manifest_digest_sha256": "<64-lowercase-hex>",
  "package_ref": "packages/<publication_id>/<lineage_id>"
}
```

No CK, wrapped CK, KEK, viewer/invite/device state, business authorization, DB credential, JWT signing material, or service private key is legal request content.

### Success evidence and idempotency

First confirmed seed/open returns HTTP `201`; an idempotent replay may return `200`. Both return:

- contract version;
- publication id;
- lineage id;
- manifest digest;
- stable `external_publication_id`;
- stable `evidence_id` for the same logical publication result;
- RFC3339 UTC `confirmed_at`.

Idempotency key is `(publication_id, lineage_id)` with the manifest digest as conflict evidence:

- same publication + lineage + digest => return stable existing evidence;
- same publication with different lineage/digest, or same lineage with different digest => **409 conflict**, fail closed;
- Availability Node success remains external evidence only. `P2P_READY` exists only after the backend durably persists the same-lineage confirmation in PostgreSQL.

### Error contract

- `400 invalid_contract` — malformed/unsupported contract.
- TLS handshake failure — client identity is absent/untrusted.
- `403 service_identity_rejected` — authenticated identity is not authorized, if application-level identity policy rejects it.
- `409 publication_conflict` — logical identity conflicts with lineage or manifest evidence.
- `422 package_invalid` — package_ref, manifest, normalized path, or ciphertext digest validation fails.
- `503 publication_unavailable` — transient storage/Hyperdrive failure.

Any network/5xx/ambiguous response is an **unknown external outcome** to the backend: PostgreSQL stays non-ready and T4 reconciliation operates on the same lineage.

## 4. ADR-018 P2 audit/correlation contract

The accepted T0 minimum event inventory is frozen with these event-kind strings:

1. `p2p_publication_intent_created`
2. `p2p_lineage_sealed`
3. `p2p_publication_confirmed`
4. `p2p_publication_reconciliation_entered`
5. `p2p_publication_ready`
6. `p2p_publication_failed`

The existing `audit_events.ingest_token UUID NOT NULL` schema is ingestion-specific and **must not be populated with a fabricated publication/lineage UUID**. T6 owns a backward-compatible audit-correlation extension. Existing ingestion events retain their current semantics.

Frozen P2 correlation model:

- `correlation_id = publication_id` for the P2 lifecycle;
- persist explicit nullable `publication_id` and `lineage_id` correlation fields alongside existing `asset_id`;
- P2 audit rows do not require a synthetic ingest token;
- schema must require at least one valid correlation route while preserving existing rows;
- audit `detail` may contain bounded non-secret reason/evidence identifiers, never CK/KEK/cert-private-key/raw secret material.

Transaction boundary by event:

| Event | Durable transaction boundary |
|---|---|
| intent created | same transaction as publication + initial outbox obligation |
| lineage sealed | same transaction that persists wrapped-CK reference/KEK version + manifest digest/package ref |
| publication confirmed | same transaction that persists same-lineage external evidence |
| reconciliation entered | same state-changing transaction that records the unknown/recovery condition |
| publication ready | same transaction as authoritative `ready` transition |
| publication failed | same transaction as terminal failure transition |

Required audit persistence is part of the success path per ADR-018. Structured trace emission shares correlation identifiers and follows the durable write; an audit persistence failure fails closed where the event is required for the operation's success.

## 5. Minimal P3 descriptor

P2 exposes no invitation/device-envelope API. The only frozen P3 handoff is an internal `p2p-ready-descriptor-v1` read model, materialized only when authoritative PostgreSQL state is `P2P_READY`:

```json
{
  "descriptor_version": "p2p-ready-descriptor-v1",
  "asset_id": "<uuid>",
  "publication_id": "<uuid>",
  "lineage_id": "<uuid>",
  "manifest_version": "p2p-manifest-v1",
  "manifest_digest_sha256": "<64-lowercase-hex>",
  "external_publication_id": "<opaque-stable-id>",
  "ck_wrap_ref": "<opaque-server-side-reference>",
  "kek_id": "<opaque-id>",
  "kek_version": 1,
  "ready_at": "<RFC3339-UTC>"
}
```

The descriptor contains no plaintext CK, wrapped CK bytes, invitation/viewer/device state, or device envelope. P3 resolves `ck_wrap_ref` inside the trusted backend boundary and separately enforces D1/O3 authorization plus every D2/K1 fail-closed device-envelope predicate.

## 6. Downstream decomposition and exact path ownership

The parents T2–T6 remain non-executable. The following leaves are the frozen implementation order and expected writable paths. Each leaf must run `scripts/rri.py` against its exact current paths immediately before presentation/execution; the values are intentionally not pre-scored here.

Shared-file rule: task IDs remain sequential. Within an eligible ADR-040 multi-writer task, only the orchestrator may write shared registries/module exports/lockfiles; all other writers receive disjoint paths and the same base SHA. `Cargo.lock`, root workspace membership, module-export files, and status documents are always integration-owner paths.

### T2 — K1 package construction

| Leaf | Scope | Writable paths | Depends on |
|---|---|---|---|
| `P2.T2a` | bootstrap dedicated P2 package crate + manifest/path/digest contract | `Cargo.toml`, `Cargo.lock`, `crates/p2p/Cargo.toml`, `crates/p2p/src/lib.rs`, `crates/p2p/src/manifest.rs` | C0 PASS |
| `P2.T2b` | read/snapshot existing prepared HLS package through storage seam | `crates/p2p/src/source.rs`, `crates/p2p/src/lib.rs`, `crates/p2p/Cargo.toml`, `Cargo.lock` | T2a |
| `P2.T2c` | AES-256-GCM per-file encryption and canonical AAD baseline | `crates/p2p/src/crypto.rs`, `crates/p2p/src/lib.rs`, `crates/p2p/Cargo.toml`, `Cargo.lock` | T2a |
| `P2.T2c-r1a` | additive internal assigned-nonce encryption primitive | `crates/p2p/src/crypto.rs` | T2c |
| `P2.T2c-r1b` | route the public CSPRNG entry through the assigned-nonce primitive | `crates/p2p/src/crypto.rs` | T2c-r1a |
| `P2.T2c-r2` | pure per-build nonce collision tracker and module export | `crates/p2p/src/nonce_tracker.rs`, `crates/p2p/src/lib.rs` | T2c-r1b |
| `P2.T2c-r3a` | private builder nonce-source seam with unchanged production behavior | `crates/p2p/src/package_builder.rs` | T2c-r1b; T2f |
| `P2.T2c-r3b` | tracker-before-encryption wiring and typed collision error | `crates/p2p/src/package_builder.rs` | T2c-r2; T2c-r3a |
| `P2.T2c-r3c` | deterministic full-build collision rejection evidence | `crates/p2p/src/package_builder.rs` | T2c-r3b |
| `P2.T2d` | generate-once CK + versioned KEK wrap/unwrap primitive and zeroization boundary | `crates/p2p/src/key_wrap.rs`, `crates/p2p/src/lib.rs`, `crates/p2p/Cargo.toml`, `Cargo.lock` | T2c |
| `P2.T2e` | persist sealed K1 metadata/wrapped-CK reference without changing T1 identity semantics | `infra/migrations/0033_extend_p2p_publications_k1.sql`, `crates/db/src/p2p_publication_repo.rs` | T2d; T1 accepted base |
| `P2.T2f` | assemble/seal ciphertext package and persist manifest/package evidence | `crates/p2p/src/package_builder.rs`, `crates/p2p/src/lib.rs`, `crates/p2p/Cargo.toml`, `Cargo.lock`, `crates/db/src/p2p_publication_repo.rs` | T2b–T2e |
| `P2.T2g` | K1/golden/cross-runtime certification | `crates/p2p/tests/k1_contract.rs` | T2f; T2c-r3c |

The new `crates/p2p` bounded context is intentional: cryptographic package/key-custody logic does not belong in generic `crates/media` or `crates/storage`.

### T3 — Availability Node

| Leaf | Scope | Writable paths | Depends on |
|---|---|---|---|
| `P2.T3a` | Node/TS service bootstrap + v1 request/response validation | `apps/availability-node/package.json`, `apps/availability-node/package-lock.json`, `apps/availability-node/tsconfig.json`, `apps/availability-node/src/contract.ts`, `apps/availability-node/src/server.ts` | C0 PASS |
| `P2.T3b` | private mTLS listener/client-identity policy; no public control listener | `apps/availability-node/src/mtls.ts`, `apps/availability-node/src/server.ts` | T3a |
| `P2.T3c` | persistent Hyperdrive seed/open + same-identity idempotency/conflict semantics | `apps/availability-node/src/hyperdrive_store.ts`, `apps/availability-node/src/server.ts` | T3b |
| `P2.T3d` | contract, mTLS, idempotency, traversal, and secret-deny certification | `apps/availability-node/test/publication_contract.test.ts`, `apps/availability-node/test/fixtures.ts` | T3c |

### T4 — O4 dispatcher/recovery

Optional queue acceleration is intentionally **not** a critical-path leaf. Direct PostgreSQL outbox dispatch + reconciliation must close P2 without a queue.

| Leaf | Scope | Writable paths | Depends on |
|---|---|---|---|
| `P2.T4a` | pure recovery decision kernel for pending/stale/unknown/ready states | `crates/domain/src/p2p_recovery.rs`, `crates/domain/src/lib.rs` | C0 PASS; T1 accepted base |
| `P2.T4b` | bounded PostgreSQL claim/lease/release operations | `infra/migrations/0034_add_p2p_publication_claim_leases.sql`, `crates/db/src/p2p_publication_repo.rs` | T4a |
| `P2.T4c` | backend mTLS Availability Node client implementing the C0 v1 contract | `crates/connectors/src/p2p_availability.rs`, `crates/connectors/src/lib.rs`, `crates/connectors/Cargo.toml`, `Cargo.lock` | T3 contract PASS |
| `P2.T4d` | outbox dispatcher using PG authority and AN client | `crates/jobs/src/p2p_publication_job.rs`, `crates/jobs/src/lib.rs`, `crates/jobs/Cargo.toml`, `Cargo.lock` | T4b; T4c |
| `P2.T4e` | worker runtime + PostgreSQL reconciler join | `apps/worker-runner/src/p2p_publication_runtime.rs`, `apps/worker-runner/src/main.rs`, `apps/worker-runner/Cargo.toml`, `Cargo.lock` | T4d |
| `P2.T4f` | O4 lost-dispatch/lost-ACK/stale-lease/duplicate certification | `apps/worker-runner/tests/p2p_publication_recovery_test.rs` | T4e |

A later optional queue accelerator, if desired after October critical-path closure, requires its own separately-scored task. Queue ACK/delivery never becomes publication authority.

### T5 — S-120 activation and P2P_READY handoff

| Leaf | Scope | Writable paths | Depends on |
|---|---|---|---|
| `P2.T5a` | characterization test proving current S-120 Ready + transcription post-ready ordering before P2 activation | `apps/worker-runner/src/preparation_runtime_tests/p2p_activation.rs`, `apps/worker-runner/src/preparation_runtime_tests.rs` | C0 PASS |
| `P2.T5b` | inert/fail-contained P2 activation after S-120 Ready and after the transcription post-ready call | `apps/worker-runner/src/p2p_activation.rs`, `apps/worker-runner/src/preparation_runtime.rs`, `apps/worker-runner/src/main.rs` | T2 PASS; T4 integration PASS; T5a |
| `P2.T5c` | authoritative ready read model + minimal P3 descriptor | `crates/domain/src/p2p_ready_descriptor.rs`, `crates/domain/src/lib.rs`, `crates/db/src/p2p_publication_repo.rs` | T5b |
| `P2.T5d` | S-120/ASR non-regression + queue/reachability-not-ready certification | `apps/worker-runner/tests/p2p_s120_non_regression_test.rs` | T5c |

The production hook placement is frozen from the existing S-120 seam: P2 activation occurs **after** `prepare_transcription_post_ready(...)` returns in `process_preparation_job`. P2 failure is contained to P2 state and cannot undo/delay `PreparationStatus::Ready` or suppress the existing transcription enqueue attempt.

### T6 — ADR-018 and deterministic closure

| Leaf | Scope | Writable paths | Depends on |
|---|---|---|---|
| `P2.T6a` | backward-compatible P2 audit correlation schema | `infra/migrations/0035_extend_audit_events_p2p_correlation.sql` | C0 PASS |
| `P2.T6b` | six P2 audit event kinds + durable emitter/repository correlation support | `crates/domain/src/audit/kind.rs`, `crates/domain/src/audit/event.rs`, `crates/domain/src/audit/tests.rs`, `crates/db/src/audit_repo.rs`, `crates/audit/src/lib.rs` | T6a |
| `P2.T6c` | deterministic six-window crash/recovery harness | `apps/worker-runner/tests/p2p_crash_windows_test.rs` | T2–T5 integration PASS; T6b |
| `P2.T6d` | ciphertext-only + secret-deny boundary certification | `apps/worker-runner/tests/p2p_secret_boundary_test.rs`, `apps/availability-node/test/secret_boundary.test.ts` | T3 PASS; T6b |
| `P2.T6e` | integrated P2 closure/evidence/status only | `docs/audit/mvp0-p2p-p2-t6-closure.md`, `docs/plan/mvp0-p2p-p2-encrypted-publication.md`, `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`, `docs/plan/mvp0-p2p-first.md`, `docs/tasks/mvp0-p2p-first.md`, `docs/plan/roadmap.md` | T6c; T6d; all P2 HP/EC evidence PASS |

Migration numbers `0033`, `0034`, `0035` are reserved by this integration order only; C0 does **not** create or modify migrations.

## 7. S-230 deployment-lane dependency freeze

C0 synchronizes the cross-slice contract as follows:

> **Documentation sync (2026-09-06):** this section records the owner-directed
> downstream S-230 sequence; it does not alter the frozen C0 package,
> cryptographic, publication, audit, fixture, or path-ownership contracts.

```text
P2.C0 PASS
     |
     v
S-230-T6p-a deployment ownership/config freeze
     |
     v
P2.T2 + P2.T3 stable contract implementation
     |
     v
S-230-T6p-b -> S-230-T6p-c
                         |
          S-230-T6 PASS + P2 PASS
                         |
                         v
                    S-230-T6p-d
       ciphertext publication + durable P2P_READY only
```

Exact dependency semantics:

- `T6p-a` depends on `P2.C0 PASS` and freezes only deployment-specific ownership/configuration. It consumes these C0 contracts/fixtures and does not redefine them.
- `T6p-b` cannot begin until `T6p-a PASS` plus stable `P2.T2` and `P2.T3` contract implementations. Package layout/crypto serialization remain frozen by C0.
- `T6p-c` depends on `T6p-b PASS` and proves the local deployment contract only.
- `T6p-d` depends on `S-230-T6 PASS + T6p-c PASS + P2 PASS` and proves only backend ciphertext publication plus durable `P2P_READY`.
- Invited playback remains outside T6p-d and requires P3-P6, T7p, P7, and T9g, including the X29/release requirements recorded by S-230.

## 8. Integration order

Critical-path source order is:

```text
C0 PASS
 -> T2a..T2g
 -> T3a..T3d
 -> T4a..T4f
 -> T5a..T5d
 -> T6a..T6e
 -> P2 PASS
```

Planning/test-fixture design for later workstreams may happen concurrently, but repository source execution remains one approved task ID at a time. T3 may start after C0/T2 contract stability without waiting for all T2 code if its exact leaf gate is separately approved; any cross-workstream shared-file collision forces sequential integration ownership. This does not authorize concurrent executable task IDs.

## 9. C0 Reflection — four required passes

### Pass 1 — cross-runtime determinism and K1 lineage

**Draft:** manifest-v1, canonical AAD, path order/digest encoding, nonce rules, and generate-once CK semantics were defined.

**Critique:** merely saying "canonical JSON" would leave Rust/TypeScript divergence around ordering, Unicode, numeric representation, and path aliases. Retrying encryption could also violate GCM nonce discipline while preserving a misleading lineage id.

**Revision:** froze restricted JCS/no-floats, NFC path rules, UTF-8 byte ordering, lowercase-hex SHA-256, base64url 96-bit nonces, and the rule that same-lineage retry reuses sealed ciphertext rather than re-encrypting.

**Verdict:** PASS.

### Pass 2 — trust and secret boundaries

**Draft:** mTLS and ciphertext-only AN boundary matched T0.

**Critique:** a filesystem path supplied by the backend could accidentally turn the Availability Node into an arbitrary file reader; KEK rotation language could also be interpreted as silent retry-time rewrap/lineage mutation.

**Revision:** made `package_ref` relative/opaque under one configured ciphertext root with traversal rejection; froze active-vs-historical KEK semantics, no retry-time version switch, and explicit secret deny-list.

**Verdict:** PASS.

### Pass 3 — crash consistency, audit, and S-120/S-230 joins

**Draft:** six T0 audit events were mapped onto P2 state transitions and S-230 deployment dependencies.

**Critique:** current `audit_events.ingest_token` is non-null and ingestion-specific; overloading it with P2 IDs would corrupt audit meaning. Also, placing P2 activation before transcription post-ready could make P2 availability delay the existing S-120 downstream path.

**Revision:** reserved T6a backward-compatible P2 correlation columns with no synthetic ingest token; froze P2 activation after `prepare_transcription_post_ready(...)`; added C0 as an explicit T6p-a completion dependency and kept T6p-d backend-only.

**Verdict:** PASS.

### Pass 4 — scope, ownership, and governance

**Draft:** T2–T6 were decomposed into implementation leaves.

**Critique:** generic shared modules and lockfiles could create path collisions; a new crypto package inside `media` would blur responsibilities; optional queue work could expand the October critical path. C0 also must not imply retrospective reopening of accepted T1.

**Revision:** created a planned dedicated `crates/p2p` ownership boundary, assigned exact paths and integration-owner treatment for shared exports/lockfiles, reserved only additive future migrations, explicitly preserved T1 as accepted input, and removed optional queue acceleration from the critical-path decomposition.

**Verdict:** PASS.

## 10. C0 closure

C0 is complete when its fixtures, this audit, P2 plan/task, S-230 plan/task, roadmap, ADR-044 status mirror, and RRI evidence are synchronized and documentation QA is green. No downstream source task becomes implicitly approved by C0 completion.
