---
type: Audit
title: "P6.T1 Block 3 — Navigation and aggregate evidence"
status: pass
task: P6.T1
block: T1.F+G
date: 2026-09-22
---

# P6.T1 Block 3 — Navigation + aggregate evidence

## Scope

This evidence closes:

- **T1.F — Home → My Content navigation**
- **T1.G — remaining component/integration tests and evidence**

It does **not** close T1.H. Aggregate T1 certification and owner verification
remain a separate owner checkpoint.

No HPKE emulator path was enabled or changed.

## T1.F delivered behavior

Navigation implementation head:

`32ac26c0849acab24d3d96a92806c5a59ec6e418`

GitHub Actions run:

`35788610849`

Result: **15/15 CI PASS**.

The existing authenticated stack now provides:

```text
Home
  -> My Content
       -> Back to Home
```

Properties verified by component/navigation tests:

- Home exposes the My Content entry point.
- My Content is registered only in the authenticated navigator.
- The existing gateway base URL and auth provider are reused.
- Back uses the existing stack; it does not create a parallel navigator.
- Leaving My Content unmounts the volatile invite-token state.
- Re-entering My Content reloads `/api/p2p/content`.
- A raw one-time token shown before Back is not reconstructed on re-entry.
- Transition to unauthenticated state removes My Content from the rendered tree.

## T1.G gap review

The A–F evidence already covered:

- authoritative Processing / Ready / Failed projection;
- loading, empty, retryable error and session-expiry behavior;
- exact descriptor-bound Invite eligibility;
- successful P3 Create Invite;
- one-time token Copy / Done / remount behavior;
- Home → My Content → Back lifecycle;
- authenticated navigation boundary.

Two remaining contract gaps were found:

1. stale Create rejection was explicitly tested only for HTTP 409 even though the
   delivered contract refreshes after 403/404/409;
2. the single-visible-token rule lacked a direct multi-Ready-item test.

No production change was required.

## T1.G added evidence

Evidence head:

`47bc3e8b3219f68bc0bbfce5f97538e39e8aa0a2`

GitHub Actions run:

`35789298099`

Result:

- **15/15 jobs PASS**
- mobile: **63/63 suites PASS**
- mobile: **461/461 tests PASS**
- `MyContentScreen.test.tsx`: PASS
- `RootNavigator.test.tsx`: PASS
- P3 T2c3 local-certification harness: PASS
- workspace line coverage: **90.43%**
- workspace tests / Redis integration: PASS
- maintainability / fmt / clippy / cargo-check / release-build / deny /
  config-secrets / qa-docs / peer-workflow-review / s3-integration /
  python-complexity: PASS

New explicit component evidence proves:

- forbidden/403 rejection refreshes authoritative owner content and fails closed;
- HTTP 404 rejection refreshes authoritative owner content and fails closed;
- HTTP 409 rejection refreshes authoritative owner content and fails closed;
- while one raw token is visible, another Ready item's Create action is disabled
  and no second POST is sent.

## Owner scoping evidence

P6 intentionally does not add client-side owner filtering.

The backend read model remains authoritative:

`crates/db/tests/p2p_dashboard_repo.rs::owner_content_is_scoped_to_owned_assets_and_preserves_publication_state`

That DB integration test inserts one owned publication and one foreign-owner
publication, then proves the owner query returns exactly the owned asset. The
mobile dashboard contract test additionally proves the product reads
`GET /api/p2p/content` rather than a generic asset list.

This preserves **EC-P6.T1-1** without moving authorization policy into the UI.

## Acceptance mapping

### HP-P6.T1-1

> Owner sees Processing → Ready and can create an invite for their ready package.

Covered by:

- `MyContentScreen.test.tsx` authoritative state projection;
- exact Ready descriptor predicate tests;
- real P3 Create Invite request test;
- Copy / Done / remount lifecycle tests;
- `RootNavigator.test.tsx` Home → My Content flow.

**T1.G evidence: PASS.**

### EC-P6.T1-1

> Other-owner assets or failed/non-ready publications do not expose an invite
> action; server rejection remains enforced.

Covered by:

- DB integration owner-scoping test;
- Processing / Failed / descriptor-mismatch component tests;
- 403 / 404 / 409 stale-authority refresh tests;
- session-expired logout behavior;
- one-visible-token lock preventing accidental replacement of the only raw token.

**T1.G evidence: PASS.**

## Status

- T1.A — PASS
- T1.B — PASS
- T1.C — PASS
- T1.D — PASS
- T1.E — PASS
- **T1.F — PASS**
- **T1.G — PASS**
- T1.H — Pending

**P6.T1 remains IN PROGRESS until T1.H aggregate certification and explicit owner
verification are complete. P6.T2 remains blocked.**
