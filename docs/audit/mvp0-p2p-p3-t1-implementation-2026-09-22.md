---
type: Audit
title: "MVP0-P2P P3.T1 implementation and certification evidence"
task: P3.T1
date: 2026-09-22
status: pass
---

# P3.T1 — invitation, claim and inbox implementation evidence

## Disposition

The owner approved the P3.T1 implementation block after P3.T0 PASS. T1a, T1b
and T1c are implemented. Final implementation head
`4dede25dda68190d556958e4cd9c35c2ad326b5b` completed **15/15 CI checks
successfully**. P3.T1 is therefore **PASS 2026-09-22** after owner verification of the implemented block.

No P3.T2 native/envelope-certification work is claimed here.

## T1a — concurrent claim race

Changed:
- `crates/db/tests/p2p_audience_repo.rs`

Added a real concurrent different-viewer claim test over the existing PostgreSQL
`FOR UPDATE` path. The evidence proves:

- two distinct viewers/devices race on the same invitation token hash;
- exactly one claim succeeds;
- exactly one loser receives `DbError::Conflict`;
- exactly one audience authorization row exists for the invitation;
- repeating the winning viewer/device claim remains idempotent and returns the
  same authorization identity.

This closes the previously missing EC-P3.T1-1 race evidence.

## T1b — exact-lineage descriptor handoff

Changed:
- `apps/api/src/routes/p2p_audience.rs`

The post-claim handoff still reads the current ready descriptor through the
authoritative P2 read model, but the response now explicitly fails closed unless
all identities agree:

```text
descriptor.asset_id       == invitation.asset_id
descriptor.publication_id == invitation.publication_id
descriptor.lineage_id     == invitation.lineage_id

authorization.invitation_id == invitation.id
authorization.asset_id       == invitation.asset_id
authorization.publication_id == invitation.publication_id
authorization.lineage_id     == invitation.lineage_id
```

A focused unit test proves both lineage and publication mismatch rejection.
This removes reliance on the incidental one-publication-per-asset schema
constraint from the P3 security contract.

## T1c — durable P3 audience audit

Changed:
- `crates/domain/src/audit/kind.rs`
- `crates/domain/src/audit/event.rs`
- `crates/domain/src/audit/tests.rs`
- `crates/db/src/audit_repo.rs`
- `crates/audit/src/lib.rs`
- `apps/api/src/routes/p2p_audience.rs`

Added P3 governance event kinds:

- `p2p_device_registered`
- `p2p_invitation_created`
- `p2p_invitation_claimed`
- `p2p_audience_authorization_issued`
- `p2p_audience_access_denied`
- `p2p_device_envelope_released` is registered for the already-frozen T2
  contract but is not emitted/claimed by T1.

The governance audit boundary now validates the P3 correlation family and the DB
reader round-trips those event kinds. Invitation/authorization success events
require asset + publication + lineage correlation. Device-registration and
pre-resolution denial events do not fabricate unavailable package identity.

The API emits durable audit for successful device registration, invitation
creation, claim, authorization issuance, and bounded denial paths. Audit
persistence remains fail-closed through `emit_governance_audit`.

## Secret boundary

No T1 audit detail contains or intentionally derives:

- raw invitation token;
- invitation token hash;
- plaintext CK;
- raw KEK;
- wrapped-CK bytes / sealed nonce;
- Android private-key material;
- bearer JWT/session credentials.

Details are limited to bounded identifiers, operation names, expiry and stable
non-secret reason codes. `config-secrets` passed on the exact implementation
head.

## Implementation lineage

```text
cbb00567  test(p3): prove claim race and exact lineage handoff
ec1ce7ea  feat(p3): add durable audience audit contract
e7e851a3  fix(p3): preserve audit error values
ea8b7d8c  style(p3): apply rustfmt output
e8df1ae9  refactor(p3): split claim audit flow
d7eb83fe  style(p3): finish claim rustfmt
94ba73fc  fix(p3): box claim helper responses
4dede25d  style(p3): align descriptor helper rustfmt
```

The intermediate commits intentionally record CI-discovered repairs:
duplicate/test-scope imports, `rustfmt`, Clippy's `too_many_lines` and
`result_large_err`. The final head, not any intermediate red commit, is the
certification artifact.

## Final CI evidence

Exact implementation head: `4dede25dda68190d556958e4cd9c35c2ad326b5b`.

All 15 checks passed:

1. coverage
2. clippy
3. qa-docs
4. deny
5. cargo-check
6. s3-integration
7. test
8. python-complexity
9. release-build
10. peer-workflow-review
11. maintainability
12. fmt
13. mobile
14. roadmap-drift
15. config-secrets

The concurrent DB race test executed as part of the green workspace test and
coverage gates; this is not an inferred PASS from source inspection.

## Acceptance mapping

- **HP-P3.T1-1:** existing owner-only ready invitation creation + raw-token-once
  API, same-viewer/device idempotency, scoped inbox and the newly certified
  exact-lineage post-claim handoff collectively satisfy the happy path.
- **EC-P3.T1-1:** the new concurrent race proves exactly one winner; existing
  expiry/non-owner/non-ready tests remain green; denial audit carries no raw
  token material.

## Handoff

P3.T1 is **PASS / owner-verified 2026-09-22**.

P3.T2 may now activate. P3.T2 remains the separate O3 envelope-release/native K1
certification block and is not implicitly approved by this evidence.
