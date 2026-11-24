---
type: Plan
title: HAA security rebaseline S1–S4
status: proposed
date: 2026-11-24
---

# HAA security rebaseline

Owner approved preparation of S1–S4 after E1–E3. The concrete architecture
in ADR-049 and new task cards are proposed; do not infer blanket authorization
for host account changes or RRI-100 implementation.

Sources: `docs/audit/haa-e1-e3-local-validation-2026-11-24.md`;
`docs/adr/ADR-049-haa-protected-executor-and-volatile-authority.md`;
`docs/tasks/haa-security-rebaseline.md`.

## S1 — Identity and protected storage
Threat model and deployment contract first. Keep the existing Mac and
interactive Touch ID approver. Isolate HAA authority service and executor
from agent-controlled code, credentials, worktrees and commands. Use one
protected service identity and a separate worker boundary. Define trusted
installation, IPC, source-of-truth, Git/evidence projection and secret
provisioning. A service UID alone does not certify the whole workflow.

## S2 — Replay and rollback
Replace persisted ACTIVE authority with service-generation-bound volatile
leases. Keep durable terminal journal in protected storage. Explicitly
invalidate all old leases on restart, no auto-retry after uncertain POST
or consume. Do not treat prior HMAC lease files as migration input. Bind
all closure transactions to a fresh grant and protected canonical state.

## S3 — Canonical RRI remediation and implementation decomposition
Freeze the original RRI 100 evidence. Q=min(4,D) makes ICI 100 when D>=4;
splitting a file or lowering C alone cannot fix the domain floor. Do not
change the rubric, assign arbitrary lower values, or claim tests reduce
security-domain risk. Separate analysis/fixtures from authority code;
compute actual canonical leaf scores at the approved source and record
all inputs, floor, confidence, penalties and parent envelope. Under the
current policy a required executable leaf with RRI >=56 remains blocked
until legitimate re-scope or explicit policy-level decision. Record
remaining irreducible security work honestly.

## S4 — Certification
Prepare negative fixtures before host changes. Consolidate local execution
at the end: verify identity and service protection, key isolation, IPC
identity/scope, worker privilege boundaries, rollback, restart, race,
actual runner and canonical closure source, exact checkout and regression.
Do not test with real credentials or modify an existing production plan.
E4 Touch ID only after all these gates pass and owner authorizes activation
testing. E5 reports GO/NO-GO; actual activation/merge is separate.

## Dependencies
S1 design -> S2 design -> S3 scoring/re-approval -> bounded implementation
-> S4 non-physical host tests -> E4 integrated physical test -> E5 decision.
No independent reviewer (existing bounded owner waiver); no new branches
or CI. Maximize ChatGPT; host provisioning/testing is last and requires
explicit owner authorization where privileged changes are necessary.

## Confidence semantics
Confidence >85% applies to completing an explicitly bounded deliverable,
not guaranteeing a positive test or safe activation. The design and test
protocol can be completed in cloud; actual service installation, real
non-bypass enforcement and final host outcomes are unmeasured. They
cannot honestly be assigned >85% execution confidence until the host
preconditions and RRI gates are resolved. Mark those conditional/BLOCKED;
do not inflate confidence to make the plan appear executable.
