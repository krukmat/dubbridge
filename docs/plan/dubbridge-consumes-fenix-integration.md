---
type: Plan
title: "DubBridge consumes Fenix — capability-neutral integration foundation"
status: active_design
date: 2026-10-31
scope: INT
---

# Canonical integration plan — DubBridge → Fenix

## CURRENT OWNER DECISION — GO CONDICIONAL, return to B4 wiring (2026-11-13)

**GO_CONDICIONAL is approved for continued technical implementation, not for activation or production certification.** Fenix HTTP admission and DubBridge Rust client have source candidates but are not connected to application routing; Fenix default-deny capability and service tokens are not provisioned; real two-service HTTPS is unproven. No known structural architectural blocker. Historic Q5 QA NO-GO is a certification verdict, **not** a ban on development.

**Execution plan:** [Canonical B4.W task ledger in Fenix](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/tasks/task_dubbridge_fenix_b4_operational_wiring.md). **W1** dedicated Fenix default-OFF S2S route and auth chain; **W2** isolated deterministic nonproduction `integration.diagnostics` capability; **W3** opt-in DubBridge API caller with trusted PostgreSQL scope, persisted work-item identity and request correlation; **W5** explicit indeterminate outcome/no auto-retry; **W4** documented nonproduction activation requires **separate owner approval** (secrets, routes, capabilities). Only then B4.T1–T3 real HTTPS, denied scopes and idempotency/outage evidence; B5 later certifies the foundation.

**Scope:** No CI, new branches, deployments, secret issuance, route enablement, production capability, or main merge. C4 GET/status stays deferred; no arbitrary retry of ambiguous outcome and no real workflow until B6. ChatGPT owns all source integration and plan steps; the local runtime/reviewer is only required at real operational validation. Historical claims below that B2–B5 are NOT IMPLEMENTED or that the next phase is B1 are obsolete snapshots.


## B4.Q5 Cloud remediation checkpoint — 2026-11-13

**ChatGPT owns formatting remediation and review preparation.** The [deterministic Go/Rust format-only runner](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/scripts/int_b4q_format_only.sh) is committed for **future isolated worktree execution**, not executed or certified; it invokes real gofmt/rustfmt, verifies current SHAs, protects original checkouts and checks that resulting diffs are whitespace-only. `cargo fmt --all` still reports `fenix_s2s.rs`, unrelated `apps/api/src/routes/workspace.rs` and `apps/gateway/src/proxy.rs`. No local agents have been used for source implementation.

[Independent reviewer **full-diff** packet](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b4q-q5-independent-review-packet-2026-11-13.md) indexes all 157 Fenix + 18 DubBridge changed files at its GitHub comparison snapshot, plus complete-diff URLs, review boundaries and recorded false-positive controls. **This packet is not an independent review verdict.** Post-formatter SHA pair is deliberately **NOT FROZEN**, because source formatting remains pending. Rust `manual_async_fn` implementation fix has been published as candidate but not retested; overall NO-GO.


> **Cloud development owner decision (2026-10-31):** RRI is **not applicable** to this integration when built with cloud coding agents. It cannot block B2.A or determine model selection. Retain independent peer/security review, Go/Rust QA and owner approval for auth/migrations/deployment. Historical RRI forecasts are informational only.



**Owner decision (2026-10-31):** This program **does not define or validate a real business workflow now**. Workflows, AI agents, prompts and business schemas are introduced incrementally **when usage establishes the need**. The original `review_backlog.analyze` example was wrongly made a prerequisite; it is **deferred and nonbinding**. It is not part of B1/B2/B3/B4 acceptance.

## B4.Q5 — Actual frozen-pair local gate evidence and targeted repair (2026-11-13)

**Owner's local evidence for frozen pair:** Fenix `d07adb66`, DubBridge `bf36a9d3`. Source was verified in clean detached worktrees (original checkout mismatch is a procedural L0 FAIL, not a test-source mismatch). Actual L1 Python SQLite **23 PASS**, Go focused **PASS**, vet **PASS**; L1 prepush **FAIL** at 11-file `fmt-check` with later subgates NOT_RUN. L2 Cargo check **PASS**, tests **321 PASS / 1 ignored**, global fmt **FAIL** (connector + two pre-existing unrelated workspace/gateway files), strict Clippy **FAIL** for 5 `manual_async_fn`. L3 reviewer output unreliable/false positives, independent security approval **absent**; no confirmed H/M vulnerability established.

