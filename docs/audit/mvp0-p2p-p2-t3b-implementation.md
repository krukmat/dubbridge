---
type: Audit
title: "P2.T3b implementation evidence — private mTLS publication ingress"
status: complete
slice: MVP0-P2P
parent: P2.T3b
---

# P2.T3b implementation evidence

## Outcome

P2.T3b now provides an unbound TLS-1.3-minimum HTTPS server that requires a
CA-trusted client certificate and authorizes the exact SHA-256 certificate
fingerprint before delegating to the existing publication handler. A trusted
but unlisted identity receives the frozen `403 service_identity_rejected`
response; missing or untrusted certificates fail during TLS negotiation.

Implementation is complete and verified by Codex and by owner Matias on
2026-09-09. T3c has not started.

## Product and executable-evidence paths

- `apps/availability-node/src/mtls.ts` — fingerprint normalization, immutable
  allow-list policy, credential validation, and unbound HTTPS factory.
- `apps/availability-node/src/server.ts` — TLS authorization and exact-pin
  guard composed before the existing publication handler.
- `apps/availability-node/test/client-fingerprint-policy.test.js` — exact
  fingerprint/rotation overlap and invalid-configuration evidence.
- `apps/availability-node/test/private-mtls-server.test.js` — unbound,
  fail-closed transport-option evidence.
- `apps/availability-node/test/private-publication-ingress.test.js` — real TLS
  handshakes for missing, untrusted, trusted-unlisted, and allow-listed clients.

The test files were moved out of `docs/audit/` and renamed by behavior during
implementation. The canonical workflow now records that executable product
tests belong beside their package and must not encode roadmap/slice/task IDs
in filenames.

## RED/GREEN record

- Policy RED: importing `dist/mtls.js` failed before `mtls.ts` existed.
- Policy/transport GREEN: typecheck/build and four policy/transport cases
  passed after the policy and HTTPS factory were implemented.
- Ingress RED: certificate generation and module loading succeeded, then the
  harness failed only because `createPrivatePublicationServer` was absent.
- Integrated GREEN: all five T3b cases plus all four T3a regression cases
  passed after the authorized composition was added.

## Delegation and fallback lineage

- `T3b-i`: Qwen `qwen3.8:27b-mlx` initial test plus one bounded marker-removal
  repair; accepted after syntax and RED verification.
- `T3b-iii`: Qwen initial output required repair; the single local repair was
  rejected safely by the wrapper's 40-line replacement cap. The approved
  terminal fallback selected `gpt-5.6-luna` / `low` with
  `fallback-selection-v1`; packet SHA-256
  `15db9ddcaf0b822c846115e293e75ab40575e3bb0ece4449c2ac642d41cf0193`
  and receipt SHA-256
  `2809fc4d296f0ab16f155605ce20d99f9d95232f5abb3f8f10a8134aabfcc38e`.
  The cloud implementer changed only the transport test.
- `T3b-v`: Qwen initial test plus the one permitted bounded repair; accepted
  after reaching the intended missing-factory RED. No cloud fallback used.
- Integrated local advisory: Qwen remained responsive but exhausted its 4096
  output-token limit, so the read-only wrapper rejected the truncated result.
  This was non-gating because the active owner override makes phase 2 `n/a`;
  no cloud reviewer was invoked. Codex completed the required evidence-bound
  Reflection passes.

## Reflection log

Required passes: 4 (`100` Very high parent; four-pass Complex closure floor)

### Pass 1 — contract and identity semantics

- **Draft verdict:** the implementation enforces CA trust and an exact
  normalized SHA-256 pin before publication ingress.
- **Critique findings:** no production defect; exact comparison, rotation
  overlap, malformed pins, duplicates, and absent presented identity were
  covered.
- **Revisions applied:** none.

### Pass 2 — failure boundaries

- **Draft verdict:** TLS rejects missing/untrusted certificates and the
  application rejects trusted-unlisted identities without publisher calls.
