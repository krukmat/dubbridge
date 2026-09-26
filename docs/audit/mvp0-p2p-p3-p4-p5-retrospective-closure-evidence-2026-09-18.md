---
type: Audit
title: "MVP0-P2P P3/P4/P5 — retrospective-closure verification (CONS-T6/T7/T8a)"
status: closed
---

# MVP0-P2P P3/P4/P5 — retrospective-closure verification

## Scope

Verification pass for `docs/tasks/mvp0-p2p-s230-consistency-remediation.md`
§ CONS-T6/T7/T8a: check whether P3 (Invitation/Envelope), P4 (Mobile Sync),
and P5.T0-T2 (Local Playback) can be retrospectively closed under the
owner-granted D0-a waiver. D0-a waives phase-1/phase-2 peer review; it does
**not** waive verifying that the pre-existing code/tests actually satisfy
each leaf's real acceptance criteria (HP-#/EC-#) before recording it Done —
same standard already applied in CONS-T4.

## Method

Three independent read-only investigations (2026-09-18, one per phase):
read the phase's task + plan ledger, located existing source implementing
each leaf, ran the real test suites, and assessed HP-#/EC-# coverage per
leaf against actual (not assumed) test evidence. Full per-leaf detail is in
each investigation's transcript; this document records the synthesized
verdict used to disposition CONS-T6/T7/T8a.

## P3 — Invitation/Envelope: no leaf is closeable

