---
type: Audit
title: "MVP0-P2P ADR-044 D2 key-envelope decision"
task: ADR044-D2
date: 2026-09-05
---

# MVP0-P2P ADR-044 D2 — key-envelope decision

## Decision boundary

This audit closes only ADR-044 D2: the content-key and device-envelope contract. It does not accept ADR-044, resolve D3 publication/outbox semantics, authorize P2/P3 source work, or modify ADR-032.

Parent RRI: **70 / Complex / Effort L**. The owner approved the parent envelope and selected **K1** at `ADR044-D2-OWNER`.

## Selected K1 contract

- One fresh random 256-bit CK per P2P package; package media uses AES-256-GCM.
- CK is persisted only server-wrapped under a versioned AES-256-GCM KEK; plaintext CK is transient and never persisted/logged.
- Android creates a P-256 ECDH key pair through Android Keystore; the private key is non-exportable by contract.
- StrongBox is optional, not required. No external hardware/HSM/USB token/TPM peripheral is required.
- Missing required Keystore/ECDH capability fails closed; there is no silent software-private-key fallback to K2.
- Device envelope uses HPKE Base `DHKEM(P-256, HKDF-SHA256)` / `HKDF-SHA256` / `AES-256-GCM`.
- The envelope semantically binds device-key id, invitation id, viewer id, asset id, package id, O3 audience-authorization id, profile version, and expiry.
- Envelope release fails closed unless viewer, claimed invitation, active device, O3 authorization, asset/package, D3-defined P2P readiness/publication predicate, expiry, and revocation all validate.
- Revocation prevents new releases; MVP-0 does not claim remote erasure/DRM for a CK already legitimately released to volatile memory.
- Bare may receive only a transient plaintext CK for the authorized playback session; it never receives device private key or backend secrets.
- Availability Node receives/serves ciphertext only and receives no plaintext CK, invitation/viewer authority, DB credentials, KEK, or backend signing keys.

## Implementation STOP condition

Before P3 may close, the Android native adapter must prove HPKE/P-256 can operate against the opaque Android Keystore private key without exporting it. If not, implementation must STOP and reopen D2; it may not silently downgrade to K2.

## Alternatives not selected

- `K2 portable HPKE`: not selected because software-private-key portability weakens K1's non-exportable key property.
- `K3 hardware JWE`: not selected because JOSE/JWE complexity is not justified for MVP absent an external interoperability requirement.

Neither is an automatic runtime fallback.

## Integrated Reflection

1. Crypto/key custody — PASS.
2. Binding/expiry/revocation — PASS.
3. Runtime authority/secret leakage — PASS.
4. Status/scope discipline — PASS.

## Verification disposition

Cloud environment has no Ollama, local models, Android device, or emulator. Local/device precheck is `n/a`; no evidence is simulated. Phase-1/phase-2 review is `n/a` under the ADR/plan/task-ledger-only exemption. ADR-044 remains `Proposed`; D3 and D4 remain gated; P2/P3 source work is not authorized by D2.
