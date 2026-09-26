---
type: Audit
title: "MVP0-P2P P2.T3b RRI and honest Low-band decomposition"
date: 2026-09-09
task: MVP0-P2P-P2-T3b
---

# P2.T3b — RRI and honest Low-band decomposition

## Parent outcome envelope

Command:

```bash
python3 scripts/rri.py --platform generic \
  --touches apps/availability-node/src/mtls.ts \
  --touches apps/availability-node/src/server.ts \
  --cc 10 --D 3 --K 3 --P 5 --T 4 --A 0 --X 3 \
  --penalty auth_security
```

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 1 | raw CC 10, bounded policy/transport/guard branches | High |
| F files | 1 | two C0-frozen product paths | High |
| D domain | 3 | Node HTTPS integration implementing the frozen mTLS boundary | High |
| T coverage | 4 | T3a tests exist, but no task-specific mTLS evidence exists | High |
| A ambiguity | 0 | objective, boundaries, HP/EC cases, and leaves are frozen below | High |
| K coupling | 3 | TLS socket, certificate, HTTPS handler, and future T3c/T4c integration | High |
| P impact | 5 | authentication/security boundary | High |
| X context | 3 | T0/C0, ADR-044, T3a contract/server, and downstream ownership | High |

**Technical profile (v2, ADR-045):** L=1 I=3 Q=3 V=4 -> bottleneck B=4 -> ICI=100
**Risk/domain band input:** 46 (D/P/K weighted + penalties 20)
**Penalties applied:** `auth_security` (+10); `no_tests_high_impact` (+10)
**Final RRI:** **100 — Very high (86–100), Effort XL.**
**Gates:** no direct parent implementation; contract-first evidence and mandatory
decomposition; one parent approval; at least four integrated Reflection passes.

## Frozen design boundary

- `mtls.ts` owns exact SHA-256 client-certificate fingerprint normalization,
  allow-list policy, mandatory `requestCert=true` / `rejectUnauthorized=true`
  TLS options, TLS 1.3 minimum, and construction of an **unbound**
  `https.Server`.
- `server.ts` composes the already-authenticated TLS request with the
  application fingerprint policy. A trusted-but-unlisted certificate receives
  `403 service_identity_rejected`; the publication executor is not invoked.
- Missing/untrusted certificates fail during TLS negotiation. No plain HTTP
  listener exists.
- T3b does not call `listen()`, read deployment environment variables, or load
  credential files. T3c will inject the real publisher; T6p owns bind address,
  port exposure, credential provisioning, and rotation. This resolves the
  Local Architect's startup-wiring uncertainty without broadening C0 paths.
- Fingerprints are exact normalized SHA-256 pins. Multiple pins permit a later
  explicit overlap window for rotation; CN/SAN fallback is forbidden.

Local Architect evidence: ignored packet/artifact under
`.agent/local-architect/adr037/P2.T3b/`; model
`qwen3.6:27b-q4_K_M`, digest
`a50eda8ed977ab48a12431878896b27ffd5cef552c17af3317d9623b939a7f1e`,
`num_ctx=65536`, `done_reason=stop`, non-empty validated output. Adopted:
fail-closed options, exact allow-list, handshake/application-policy separation.
Rejected/narrowed: public-bind classification is deferred because this leaf
returns an unbound server and T6p owns deployment exposure.

## Executable Low leaves

The initial parent score activates the test-first decomposition trigger. Six
sequential leaves follow three real behavioral seams. Test leaves are eligible
for Qwen Developer; security-sensitive product leaves remain primary-agent
authorship despite their bounded Low score.

| Leaf | Objective and exact writable paths | RRI | Route |
|---|---|---:|---|
| `T3b-i` | Policy contract-first evidence: `apps/availability-node/test/client-fingerprint-policy.test.js` | 25 Low / S | Qwen Developer |
| `T3b-ii` | Exact fingerprint normalization and allow-list decision: `apps/availability-node/src/mtls.ts` | 25 Low / S | primary Codex; security-sensitive |
| `T3b-iii` | Transport contract-first evidence: `apps/availability-node/test/private-mtls-server.test.js` | 25 Low / S | Qwen Developer |
| `T3b-iv` | Fail-closed TLS options + unbound HTTPS server factory: `apps/availability-node/src/mtls.ts` | 25 Low / S | primary Codex; security-sensitive |
| `T3b-v` | Ingress-guard contract-first evidence: `apps/availability-node/test/private-publication-ingress.test.js` | 25 Low / S | Qwen Developer |
| `T3b-vi` | Compose authorized mTLS requests with existing publication handler: `apps/availability-node/src/server.ts` | 25 Low / S | primary Codex; security-sensitive |

### Test-leaf RRI profile (`T3b-i`, `T3b-iii`, `T3b-v`)

Each command substitutes that leaf's one exact audit-test path:

```bash
python3 scripts/rri.py --platform generic --touches <audit-test-path> \
  --cc 6 --D 1 --K 1 --P 1 --T 1 --A 0 --X 2
```

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C | 1 | raw CC 6 | High |
| F | 0 | one file | High |
| D | 1 | one bounded executable contract | High |
| T | 1 | existing T3a harness/toolchain provides the test substrate | High |
| A | 0 | exact cases frozen | High |
| K | 1 | isolated audit harness | High |
| P | 1 | evidence only; no product authority | High |
| X | 2 | target contract plus one source boundary | High |

Technical bottleneck B=1 -> ICI 25; risk input 7; penalties none.
**Final RRI: 25 — Low / Effort S.**

### Product-leaf RRI profile (`T3b-ii`, `T3b-iv`, `T3b-vi`)

Each command substitutes its one exact product path and raw CC 5–6:

```bash
python3 scripts/rri.py --platform generic --touches <product-path> \
  --cc <5-or-6> --D 1 --K 1 --P 5 --T 1 --A 0 --X 2 \
  --penalty auth_security
```

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C | 0–1 | raw CC 5–6 for one localized invariant | High |
| F | 0 | one file | High |
| D | 1 | one localized policy/factory/composition invariant | High |
| T | 1 | immediately preceding contract-first leaf | High |
| A | 0 | exact behavior and stop conditions frozen | High |
| K | 1 | isolated module/file increment; parent retains cross-boundary coupling | High |
| P | 5 | authentication/security impact remains explicit | High |
| X | 2 | target file plus frozen contract/evidence | High |

Technical bottleneck B=1 -> ICI 25; risk input 25 including
`auth_security` (+10).
**Final RRI: 25 — Low / Effort S.**

The product leaves are not delegated merely because their score is Low: mTLS
policy has high editorial/security risk and is outside the eligible simple
local-patch lane. The parent RRI 100 remains the approval, Reflection, and
integrated-closure envelope. Further splitting would fragment one of the three
security invariants and is rejected as score gaming.

## Advisory and review disposition

- Antares refinement: typed skip — the current watchlist has no hypothesis for
  `apps/availability-node`; no generic security sweep is permitted.
- Task-analysis review: n/a — `REVIEW-OVERRIDE: urgency`, explicit
  owner-directed MVP0-P2P exception in
  `docs/audit/mvp0-p2p-review-exception.md`.
