---
doc_type: audit
id: DB-FENIX-B4.W3.1
date: 2026-11-13
status: inventoried_no_capability_neutral_durable_work_item
owner: ChatGPT Cloud
---

# B4.W3.1 — Identify a durable DubBridge work-item for Fenix operation IDs

## Source inspection (current integration branch)

- `crates/connectors/src/fenix_s2s.rs`: `OperationParts::from_persisted_work_item(id, capability_key, ...)` derives stable UUIDv5 `operation_id` and `trace_id` from the caller's **already persisted** UUID. It does **not** verify that a row exists or that the ID belongs to the authorized actor, organization or project. Reconstructing with a fresh random ID per attempt destroys idempotency; using a different payload with the same ID/key triggers a conflict.
- `infra/migrations/0014_create_review_tasks.sql`: `review_tasks.id` is a persistent UUID bound to a specific organization/project/asset/target language. **Not a generic Fenix work item** and inappropriate as a free-floating diagnostics ID.
- `infra/migrations/0028_add_localization_generation_claims_and_exact_pointers.sql` and `0029_create_translation_dispatch_outbox.sql`: `generation_request_id` is durable **only** within a composite, translation/dubbing-specific generation identity. Not usable for an arbitrary neutral capability.
- `infra/migrations/0032_create_p2p_publications_and_outbox.sql`: `p2p_publications.id` and `p2p_publication_outbox.id` are durable but belong to a publication/lineage and to its own state machine. Do not attach arbitrary Fenix requests to them.
- `infra/migrations/0005_create_pending_ingestions.sql` and `0008_create_recording_sessions.sql`: ingestion/recording UUIDs have different ownership and lifecycle; no generic external-operation dispatch contract.
- `crates/jobs/src/lib.rs`: `JobEnvelope<T>` is a typed in-memory/queue message abstraction, not a durable generic Fenix operation-intent table.
- Migrations 0001–0039 contain **no capability-neutral Fenix dispatch intent ledger**. The existing Fenix client and PostgreSQL scope authorizer are not wired into an API caller.

## Conclusion

**W3.1 inventory COMPLETE; no safe existing generic persistent work-item source identified.** This is a scoped implementation dependency, **not an architectural blocker** and not permission to improvise a UUID.

**W3.2 prerequisite / ChatGPT task:** add a minimal, opt-in, capability-neutral PostgreSQL `fenix_dispatch_intents` ledger as a proposed migration + typed repository, rather than reusing review/publication/generation tables. A new intent is created and committed **after trusted DubBridge actor/org/project authorization and before any network send**. Required invariants:

1. Durable `id UUID` (immutable), creator, trusted `org_id` and optional scoped `project_id`, `capability_key`, schema version, immutable canonical request digest, stored `operation_id` and `trace_id`, explicit state, timestamps and safe/opaque error code; no bearer token or freeform secret payload.
2. `operation_id` and `trace_id` derived once using `OperationParts::from_persisted_work_item(id, capability_key,...)`, stored or reproducibly checked and never regenerated with an unrelated ID; authorization must revalidate current local membership and project-to-org relation immediately before dispatch.
3. Record intent before POST, transition to `submitted`/`indeterminate` durably on uncertainty, and do not automatically retry. A transport acknowledgement is not a business or agent success. Reconciliation requires Fenix evidence; C4 GET/status remains deferred.
4. PostgreSQL uniqueness, transaction/CAS controls and immutable payload digest prevent concurrent dispatch with different inputs for the same intent. A changed semantic request gets a **new** intent ID.
5. The migration is source-only until independently authorized for a nonproduction environment. No services, queues, operators, credentials, capabilities, deployments or production DB are changed in W3.1.

**Follow-on order:** W3.2.a design immutable ledger/migration source → W3.2.b repository and transactional create/read/claim semantics → W3.3 opt-in adapter to the existing `FenixOperationRequest::build`, `PgFenixScopeAuthorizer` and `FenixClient::submit_once`. W5 adds indeterminate and operator-led reconciliation. Review candidate schema and no-background-retry policy before any live nonproduction approval.

**W3.1 outcome:** `INVENTORY_COMPLETE_NEW_NEUTRAL_INTENT_REQUIRED`; W3.2/W3.3 not yet implemented.
