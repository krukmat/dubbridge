---
type: Audit
title: "HAA W2/W3 owner-waived review and static hardening"
date: 2026-11-24
---

# Scope and disposition

Source: DubBridge c19445849f641d8a8bc20a2a0e96bdd877eb1f2e, HAA physical certification recorded on 2026-11-24.
The owner explicitly waived independent reviewers for this HAA/DubBridge
integration and approved maximizing ChatGPT implementation and self-audit.
This is NOT an independent review PASS or a project-wide policy change.
No test rerun, new biometric ceremony, production activation or merge is
claimed for the changes below.

| Finding | Disposition |
|---|---|
| H1 — LeaseStore accepted new lease IDs with the same request or execution ID | Patched: locked uniqueness check, including revoked leases; negative test added |
| H2 — Persisted record identity was not compared with its filename | Patched for leases and checkpoints; negative tests added |
| H3 — Live task-card symlink check happened only after resolution | Patched: reject symlinked card and in-repo parent symlinks; regression test added |
| H4 — Lease guard/recovery accepted a clock earlier than issuance | Patched: revoke and fail closed; negative tests added |
| M1 — Lease parser accepted TTL below the W2 minimum | Patched: enforce 60–1800 seconds |
| H5 — W3 runtime trust boundary is not implemented | OPEN, T3.8 blocker: trusted launcher must control card/RRI/session, credential/key isolation, actual operation scope and operation-time guard; no caller-supplied ALLOW flags |
| H6 — Action-derived W2 actual-state helpers are synthetic | OPEN for T3.8/W4: never use them to authorize real work or closure |
| H7 — Card contents and RRI are not fully cryptographically committed by v1 scope | OPEN for T3.8: pin full canonical card + RRI in executor-owned trusted context; if scope contract must change, require explicit profile compatibility review and re-certification |
| H8 — Checkpoint permissions/HMAC lease storage alone do not isolate an agent sharing the executor OS identity | OPEN: demonstrate process/filesystem isolation and revocation/recovery semantics before enforcement; no claim of a same-user hostile sandbox |
| M2 — Policy names a grant digest in lease evidence; implementation records action digest | OPEN: reconcile evidence binding and retain verified grant digest without exposing credentials or reusable grants |
| M3 — Existing audit changes included unrelated Contabo ignore/frontmatter edits | Documented; preserve owner changes; do not revert silently |

## Gates

- Physical W2 smoke: recorded PASS for task-start and synthetic plan-close.
- Prior local gate: recorded 59 tests (1 skipped) before these patches.
- New regression tests: authored; full local execution NOT RUN.
- Independent reviews: owner-waived for this integration, not represented as PASS.
- Canonical task-specific RRI execution: pending; do not invent scores.
- W2 branch integration: pending; no implicit merge authorization.
- T3.8 must remain fail-closed and inactive until remaining trust-boundary
  issues and validation gates are resolved.
- No new branches, CI, PR or merges.

Next: design and implement the trusted runner seam and lifecycle/operation
boundary, preserving existing HITL and sandbox constraints. If the isolation
cannot be established, report BLOCKED rather than enabling HAA enforcement.
