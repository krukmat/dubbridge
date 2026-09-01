---
type: Tasks
title: "Local Code Intelligence Boundary"
status: active
plan: docs/plan/local-code-intelligence-boundary.md
behavioral_coverage_contract: unit-v1
---

# Local Code Intelligence Boundary Tasks

## T1 — Backend-neutral graph contract

**Status:** in progress  
**Effort:** M  
**Depends on:** none

Implement a narrow local graph-result contract and a deterministic JSON-backed adapter suitable for fixtures and external backend bridges.

### Acceptance criteria
- Backend output is normalized into anchors, files, symbols, relationships, tests, boundaries, and source fragments.
- Malformed or incomplete backend payloads fail closed with an actionable error.
- No model/vendor name appears in the core contract.

### Behavioral examples
- **HP-1:** valid graph payload for a task returns normalized anchors, related files, tests, and selected source fragments.
- **EC-1:** malformed graph payload or missing required revision metadata is rejected before any cloud capsule can be produced.

**Evidence to emit:** unit-test results for HP-1/EC-1.  
**Status artifacts affected:** this ledger, `docs/plan/local-code-intelligence-boundary.md`.

**Agent handoff:** Implement the backend-neutral graph contract first. Keep it deterministic, local, stdlib-only, and independent of any specific CKG or LLM.

---

## T2 — Context Receipt and bounded Context Capsule

**Status:** pending  
**Effort:** M  
**Depends on:** T1

Implement receipt generation and minimum-disclosure capsule export.

### Acceptance criteria
- Receipt binds task, git revision, graph revision, anchors, impact, boundaries, governance, exported context, exclusions, and expansion history.
- Cloud-safe capsule omits global topology and denied classes by default.
- Local capsule may preserve richer relationships while still excluding secrets/runtime data.
- Payload hash is deterministic for identical normalized inputs.

### Behavioral examples
- **HP-2:** task-local symbols/tests/fragments produce a deterministic receipt plus a cloud-safe capsule containing only approved fields.
- **EC-2:** a graph result classified as secret/runtime data is excluded from both local and cloud capsules.
- **EC-3:** global architecture/cross-boundary data is absent from cloud output unless explicitly selected by a future policy extension.

**Evidence to emit:** unit-test results for HP-2/EC-2/EC-3 and example receipt/capsule fixture.  
**Status artifacts affected:** this ledger, schema documentation.

**Agent handoff:** Build deterministic receipt/capsule serialization on top of T1. Enforce least privilege at the export boundary, not in the CKG backend.

---

## T3 — CLI and workflow integration surface

**Status:** pending  
**Effort:** S  
**Depends on:** T2

Expose the gateway as a small CLI usable from DubBridge Analyze/handoff phases without coupling it to a specific agent runner.

### Acceptance criteria
- CLI accepts task text/id, backend JSON, target (`local`/`cloud`), and output directory.
- CLI writes `context-receipt.json` and `context-capsule.json` atomically enough for normal local tooling use.
- Non-zero exit on invalid input or policy rejection.
- README documents intended Analyze-phase usage and trust boundary.

### Behavioral examples
- **HP-3:** valid local graph fixture produces both output artifacts for a cloud target.
- **EC-4:** invalid target or invalid backend payload exits non-zero and leaves no misleading successful receipt.

**Evidence to emit:** CLI/unit test results.  
**Status artifacts affected:** this ledger, tooling README.

**Agent handoff:** Keep integration thin. Do not edit existing model-routing code in this slice; the gateway must be consumable by current or future runners.

---

## T4 — Verification and closure

**Status:** pending  
**Effort:** S  
**Depends on:** T1, T2, T3

Run targeted Python unit tests and repository documentation checks available in the execution environment. Record any checks that cannot be run by the GitHub-only orchestrator for local follow-up.

### Acceptance criteria
- All new behavioral cases map to concrete tests.
- No production Rust/mobile/runtime paths are changed.
- Branch diff remains scoped to code-intelligence tooling/docs.
- Completion record contains exact verification commands and limitations.

**Evidence to emit:** test/check outputs or explicit environment limitation.  
**Status artifacts affected:** this ledger and plan status.

**Agent handoff:** Verify the slice as tooling infrastructure. Do not broaden scope to install or vendor a CKG backend.
