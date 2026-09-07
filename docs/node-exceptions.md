---
type: Policy
title: "Node.js/TypeScript Exception Boundary"
status: active
---

# Node.js/TypeScript Exception Boundary

DubBridge is intentionally Rust-first. Node.js/TypeScript exists only where an
established JavaScript-ecosystem library makes a full Rust replacement
disproportionately expensive or unavailable, and only inside the one scoped
service this exception names.

## Allowed service

- **Availability Node** (`apps/availability-node`, MVP0-P2P P2, ADR-044): a
  dedicated Node.js/TypeScript service using the Hyperdrive/Hyperswarm
  JavaScript ecosystem to seed encrypted P2P package ciphertext, behind a
  private mTLS-authenticated publication-control surface (`AN-R1 + AN-A1`,
  frozen by P2.T0).

No other `apps/*` or `crates/*` surface is covered by this exception. A new
Node.js/TypeScript service requires its own ADR-level justification and an
amendment to this file — it is not implicitly granted by this precedent.

## Constraints

- The Availability Node is **ciphertext-only**: it stores and seeds encrypted
  package bytes and has no visibility into plaintext content or keys.
- It holds **no PostgreSQL connection and no database of its own** — durable
  publication state and authorization decisions remain in `apps/api` (Rust),
  per ADR-044 D3 (`O4`: PostgreSQL + transactional outbox as durable
  authority).
- It has **no business-authorization authority** — it never decides who may
  access a publication; that remains `apps/api`'s sole responsibility
  (ADR-032, ADR-044).
- It holds **no plaintext key material** — K1 custody (AES-256-GCM package
  encryption, server-wrapped CK, HPKE P-256 device envelope) stays with
  `apps/api` and the mobile device's non-exportable Android Keystore key,
  never the Availability Node.
- It has **no backend-signing authority** — it does not issue JWTs, playback
  grants, or any other credential `apps/api` is authoritative for.
- Communication with `apps/api` is restricted to the private mTLS-
  authenticated publication-control surface defined by P2.T0; it is never
  exposed as a public/unauthenticated endpoint.
- Governance-significant events crossing this boundary still require the
  minimum ADR-018 P2 audit set frozen by P2.T0 — this exception does not
  relax the durable-audit invariant.

## Operational model

The Availability Node directory follows the same contract-isolation
discipline as a Python worker (`docs/python-exceptions.md`), adapted to its
own runtime:

- `package.json` / `tsconfig.json` — dependency and build boundary.
- `src/contract.ts` — the typed publication contract (`p2p-manifest-v1`,
  `availability-publication-v1`) it accepts and returns.
- `src/mtls.ts` — the private mTLS transport boundary to `apps/api`.
- `src/server.ts`, `src/hyperdrive_store.ts` — service implementation.
- `test/` — contract-level tests proving the ciphertext-only, no-DB,
  no-authorization boundary above.

This keeps the contract stable while allowing implementation changes behind
the service boundary, exactly as Python workers are isolated behind
`input.schema.json`/`output.schema.json`/`error.schema.json`.

## Related

- ADR-044 — P2P audience delivery boundary (accepted; source of this exception)
- ADR-043 — mobile P2P runtime ownership and proof isolation
- `docs/python-exceptions.md` — sibling exception boundary for Python ML workers
- `docs/architecture.md` § Core principles
