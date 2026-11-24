---
type: Audit
title: "HAA ↔ DubBridge local macOS integration certification"
date: 2026-11-24
---

# HAA ↔ DubBridge local macOS certification — 2026-11-24

## Scope and source state

- DubBridge: `feature/haa-hitl-w3-request-verify`, HEAD `630ce8335e49e28725fae46f48a70686e00f5937`.
- HAA: `feature/dubbridge-action-profiles`, HEAD `c97020069179266d63a81ace25adcd93a0634446`.
- Both branches fetched and confirmed up to date with origin. During certification, no branch, commit, push, merge, PR, deployment, or GitHub Actions run was made.
- Existing DubBridge worktree changes in `.gitignore` and `scripts/haa/authority_lease_test.py` were preserved and included in the local unit test run. HAA was clean before validation.

## Environment and commands

- Node.js `v26.13.0`; Python `/opt/homebrew/bin/python3` `3.15.2`; Xcode `27.2`; one valid Apple Development signing identity.
- `python3 -m venv /tmp/dubbridge-haa-cert-venv`, then installed `scripts/haa/requirements.txt` (`cryptography 45.0.7`) and ran `PATH=/tmp/dubbridge-haa-cert-venv/bin:$PATH sh scripts/haa/run-tests.sh`: **PASS**, 59 tests, 1 skipped. The skipped case exercises the missing-dependency path and skips when `cryptography` is installed.
- HAA dependencies were already installed (`npm ls --depth=0`: PASS). `node --experimental-strip-types --test tests/dubbridge-profiles.test.ts`: **PASS**, 4/4.
- From HAA, `HAA_PYTHON=/opt/homebrew/bin/python3 HAA_DUBBRIDGE_REPO_PATH=/Users/matias/dubbridge npm run validate:dubbridge-macos`: **PASS**. Its isolated Python gate again passed 59 tests, 1 skipped. The signed, provisioned macOS approver built and passed signature verification; the temporary Secure Enclave authenticator registered.

The first validator attempt selected system Python 3.9 and failed importing `datetime.UTC`; setting `HAA_PYTHON` to Python 3.15 resolved this. The next attempt stopped at an unaccepted Xcode license. The owner accepted the license; the final run succeeded. Xcode emitted a `DVTCoreDeviceCore` plug-in warning during the successful build, with no observed effect on signing or the ceremony.

## Physical ceremonies

| Action | Before Touch ID | Physical approval | Detached result | Final HAA state |
|---|---|---|---|---|
| `dubbridge.task.start.v1` | Executor denied while `PENDING` | PASS | `authorizeAndConsume` and detached grant verification PASS | `CONSUMED` |
| `dubbridge.plan.close.v1` | Executor denied while `PENDING` | PASS | `authorizeAndConsume` and detached grant verification PASS | `CONSUMED` |

The final terminal line was `PASS DubBridge W2: task-start + plan-close Touch ID authority smoke`. The owner separately confirmed in conversation that they physically authorized both Touch ID prompts and reviewed the displayed actions. The validator used a synthetic W2 smoke task and closure manifest; it did not activate a DubBridge lease, run a production task, or close a real plan. Its cleanup path attempted to remove the temporary authenticator and state; the terminal result did not separately attest deletion.

## Disposition

**W2.T2g physical MacBook gate: PASS.** The local DubBridge W2/W3 unit gate passes, but this does not certify W3 runtime enforcement. W3.T3.2–T3.7 still need band-routed independent code-solution reviews and status closure. Both W2 integration branches remain unmerged. W3.T3.8 must implement runner guards and negative-path certification after those prerequisites; no runner enforcement was tested here.
