# MVP0-P2P P2 RRI and decomposition audit

Date: 2026-09-05  
Branch: `feature/p2p-mvp-core`

## Parent assessment

P2 spans database/migrations, K1 cryptography/key custody, storage reads, async
workers, O4 outbox/queue/reconciliation, Availability Node publication, audit, and
cross-component failure injection. It is not a valid direct implementation unit.

Conservative planning inputs:

- C=4 — expected high aggregate branching across orchestration/recovery paths;
- F=5 — phase is expected to touch more than 20 code/test/config/doc files;
- D=5 — security-sensitive distributed publication/key-custody domain;
- T=4 — new P2 product area has no existing direct implementation coverage;
- A=0 — acceptance behavior is explicit in ADR-044 and the P2 task ledger;
- K=5 — PostgreSQL + worker + queue + external Availability Node distributed side effects;
- P=5 — persisted readiness + cryptographic key/data-security impact;
- X=5 — several crates/apps plus cross-runtime contract.

Penalties retained conservatively:

- security/sensitive-key handling +10;
- >10 files +8;
- missing-new-area tests with high impact +10;
- high complexity + high domain coupling +10;
- architecture decision still required for P2.T0 Availability Node operation/auth +12.

Policy-equivalent result: **RRI 131 — Excessive — Effort XL**.

This parent is planning/re-scope only. No direct source execution is allowed.

> The active session has connector access to the repository but no executable
> checkout, so this artifact does not falsely claim a `scripts/rri.py` command run.
> Every executable child must run the repository script against its frozen exact
> paths immediately before presentation; any materially different result replaces
> the planning score and re-applies the normal gate.

## Child planning bands

| Child | Planning RRI | Band | Why it remains separate |
|---|---:|---|---|
| P2.T0 | 64 | Complex | unresolved Availability Node runtime/auth + state/audit contract; architecture/security choice |
| P2.T1 | 78 | High | migration + transactional publication/outbox state + fail-closed persisted readiness |
| P2.T2 | 82 | High | AES-GCM/KEK custody + storage/package construction + confidentiality |
| P2.T3 | 72 | High | authenticated external service + Hyperdrive seeding + idempotent conflict semantics |
| P2.T4 | 84 | High | distributed at-least-once dispatch, optional queue, reconciliation, crash recovery |
| P2.T5 | 74 | High | existing S-120/ASR sequencing + new durable readiness integration |
| P2.T6 | 76 | High | governance audit + multi-component crash-window certification and closure |

These are parent envelopes, not implementation-authoring units. Before source work,
each is re-scored and decomposed. Parent-level HITL/review/Reflection rules remain in
force even when individual pure/mechanical leaves honestly fall in Low.

## Honest Low-band maximization result

Potential Low seams exist, but only after the containing parent freezes interfaces:

- pure stable-id/value validation;
- deterministic non-cryptographic manifest serialization/hash formatting;
- pure retry/backoff decision helpers;
- redaction/diagnostic formatting;
- deterministic test-fixture/scenario builders.

The following are explicitly **not** Low-band candidates merely for routing benefit:

- migration constraints and durable state transitions;
- key generation, AES-GCM encryption, KEK wrapping, nonce lifecycle;
- outbox claiming/lease/retry DB effects;
- Availability Node authentication;
- external publish/reconcile side effects;
- `P2P_READY` transition;
- required ADR-018 durable audit emission.

## Next gate

`P2.T0` is the next approval checkpoint. It is docs/architecture-only, but RRI 64
retains explicit HITL because it selects the Availability Node trust/operation
boundary that T1-T4 will implement.
