---
type: Audit
title: "MVP0-P2P P3.T3c secret-boundary certification"
task: P3.T3
leaf: T3c
date: 2026-09-22
status: pass
---

# P3.T3c — secret-boundary certification

## Result

**PASS** against implementation head `6c3a565c2a33ae6253164d1ff3b6507e028e3e13`.

The exact head completed **15/15 GitHub CI checks successfully**. The P3
integration suite executed:

- `p3_t3a_owner_invite_claim_o3_and_envelope_binding_are_integrated` — PASS;
- `p3_t3b_envelope_release_fails_closed_across_live_o3_device_and_package_boundaries` — PASS;
- `p3_t3c_backend_audit_and_storage_are_secret_boundary_clean` — PASS.

The same three cases passed under the coverage runner. Workspace line coverage
remained **90.43%**, above the 90% gate.

The static T3c guard also executed in the `config-secrets` job and returned:

```text
P3_T3C_SECRET_BOUNDARY=PASS
```

## T3c.1 — API/backend log boundary

`scripts/p3-t3c-secret-boundary.py` rejects direct debug/print/tracing calls
inside the P3 audience/envelope route sources. The governance audit emitter
continues to trace only correlation/session/publication/lineage identifiers and
`event_kind`; it does not trace the durable `detail` payload.

Result: **PASS**.

## T3c.2 — durable audit events

The integrated HTTP/PostgreSQL test exercises device registration, invitation
creation, claim, O3 authorization, envelope release and an access denial. It
requires the correlated inventory:

- `p2p_device_registered`;
- `p2p_invitation_created`;
- `p2p_invitation_claimed`;
- `p2p_audience_authorization_issued`;
- `p2p_device_envelope_released`;
- `p2p_audience_access_denied`.

Every JSON detail is recursively checked for forbidden secret keys and known
secret fixture values.

Result: **PASS**.

## T3c.3 — mobile/native logging boundary

Static certification covers the P3 device TypeScript boundary and the Android
Keystore module. It rejects console/Android logging and local persistence APIs
at this boundary. The native module still requires:

- `AndroidKeyStore`;
- `privateKey.encoded == null`;
- zeroization of ECDH/shared-secret, ciphertext, plaintext CK, derived HPKE key
  and nonce buffers.

This reuses the already-certified T2c3 Android Keystore/HPKE behavior; no
GitHub-hosted Android emulator was re-enabled or used.

Result: **PASS**.

## T3c.4 — persisted storage inspection

The integrated DB test inspects the relevant P3 tables and rejects plaintext
secret columns. It additionally proves:

- invitation persistence remains hash-only;
- the persisted token hash is 32 bytes and differs from the raw invitation
  token;
- persisted `sealed_nonce` and `sealed_wrapped_ck` are not the plaintext CK;
- device persistence contains public identity only; no private-key field is
  present;
- Android private-key custody remains the native Keystore boundary certified
  by T2c3.

Result: **PASS**.

## T3c.5 — deny-list certification

The executable checks cover the frozen P3 deny list:

| Secret/material | Certification |
|---|---|
| raw invitation token | absent from envelope/audit; hash-only DB persistence |
| token hash | DB-only; rejected from audit/envelope |
| plaintext CK | rejected from envelope/audit/persisted K1 fields |
| KEK | rejected from envelope/audit |
| wrapped CK / nonce diagnostics | rejected from audit/log boundary |
| Android private-key bytes | opaque Keystore key; static no-log/no-storage guard |
| HPKE shared/transient material | native zeroization guards present; no logging |
| JWT/session bearer material | rejected from audit/envelope fixture values |

Result: **PASS**.

## T3c.6 — correlation-safe evidence

Troubleshooting correlation remains available without secret material through
bounded IDs and reason codes: invitation, device, authorization, asset,
publication and lineage identities plus P3 event kind / denial reason.

Result: **PASS**.

## Closure boundary

**T3c PASS.**

This does **not** close P3.T3 or aggregate P3. T3d (evidence map, downstream
handoff, final exact-head certification and owner verification) remains
pending.

GitHub-hosted Android HPKE emulator execution remains hard-disabled by owner
direction.
