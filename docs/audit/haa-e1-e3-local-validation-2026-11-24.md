---
type: Audit
title: "HAA candidate E1–E3 local validation"
status: final
date: 2026-11-24
---

# HAA candidate E1–E3 local validation — 2026-11-24

Branch `feature/haa-hitl-w3-request-verify`, reference HEAD `12aa926`.
Executed by Claude Code (Opus 5.5) on the owner's macOS host. No CI, no new
branch, no merge, no independent reviewer (owner waiver), no enforcement
activation, no Touch ID request, no HAA network call.

## E1 — Executor isolation (credentials and storage vs agent processes)

### Demonstrated (empirical, this host)

| Check | Result |
|---|---|
| Local macOS accounts (non-system) | only `matias` (uid 501); no dedicated executor user |
| UID of agent processes | Claude Code, Claude.app, Codex, Ollama all run as uid 501 |
| Lease/checkpoint store permissions | `0700` dir / `0600` file, owner uid 501 — same UID as the agent |
| Same-UID agent read of lease file | succeeds |
| Forge lease without signing key | rejected `LEASE_INTEGRITY_FAILED` |
| **Rollback**: agent restores pre-revoke bytes of a `REVOKED` lease | `load()` returns **`ACTIVE`** — revocation undone, undetected |
| Agent deletes lease file | `FileNotFoundError` (not a typed `LeaseError`) |
| SIP | enabled (does not separate same-UID user processes) |

Probe: `isolation_probe.py` (scratchpad, not committed) reusing
`scripts/haa/authority_lease_test.py::LeaseTest` fixtures with a throwaway
directory and a random 32-byte key.

### Theoretical only (not demonstrated)

- HMAC keys are constructor-injected; **no key provisioning exists**
  (no Keychain, no file, no separate process). Whether the agent can read the
  key depends on a deployment that does not exist yet.
- `0600` permissions, `fcntl` locks and HMAC give integrity against
  *other* UIDs and against key-less forgery only. They do not resist
  rollback, deletion or key/memory access by a same-UID process.
- `IsolationBoundary` (`runner_authority.py`) is a Protocol with no
  implementation; nothing proves command isolation.

### Verdict E1

**Isolation NOT achieved.** The current host provides no boundary between the
executor and agent processes. The previous checkpoint's limitation
("MAC storage does not protect against an attacker with executor OS identity
and rollback access") is now confirmed empirically, and that identity is the
one every agent process holds.
Minimum requirement before activation: a separate executor identity (dedicated
macOS user or a root-owned launchd daemon) holding keys in its own Keychain or
Secure Enclave, plus a monotonic anti-rollback counter (e.g. HAA-side sequence
or a Keychain-stored epoch). That is an architecture change and outside this
execution.

## E2 — Canonical RRI (`scripts/rri.py --auto-cc`, radon)

Inputs common to all leaves: `D=5` (rubric: security/authentication),
`K=3`, `P=3`, `T=1` (reasonable tests), penalty `auth_security`. `X`/`A` per
leaf. JSON outputs reproducible with the same flags.

| Leaf | Touches | C (max CC) | X | A | risk band | ICI band | **Final** |
|---|---|---|---|---|---|---|---|
| W3.T3.2 boundary detector | 2 | 2 (15) | 0 | 2 | 38 | 100 | **100 Very high** |
| W3.T3.3 request checkpoints | 2 | 2 (19) | 1 | 2 | 38 | 100 | **100 Very high** |
| W3.T3.4 live verify | 2 | 2 (17) | 3 | 2 | 38 | 100 | **100 Very high** |
| W3.T3.5 authority lease | 2 | 4 (35) | 1 | 2 | 48 | 100 | **100 Very high** |
| W3.T3.6 lease guard | 2 | 4 (35) | 2 | 2 | 48 | 100 | **100 Very high** |
| W3.T3.7 recovery | 2 | 1 (8) | 2 | 2 | 38 | 100 | **100 Very high** |
| W3.T3.8 runner authority + gate | 4 | 2 (13) | 3 | 3 | 38 | 100 | **100 Very high** |
| W4 candidate closure journal | 2 | 2 (14) | 3 | 3 | 38 | 100 | **100 Very high** |
| E3 test fix (this run) | 1 | — | 0 | 0 | — | — | **25 Low** |

Driver: `D=5 → Q=4 → bottleneck 4 → ICI 100`, which dominates the risk band
(38–48). Even at `D=4` (permissions), Q stays 4, so the result does not
depend on that choice. Policy gate for 86–100: *do not implement directly;
ADR + risk analysis + decomposition*. The A–D candidate code was authored
before this gate; **every W3/W4 leaf must be decomposed and re-approved
before activation**. `W3.T3.5`/`T3.6` also contain functions with CC 35
(rubric "Very High"), which are the first refactor candidates.

## E3 — Candidate regression gate

Environment: Python 3.12.15 venv outside the repository,
`cryptography 45.0.7` (`scripts/haa/requirements.txt`), clean checkout.

| Run | HEAD | Status | Tests |
|---|---|---|---|
| 1 | `12aa926` | `FAIL_CANDIDATE_TESTS` | 82 run, 1 error, 1 skip |
| 2 | `0713e1f` | `PASS_CANDIDATE_TESTS` | 82 run, 0 failures, 1 skip |

Failure 1: `live_verify_test::test_bound_session_pins_exact_card_and_rri`
→ `TASK_CARD_OUTSIDE_REPOSITORY`. Cause: the test passed an unresolved temp
root (`/var/folders/...`) while `_trusted_card` compares against the resolved
card path (`/private/var/...`, a macOS alias). Every production caller already
passes `repo_root.resolve()`. Fix (test-only, `0713e1f`): pass
`self.root.resolve()`. No production code changed.

Skip: `grants_test::DependencyGateTest::test_missing_crypto_dependency_fails_closed`
(skipped by design when `cryptography` is installed).

`PASS_CANDIDATE_TESTS` is a unit/regression gate only: not isolation, not
real-runner enforcement, not plan closure, not activation.

## Findings

1. **E1-F1 (blocking for activation):** same-UID rollback reactivates a
   revoked lease. No anti-rollback mechanism exists.
2. **E1-F2 (blocking):** no executor identity separate from the agent; no key
   provisioning; `IsolationBoundary` unimplemented.
3. **E1-F3 (minor):** a deleted lease raises `FileNotFoundError` instead of a
   typed `LeaseError`. It fails closed but is untyped. Not fixed (production
   code, out of scope for minor test fixes).
4. **E2-F1 (process):** all W3/W4 leaves score RRI 100. The mandatory
   decomposition/ADR gate was not applied before the candidate code was written.
5. **E3-F1 (fixed):** macOS `/var` alias false negative in one test.

## GO/NO-GO for E4

- **E4 as a further local, non-physical validation step: GO.** The regression
  gate passes on `0713e1f`.
- **Any physical Touch ID approval, enforcement or activation: NO-GO.** It
  stays NO-GO until E1-F1/F2 are resolved by an approved isolation
  architecture and the RRI-100 leaves are decomposed and re-approved.