**ChatGPT Q5 fix:** Rust connector's 5 manual async implementations converted to `async fn` in [DubBridge `2fc5b2b`](https://github.com/krukmat/dubbridge/commit/2fc5b2b7a38222a9c9a65c91c35c45de30207fd5), preserving `impl Future + Send` trait signatures. **New code not cargo-retested**, and Go formatting is **still unresolved**. [Full gate triage](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b4q-q5-gate-triage-2026-11-13.md). Next run only affected format/prepush and Cargo check/test/clippy on updated SHAs, followed by an independent review of the **complete** integration diff. Reuse SQLite and already-green Go focused/vet evidence where unchanged. **NO overall QA PASS** and no route/credential/capability/CI/deploy/merge activation.

## B4.Q C1–C4 Cloud delivery — local certification remains last (2026-11-13)

**Published C1 findings/fixes:** Go API router test helper now receives `testing.T`; Go forged-workspace negative test correctly mutates the B2.E `"input":{"v":1}` fixture, and an explicit no-op mutation guard prevents recurrence. Rust `reqwest::Response::chunk` API checked; no extra stream feature needed, no dependency churn.

**C2:** confirmed **5/5 exact Git blob identities** for Fenix SQLite 040 migration, DubBridge Node B4 checker and all three pinned schemas against prior executed evidence. Earlier 24/24 actual isolated SQLite + 29/29 actual Node contract checks remain applicable to unchanged inputs; **no duplicate execution in this checkpoint** and the new Fenix Python wrapper **NOT_RUN**. **C3:** 13/13 targeted source fixture/mutation checks PASS (not Go/Rust QA). **C4:** [preflight and explicit verification boundary](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b4q-cloud-first-preflight-2026-11-13.md), [single deferred local batch handoff](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/handoffs/dubbridge-fenix-b4q-final-local-batch-2026-11-13.md), and exact final source SHA freeze in Fenix issue #10/DubBridge issue #13 once documentation is committed.

**Status:** C1–C4 Cloud authoring delivered; Q4 formal Go1.25.13/Cargo/independent review and real authenticated E2E remain **NOT_RUN**. No local agents, CI, route, operator grants, service tokens, deployment, status GET or merge. User explicitly requested **all local work LAST**. Q5 remediation remains ChatGPT-owned after consolidated verification.

## B4.Q owner sequence — Cloud-first, local-final (2026-11-13)

**Owner instruction (supersedes earlier Q4/Q5 ordering):** prioritize **all ChatGPT Cloud work**, defer **all local/toolchain/reviewer execution** until the last phase. Authoritative two-level [B4.Q task ledger](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/tasks/task_dubbridge_fenix_b4q_cloud_first_execution.md): **C1** review/fix changed Go/Rust source and compile-risk issues; **C2** run committed Python SQLite 038–040 and Node pinned-contract scripts from Cloud; **C3** reconcile cross-repo negative contracts and test source; **C4** audit scoped diffs, freeze both SHA references and prepare **one** local handoff. Completed B4.Q1–Q3 evidence remains historical and must not be needlessly rerun.

Only after Cloud C1–C4 closure: **L1** Fenix Go1.25.13 build/tests/race/vet/prepush and full migrations, **L2** Rust fmt/check/tests/clippy, **L3** independent focused code/security review; run as **one compact local session/batch**, no parallel code implementation by local agents and no repeated unmodified suites. Then ChatGPT Q5 triages findings. Any source changes invalidate affected evidence only. **No** local-agent requests before C4 freeze; no CI, live route/credential/capability, paid infrastructure, status GET, deployment or merge. Source candidate status is unchanged: Q4/Q5 QA NOT_RUN.

## B4.Q1–Q3 Cloud verification — 2026-11-11

**Approved and executed:** Fenix [B4.Q Cloud audit](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b4q-cloud-validation-2026-11-11.md) covers Q1 source integrity and Go token canonicalization (21/21 static checks + isolated Go stdlib regression), Q2 actual SQLite 040 checks **24/24 PASS**, and Q3 execution of the byte-identical [Node B4 checker](../../scripts/check-fenix-b4-contract.mjs) **29/29 PASS** against exact provider schema Git blobs.

**DubBridge fixes:** `crates/connectors/src/fenix_s2s.rs` now rejects Base64URL token tails that Fenix Go `ParseServiceToken` rejects; `scripts/check-fenix-b4-contract.mjs` explicitly errors on unsupported JSON-Schema keywords/types. No human JWT substitution, S2S route mount or retry is introduced. The exact 29-check Node script was actually executed on Node 22.16.0; Rust cargo tests remain unrun.

**Formal status:** Cloud verification Q1–Q3 delivered; Fenix full Go1.25.13, cargo fmt/check/test/clippy, full migrations/modernc, independent security/code review and real HTTPS E2E **NOT_RUN**. C4 GET/status deferred, no provisioned service capability/credential/route, local agent tokens, CI, deployment or merge.

## B4 offline interop checkpoint — 2026-11-11

