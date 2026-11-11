---
type: Audit
title: "DubBridge → Fenix B3 cloud source-only audit"
status: source_candidate_29_static_checks_pass_rust_qa_pending
date: 2026-11-11
slice: INT-B3
---

# B3 — Cloud Rust consumer audit

**Branch:** `feature/int-v2-dubbridge-consumes-fenix`. **Source baseline:** `9bf1d9ebeccd41100f384a67178aae529dc2614b`. **Last source implementation commit:** `db8413e952544d3ef782ad9e7654363367f8aad5` (later document updates only). **Tracker:** [DubBridge #12](https://github.com/krukmat/dubbridge/issues/12). **Author:** ChatGPT Cloud, RRI N/A for DB-FENIX-INT cloud. The approved block spans B3.T1–T4.

## Implemented source only

| Task | Source | Concrete behavior | Not claimed |
|---|---|---|---|
| **B3.T1** | `crates/connectors/src/fenix_s2s.rs`, `crates/connectors/{Cargo.toml,src/lib.rs}` | Explicitly constructed HTTPS-only reqwest client, redirects forbidden, timeout bounded to 30 seconds, Fenix service token format with redacted Debug, 64-KiB request and 8-KiB streaming response cap | Any live request or operator credential provisioning |
| **B3.T2** | Same Rust client; `apps/api/src/fenix_scope.rs`, `apps/api/{Cargo.toml,src/lib.rs}` | B1 generic JSON request builder; async local policy interface; PostgreSQL membership-role + project-parent checks; exact authorized scope; deny on invalid/missing memberships or SQL errors | Active API route, per-capability role activation, end-to-end authorization |
| **B3.T3** | Same Rust client | Deterministic UUIDv5 operation/trace IDs from an **already persisted** local work-item ID and capability, no hidden retry, ambiguous result requires reconciliation, strict proposal-only acknowledgement | Writing a PostgreSQL linkage row, real C4 GET/status polling, live reconciliation |
| **B3.T4** | `fenix_s2s.rs` test module and this audit | Test source for fake transport, idempotency key, wrong scope, invalid bearer, HTTPS URL restrictions, invalid/mismatched acknowledgement and once-only sends | Executed cargo test, local HTTP, coverage or peer review |

Existing connector boundary (ADR-025) is preserved: connectors has no DB dependency; actual authorization queries reside in the API crate. Added only existing workspace dependencies, plus matching package entries in `Cargo.lock`. No new database migration.

## Static Cloud checks — 29/29 PASS

Read-only code/content checks verified: B1 required fields; stable UUIDv5 keys; authorized scope before serialization; actual `org_members`/`projects` queries; role threshold and deny-on-error; 64-KiB JSON, 8-KiB streamed response, 32-level input; HTTPS-only + no redirects + 30-second cap; S2S bearer prefix and redacted Debug; no derived Debug on request/scope; matching Idempotency-Key; POST-only; **no GET/status transport**; strict candidate acknowledgement identity and unknown-outcome handling; no hidden automatic resend; negative-test source presence; consistent workspace Cargo manifests and manually reconciled `Cargo.lock`.

**Scope of this PASS is static source assertions only**. It neither proves Rust compilation nor certifies any test or security property. The earlier first-pass static probe falsely flagged a `serde_json::Value::get` call as HTTP GET; narrowed transport-specific matching confirmed there is **no HTTP GET implementation** and the final source-only matrix is 29/29 PASS.

## Source-backed outstanding gates and limits

| ID | Evidence / issue | Next owner | Disposition |
|---|---|---|---|
| B3-R1 | Cloud environment has no `rustc`, `cargo`, `rustfmt`; network git remote unavailable | Rust QA execution channel (when available), **not currently local agents** | **NOT_RUN:** build, fmt, clippy, tests, coverage and independent code review |
| B3-R2 | Public Fenix B2.E admission implementation is an **in-process source-only candidate**, with no mounted HTTPS endpoint; response-ack JSON proposed by DubBridge has not been implemented/ratified by Fenix | Future cross-repo provider transport contract | **BLOCKED FOR LIVE INTEGRATION**, not a source-code authoring blocker |
| B3-R3 | Local actor scope authorizer checks current live DubBridge `org_members` and `projects` from PostgreSQL, but no application handler selects role minimum or invokes it | Future authorized consumer service composition | **UNWIRED**, default no traffic |
| B3-R4 | Deterministic operation ID reconstructs from a caller-provided persisted work-item UUID; DubBridge does not yet store a new integration mapping or reconciliation checkpoint | Future B3/B4 durable linkage design | **NOT IMPLEMENTED** as a new DB persistence feature |
| B3-R5 | A network error or unknown response may occur after Fenix accepted a request; no automatically repeated submit | B3 safe policy already enforced in source; future C4 status process | **SAFE DEFAULT:** hold for explicit reconciliation |
| B3-R6 | Per-capability permission role and credential mapping must follow operator policy, not caller-controlled JSON | B2.B4/operator configuration; live security approval | **UNCONFIGURED / DENY-BY-NO-WIRING** |
| B3-R7 | Fenix C4 GET/status is deferred by owner | No polling in Rust; no fake status or success | **INTENTIONALLY OUT OF SCOPE** |

## Boundaries preserved

- DubBridge is the consumer; Fenix verifies a **separate S2S identity** and its own tenant/capability boundaries. No human JWT reuse.
- The builder accepts only a local `FenixScopeAuthorizer` result; a forged body does not directly populate the authorized scope. The actual API module query is inert.
- Rust code is opt-in and not mounted in `build_app`, any worker, or gateway. No service-to-service token, model, workflow, deployment, CI, new paid hosting, destructive change or main merge.
- No live B4 E2E or B5 security/go-live claim; independent Fenix T4 security [#8](https://github.com/krukmat/fenix/issues/8) remains separate.

**Result:** B3 Cloud code/test sources and architecture review delivered as a **feature-branch candidate**. **Formal closure remains pending** runtime tests and independent review. No additional local-agent token consumption is requested.
