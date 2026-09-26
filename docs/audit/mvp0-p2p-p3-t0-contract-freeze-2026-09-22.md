---
type: Audit
title: "MVP0-P2P P3.T0 contract and executable-path freeze"
task: P3.T0
date: 2026-09-22
status: pass
---

# P3.T0 — contract and executable-path freeze

## Disposition

P3.T0a–T0g are complete as a planning freeze. No runtime source is changed by
this artifact. The owner approved the subsequent P3.T1 block on 2026-09-22,
providing explicit T0 owner verification. Commit `ffc7ffa` also completed 15/15
CI checks successfully. P3.T0 is therefore PASS and P3.T1 is activated.

The 2026-09-18 retrospective was directionally correct that P3 lacked its own
contract artifact, but parts of its test inventory are now stale: current source
already contains repository integration coverage for invitation/authorization and
one envelope-release denial case.

## T0a — current implementation inventory

Current implementation surfaces:

- schema: `infra/migrations/0038_create_p2p_audience.sql`;
- invitation/device/O3 persistence: `crates/db/src/p2p_audience_repo.rs`;
- audience API: `apps/api/src/routes/p2p_audience.rs`;
- exact P2 ready read model: `crates/db/src/p2p_ready_repo.rs`;
- envelope release predicate join: `crates/db/src/p2p_envelope_repo.rs`;
- envelope API + server KEK unwrap: `apps/api/src/routes/p2p_envelope.rs`;
- HPKE device-envelope construction: `crates/p2p/src/device_envelope.rs`;
- JS native boundary: `mobile/src/p2p/device/DeviceIdentity.ts`;
- Android opaque-key implementation:
  `mobile/modules/dubbridge-p2p-keystore/android/src/main/java/com/dubbridge/p2pkeystore/DubBridgeP2PKeyStoreModule.kt`;
- DB integration evidence: `crates/db/tests/p2p_audience_repo.rs`;
- mobile audience client evidence:
  `mobile/__tests__/p2p/p2p-audience-service.test.ts`.

## T0b — invitation / claim / inbox freeze

The current contract is frozen as follows.

1. Invitation creation is owner-scoped and only succeeds for authoritative
   P2P_READY publication state.
2. Default invite TTL is 24h; maximum accepted TTL is 7d; non-positive/expired
   creation fails closed.
3. The raw invitation token is returned only by the create response. PostgreSQL
   stores only a 32-byte SHA-256 token hash.
4. One active device per subject is the MVP-0 model. Re-registering the same
   key identity is idempotent; a conflicting active key fails.
5. Claim locks the invitation row with `FOR UPDATE`, revalidates expiry,
   revocation, active viewer device and exact invitation publication readiness,
   then binds viewer + device atomically.
6. Repeating claim by the same viewer/device is idempotent. A different
   viewer/device loses and must fail closed.
7. Inbox lists only invitations already claimed by the authenticated viewer.
8. Claim creates/returns a distinct O3 authorization record. **Claim is not
   authorization to release CK** and possession of invitation/ciphertext is
   never sufficient for envelope release.

## T0c — O3 authorization and K1 device freeze

Envelope release remains a separate backend decision. Before CK may be sealed
to a device, the current join must prove all of:

- requested authorization belongs to the authenticated viewer;
- authorization is not revoked or expired;
- invitation identity equals authorization asset/publication/lineage;
- invitation is claimed by the same viewer and device, not revoked/expired;
- device belongs to that viewer and is still active;
- publication id + asset + lineage match;
- publication is `ready` with same confirmed lineage;
- external publication/manifest/sealed-K1 evidence is complete;
- matching outbox delivery is durably `delivered`.

The envelope binds profile, device key id, invitation id, viewer id, asset id,
publication id, lineage id, authorization id and expiry. Android owns the P-256
private key through Android Keystore, asserts that the private encoding is null,
performs ECDH natively and exposes no JavaScript/software-private-key fallback.
Missing native capability remains a STOP/fail-closed condition under D2/K1.

## T0d — secret and audit contract

### Secret deny-list

The following must never enter logs, audit detail, analytics, persistent mobile
storage, or API error text:

- raw invitation token or token hash;
- plaintext CK;
- raw server KEK;
- wrapped-CK bytes / sealed nonce as diagnostic payload;
- Android private-key material;
- HPKE plaintext or transient native ECDH/HKDF material;
- bearer JWT/session credentials.

Identifiers needed for correlation (asset/publication/lineage/invitation/
authorization/device ids, key id) are not secrets but must remain bounded.

### Frozen P3 audit inventory

P3 must add durable ADR-018-compatible events:

1. `p2p_device_registered`;
2. `p2p_invitation_created`;
3. `p2p_invitation_claimed`;
4. `p2p_audience_authorization_issued`;
5. `p2p_device_envelope_released`;
6. `p2p_audience_access_denied` with a bounded non-secret reason code.

Invitation lifecycle uses `correlation_id = invitation_id`; authorization/
envelope lifecycle uses `correlation_id = authorization_id`. Asset,
publication and lineage correlation is populated whenever known. Audit detail
may carry bounded ids/reason codes, never secret payloads.

## T0e — exact writable-path ownership

The implementation parent is intentionally decomposed. Shared paths cannot be
modified concurrently.

