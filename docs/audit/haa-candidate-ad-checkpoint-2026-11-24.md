---
type: Audit
date: 2026-11-24
---

# A–D autonomous candidate checkpoint

Owner approved the A–D plan, no independent reviewer, no new branches or CI.
W2 historical physical gate: PASS (59 tests, one skip at the prior tested
worktree, HAA 4/4). No subsequent tests were executed by ChatGPT: the cloud
runtime cannot resolve github.com to clone the checkout. New code/tests are
not certified.

| Task | Deliverable state | Remaining |
|---|---|---|
| A1 | Evidence and waiver consolidated | W2 review-policy/status reconciliation and merge not done |
| A2 | Digest-bound task session; v1 wire unchanged | trusted RRI source; host isolation; local regression |
| A3 | Verified grant digest in sealed lease | local regression |
| A4 | MAC checkpoints, duplicate-session guard and recovery hardening | protected host storage; anti-rollback/cleanup; local regression |
| B1 | Candidate preflight authored | connect trusted deployment to real runner |
| B2 | Candidate model-turn check + request/activation authored | connect actual model entry; direct CLI bypass |
| B3 | Candidate edit/command/review checks authored | connect actual file/format/test operations; commit adapter not implemented |
| B4 | candidate lifecycle/revoke authored | host crash/restart integration |
| B5 | negative/concurrent request tests authored | run real tests and direct runner bypass tests |
| B6 | isolation interface authored | real isolation implementation / proof; no default |
| C1 | candidate manifest from source + actual Git | actual canonical plan/task source adapter |
| C2 | pre/post snapshot re-read authored | verify real ledger/validation/evidence source and lock |
| C3 | pinned detached closure authorization authored | real Touch ID closure validation |
| C4 | atomic private candidate PLAN_DONE journal | real plan transition and projection not wired |
| C5 | stale/replay/pending/timeout/dirty/invalid grant tests | actual bypass/closure tests |
| D1 | static audit and limitations recorded | no independent review; no claim of complete audit |
| D2 | new recovery/negative/concurrency tests authored | execute tests; security gate |
| D3 | candidate exact-checkout test gate authored | run locally; macOS isolation/integrated gate separate |
| D4 | candidate evidence/status synchronized | final evidence/GO-NO-GO pending |

No existing runtime entrypoint was changed. The candidate controller is not
a production security boundary; an ordinary invocation of the old CLI does
not enforce HAA. The plan closure journal is not the real Markdown ledger.
HAA v1 sessionId carries a commitment to the card bytes and authoritative
RRI, not a separate human-readable RRI display. That commitment is only as
trustworthy as the supervisor's source and isolation. Neither MAC storage
nor Python locks protect against an attacker with executor OS identity and
rollback access. No claim of non-bypassability is made.

## Local gate (no CI)

Run from the exact clean DubBridge candidate checkout in a Python 3.11+
virtual environment with scripts/haa/requirements.txt installed:

    python3 scripts/haa/certify_candidate.py

A PASS_CANDIDATE_TESTS result is only a unit/regression gate. It is not
the historical W2 smoke, an isolation test, a real-runner enforcement
test, a plan closure certification, or an activation authorization.

## Disposition

A–D candidate artifacts authored. **Activation NO-GO**. Real runner and
plan state integration are open, not tasks silently delegated to the user.
E1–E5 remain local validation; do not start a physical approval if the
candidate regression gate fails. Canonical RRI scoring, protected deployment,
real runner integration, real closure projection, and branch integration
remain prerequisites. No new GitHub Actions, branch or merge.

## E1–E3 result (2026-11-24)

Candidate gate `PASS_CANDIDATE_TESTS` on `0713e1f` (82 tests, 1 by-design
skip) after a test-only macOS path fix. Isolation is not achieved: a same-UID
rollback is demonstrated. Every W3/W4 leaf scores canonical RRI 100. E4 local
validation is GO; physical/activation is NO-GO. See
`docs/audit/haa-e1-e3-local-validation-2026-11-24.md`.