- **Critique findings:** the missing-certificate request passed explicit
  `undefined` key/cert properties, and missing top-level configuration was not
  asserted directly.
- **Revisions applied:** omitted client credential properties entirely for the
  missing-certificate case; added absent allow-list and absent credential
  assertions.

### Pass 3 — scope and downgrade resistance

- **Draft verdict:** caller-supplied downgrade fields cannot override the
  factory's TLS settings; the returned server remains unbound.
- **Critique findings:** no source-scope violation or identity fallback found.
- **Revisions applied:** none; negative scan confirmed no `listen()`,
  environment/file credential loading, Hyperdrive/Postgres, readiness, secret,
  or CN/SAN fallback logic in the touched source boundary.

### Pass 4 — executable evidence and regression

- **Draft verdict:** T3b behavior and T3a ingress behavior pass together.
- **Critique findings:** CommonJS `.test.cjs` would execute but was not a
  supported `behavior-v2` evidence suffix.
- **Revisions applied:** converted all three package-local tests to native ESM
  `.test.js`; repeated syntax, typecheck, build, integrated tests, scope scan,
  and diff checks successfully.

## Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | trusted and allow-listed certificate reaches the publisher exactly once | integration | `apps/availability-node/test/private-publication-ingress.test.js::private publication ingress rejects unauthorized client identities` | passed |
| HP-2 | Happy path | Node and compact SHA-256 forms normalize exactly and rotation overlap accepts both pins | unit | `apps/availability-node/test/client-fingerprint-policy.test.js::normalizeSha256Fingerprint and createClientFingerprintPolicy accept exact identities` | passed |
| EC-1 | Edge case | missing or untrusted client certificate fails TLS and never calls publisher | integration | `apps/availability-node/test/private-publication-ingress.test.js::private publication ingress rejects unauthorized client identities` | passed |
| EC-2 | Edge case | trusted but unlisted certificate receives exact 403 with zero publisher calls | integration | `apps/availability-node/test/private-publication-ingress.test.js::private publication ingress rejects unauthorized client identities` | passed |
| EC-3 | Edge case | absent/malformed/duplicate pins and empty/missing credentials fail closed; TLS downgrade inputs are ignored | component | `apps/availability-node/test/client-fingerprint-policy.test.js::Configuration fail-closed`; `apps/availability-node/test/private-mtls-server.test.js::unbound HTTPS server factory enforces fail-closed mTLS options` | passed |

## Verification run by Codex

```text
npm --prefix apps/availability-node run typecheck
npm --prefix apps/availability-node run build
node --test apps/availability-node/test/*.test.js docs/audit/mvp0-p2p-p2-t3a-contract.test.js docs/audit/mvp0-p2p-p2-t3a-http.test.js
(cd apps/availability-node && node --test test/*.test.js)
if rg -n 'listen\(|process\.env|node:fs|readFile|hyperdrive|postgres|plaintext_ck|server_kek|P2P_READY|subjectAltName|commonName' apps/availability-node/src/mtls.ts apps/availability-node/src/server.ts; then exit 1; fi
git diff --check
```

Result: typecheck/build passed; 9/9 integrated tests and 5/5 package-local
tests passed; negative scope scan and diff check passed.

Code-solution review: n/a - REVIEW-OVERRIDE: urgency, see
`docs/audit/mvp0-p2p-review-exception.md`

## Owner final verification

- Owner: Matias
- Date: 2026-09-09
- Statement: Owner supplied the integrated verification result confirming that
  every mapped T3b happy path and edge case, together with all T3a regressions,
  passed: 9/9 tests, 0 failures, 0 skipped.
- Commands run: `node --test apps/availability-node/test/*.test.js docs/audit/mvp0-p2p-p2-t3a-contract.test.js docs/audit/mvp0-p2p-p2-t3a-http.test.js`.