| Leaf | Purpose | Frozen writable paths |
|---|---|---|
| P3.T1a | concurrent different-viewer claim-race evidence | `crates/db/tests/p2p_audience_repo.rs` |
| P3.T1b | fail closed unless post-claim descriptor matches exact publication + lineage | `apps/api/src/routes/p2p_audience.rs` |
| P3.T1c | invitation/device/claim/authorization audit kinds + persistence/emission | `crates/domain/src/audit/{kind,event,tests}.rs`, `crates/db/src/audit_repo.rs`, `apps/api/src/routes/p2p_audience.rs`, relevant focused tests |
| P3.T2a | complete DB release-predicate denial matrix | `crates/db/tests/p2p_audience_repo.rs` |
| P3.T2b | envelope handler/binding and denial behavior | `apps/api/src/routes/p2p_envelope.rs`, focused API/unit tests |
| P3.T2c | JS no-fallback + Android opaque P-256/HPKE interoperability evidence | `mobile/src/p2p/device/DeviceIdentity.ts`, `mobile/modules/dubbridge-p2p-keystore/**`, focused mobile/native tests/evidence |
| P3.T2d | envelope-release/denial audit emission | audit paths above + `apps/api/src/routes/p2p_envelope.rs` |
| P3.T3 | integrated P3 certification/status only; source changes only for defects found by certification | P3 certification evidence + status docs; any repair gets its own scored leaf |

If T1b needs a new DB identity-specific read seam rather than a local explicit
identity check, `crates/db/src/p2p_ready_repo.rs` must be added to that leaf
before execution and the leaf must be rescored.

## T0f — evidence map and RRI decomposition

### Current evidence already present

`crates/db/tests/p2p_audience_repo.rs` currently proves:

- active-device registration idempotence/conflict;
- owner-only invitation creation and invalid expiry denial;
- invitation publication/lineage binding;
- same-viewer/device claim idempotence;
- foreign-viewer claim denial;
- viewer-scoped inbox;
- active authorization viewer scoping;
- claim denial after publication state drift;
- valid envelope release context;
- release denial after device revocation.

Therefore those behaviors are not to be reimplemented merely to satisfy an old
ledger statement.

### Gaps carried into executable leaves

- **G1 / T1a:** no true concurrent different-viewer race test currently proves
  one winner under the `FOR UPDATE` path.
- **G2 / T1b:** after claim commits, the API calls
  `get_ready_descriptor_by_asset(asset_id)`; it does not explicitly assert the
  returned descriptor's publication + lineage equal the invitation/
  authorization identity. Current schema has one publication per asset, but the
  P3 contract remains exact-lineage and must not depend on that incidental
  uniqueness.
- **G3 / T1c/T2d:** current audit event inventory contains P2 publication events
  but no P3 invitation/device/authorization/envelope events.
- **G4 / T2a:** release-context testing covers the happy path and revoked device,
  not the full wrong-viewer/expired-or-revoked auth/invite/non-ready publication
  matrix.
- **G5 / T2b:** envelope handler response/binding/error branches lack focused
  contract evidence.
- **G6 / T2c:** JS boundary is explicitly no-fallback in source and native code
  asserts non-exportability, but P3 still needs focused no-fallback and actual
  opaque-key HPKE interop certification before T3 closure.

### RRI envelope

The coherent P3 implementation parent is **RRI 100 / Very high**: authorization,
persistent state, audit, native crypto and secret custody make direct execution
as one patch invalid. T0 therefore decomposes it before implementation.

Planning scores for the leaves, to be mechanically rerun against the exact
current diff at activation:

- T1a: **RRI 70 / Complex** — transactional concurrency test.
- T1b: **RRI 70 / Complex** — authorization-adjacent exact-identity guard.
- T1c: **RRI 100 / Very high** — audit + authorization cross-module change.
- T2a: **RRI 70 / Complex** — fail-closed DB predicate matrix.
- T2b: **RRI 100 / Very high** — envelope/API security boundary.
- T2c: **RRI 100 / Very high** — native cryptographic/Keystore boundary.
- T2d: **RRI 100 / Very high** — security audit emission.
- T3: **RRI 100 / Very high** — cross-stack certification.

These are planning envelopes, not permission to execute. `scripts/rri.py`
remains authoritative immediately before each leaf; a lower exact score may be
recorded only from the then-current path/diff evidence.

## T0g — closure handoff

HP-P3.T0-1 is covered by the frozen API/schema/O3/K1/path contract above.
EC-P3.T0-1 is covered by explicit fail-closed readiness and Keystore STOP rules;
claim alone never grants envelope release.

P3.T0 is therefore **PASS 2026-09-22** (owner-verified; 15/15 CI green).
After PASS, execution order is:

```text
T1a claim-race evidence
 -> T1b exact-lineage descriptor guard
 -> T1c durable P3 audience audit
 -> P3.T1 PASS
 -> T2a predicate matrix
 -> T2b envelope API/binding
 -> T2c Android opaque-key/no-fallback proof
 -> T2d envelope audit
 -> P3.T2 PASS
 -> P3.T3 integrated certification
 -> P3 PASS
```

This does not alter the 2026-09-22 sequencing amendment: P3 PASS remains
required before P4 aggregate closure can satisfy the development chain, and
P5.T3 remains deferred to the release-certification lane.
