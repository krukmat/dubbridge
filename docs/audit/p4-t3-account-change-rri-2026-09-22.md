---
type: Audit
title: "P4.T3 account-change lifecycle test RRI"
date: 2026-09-22
task: P4.T3-account-change
---

# P4.T3 account-change lifecycle test — RRI

Touched path:
- `mobile/__tests__/p2p.provider-account-change.test.tsx`

This is a test-only closure leaf exercising existing production behavior in
`P2PProvider.tsx`; no production contract, auth implementation, crypto,
schema or runtime protocol changes.

Scores: C=1, F=0, D=1, T=0, A=0, K=1, P=1, X=1.
Derived technical bottleneck=1, ICI=25, final RRI=25 (Low).

Acceptance evidence:
- Account A -> Account B invokes `P2PSyncController.clearAccount(A)` exactly
  once and does not clear again for a stable B identity.
- Account A -> unauthenticated also invokes `clearAccount(A)`.
- Existing `P2pProductSync.clearAccount` tests independently prove that
  `clearAccount` cancels active replication and removes only that account's
  cache, so the provider test closes the previously missing account-change
  trigger rather than duplicating lower-level cache tests.
