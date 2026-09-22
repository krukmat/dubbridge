---
type: Audit
title: "P4.T1 bounded reconnect RRI"
date: 2026-09-22
task: P4.T1
---

# P4.T1 bounded reconnect — execution RRI

Exact proposed production/test paths:

- `mobile/src/p2p/sync/P2pProductSync.ts`
- `mobile/__tests__/p2p.product-sync.test.ts`

Scoring inputs used for the coherent bounded-reconnect leaf:

| Variable | Score | Rationale |
|---|---:|---|
| C | 2 | bounded retry loop plus transport/error branches; no new recursive/global engine |
| F | 1 | two touched source/test files |
| D | 2 | existing foreground sync state machine; no new distributed state/schema |
| T | 1 | focused product-sync suite already exists and is extended with retry negatives |
| A | 1 | acceptance criterion is explicit; retry count frozen to one automatic reconnect |
| K | 2 | couples existing sync state, cache and existing reconnect-budget utility |
| P | 2 | affects P4 product sync correctness but not auth/crypto/schema |
| X | 1 | localized mobile TypeScript context |

No manual penalties apply: this is not an auth/security contract change, broad
refactor, architecture decision, or unverified high-impact patch.

Derived ADR-045 technical profile:
- L=2, I=2, Q=2, V=1
- bottleneck=2
- ICI=50
- ICI band input=55
- risk/domain input remains below the ICI band

**Final RRI: 55 — Med-high.**

Owner execution approval: explicit instruction in the active session to continue
the plan assuming P5.T3 PASS. This authorizes implementation; independent
band-routed code-solution review remains required before marking P4.T1 Done.

Implementation boundary remains exactly the existing P4 contract:
one automatic reconnect by default, reuse verified ciphertext cache, bounded
exhaustion, cancellation/sign-out domination, no READY before complete
verification, no new schema/protocol/crypto/fallback path.
