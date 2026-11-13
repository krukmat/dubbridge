---
type: Handoff
title: "DubBridge → Fenix B4.W5 — no-auto-retry ambiguity handling"
status: source_prepared_not_activated
date: 2026-11-13
---

# W5 — Operation ambiguity and manual reconciliation (without C4 GET/status)

**Applies to:** candidate `fenix_dispatch_intents` ledger and dormant `FenixDispatchAdapter`; this is **not** a live endpoint or authorized operator procedure yet. Any nonproduction activation requires a separate W4 owner decision. Production remains disabled.

## State and ownership

1. A trusted DubBridge actor/org/project check completes **before** the intent is written. One immutable intent UUID is generated and committed before HTTP I/O. The existing shared UUIDv5 helper derives stable `operation_id` and `trace_id` from that UUID and capability key.
2. On a deliberately enabled one-shot send, DubBridge re-checks active scope and exact outbound payload digest, then atomically changes `prepared → sending` with a single `send_token`, before any POST. **Never retry automatically** once this CAS succeeded.
3. A sanitized Fenix admission acknowledgement results in `submitted` (accepted/reserved; **not** workflow completion). A canonical remote rejection is `rejected`. A transport timeout, invalid response, provider uncertainty, or persistence failure results in `indeterminate` or leaves `sending` if the local finalization cannot commit. No implicit attempt 2, queue re-enqueue, reset to `prepared`, hidden poll, or GET/status C4.
4. Source-only Rust DB methods `list_unresolved_fenix_dispatch_intents` (bounded to 100 own items) and `mark_stale_fenix_send_indeterminate` (actor/org-scoped, explicit invocation, at least 5 minutes old) support **manual** classification of orphaned `sending` rows. They do not contact Fenix or resend. No background scheduler or public route invokes them.
5. For a real incident, an explicitly authorized operator may inspect Fenix's **read-only** SQLite `service_operation` record by verified Fenix service principal + `operation_id` and compare stored authenticated scope, capability/version, trace and reservation status with DubBridge's immutable intent. Obtain an auditable, redacted evidence reference outside payload/secret stores; no raw service bearer, JWT, secret, private input, or attacker-supplied URL in logs.
6. **Do not compare hashes without accounting for different canonicalization:** DubBridge's `request_sha256` is SHA-256 of the exact Rust HTTP payload; Fenix's `service_operation.request_sha256` is SHA-256 of **Go-normalized JSON**, which sorts object keys and may normalize escapes. They are not guaranteed equal even for semantically equivalent wire input. For proof use a canonicalized pair under the Fenix spec or independent immutable source records, not naive byte-for-byte digest comparison.
7. An indeterminate intent is terminal in the candidate SQL schema; operator evidence does not automatically change it to success or retry it. A fresh request/intent is a **new owner-authorized operation**, permitted only after disproving prior acceptance/effects or a separately designed duplicate-safe protocol. No automatic new UUID for retry.

## Failure classifications

| Condition | Durable representation | Automatic retry |
|---|---|---|
| Local auth/payload rejected before CAS | `prepared` (or no row for failed prepare) | No |
| CAS succeeds, call returns HTTP 202/200 and ack is persisted | `submitted` | No |
| HTTP canonical 400/403/409 denial persisted | `rejected` | No |
| Timeout, no reply, malformed reply, provider 503 | `indeterminate` | No |
| Process crash/cancel after `sending` but before finalization | `sending` until explicit stale classification | No |
| DB unavailable during finalization | `sending` and error | No |
| Operator cannot prove remote outcome | `indeterminate` remains unresolved | No |

## Kill switch and rollout boundary

- Default-off `FenixDispatchOptIn` requires both `enabled=true` and `environment="integration-test"`. No API handler/worker instantiates this adapter in main runtime.
- No runtime token provisioned by this code, no table applied (candidate in `docs/proposals`), no service endpoint mounted, no diagnostic capability registered/enabled. W4 requires explicit owner approval before any of those steps.
- Roll back operational enablement by disabling any explicitly approved dedicated router and revoking service principals/capability policy. **Do not delete the ledger**, since it is the sole local proof for ambiguous sends.
- C4 GET/status is deferred, and no business workflow or agent execution belongs to B4.

**W5 source status:** ambiguity behavior and explicit stale classification are implemented as inert Rust code plus this handoff. No Cargo/PostgreSQL run or remote E2E executed; independent security/operational review remains pending.
