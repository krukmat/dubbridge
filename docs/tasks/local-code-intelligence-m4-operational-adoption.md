---
type: TaskList
title: "Local Code Intelligence M4 — Operational Adoption"
status: hardening_implemented_pending_pre_t4_audit
plan: docs/plan/local-code-intelligence-m4-operational-adoption.md
behavioral_coverage_contract: unit-v1
---

# Local Code Intelligence M4 — Operational Adoption Tasks

## Pre-T4 audit entry point

Before starting M4-T4, run and record the dedicated readiness audit:

`docs/audit/local-code-intelligence-m4-pre-t4-audit.md`

That audit owns the mandatory S0-S10 branch-local sequence, readiness gates R0-R12, verdict vocabulary, and the operational handoff into T4. The older `docs/audit/local-code-intelligence-boundary-audit.md` remains M1-M3/background evidence and must not be used as a substitute for the M4 pre-T4 readiness verdict.

## Execution state

```text
M3 CLOSED
   |
   v
M4-T0  DONE
   |
   +--> M4-T1  IMPLEMENTED + unit verified
   |
   +--> M4-T2  IMPLEMENTED + unit verified
               |
               v
            M4-T3  IMPLEMENTED + unit verified
               |
               v
      M4 pre-T4 audit S0-S10  PENDING
               |
        READY_FOR_T4 verdict
               |
               v
            M4-T4  NEXT
               |
        findings only
               v
            M4-T5
               |
               v
            M4-T6
```

T1/T2 were executed sequentially on `feature/local-code-intelligence-boundary`; no separate M4 branch was created because M3 has not yet been merged into `main`.

---

## M4-T0 — Reconcile operational baseline

**Status:** done  
**Effort:** S  
**Type:** docs/status only  
**Depends on:** M3 closed

### Result

- M3 remains closed by project decision.
- The existing operational Analyze/handoff entry point is explicitly documented as `scripts/code_intelligence/context_gateway.py`.
- M3 closure does not imply a model-specific hook in `run_local_task.py`.
- `JsonGraphBackend` remains the stable backend-neutral interchange contract in this branch.

**Evidence:** `scripts/code_intelligence/README.md`, M4 plan.

---

## M4-T1 — Enforce graph freshness

**Status:** implemented; unit verified; pre-T4 audit pending  
**Effort:** S/M  
**Type:** development  
**Depends on:** T0

### Implemented behavior

- `build_artifacts()` requires `expected_git_revision`.
- CLI requires `--expected-git-revision`.
- Graph `git_revision` must equal the expected operational baseline before artifacts are produced.
- Expansion additionally binds to the base receipt Git and graph revisions.

### Behavioral cases

- **HP-41:** expected revision `A` + graph revision bound to Git `A` -> receipt/capsule produced.
- **EC-41:** expected revision `A` + graph built from Git `B` -> fail closed before success artifacts.

### Unit coverage certification

| Case ID | Type | Unit test evidence | Result |
|---|---|---|---|
| HP-41 | Happy path | `scripts/code_intelligence/context_gateway_test.py::ContextGatewayTests::test_hp41_matching_revision_produces_artifacts` | passed in isolated exact-source run |
| EC-41 | Edge case | `scripts/code_intelligence/context_gateway_test.py::ContextGatewayTests::test_ec41_stale_graph_is_rejected` | passed in isolated exact-source run |

---

## M4-T2 — Harden minimum disclosure beyond backend labels

**Status:** implemented; unit verified; pre-T4 audit pending  
**Effort:** M  
**Type:** development  
**Depends on:** T0; executed after T1

### Implemented behavior

- Cloud `files`, `symbols`, `anchors`, `tests`, `boundaries`, and `governance` are minimized from allowed task-local evidence rather than copied wholesale.
- Explicit secret/runtime classifications remain denied.
- Deterministic defense-in-depth blocks clearly unsafe paths/content such as `.env*`, credential/private-key material, temporary/runtime roots, and selected secret markers even when mislabeled `task_local`.
- Local remains richer than cloud while unsafe secret/runtime material stays denied.
- No generic policy DSL was introduced.

### Behavioral cases

- **HP-42:** task-local evidence for `crates/alpha/...` exports only justified alpha metadata to cloud.
- **EC-42:** unrelated auth/topology metadata is omitted when no allowed evidence justifies it.
- **EC-43:** a `.env.local` fragment containing a token marker and mislabeled `task_local` is still blocked.

### Unit coverage certification

| Case ID | Type | Unit test evidence | Result |
|---|---|---|---|
| HP-42 | Happy path | `scripts/code_intelligence/context_gateway_test.py::ContextGatewayTests::test_hp42_cloud_exports_only_justified_metadata` | passed in isolated exact-source run |
| EC-42 | Edge case | `scripts/code_intelligence/context_gateway_test.py::ContextGatewayTests::test_ec42_unrelated_metadata_is_omitted_from_cloud` | passed in isolated exact-source run |
| EC-43 | Edge case | `scripts/code_intelligence/context_gateway_test.py::ContextGatewayTests::test_ec43_mislabeled_unsafe_fragment_is_still_denied` | passed in isolated exact-source run |

Additional regression: `ContextGatewayTests::test_local_target_remains_richer_than_cloud` passed.

---

## M4-T3 — Bounded context expansion

