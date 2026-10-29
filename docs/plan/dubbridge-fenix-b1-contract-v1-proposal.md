---
type: ContractProposal
title: "B1 — DubBridge consumer / Fenix provider v1 wire contract"
status: proposed
date: 2026-10-29
---

# B1.T3 — Proposed DubBridge → Fenix service contract v1

**Status: CANDIDATE, NOT IMPLEMENTED OR APPROVED.** [Fenix provider source review](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/plans/dubbridge-fenix-b1-t1-api-runtime-assessment.md) and [security ADR proposal](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/decisions/ADR-DB-FENIX-S2S-PROPOSED.md).

## Responsibility and initial useful scenario

DubBridge is the caller, owns org/project authorization and queries its existing review repository. Fenix is the agent/workflow execution service. First scenario: *analyze review backlog statistics* and return an advisory action plan. This does **not** approve/reject/publish, read third-party media or touch P2P. A successful API call is not an approval.

## Proposed HTTP surface (new endpoints, NOT existing)

```text
POST /integrations/dubbridge/runs
Idempotency-Key: <opaque stable operation ID>
Authorization: Bearer <restricted Fenix-issued S2S credential>

GET /integrations/dubbridge/runs/{run_id}
Authorization: Bearer <same class of restricted credential>
```

No arbitrary Fenix workspace/agent/workflow selection by the DubBridge user: mapping to a single allowed Fenix workspace/workflow lives server-side at Fenix. Credentials are operator-configured and never derived from the DubBridge human JWT. TLS required except isolated loopback tests.

## Request candidate

```json
{
  "contract_version": "1",
  "operation_id": "db-op-unique",
  "operation": "dubbridge.review_backlog.analyze",
  "trace_id": "bounded-correlation",
  "source": {"product": "dubbridge", "org_id": "<uuid>", "project_id": "<uuid>"},
  "input": {
    "review_counts": {"pending": 4, "approved": 2, "rejected": 1}
  }
}
```

No raw reviews, assets, user identifiers, credentials or model instructions. DubBridge must prove user authorization to the specified project locally. Numbers originate from real database; a test-only real DB seed is allowed, fake API results are not E2E evidence. Enforce byte limits and the numeric bounds in a versioned schema.

## Response candidate (not a claim about current Fenix)

```json
{
  "contract_version": "1",
  "operation_id": "db-op-unique",
  "run_id": "fenix-run-id",
  "status": "accepted",
  "trace_id": "fenix-trace-id",
  "result": null
}
```

Initial response is **202 Accepted only after durable reservation**. `GET` retrieves `accepted | running | awaiting_approval | succeeded | failed | denied | indeterminate` plus a typed minimal `result` or `error_code`. A result is **advisory**; no mutation of DubBridge. Failures are explicit; never invent a recommendation if Fenix/model runtime fails.

The current `POST /api/v1/workflows/{id}/execute` runs the DSL handler **inline** and returns HTTP 200. Therefore these 202/poll endpoints **must be implemented** as B2 work with durable execution/reconciliation, not created by pretending the current endpoint is already async.

## Idempotency and recovery

- DubBridge derives and stores one stable `operation_id` per authorized requested analysis. Reuse on retry, including after TCP timeout.
- Fenix stores `service_principal_id + operation_id` uniquely with request fingerprint and run mapping before executing work. Same ID/payload returns same run; changed payload returns 409.
- If accepted but execution state unknown, DubBridge polls the existing operation rather than creating a second one. Backoff bounded; no unbounded retries or repeated side effects.
- Propagate stable `trace_id` to both operational audits; do not grant authority from a request header.

## Preconditions and acceptance

1. B1.T1 source audit and **live run** proving runner/response semantics.
2. B1.T2 Fenix-owned service identity, limited workspace/allowed workflow, rotation and revocation ADR **accepted after independent review**.
3. B1.T3 frozen v1 schema + errors/limit/risk matrix + real RRI and peer readiness.
4. B2 Fenix S2S API implemented, local QA complete; B3 DubBridge client and local DB/authorization.
5. **Real E2E:** DubBridge API/DB → Fenix agent/workflow run → persisted Fenix result → DubBridge; negative tests for cross-tenant, duplicate keys, revocation and no domain writes.

No new paid cloud, GitHub Actions, deployment or merge permitted by this proposal.

## Machine-readable contract evidence (2026-10-29)

The following **candidate** JSON Schema Draft 2020-12 files are versioned on Fenix's clean branch:

- [Request v1 schema](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-v1-request.schema.json) — strict top-level and nested `additionalProperties:false`, UUID syntax, fixed operation and nonnegative bounded counts.
- [Run/result v1 schema](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-v1-run.schema.json) — restricted status enum, small advisory summary/recommendations only, no reasoning trace or tool calls.
- [Schema selftest](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/scripts/int_b1_contract_selftest.py) — positive and negative contract cases for PII, malformed tenant, counts, workflow selection and response leaks. Tests were written but **have not been executed in the full repo**.

**Wire rules requiring B1 approval:**

- `Idempotency-Key` header must equal JSON `operation_id` exactly. Fenix binds a stable payload hash to `(service principal, operation_id)`, selected immutable workflow version and org/project permission. Identical replay reads the prior execution; mismatch → 409.
- v1 S2S principal is bound at provisioning to **one approved DubBridge organization/project pair** plus one Fenix workspace/workflow version. Values in request `source` are checked against that binding, never accepted as granting authority.
- The response schema's v1 normalized states are `accepted, running, awaiting_approval, handed_off, succeeded, completed_with_warnings, abstained, failed, denied, indeterminate`. Mapping from Fenix internal/public statuses requires separate implementation and tests. `handed_off` and `awaiting_approval` are **not** successful terminal approvals.
- No S2S result may contain private prompts, raw agent tool calls/reasoning, reviewer identities or raw project assets. Restrict model output to the allowlisted schema and validate before returning it.

**Implementation note:** schema `format:uuid` must be enforced with an enabled format checker (or an equivalent explicit validator); merely declaring `format` without enforcement is insufficient.

**Status: schema files and test source COMMITTED; peer/security review, script execution, actual Fenix API behavior and end-to-end evidence are PENDING.**

## Verified offline schema invariants — 2026-10-29

[Local test evidence](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b1-contract-check-2026-10-29.md): **18/18** directed cases and **60/60** status/result/error combinations passed using scripts and JSON schemas whose exact Git blob SHAs matched those committed on Fenix's clean branch.

Response status/result constraints now enforced by `run.schema.json`:
- `succeeded | completed_with_warnings`: typed advisory result required, no `error_code`.
- `accepted | running | awaiting_approval | handed_off | abstained`: no result or error (JSON `result:null`).
- `failed | denied | indeterminate`: `result:null`, `error_code` required.

This does **not** certify that a real Fenix provider implements these states. Server-side auth, scoped principal, HTTP idempotency enforcement, durable scheduler and client parsing still require actual runtime work and evidence; B1 security/peer gates remain open.
