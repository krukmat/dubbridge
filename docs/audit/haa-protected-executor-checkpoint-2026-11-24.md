---
type: Audit
status: candidate
date: 2026-11-24
---

# Protected executor S1–S3 checkpoint

ADR-049 ratified. Development RRI calculation suspended by owner, no
independent reviewer. Existing runtime approval policy is unchanged.

Implemented candidate modules:
- protected_host: read-only identity and DAC checks; explicit deployment runbook;
- volatile_authority: memory-only leases, monotonic expiry, unique durable
  admission tombstones, service singleton lock; no disk restore or migration;
- protected_broker: peer UID, bounded framing, exact existing-file replacement
  with preimage, symlink/hardlink rejection; no arbitrary command or authority;
- worker_runner: actual legacy runner on scratch, sends one proposal;
- protected_plan: canonical SQLite snapshot and HAA closure record updated
  atomically on verified grant, no Markdown import;
- protected_session: trusted fresh request/poll/activate and bounded socket loop;
- regression fixtures registered in run-tests.sh, available through
  certify_candidate.py.

Not certified: no execution of new tests, actual service/worker accounts,
root-owned interpreter/dependencies, launchd definition, socket and HAA
credential provisioning, signed app isolation, ACLs or elevation checks.
No protected commit or independent verification-command provenance adapter.
The worker's success output is not trusted PASS evidence. Real workflow
plan ingestion/projection and default runner routing remain unwired.
The legacy CLI still runs normally; the intended isolation is on the
protected target, not a prohibition on running arbitrary scratch code.
No real Touch ID, closure, activation, CI, new branch or merge.

## Local next gate (nonphysical)

Use clean candidate checkout and Python 3.11+ isolated environment with
scripts/haa/requirements.txt. Execute:

    python3 scripts/haa/certify_candidate.py

Capture output and exact HEAD; if any test fails, correct/retest before
provisioning. This gate does not install a daemon, create users, handle real
credentials or perform biometric ceremonies. Do not run privileged changes
until the actual host layout and owner-approved deployment are agreed.

## S3.7 continuation — 2026-11-24

See `docs/audit/haa-s37-integrated-candidate-2026-11-24.md`. Bounded commit, task intent, pinned plan
initialization, independent closure and signed integration fixture prepared.
S3.7b remains partial (actual isolated verification backend unavailable);
S3.7e suite authored, full execution pending. Code is not activated.
Next gate S4.0 is nonprivileged exact-checkout regression; no Touch ID.