| Leaf | Source exists | Test evidence | Verdict |
|---|---|---|---|
| P3.T0 (contract freeze) | `infra/migrations/0038_create_p2p_audience.sql` | No contract/decision artifact exists at all | Not closeable — no work product for the leaf's own deliverable |
| P3.T1 (invitation persistence/claim/inbox) | `crates/db/src/p2p_audience_repo.rs`, `apps/api/src/routes/p2p_audience.rs` | 0 repo-layer tests (`cargo test -p dubbridge-db p2p_audience` matches nothing); only 2 pure-utility API tests (token format, status derivation) | Not closeable — the concurrent-claim row-lock (EC-1's actual race scenario) has zero test evidence |
| P3.T2 (O3 authorization + K1 envelope delivery) | `crates/p2p/src/device_envelope.rs`, `crates/db/src/p2p_envelope_repo.rs`, `apps/api/src/routes/p2p_envelope.rs`, mobile `DeviceIdentity.ts` | Only the pure crypto-sealing step is tested (3 tests); the DB-side fail-closed join (device/viewer/publication/expiry/revocation) has 0 tests; handler fail-closed status codes (503/409) have 0 tests; mobile no-software-fallback throw path has 0 tests | Not closeable — the fail-closed authorization predicate itself, which is this leaf's core acceptance criterion, is unverified |
| P3.T3 (integration certification) | none | none | Not closeable — no work product exists |

**Assessment:** P3's source is real and structurally coherent with
ADR-044/D2, but the specific behaviors the ledger requires as acceptance
criteria — concurrency/race handling, fail-closed authorization/expiry/
revocation, "never logs raw tokens" — are asserted only by code inspection,
not by any executable test. This is a materially different situation from
the P2.T4b-T4f/T5/T6 precedent, where the gaps closed under waiver were
individually-recorded, non-blocking residuals against an otherwise solid
evidence base. Here the untested surface is the fail-closed
security/authorization logic itself.

## P4 — Mobile Sync: partially closeable

| Leaf | Test evidence | Verdict |
|---|---|---|
| P4.T0 (lifecycle/cache/RPC freeze) | 14/14 tests pass; state machine, READY-invariant, per-account cache isolation, and sign-out cancellation all directly tested | **Close-ready** — no material gap found |
| P4.T1 (product replication + bounded resume) | 11/11 tests pass, but the "bounded reconnect against unavailable peers" criterion is not actually implemented — `reconnect-budget.ts` is wired only into the P1/P2 proof-runner topology, never into `P2pProductSync.ts`/`product-package-runtime.ts`; the product path is single-attempt-then-fail | **Not closeable** — this is a criterion the code does not meet, not merely an under-tested one |
| P4.T2 (manifest verification + lifecycle isolation) | 14/14 tests pass; digest/missing-file/identity-mismatch all fail closed with dedicated tests | **Close-ready with named residual** — no test isolates cancellation during VERIFYING specifically (only DOWNLOADING is tested), and "secrets never reach Bare" is a structural/interface guarantee, not an explicit negative-assertion test |
| P4.T3 (P4 certification + P5 handoff) | 6/6 tests pass at the controller/handle layer | **Not closeable as stated** — the leaf's own acceptance text requires a discrete certification pass (P5 has an analogous `P5DeviceCertification.ts`); no equivalent P4 artifact exists, and "account-change" (as distinct from sign-out) is not separately exercised |

Full `__tests__/p2p` regression sanity check: 37 suites / 179 tests, all
passing — the failures above are coverage/criterion gaps in specific
leaves, not workspace regressions.

## P5.T0-T2 — Local Playback: partially closeable

(P5.T3 — physical Android device certification — is out of scope; already
tracked separately as CONS-T8b.)

| Leaf | Test evidence | Verdict |
|---|---|---|
| P5.T0 (contract freeze) | 6/6 tests pass; the tested authorization branch (wrong-viewer) is covered, but other `assertAuthorization` OR-branches (asset/publication/lineage mismatch, expiry) are implemented but never exercised; no standalone contract/decision artifact exists | **Close-ready with named residual** — untested branches are single OR-conditions in an otherwise-implemented check, not missing implementation |
| P5.T1 (loopback decryption gateway) | 3/3 tests pass, but they cover only 3 extracted pure helper functions (decrypt, manifest rewrite, range parsing). The `ProductPlaybackRuntime` class itself — the actual loopback HTTP/TCP server, including traversal rejection, session-token scoping, digest-mismatch rejection, and CK zeroization on stop/error — has **zero test coverage anywhere in the repo** | **Not closeable** — this is a network-facing local server whose fail-closed security behavior (traversal, session scoping, key zeroization) is asserted only by code inspection |
| P5.T2 (VideoPlayer + teardown) | 4/4 tests pass; wiring and the lease's idempotent release/fail-closed-retry are directly tested | **Not closeable independently** — its own boundary is met, but the acceptance claim ("release transient CK") transitively depends on P5.T1's untested zeroization; closing T2 while T1 stays open would misrepresent what has actually been proven |

## Disposition

**Close-ready pending owner sign-off on named residuals** (minor,
non-security gaps, same class as the P2.T4e-cov/P2.T5a precedents already
accepted in this slice): P4.T0, P4.T2, P5.T0.

**Not closeable without either real engineering work (add the missing
tests / implement the missing behavior) or an explicit, owner-acknowledged
risk-acceptance naming the exact gap:**
- P3 (all four leaves) — fail-closed authorization/expiry/revocation and
  claim-race logic untested; T3 has no work product at all.
- P4.T1 — the "bounded reconnect" acceptance criterion is not met by the
  current implementation (single-attempt-then-fail), not just under-tested.
- P4.T3 — no certification artifact exists; account-change path untested.
- P5.T1 — the loopback server's traversal/session-scoping/key-zeroization
  behavior, a security-relevant fail-closed boundary, is untested.
- P5.T2 — its closure claim is transitively gated on P5.T1's open gap.

None of these leaves was closed in this pass. Closing the security/
fail-closed gaps (P3.T1/T2, P5.T1) via authorization alone, without either
tests or an owner risk-acceptance specific to each named gap, would
contradict ADR-008's fail-closed governance principle and this repository's
own "no hallucination / cite your evidence" communication contract — the
same standard already applied when declining to retro-close CONS-T8b/T10a/
T10c.

## Recommendation

Two independent paths forward, owner's choice:

1. **Real engineering work** on the named gaps before any further
   closure — highest-value targets: P3.T1's claim-race test, P3.T2's
   fail-closed-join test, P5.T1's `ProductPlaybackRuntime` integration test,
   and either implementing P4.T1's bounded reconnect or revising its
   acceptance criterion to match what the product path actually needs.
2. **Owner risk-acceptance** on specific named gaps (not a blanket
   waiver) — if the owner judges a given gap acceptable to carry forward
   (e.g., P4.T1's single-attempt behavior is acceptable for the October
   scope), record that decision explicitly per gap in this document or the
   task ledger, and only then close the affected leaf with the residual
   recorded — mirroring how P2.T4e-cov and P2.T5a's residuals were handled.

Either path is compatible with the existing D0-a waiver (it still covers
skipping peer review); what it cannot substitute for is the underlying
evidence or an explicit owner decision on each named gap.

## Related

- `docs/tasks/mvp0-p2p-s230-consistency-remediation.md` § CONS-T6/T7/T8a
- `docs/tasks/mvp0-p2p-p3-invitation-envelope.md`
- `docs/tasks/mvp0-p2p-p4-mobile-sync.md`
- `docs/tasks/mvp0-p2p-p5-local-playback.md`