**Actual status:** [Cloud B4 audit](../audit/dubbridge-fenix-b4-offline-interop-2026-11-11.md) and [issue #13](https://github.com/krukmat/dubbridge/issues/13). **B4.T1 live authenticated HTTP remains blocked** by unmounted Fenix endpoint, operator capability/token provisioning, security T4 and Go/Rust gates. **B4.T2/T3 have source tests and an executed 28/28 Cloud contract simulation**, not real two-service E2E.

Concrete cross-repo changes: Fenix proposed `admission-ack`/`admission-error` schemas and a Go `S2SAdmissionReceipt` JSON projector; DubBridge pinned those exact schemas with blob SHA, strengthened Rust success/denial parsing, and added an offline dependency-free Node contract checker. 24/24 source assertions also passed, but Node CLI, Go/Rust tests and independent review remain NOT_RUN. `202` is new reservation, `200` is replay; both require the same operation and trace IDs. `503` and unknown delivery states are ambiguous, not automatic retries. No C4 GET/status is implemented or required for this offline preparation.

**No production route, token, capability activation, hosted E2E, database migration, CI, worker, agent or main merge.** B4 is not closed; B5 release readiness cannot be claimed. B6 remains dependent on real future usage.

## Current B3 Cloud implementation checkpoint — 2026-11-11

**Source-only B3.T1–T4 implementation** is now published on `feature/int-v2-dubbridge-consumes-fenix` in the existing connectors and API crates. [Task ledger](../tasks/dubbridge-fenix-service-consumption.md) and [issue #12](https://github.com/krukmat/dubbridge/issues/12) hold paths and verification limits.

- **T1:** opt-in HTTPS client in `crates/connectors/src/fenix_s2s.rs`, with Rustls, redirects disabled, bounded 64-KiB envelopes/8-KiB responses, 30-second timeout ceiling, a short-lived Fenix-issued bearer (never human JWT), and redacted Debug.
- **T2:** async `FenixScopeAuthorizer` invoked *before* constructing the neutral B1 request; dormant `apps/api/src/fenix_scope.rs` queries live PostgreSQL org-membership role and project ownership, rejecting errors or mismatches. Actual caller wiring and capability-specific role decision remain later.
- **T3:** caller must already hold a durable local work-item ID; key/trace UUIDv5 derivation is stable, conflicts don't rewrite the key, and ambiguous sends are never auto-retried. **No polling:** B2.C4 GET/status remains deferred, so there is no fake resolver. Actual durable local linkage/reconciliation and the public Fenix acknowledgement remain later contracts.
- **T4:** static fake-transport/unit tests authored in Rust; [Cloud audit](../audit/dubbridge-fenix-b3-cloud-source-audit-2026-11-11.md) records **33/33 source predicates PASS**, not executed Rust QA. The operation request's fields are private so production callers must use a stable work-item factory.

**Source snapshot:** `72c356801b630a8d5d02619ed904b521bd549c84`. **Gate status:** no `rustc`, `cargo`, local tokens, independently authorized review or actual Fenix HTTP route available in this Cloud session. B3 is therefore **published source candidate / QA and review pending**, not closed or deployable. Current Fenix B2.E admission remains dormant, without production URL or approved input schemas; B2.B4/T4 and owner's activation/security approvals are still distinct gates. No token, paid hosting, production route, CI, migration, real workflow, deployment or main merge was authorized or used here. **RRI N/A for cloud DB-FENIX-INT**.

## Intent and domain boundary

DubBridge is the consuming product: owns authorized users, organizations/projects, multimedia, review/rights, approval, publication and P2P. Fenix provides a platform-side identity boundary, governed capability registry, agent/workflow orchestration engine, Blackboard, ToolRegistry and persisted execution status. Integration does not transfer domain decision-making to Fenix.

```text
DubBridge user/service -> DubBridge API authorization
 -> DubBridge S2S client -> Fenix S2S API
 -> check service principal + mapped tenant/workspace + capability status
 -> [registered and enabled capability?] yes -> future workflow/agent dispatch
                                 no  -> explicit rejection, no run
 -> status/trace/result correlated by operation_id -> DubBridge
```

`capability_key` is an abstract **configured logical capability**, not a user-selected workflow ID. The initial foundation ships with **no domain capability enabled**. The registry is deny-by-default; unknown capability requests do not launch any model or fabricated operation.

## Technical baseline and contract

- Fenix's existing routes use human JWTs; `workflows/{id}/execute` invokes DSL inline and `agents/trigger` by itself creates a run, not actual execution. Design a **separate, restricted S2S boundary**.
- Fenix persists in its own **SQLite**, DubBridge in PostgreSQL. No shared token signing secret, login credential or DB.
- Versioned generic envelope with `contract_version`, `operation_id`, `capability_key`, `input_schema_version`, `trace_id`, authorized context, bounded `input`; no embedded DSL/prompt/arbitrary workflow ID.
- On registered capability, input/output schema is required **per capability**; until then unknown capabilities are rejected before execution.
- Fenix controls operator provisioning, credential revocation/rotation, allowed workspace, permitted capabilities and tenant mapping; DubBridge authorizes local user/org/project before sending a request.
- Durable operation reservation and idempotency are backend capabilities. `operation_id` is the stable GET key; `run_id: null` is expected until a real run is allocated; no duplicate execution retries after uncertain acceptance.
- Responses are sanitized, status-oriented. Generic envelope cannot return unvalidated model internals or allow direct mutation of DubBridge.

Canonical [generic wire contract](dubbridge-fenix-b1-contract-v1-proposal.md), [Fenix source/provider plan](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/plans/dubbridge-consumes-fenix-integration.md).

**Testability without an actual workflow:** B4 may enable a strictly isolated nonproduction `integration.diagnostics` capability backed by a bounded deterministic Fenix provider diagnostic handler. It exercises **real authentication, durable SQLite operations, HTTP status, idempotency and recovery**, without impersonating an agent or providing business/model results. Production registry remains default-deny and has no domain capabilities registered. Label such tests **infrastructure E2E**, not functional agent E2E.

## Blocks, tasks, dependencies (two-level)

| Block | Tasks | Dependency / acceptance |
|---|---|---|
| **B0 — Orientation** | B0.T1 caller/provider direction; B0.T2 archive inverse artifacts; B0.T3 source inventory | DONE |
| **B1 — Foundation contract** | B1.T1 source API inventory only; B1.T2 S2S principal and tenant/workspace policy; B1.T3 generic request/status/idempotency schema; B1.T4 security/RRI review | B0; **no live workflow required** |
| **B2 — Fenix foundation** | B2.A separate S2S auth and revocation; B2.B registry/permission denial; B2.C SQLite operation dedupe; B2.D status/trace and recovery; B2.E real provider-side negative tests without registered domain capability | B1 security and task-level approval |
| **B3 — DubBridge client** | B3.T1 opt-in Rust TLS/secret client; B3.T2 authorized local org/project mapping and neutral request; B3.T3 stable idempotency/timeout/ambiguous-outcome policy (**GET/status deferred**); B3.T4 fake transport/tests and Cloud audit | B1 schema + inert B2 source; actual API activation remains gated |
| **B4 — Infrastructure E2E** | B4.T1 authenticated HTTP between real apps (**blocked, endpoint not mounted**); B4.T2 denied identity/capability/tenant (**offline source + simulation**); B4.T3 idempotency/trace/ambiguous outcomes (**offline source + simulation; GET/status deferred**) | B2+B3; **real E2E pending**, no LLM/workflow needed |
| **B5 — Foundation closure** | B5.T1 local Go/Rust QA, security review; B5.T2 evidence/runbook; B5.T3 GO/NO-GO | B4 |
| **B6 — Capabilities on demand** | B6.T1 choose use case from actual needs; B6.T2 register capability/schema/policy; B6.T3 implement and certify real workflow/agent + business E2E | B5 + separate use-case decision |

## Corrected sequencing and gates

The prior [Fenix B1.C historical NO-GO](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b1c-gate-2026-10-31.md) conflated foundation with a **specific workflow**. It is superseded as a roadmap gate: B1 is **not blocked** by a missing model, active workflow or human-user test JWT. The older RRI 122 is a **hypothetical monolithic forecast** requiring task decomposition, not a measured security approval.

Security is **not** deferred: isolated service identities, tenant binding, capability allowlisting, negative auth tests, durable operation handling and independent review remain mandatory before deployment. No arbitrary capability can be executed before registration.

Historical review-count JSON schemas and the local live-workflow probe are **archived concept artifacts** for possible B6 usage; previous offline schema receipts do not certify this neutral contract.

No paid hosting, main merge, GitHub Actions workflows, S2S tokens or application runtime have been changed by the planning correction.

## Current checkpoint

**B0 DONE. B1 design/source audit PARTIAL (workflows no longer a blocker). B2–B5 NOT IMPLEMENTED. B6 DEFERRED until usage.** Next work: neutral envelope security/schema review → task-scoped B2.A identity layer.


## B1 contract validation checkpoint — 2026-10-31

Provider evidence: [Fenix generic envelope audit](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b1-neutral-checkpoint-2026-10-31.md) reports **17 directed PASS + 120 matrix PASS** against the *current* generic request/response/error schemas using matching Git blob SHA. This verifies **schema shape only**, not S2S authorization or live transport. Fenix security ADR remains proposed; independent security/official RRI gates are still open. [Decomposed B2.A](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/tasks/dubbridge-fenix-b2a-s2s-identity.md) is the next task-readiness scope. B6 alone defines real business workflows when needed; no B1–B5 business-workflow dependency.
