---
type: Plan
title: "Local Code Intelligence M4 — Operational Adoption"
status: hardening_implemented_pending_branch_local_qa
branch: feature/local-code-intelligence-boundary
predecessor: M3 closed
---

# Local Code Intelligence M4 — Operational Adoption

## Objective

Move the Local Code Intelligence Boundary from an implemented/integrated capability into routine DubBridge agent use without reopening M1–M3 or expanding it into a separate platform.

M4 is a **use-and-adjust** milestone. It hardens the trust-boundary gaps that matter for real operation, adds bounded context expansion, and then lets normal DubBridge tasks drive further refinement.

## Starting assumptions

- M1–M3 are closed by project decision.
- The M3 operational entry point is `scripts/code_intelligence/context_gateway.py`, invoked during Analyze/handoff.
- M3 intentionally does not require model-specific hooks in `run_local_task.py` or another runner.
- `JsonGraphBackend` is the stable backend-neutral interchange boundary in this branch; concrete local CKG production may remain external/replaceable.
- Existing agent workflow integration is authoritative; M4 does not redesign model routing, RRI, or Analyze.
- Source, tests, ADRs, and repository policy remain authoritative over graph-derived context.
- Cloud agents receive bounded artifacts, never unrestricted graph traversal.
- No formal benchmark/POC program, metrics dashboard, Neo4j migration, or GraphRAG platform is introduced.

## M4 architecture

```text
M3 CLOSED Analyze/handoff path
        |
        v
Local graph result
        |
        v
Freshness gate ---------------------> reject stale graph
        |
        v
Defense-in-depth export policy ----> minimize cloud metadata/content
        |
        v
Context Receipt + Capsule
        |
        +-----------------------------+
        |                             |
        v                             v
   normal agent use            bounded expansion request
                                      |
                                      v
                              local policy evaluation
                                      |
                                      v
                               revised receipt/capsule
```

## Implemented M4 hardening

### M4-T0 — baseline reconciliation

Closed. Branch docs now name the existing M3 entry point explicitly and no longer imply that M3 must be reimplemented.

### M4-T1 — freshness invariant

Implemented. The gateway now requires an explicit expected Git revision and rejects graph results whose `git_revision` does not match before artifacts are published.

CLI contract:

```text
--expected-git-revision <sha>
```

For real Analyze usage the orchestrator should supply the active checkout revision (`git rev-parse HEAD`). Synthetic fixtures may use their declared synthetic revision for deterministic audit smokes.

### M4-T2 — minimum-disclosure hardening

Implemented.

- Cloud metadata is derived from allowed task-local fragments/relationships rather than copied wholesale from graph arrays.
- Clearly unsafe paths/content receive deterministic gateway-side deny handling even if a backend mislabels them `task_local`.
- Local target remains richer but explicit secret/runtime records and unsafe runtime/credential material remain denied.
- The policy remains deliberately small; no generic classification DSL was introduced.

### M4-T3 — bounded expansion

Implemented.

Expansion is another local gateway evaluation, not cloud graph traversal. It requires:

- a hash-valid base receipt;
- same target;
- same expected/current Git revision;
- same graph revision;
- a non-empty reason;
- the same export policy as initial context.

The new receipt records the base receipt SHA-256, reason, decision (`allow`, `reduce`, `deny`), and the exported delta.

## Verification state

The orchestrator executed the exact new Python sources in an isolated temporary runtime:

```text
python3 scripts/code_intelligence/context_gateway_test.py -> 15 tests OK
python3 -m py_compile backend.py context_gateway.py context_gateway_test.py -> exit 0
```

The environment emitted an unrelated internal Python/artifact-tool startup warning; process exit codes and DubBridge test results were successful.

Repository-checkout verification still remains before T4 operational use:

```bash
python3 scripts/code_intelligence/context_gateway_test.py
python3 -m py_compile \
  scripts/code_intelligence/backend.py \
  scripts/code_intelligence/context_gateway.py \
  scripts/code_intelligence/context_gateway_test.py
make qa-docs
```

Full local audit guidance is in `docs/audit/local-code-intelligence-boundary-audit.md` (S0–S10).

## Task dependency state

```text
M3 CLOSED
   |
   v
M4-T0 Baseline reconciliation          DONE
   |
   +-------------------+
   |                   |
   v                   v
M4-T1 Freshness     M4-T2 Export hardening
   DONE                DONE
   |                   |
   +---------+---------+
             |
             v
      M4-T3 Bounded expansion           DONE
             |
             v
      branch-local QA/audit             PENDING
             |
             v
      M4-T4 Operational adoption        NEXT
             |
        findings only
             v
      M4-T5 Conditional hardening
             |
             v
      M4-T6 Closure
```

## M4-T4 — operational adoption

After branch-local S0–S10 verification, use the path on ordinary DubBridge tasks. Record only actionable friction:

- stale graph race;
- missing relevant context;
- irrelevant context;
- policy over-blocking or under-blocking;
- host resource regression;
- receipt/capsule consumer race.

No synthetic benchmark task or A/B program is required.

## M4-T5 — conditional only

Create a narrowly scoped hardening task only from reproducible T4 evidence. Examples include pair-level artifact race, recurring classification false positives/negatives, repeated freshness lifecycle races, or context-selection friction.

Do not pre-schedule dashboards, policy DSLs, Neo4j/GraphRAG work, automatic RRI changes, or other speculative infrastructure.

## Resource behavior

Keep the sequential host model:

```text
index/query -> receipt/capsule -> graph mostly idle -> one heavy local model -> review
```

M4 must not make the CKG a permanently resident reasoning agent.

## Milestone closure condition

M4 is complete when:

1. stale graph consumption fails closed on the operational path;
2. cloud output exposes only justified/minimized metadata/content;
3. missing context can be expanded through a bounded local path without graph traversal;
4. the mechanism has been used on normal DubBridge work and material friction is fixed or explicitly deferred;
5. plan/task/audit/operator docs describe the same contract;
6. branch-local verification evidence is recorded.

No formal performance target or benchmark threshold is required.