**Status:** implemented; unit verified; pre-T4 audit pending  
**Effort:** M  
**Type:** development  
**Depends on:** T1, T2

### Implemented behavior

Expansion is another local gateway evaluation. CLI supports:

```text
--base-receipt <path>
--expansion-reason <reason>
```

Both must be supplied together.

The base receipt must:

- verify its SHA-256;
- use `context-receipt-v1`;
- match target;
- match expected/current Git revision;
- match graph revision.

The resulting receipt records the base receipt SHA, reason, decision (`allow`, `reduce`, `deny`), and added context delta. The same export policy applies to initial and expanded context.

### Behavioral cases

- **HP-43:** a new adjacent task-local helper under the same Git/graph revisions is added through bounded expansion.
- **EC-44:** global-architecture context requested for convenience does not bypass cloud policy.
- **EC-45:** expansion with a different graph revision fails closed.

### Unit coverage certification

| Case ID | Type | Unit test evidence | Result |
|---|---|---|---|
| HP-43 | Happy path | `scripts/code_intelligence/context_gateway_test.py::ExpansionTests::test_hp43_bounded_expansion_adds_allowed_context` | passed in isolated exact-source run |
| EC-44 | Edge case | `scripts/code_intelligence/context_gateway_test.py::ExpansionTests::test_ec44_forbidden_expansion_cannot_bypass_cloud_policy` | passed in isolated exact-source run |
| EC-45 | Edge case | `scripts/code_intelligence/context_gateway_test.py::ExpansionTests::test_ec45_expansion_with_different_graph_revision_fails_closed` | passed in isolated exact-source run |

Schema evidence: `docs/schemas/context-receipt-v1.schema.json` defines the expansion record shape.

---

## Verification evidence available now

The orchestrator executed the exact new Python source content in an isolated temporary runtime:

```text
python3 context_gateway_test.py
Ran 15 tests
OK

python3 -m py_compile backend.py context_gateway.py context_gateway_test.py
exit 0
```

The Python environment emitted an unrelated artifact-tool startup warning; DubBridge commands still exited successfully.

### Required pre-T4 branch-local audit

Run the complete audit defined in:

`docs/audit/local-code-intelligence-m4-pre-t4-audit.md`

It includes:

- S0-S1 branch identity, merge-base, scope, and no product/RRI drift;
- S2 branch-local compile and behavioral unit suite;
- S3-S4 fresh/stale graph enforcement;
- S5-S6 cloud minimum disclosure, defense-in-depth, and local-richer safety;
- S7 determinism and receipt/capsule integrity;
- S8-S9 bounded expansion allow/deny/stale behavior;
- S10 model/vendor independence plus `make qa-docs`;
- readiness gates R0-R12;
- final verdict and the exact handoff procedure into T4.

### Owner final verification

Pending the dedicated pre-T4 audit. Do not represent branch-local QA, `make qa-docs`, or `READY_FOR_T4` as passed until that audit is executed from this branch checkout and its evidence is recorded.

---

## M4-T4 — Operational use-and-adjust loop

**Status:** next only after `READY_FOR_T4` or acceptable `READY_FOR_T4_WITH_CONDITIONS` verdict  
**Effort:** ongoing/S per adjustment  
**Type:** operational  
**Depends on:** T3 + successful pre-T4 audit

The operational execution contract is Section 9 of:

`docs/audit/local-code-intelligence-m4-pre-t4-audit.md`

Use the M4 path during ordinary DubBridge work. Do not create synthetic benchmark tasks or an A/B program.

### Acceptance criteria

- normal tasks use the existing Analyze/CKG path;
- sufficient initial context causes no unnecessary expansion;
- missing context uses bounded expansion instead of unrestricted repository/graph exploration;
- only actionable friction is recorded: stale graph, missing/irrelevant context, policy over/under-blocking, host pressure, artifact race, or graph/source mismatch;
- recurring/material friction becomes a narrowly scoped T5 task or is explicitly deferred;
- source/tests/ADRs/policies remain authoritative over graph-derived context.

### Behavioral examples

- **HP-44:** ordinary task proceeds with the initial bounded capsule.
- **HP-45:** one adjacent symbol is supplied through bounded expansion.
- **EC-46:** graph/receipt is not treated as authoritative over source/tests.
- **EC-47:** broad cloud architecture traversal remains denied/bounded.

---

## M4-T5 — Evidence-backed hardening

**Status:** conditional  
**Depends on:** a concrete T4 finding

Create one narrowly scoped task per reproducible issue. Valid triggers include unsafe/incorrect export behavior, pair-level artifact race, recurring export misclassification, freshness lifecycle race, repeated unnecessary expansion, or host pressure attributable to the code-intelligence lifecycle.

Dashboards, generic policy DSLs, Neo4j/GraphRAG, theoretical elegance, or automatic RRI changes are not valid triggers by themselves.

Every triggered development T5 receives its own RRI, HP/EC cases, review route, tests, and evidence.

---

## M4-T6 — Milestone closure

**Status:** pending  
**Depends on:** T4 and every blocking T5 task

Close only when:

- at least one ordinary DubBridge task has exercised the audited M4 path;
- plan, ledger, audit, schema, and operator README agree;
- material findings are fixed or deferred with rationale;
- no unresolved blocking T5 remains;
- ADR need is decided;
- RRI/model routing remains unchanged unless separately approved; and
- no product-runtime coupling was introduced.
