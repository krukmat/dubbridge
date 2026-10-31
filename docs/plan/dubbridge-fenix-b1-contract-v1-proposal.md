---
type: ContractProposal
title: "DubBridge → Fenix generic capability-neutral S2S envelope v1"
status: proposed
date: 2026-10-31
---

# Capability-neutral S2S contract v1 (PROPOSED)

**Owner decision:** The actual workflows and agents will be specified by future usage. This integration contract **does not require any business capability, review backlog analysis, model prompt or workflow ID to exist in B1–B5**. The old `review_backlog.analyze` payload and its domain-specific result/test files are archived examples for possible B6 work, **not normative v1**.

## Endpoints to implement (not existing)

```http
POST /integrations/dubbridge/operations
Authorization: Bearer <Fenix-issued restricted S2S bearer>
Idempotency-Key: <stable operation_id>
Content-Type: application/json

GET /integrations/dubbridge/operations/{operation_id}
Authorization: Bearer <same restricted class of bearer>
```

An unconfigured capability is rejected **before accepting a run**, e.g. HTTP 409/422 with `CAPABILITY_NOT_CONFIGURED`. This response must be testable against real Fenix without running a workflow. Successful HTTP 202 may only be returned after a durable operation reservation for a configured, authorized capability.

## Generic request envelope (candidate)

```json
{
  "contract_version": "1",
  "operation_id": "dubbridge-request-0001",
  "capability_key": "capability.example",
  "input_schema_version": "1",
  "trace_id": "dubbridge-trace-0001",
  "source": {
    "product": "dubbridge",
    "org_id": "550e8400-e29b-41d4-a716-446655440000",
    "project_id": "550e8400-e29b-41d4-a716-446655440001"
  },
  "input": {}
}
```

`capability.example` is a **syntactic placeholder only, not a registered capability**. Caller cannot choose workflow IDs, arbitrary agents, tools, DSL, or security scopes. Unknown capability must be rejected. The generic `input` object can be empty; any nonempty payload requires validation against an **operator-registered capability-specific schema** at runtime. No generic arbitrary object may execute.

**Principal scope:** DubBridge validates its user/org/project permissions first. Fenix authenticates a separate **Fenix-owned service principal**, checks the mapped workspace and independently provisioned permitted DubBridge org/project/installation scope, then checks the capability registration and allowlist. Never trust source JSON as authority. One-project-per-principal can be the safest initial provisioning policy but is not hardcoded into a universal envelope.

## Operation response (candidate)

```json
{
  "contract_version": "1",
  "operation_id": "dubbridge-request-0001",
  "capability_key": "capability.example",
  "status": "accepted",
  "run_id": null,
  "trace_id": "fenix-operation-trace-0001",
  "result": null
}
```

Status values reserved for when capabilities are registered: `accepted`, `running`, `awaiting_approval`, `handed_off`, `succeeded`, `completed_with_warnings`, `abstained`, `failed`, `denied`, `indeterminate`. An accepted operation has a stable `operation_id` and `trace_id` but not necessarily a `run_id`. An actual Fenix run ID only appears after a real runner allocates it. No fabricated run IDs or fake success results in the foundation.

A successful capability returns a `result` object with `schema_version` and `data` **validated and filtered according to the installed capability's output schema**. The generic response envelope is not permission to expose raw model output, internal thoughts, inputs, ToolRegistry calls, or agentRun DTOs.

## Error response (candidate)

```json
{
  "contract_version": "1",
  "error_code": "CAPABILITY_NOT_CONFIGURED",
  "message": "Capability unavailable"
}
```

Suggested errors: `INVALID_REQUEST`, `UNAUTHENTICATED`, `FORBIDDEN`, `CAPABILITY_NOT_CONFIGURED`, `CAPABILITY_DISABLED`, `CONFLICT`, `PROVIDER_UNAVAILABLE`. No secret, org existence, workflow inventory or model internals disclosed. HTTP 401 for missing/invalid identity, 403 or non-enumerating 404 for denied scope, 409/422 for unconfigured capability and 409 for conflicting idempotency digest. Rate and payload limits mandatory.

## Lifecycle and idempotency

1. Client passes bounded `operation_id` also in `Idempotency-Key`. Fenix checks equality, shape, source binding and configured capability scope before acceptance.
2. Only a configured capability can reserve `(service_principal_id,operation_id)` in Fenix **SQLite** with an immutable canonical payload digest and capability configuration version.
3. Repeating the same key+digest returns the same persisted logical operation; different payload gives 409.
4. When a real workflow is enabled in B6, runtime launch must be recoverable and linked to the operation. Unknown state is `indeterminate`, never blindly retried.
5. DubBridge polls `GET /integrations/dubbridge/operations/{operation_id}` and handles timeout/revocation, typed error and stale states safely. Correlation IDs are not authorization.
6. No business workflow exists during B1–B5: *absence* is the expected and testable foundation outcome. No data mutations or artificial result generation.

## Infrastructure-only end-to-end verification

B4 can test authenticated acceptance, real Fenix SQLite idempotency/status, crash/outage handling and DubBridge client parsing by temporarily provisioning **`integration.diagnostics`**, a deterministic, side-effect-free Fenix diagnostic capability in an isolated test environment. This is **not** a simulated AI workflow, model result or a production feature. B4 reports `INFRASTRUCTURE_E2E_PASS` only when proven, never `AGENT_E2E_PASS`. Without this test seam, cross-service rejection paths can be tested but accepted-operation deduplication cannot be genuinely certified. Production stays default-deny, with no enabled business capability until B6.

## Machine-readable generic envelope (candidate)

Fenix contains these **new generic** contract definitions:
- [request schema](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-v1-envelope-request.schema.json)
- [response schema](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-v1-envelope-response.schema.json)
- [error schema](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-v1-envelope-error.schema.json)

These schemas validate envelope **shape only**. They do not prove authorization, token revocation, capability registration, payload safety, SQLite durability or end-to-end execution. The previous review-count schemas/tests are retained as **superseded example artifacts**.

## Acceptance and scope gates

- **B1:** security/reviewed protocol and generic envelope. Source-level inventory suffices; no workflow run.
- **B2:** Fenix service identity, registry with no domain capabilities registered, operation/status and negative integration tests.
- **B3:** real DubBridge Rust client with local domain auth and polling.
- **B4:** real two-system HTTP transport, errors, scopes, revocation and unknown-capability denial; **not a simulated agent result**.
- **B5:** reviews, Go/Rust QA, deployment controls, secure foundation GO/NO-GO.
- **B6:** real capability definition, input/output schemas, workflow implementation and functional business E2E only when the product needs it.

**Contract status:** proposed, not implemented, not independently security-approved. No token, deployment, GitHub Actions or main merge.


## B1 executable neutral-envelope verification (2026-10-31)

The exact Fenix committed [request schema](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-v1-envelope-request.schema.json), [response schema](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-v1-envelope-response.schema.json), [error schema](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-v1-envelope-error.schema.json) and two test scripts passed **17 directed + 120 matrix checks**, as recorded in the [reproducible audit](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b1-neutral-checkpoint-2026-10-31.md).

**Protocol security gap still open:** generic `input` JSON Schema is syntactic only. Fenix must enforce bounded raw request bytes (initial candidate 64 KiB), recursion limits, Fenix-issued service identity, independent source/tenant authorization, strict schema of each *registered* capability, deny-by-default unknown capability, equality of `Idempotency-Key` and `operation_id`, and stable non-enumerating errors. No model/workflow is selected during foundation work. Independent security/peer review and actual S2S implementation remain pending.
