---
type: Playbook
title: "HAA W3 runner enforcement readiness and trust boundary"
status: proposed
date: 2026-11-24
---

# W3.T3.8 — implementation contract, enforcement OFF

The 2026-11-24 physical smoke passed on both existing integration branches.
Independent peer review is explicitly waived for this integration only.
The static audit and hardening commit 1d8dc8b2b7d39739274d325cfc2cdba8b961658b still need a full local
regression run. This document is design preparation, not enforcement.

## Actual seams

- `scripts/local-agent/cli.py::main` loads the card, chooses limits/model,
  creates file tools, and enters the session loop. Insert the mandatory
  trusted preflight before model activation and runner-controlled writes.
- `scripts/local-agent/run_local_task.py::main` delegates to that common
  CLI. Direct invocation must not bypass a required gate.
- `scripts/local-agent/run_normalized_task.py::main` invokes the same
  runner but creates a separate telemetry session UUID. A trusted authority
  session must remain stable and must not be replaced by this observational ID.
- `LocalAgentBoundary.check_write/check_command` and RunnerFileTools
  must preserve the existing path and exact-argv checks. A separate HAA
  lease check is necessary but cannot substitute for these checks.
- `build_default_test_runner` checks the argv before spawning. The
  launched process is still an ordinary OS process, not a filesystem sandbox.

## Staged execution

| Leaf | Owner | Acceptance |
|---|---|---|
| T3.8.1 | ChatGPT | Trusted runner preflight pins canonical card digest, authoritative RRI, actual repo/worktree identity, session, requested capabilities and approval-required decision; no untrusted CLI flag can supply permission |
| T3.8.2 | ChatGPT | Operation adapters recheck lease and live scope, actual writes and commands; check-and-effect serialized at trusted boundary; revoke on task end, cancellation, expiry or escalation |
| T3.8.3 | ChatGPT | Validate startup/restart, low-band policy, high-band hold, wrong card/RRI/session, duplicate/revoked/expired lease, direct/normalized CLI and protected operation denial; no production activation |
| T3.8.4 | ChatGPT + owner-host test | Establish and test isolation of executor secrets, key/storage, trusted code and approved task cards from agent-controlled processes; no same-UID security claims without evidence |
| T3.8.5 | Owner-host test + ChatGPT | Full local runner and HAA regression; exact real task authority path, no bypass, record evidence; enable only after explicit activation authorization |

## Trust model and activation blocker

A Python guard in an agent-editable worktree cannot protect itself from
a command executing with the same OS rights. An allow-listed test command
may execute repository-controlled code. Stripping environment variables
protects against accidental inheritance, not arbitrary same-UID access to
credential stores or files.

Separate the trusted executor from the model and untrusted child processes.
Keep HAA credentials, private HMAC key, trusted card digest, state journals,
and operation enforcement outside agent-writable paths. Test the actual
OS isolation mechanism before claiming resistance to malicious same-user
processes. For local execution without that isolation, only explicitly
document a cooperative-agent model; do not claim a cryptographic
non-bypass guarantee. No OS security settings are weakened.

Preflight and per-operation checks must fail closed for invalid, missing,
unavailable, or tampered authority. The existing HITL policy still governs
low-risk work. A high-risk task cannot become low-risk by changing card RRI.
Never derive live preconditions from a requester action. Never accept a
receipt, arbitrary lease ID, environment approval flag or synthetic smoke
state as execution permission.

No HAA task lease can authorize a plan closure, push, merge, deployment
or secrets operation. W4 requires exact plan-closure manifest verification
and a separate Touch ID ceremony. This contract does not authorize those
side effects.

## E1–E3 rebaseline

The local E1 probe confirmed same-UID rollback of sealed ACTIVE bytes;
E3 passed 82 tests (one expected skip). Proposed ADR-049 replaces persisted
active authority with protected volatile leases, separating executor and
worker identities. See `docs/plan/haa-security-rebaseline.md`. RRI 100 requires a prior decomposition
and policy disposition; this earlier seam design is not an activation plan.

## Validation sequencing

Keep both existing branches. Do not merge, introduce a branch, or use CI.
Consolidate owner-host execution as far as dependencies permit. Before
activation: integrate W2, run canonical task-specific RRI calculations
and local regression suite at the candidate HEAD, resolve all open audit
findings relevant to W3, and certify the trusted boundary. The previous
physical smoke is valid historical W2 evidence, not evidence of new
runner enforcement. The review waiver does not waive these gates.
