---
type: Playbook
status: candidate
date: 2026-11-24
---

# Exact-tree verification adapter

S4.0 regression passed at 1574d75. The next adapter exports exact Git
tree blobs (not a mutable worker checkout, git archive attributes, or
agent PASS JSON), preserves safe in-tree symlinks and rejects escaping links, submodules
and filesystem-name collisions,
and binds command exit status to tree/session/argv. Files are read-only
to the dedicated verifier group. The executor never executes candidate code.
Tests use a mocked launcher; no host proof is asserted.

## Necessary deployment boundary

Executor, model/agent worker and verification process must have distinct
non-root identities. A verifier sharing the agent UID cannot substantiate
independent process provenance. Prefer a dedicated non-login verification
identity on the same Mac; no extra hardware or cloud resource. This
refines ADR-049's worker boundary. Alternative: a host-certified equivalent
process isolation mechanism, not a claimed same-UID sandbox.

The IdentityLauncher contract is deliberately not a default subprocess or
generic sudo command. The administrator must approve a fixed, audited
identity launch mechanism that:
- accepts only trusted supervisor requests; no worker IPC access;
- executes the exact approved argv under the dedicated verifier identity,
  in the immutable exported tree, with sanitized environment and dedicated
  writable temporary/build area; no credentials or privileged execution;
- observes the actual process exit from the trusted side, not a JSON verdict
  written by the verifier;
- guarantees termination of all descendants or fails closed and requires
  explicit reconciliation; prevents stale processes from touching later runs;
- verifies group/ACL protection of the export tree and executor credentials;
- does not let untrusted commands execute as root.

A narrowly configured OS identity launcher or an equivalent protected
service are deployment alternatives, not installed or authorized here.
Host inspection determines which is actually viable. Do not configure
broad sudo privileges or run a root-owned service with arbitrary agent code.

Current adapter limitations: regular files and contained relative symlinks,
SHA-1 Git
object format, 8192 files/128 MiB, max 32 commands; commands requiring writes
inside the source tree need a separate build-output strategy. No test
coverage quality or adversarial test-honesty claim from an exit code.
Timeout/ambiguity retains the export and blocks reuse of the adapter; durable
task intent prevents automatic task completion replay. Host process cleanup
and operator reconciliation are not implemented by this library.

S3.7b now has concrete tree export/result adapter; real identity launcher
and local certification remain open. No physical Touch ID or activation.

## Actual repository compatibility inspection

At 1dcb996 the Git tree contained 2169 blobs totaling 25,648,523 bytes,
including one symlink fixture at scripts/antares/testdata/context_closure_dependency_manifest/basic_snapshot/symlink_escape/escape.rs.
Its target is relative, escapes the fixture directory but remains inside the
repository root. Export must preserve that test fixture. Defer symlink creation
until regular files are written and reject lexical/resolved escape and loops.
This is a static Git tree inspection, not an executed full export.

## Next host gate (nonprivileged)

From the already prepared Python environment and a clean checkout of the
existing branch:

    git pull --ff-only
    python3 scripts/haa/certify_candidate.py
    python3 scripts/haa/inspect_verifier_host.py

The second script reports only read-only host prerequisites and does not
certify isolation. No sudo, user creation, credentials or HAA ceremony.
New adapter tests have been authored but not executed in cloud. The last
local PASS was on 1574d75; it does not cover this new implementation.

## S4.0 expanded PASS / S4.1 inventory

Owner-reported 131 tests (130 pass, one skip), 11.607 s, clean HEAD
`3de677fd3f662a2f65343c47c49829a0d2fec08b`. See `docs/audit/haa-s41-host-inventory-2026-11-24.md`. Only UID 501 detected, admin member;
current Python owned by UID 501. Dedicated service/worker/verifier and
protected runtime must be provisioned. No real identity proof or activation.
