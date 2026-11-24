---
type: Audit
date: 2026-11-24
---

# A1–A4 — bounded consolidation

W2 physical smoke remains historical PASS, documented in
`haa-macos-integration-certification-2026-11-24.md`. The owner waived
independent reviewers for this integration, not security or test gates.

A2: new trusted session ID commits the exact task-card SHA-256 and authoritative
RRI into sessionId, already included in HAA action/scope digests. The supervisor
must supply a trusted pin and independently computed RRI. No HAA wire change.
A3: retain only the detached verified grant digest in the HMAC-sealed lease;
no reusable grant is persisted. Legacy synthetic tests can use unbound task
contexts but the runtime candidate must require binding and grant digest.
A4: tighten directory and remote audience checks. Older persisted records
missing the new lease field fail closed; do not migrate an active lease or
reuse an uncertain request. Before any runner activation, prove isolation
from untrusted code and execute the regression gate at the candidate HEAD.

Status: implementation and regressions authored, not yet executed. No
new CI, branches, merges or production authority. Confidence refers to
bounded implementation, not certification.
