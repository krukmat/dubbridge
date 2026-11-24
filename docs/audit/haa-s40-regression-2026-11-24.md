---
type: Audit
status: final
date: 2026-11-24
---

# S4.0 — Candidate regression PASS

Owner supplied the following exact gate output in the conversation.
This is a local execution report, not an independently rerun test by ChatGPT.

- Candidate HEAD: `1574d75eccdba4a6b42d86fc7f1a9c4d311b3685`
- Python 3.15.2, cryptography 50.0.2
- `PASS_CANDIDATE_TESTS`, exit code 0
- 124 tests in 10.955 seconds, OK (skipped=1), hence 123 pass, one skip, no failures
- host_isolation: NOT_TESTED
- verification_backend: NOT_CERTIFIED
- physical_certification: false
- runtime_enforcement: false
- activation: NO_GO

The exact-checkout regression gate passed, including the signed candidate
integration fixture. HAA transport and the verification backend in that
fixture are simulated. No evidence of actual UID isolation, credential
isolation, trusted verification execution or new physical ceremony is
provided by this output. The skip cause is not independently identified
by this compact output.

S4.0 PASS for `1574d75eccdba4a6b42d86fc7f1a9c4d311b3685` only. Any new code must be rerun.
S3.7b real backend/deployment remains open. S4.1 provisioning, S4.2 isolation,
S4.3 host regression/anti-rollback, S4.4 integrated Touch ID, S4.5 final decision
remain gated. Preserve the user's development RRI suspension and independent
review waiver; no CI, branch, merge or activation.
