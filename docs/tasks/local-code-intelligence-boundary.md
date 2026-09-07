---
type: TaskList
title: "Local Code Intelligence Boundary"
status: ready_for_audit_pending_local_verification
plan: docs/plan/local-code-intelligence-boundary.md
behavioral_coverage_contract: unit-v1
---

# Local Code Intelligence Boundary Tasks

## Audit entry point

Start local audit/review at:

`docs/audit/local-code-intelligence-boundary-audit.md`

That document is the single navigation point for the relevant docs, implementation files, mandatory smoke suite (`S0`–`S8`), exploratory probes (`P1`–`P4`), evidence format, and merge interpretation.

The reusable black-box fixture is:

`scripts/code_intelligence/fixtures/audit-smoke-graph.json`

---

## T1 — Backend-neutral graph contract

**Status:** implemented  
**Effort:** M  
**Depends on:** none

Implemented a narrow local graph-result contract and deterministic JSON-backed adapter suitable for fixtures and external backend bridges.

### Acceptance criteria
- Backend output is normalized into anchors, files, symbols, relationships, tests, boundaries, and source fragments.
- Malformed or incomplete backend payloads fail closed with an actionable error.
- No model/vendor name appears in the core contract.

### Behavioral examples
- **HP-1:** valid graph payload for a task returns normalized anchors, related files, tests, and selected source fragments.
- **EC-1:** malformed graph payload or missing required revision metadata is rejected before any cloud capsule can be produced.

**Implementation evidence:** `scripts/code_intelligence/backend.py`, tests in `scripts/code_intelligence/context_gateway_test.py`.  
**Execution evidence:** pending local test execution; this GitHub-only execution environment cannot run the branch checkout.

---

## T2 — Context Receipt and bounded Context Capsule

**Status:** implemented  
**Effort:** M  
**Depends on:** T1

Implemented receipt generation and minimum-disclosure capsule export.

### Acceptance criteria
- Receipt binds task, git revision, graph revision, anchors, impact, boundaries, governance, exported context, exclusions, and expansion history.
- Cloud-safe capsule omits global topology and denied classes by default.
- Local capsule may preserve richer relationships while still excluding secrets/runtime data.
- Payload hash is deterministic for identical normalized inputs.

### Behavioral examples
- **HP-2:** task-local symbols/tests/fragments produce a deterministic receipt plus a cloud-safe capsule containing only approved fields.
- **EC-2:** a graph result classified as secret/runtime data is excluded from both local and cloud capsules.
- **EC-3:** global architecture/cross-boundary data is absent from cloud output unless explicitly selected by a future policy extension.

**Implementation evidence:** `scripts/code_intelligence/context_gateway.py`, `docs/schemas/context-receipt-v1.schema.json`, tests in `scripts/code_intelligence/context_gateway_test.py`.  
**Execution evidence:** pending local test execution.

---

## T3 — CLI and workflow integration surface

**Status:** implemented  
**Effort:** S  
**Depends on:** T2

Exposed the gateway as a small CLI usable from DubBridge Analyze/handoff phases without coupling it to a specific agent runner.

### Acceptance criteria
- CLI accepts task text/id, backend JSON, target (`local`/`cloud`), and output directory.
- CLI writes `context-receipt.json` and `context-capsule.json` using temporary files and atomic replacement per artifact.
- Non-zero exit on invalid backend/policy input.
- README documents intended Analyze-phase usage and trust boundary.

### Behavioral examples
- **HP-3:** valid local graph fixture produces both output artifacts for a cloud target.
- **EC-4:** invalid backend payload exits non-zero and leaves no misleading successful receipt/capsule.

**Implementation evidence:** `scripts/code_intelligence/context_gateway.py`, `scripts/code_intelligence/README.md`, CLI test in `scripts/code_intelligence/context_gateway_test.py`.

---

## T4 — Verification and closure

**Status:** audit ready; pending local execution  
**Effort:** S  
**Depends on:** T1, T2, T3

### Verification completed from the orchestrator
- Branch scope was compared against `main`; implementation remains isolated to code-intelligence tooling/docs.
- No production Rust/mobile/runtime path was intentionally changed.
- GitHub reported no CI statuses/workflow runs for the branch head during implementation.
- A dedicated audit entry point and reusable black-box fixture are committed for local verification.

### Mandatory local audit smoke suite

Run the full `S0`–`S8` sequence documented in:

`docs/audit/local-code-intelligence-boundary-audit.md`

At minimum it covers:

| Smoke | Purpose |
|---|---|
| S0 | branch scope and no product/RRI regression |
| S1 | Python compile + behavioral unit suite |
| S2 | cloud CLI happy path and deny filtering |
| S3 | local richer context while excluding explicit secret/runtime classes |
| S4 | byte-level deterministic artifacts |
| S5 | receipt/capsule SHA-256 integrity |
| S6 | malformed graph payload fails closed with no success artifacts |
| S7 | executable core remains model/vendor agnostic |
| S8 | repository documentation QA |

### Exploratory audit probes

The same entry point documents non-automatic probes:

- **P1:** stale `git_revision` / graph-to-checkout freshness enforcement;
- **P2:** backend classification trust and deliberate misclassification;
- **P3:** cloud metadata disclosure via unclassified `files` / `symbols` / related arrays;
- **P4:** per-artifact versus pair-level atomicity.

These probes are intentionally visible to the auditor. Their current behavior should be assessed against the intended DubBridge threat model rather than silently treated as already-enforced guarantees.

### Minimal verification commands

If the auditor wants the shortest baseline before the full smoke suite:

```bash
python3 scripts/code_intelligence/context_gateway_test.py
python3 -m py_compile \
  scripts/code_intelligence/backend.py \
  scripts/code_intelligence/context_gateway.py \
  scripts/code_intelligence/context_gateway_test.py
make qa-docs
```

### Unit coverage certification — pending execution

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid graph payload normalizes | `scripts/code_intelligence/context_gateway_test.py::BackendContractTests::test_hp1_valid_payload_normalizes_graph_result` | pending |
| EC-1 | Edge case | missing graph revision fails closed | `scripts/code_intelligence/context_gateway_test.py::BackendContractTests::test_ec1_missing_revision_fails_closed` | pending |
| HP-2 | Happy path | deterministic bounded cloud artifacts | `scripts/code_intelligence/context_gateway_test.py::ContextGatewayTests::test_hp2_cloud_artifacts_are_deterministic_and_bounded` | pending |
| EC-2 | Edge case | secrets/runtime data never exported | `scripts/code_intelligence/context_gateway_test.py::ContextGatewayTests::test_ec2_secret_and_runtime_data_are_never_exported` | pending |
| EC-3 | Edge case | cloud omits cross-boundary/global architecture | `scripts/code_intelligence/context_gateway_test.py::ContextGatewayTests::test_ec3_cloud_omits_cross_boundary_and_global_architecture` | pending |
| EC-4 | Edge case | invalid CLI input creates no success artifacts | `scripts/code_intelligence/context_gateway_test.py::ContextGatewayTests::test_ec4_cli_invalid_payload_leaves_no_success_artifacts` | pending |

### Owner final verification

Pending. The orchestrator attempted to clone the branch for execution, but the container could not resolve `github.com`; therefore no test is represented as passed without evidence.

### Audit closure evidence expected

Before merge, record:

- branch/head and merge base reviewed;
- `S0`–`S8` results;
- observations for `P1`–`P4`;
- any required fixes with reproducible evidence;
- ADR recommendation (`required` / `not required yet`);
- final verdict (`PASS`, `PASS WITH CONDITIONS`, or `BLOCK`).
