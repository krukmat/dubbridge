---
type: Audit
title: "MVP0-P2P P3.T2-A backend release-boundary evidence"
task: P3.T2
block: T2-A
date: 2026-09-22
status: done
---

# P3.T2-A — O3 release predicates, envelope API and durable audit

## Disposition

Backend block T2-A (T2a + T2b + T2d) is complete. Final implementation head
`a228ddadc7056834c87aabdd9a27b52b0e4c1cf9` completed **15/15 CI PASS**.

This is a partial P3.T2 closure only. **P3.T2 remains in progress** because
T2-B/T2c (JS no-fallback, native binding/expiry enforcement and Android
Keystore opaque-key HPKE interop) is still pending.

## T2a — fail-closed release predicate matrix

Primary evidence:
- `crates/db/src/p2p_envelope_repo.rs`
- `crates/db/tests/p2p_audience_repo.rs`

The existing release query was preserved. New integration evidence proves no
release context for:

- wrong authenticated viewer;
- revoked authorization;
- expired authorization;
- revoked invitation;
- expired invitation;
- revoked device;
- authorization/device-subject binding drift;
- non-ready publication;
- reconciling publication with confirmation removed;
- missing sealed-K1 evidence;
- outbox no longer durably delivered.

Two attempted drift cases exposed stronger schema-level defenses rather than
runtime defects:

- `p2p_publications_confirmation_lineage_check` rejects a confirmed lineage
  different from the publication lineage;
- `p2p_audience_authorizations_publication_lineage_fk` rejects authorization
  publication/lineage pairs that do not exist.

The final test models those impossible states as schema-guard evidence instead
of weakening constraints merely to exercise the runtime query.

## T2b — envelope API / exact binding

Primary implementation:
- `apps/api/src/routes/p2p_envelope.rs`

Envelope construction is now isolated in a deterministic/testable backend seam.
The successful binding covers:

```text
profile_version
device_key_id
invitation_id
viewer_id
asset_id
publication_id
lineage_id
authorization_id
expires_at_unix
```

Focused tests prove exact binding serialization and fail-closed construction for:

- KEK id/version mismatch;
- invalid sealed nonce shape;
- wrapped CK authentication failure;
- invalid device P-256 SPKI.

The route retains:
- unavailable/malformed server KEK configuration -> service unavailable;
- authorization/release context unavailable -> no envelope;
- build mismatch/tamper -> conflict;
- audit persistence failure -> internal error, with no envelope returned.

No fallback media or software private-key path was added.

## T2d — durable release/denial audit

The handler now emits the P3 event kinds registered during T1:

- `p2p_device_envelope_released` after successful envelope construction and
  before response release;
- `p2p_audience_access_denied` for release-context denial, KEK unavailability
  and envelope-build failures.

Audit is fail-closed through `emit_governance_audit`: if durable persistence
fails, the endpoint returns 500 and does not release the envelope.

Bounded reason codes include:

```text
authorization_unavailable
kek_unavailable
kek_mismatch
sealed_nonce_invalid
wrapped_ck_invalid
device_key_invalid
```

Audit detail contains only identifiers and stable reason/operation names. It
does not contain plaintext CK, KEK bytes, wrapped CK, sealed nonce, device
private-key material, JWT or invitation token material.

## Implementation lineage

```text
35f04a2e  test(p3): complete envelope release denial matrix
44976e05  feat(p3): harden envelope API and durable audit
22c18fea  style(p3): apply T2-A rustfmt
a0ddc408  style(p3): trim T2-A test eof
767c2393  test(p3): model representable lineage drift
a228ddad  test(p3): align release drift matrix with schema guards
```

The red intermediate runs were diagnostic:
- first lineage mutation was rejected by the publication confirmation CHECK;
- second mutation was rejected by the publication/lineage FK;
- the final test intentionally records those as database defenses and uses
  representable states for runtime fail-closed evaluation.

## Final CI evidence

Exact implementation head:
`a228ddadc7056834c87aabdd9a27b52b0e4c1cf9`.

All 15 checks passed:

1. test
2. coverage
3. cargo-check
4. clippy
5. fmt
6. release-build
7. mobile
8. s3-integration
9. deny
10. config-secrets
11. peer-workflow-review
12. maintainability
13. python-complexity
14. roadmap-drift
15. qa-docs

## Acceptance mapping

T2-A closes the backend half of HP/EC-P3.T2-1:

- current O3 authorization + live invitation/device + exact package readiness is
  required before server CK unwrap/device envelope construction;
- wrong viewer, dead authorization/invitation/device, non-ready package,
  incomplete K1 state or missing delivery evidence yields no CK release;
- the returned envelope is cryptographically bound to the exact release
  identities and expiry;
- release/denial is durably auditable without secret payloads.

T2-A does **not** prove the Android opaque private-key STOP condition. That
remains T2-B/T2c and must close before P3.T2 PASS.

## Next block

```text
T2c1  JS native-only/no-software-fallback tests
  ↓
T2c2  parse + validate native binding and expires_at_unix before unwrap
  ↓
T2c3  Android Keystore opaque P-256 / HPKE interop evidence
  ↓
P3.T2 closure review
```
