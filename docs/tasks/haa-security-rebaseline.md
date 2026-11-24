---
type: TaskList
title: HAA security rebaseline tasks
status: proposed
plan: docs/plan/haa-security-rebaseline.md
---

# S1–S4 task ledger

**Follow-up owner decision (2026-11-24):** the RRI calculation requires reevaluation and is suspended until further notice for development of this HAA ↔ DubBridge feature. Do not recalculate or automatically reinstate it; preserve historical scores. Resumption requires an explicit owner decision after reevaluation. This does not suspend runtime HITL/authorization rules or technical security gates.


Historical parent RRI 100 Very high (E2 evidence). Owner explicitly waived
RRI as a development gate for this feature on 2026-11-24; no rescoring required.
Confidence is a planning estimate for artifact completion only.
DONE below means documented design artifact, not certified code.

| Leaf | Deliverable / acceptance | Owner | Depends | Confidence | State |
|---|---|---|---|---|---|
| S1.1 | Threat model, privilege boundary, exclusions | ChatGPT | E1 | 96% | DONE design |
| S1.2 | ADR alternatives and selected protected service topology | ChatGPT | S1.1 | 93% | PROPOSED |
| S1.3 | Define protected code, storage, credentials, owner and worker identities | ChatGPT | S1.2 | 92% | DONE design |
| S1.4 | Define bounded IPC and protected operation/target contract | ChatGPT | S1.3 | 90% | DONE design |
| S2.1 | Volatile lease + restart-generation invariant | ChatGPT | S1.2 | 95% | DONE design |
| S2.2 | Durable request/consume/closure state and no replay | ChatGPT | S2.1 | 93% | DONE design |
| S2.3 | Legacy ACTIVE records non-authorizing; no automatic migration | ChatGPT | S2.1 | 96% | DONE design |
| S3.1 | Preserve RRI 100 and demonstrate domain-floor issue | ChatGPT | E2 | 98% | DONE analysis |
| S3.2 | Split old W3/W4 code into reviewable leaves and map gates | ChatGPT | S3.1 | 92% | DONE decomposition |
| S3.3 | Record bounded feature RRI exception and preserve historical evidence | ChatGPT | Owner directive | 99% | DONE (owner waiver) |
| S3.4 | Implement bounded security leaves with acceptance criteria and regression tests | ChatGPT | S1.2 ratified; S3.3 | Unmeasured | PENDING architecture ratification |
| S4.1 | Negative-path matrix and exact-head evidence protocol | ChatGPT | S1–S3 design | 95% | DONE design |
| S4.2 | Privileged provisioning of protected service/worker on Mac | Owner + local | S1.2 ratified; S3.4 | Unmeasured | BLOCKED |
| S4.3 | Consolidated host isolation/rollback/runner/regression gate | Local + ChatGPT | S4.2 | Unmeasured | BLOCKED |
| S4.4 | E4 integrated ceremony and E5 evidence decision | Owner + local + ChatGPT | S4.3 PASS | Unmeasured | BLOCKED |

## Decomposition of existing W3/W4

Each line is a separate candidate leaf with bounded behavior and acceptance
criteria, not automatic execution approval. No feature-development RRI scoring
required. Characterize behavior before changing it; obtain architecture
ratification before authority-changing code. Earlier candidate code remains inactive.

| Original parent | Bounded analysis / verification leaf | Separate authority-changing leaves |
|---|---|---|
| T3.2 | Decision-table fixtures, unknown/negative input cases | Validate scope/risk; classify routine vs protected action; reject invalid contexts |
| T3.3 | State-transition and duplicate-session fixtures | Protected state-store API; atomic uniqueness; request-intent journal; binding validation |
| T3.4 | Exact card/session/Git test vectors | Protected source lookup; live snapshot; trusted pin validation; consume-intent; detached signature/binding; post-check |
| T3.5 | Lease schema/TTL/legacy fixtures | Volatile registry; unique grant admission; service generation; expiry/revocation; audit projection |
| T3.6 | Path/capability test matrix; characterize CC 35 function | Git identity/ancestry checks; card binding; actual operation scope; guard-and-effect serialization |
| T3.7 | Crash/rollback/uncertain-state fixtures | Startup invalidation; terminal journal recovery; no-retry reconciliation; report projection |
| T3.8 | Direct/normalized CLI and worker bypass fixtures | Protected install; typed IPC; preflight; brokered model/edit/test/commit; worker lifecycle; protected final target |
| W4 | Manifest and drift/closure fixtures | Trusted canonical plan reader; consume-intent; detached grant verification; atomic canonical close; projection/recovery |

## Acceptance for S3.3

DONE by explicit owner direction. The feature-scoped exception is recorded
in the workflow guide, RRI policy and this ledger. Prior RRI evidence is
not altered. Runtime authorization, actual isolation and regression gates
remain intact.

No owner-host task is delegated now. No production code, credential, role,
branch, merge or activation changes under this design checkpoint.

## Revised S1–S4 execution order — owner approved 2026-11-24

The earlier table remains historical. ADR-049 accepted; development RRI
suspended, independent review waived, no CI. The revised plan has 15 remote
implementation tasks and 5 local/final certification tasks. No host changes
or enforcement activation are authorized by this checkpoint.
