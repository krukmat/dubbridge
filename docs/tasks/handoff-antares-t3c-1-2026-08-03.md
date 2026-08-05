---
type: TaskList
status: in-progress
---

# Handoff: Antares T3c-1 — Deterministic dependency and manifest closure

Session started in Codex and handed off to Claude Code on 2026-08-03. This
handoff captures the verified pre-execution state. Implementation has not
started.

## Current state

- Task: `T3c-1` — Deterministic dependency and manifest closure.
- Status: `[ ] Open`; explicit human approval is still pending.
- Pre-execution RRI: `55`, band `Med-high`, `Effort L`, thinking `On`.
- Phase 1: `PASS` via final D14 fallback.
- Required implementation route after approval: ADR-038 architect-refined
  single-attempt gate.
- No T3c-1 implementation files exist yet. The frozen T3c-0 corpus and
  `scripts/antares/packet_schema.py` are read-only.

Working-tree changes already present and belonging to this preparation:

- `docs/tasks/antares-security-specialist-advisor.md`
- `docs/plan/antares-security-specialist-advisor.md`
- `docs/audit/antares-t3c-1-rri.md`

Pre-existing unrelated change: `.coverage`. Preserve it and do not include it
in a T3c-1 change.

## Governing documents

- `docs/tasks/antares-security-specialist-advisor.md`
- `docs/plan/antares-security-specialist-advisor.md`
- `docs/plan/roadmap.md`
- `docs/architecture.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md`
- `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md`
- `docs/adr/ADR-037-local-architect-complex-analyst.md`
- `docs/audit/antares-t3c-1-rri.md`

## Implementation surface

Implement only the following paths after approval:

- `scripts/antares/context_closure.py` — new closure implementation; full file.
- `scripts/antares/context_closure_test.py` — new unit/characterization tests;
  full file.
- `scripts/antares/testdata/context_closure_dependency_manifest/**` — dedicated
  fixtures only.

Do not modify `scripts/antares/packet_schema.py`, the frozen
`context_closure_characterization` corpus, T3c-2/T3d code, or any ambient
repository files.

## Goal

Given an explicit snapshot root and canonical changed paths, compute a bounded,
deduplicated, deterministic closure of local Rust/Python source dependencies
and relevant local manifests, using canonical snapshot-relative POSIX paths.
The root must be caller-supplied; never infer it from the working directory,
manifests, imports, or package tooling.

## Acceptance criteria

- HP-1: Rust local `mod` closure and applicable Cargo manifest/path-dependency
  closure are deterministic; Rust `use` statements do not create file edges.
- HP-2: Python local relative/allowed absolute import closure follows the fixed
  module mapping; external stdlib/third-party imports are ignored.
- HP-3: seed permutations produce byte-for-byte equivalent sorted output;
  duplicates are removed and changed seeds remain identifiable.
- EC-1: empty seeds produce zero derived entries and exactly the frozen
  `__seed__` / `context_closure_no_seed` omission, without scanning.
- EC-2: unsupported file types produce the frozen unsupported-file omission.
- EC-3: expansion limits stop before the next canonical pending source and emit
  the specified deterministic limit omission.
- EC-4: containment escapes are soft `path_outside_snapshot` omissions with
  canonical absolute paths.
- EC-5: unresolved local edges, missing seeds, malformed/invalid allowlisted
  manifests, and invalid encoding raise the typed
  `ContextClosureResolutionError`; no result, partial closure, or fabricated
  omission is returned.
- EC-6: cycles resolve each edge before ignoring visited back-edges and remain
  bounded and deterministic.
- EC-7: manifest ancestor discovery, allowlist, Cargo entrypoint rules, path
  dependencies, empty manifests, and manifest parse behavior match the task
  ledger exactly.
- EC-8: canonicalization is snapshot-relative POSIX, case-sensitive,
  locale-independent, and symlink escapes remain soft omissions.
- EC-9: Python relative imports remain local and fail closed when their package
  or target cannot be resolved; external absolute imports remain ignored.
- EC-10: strict/opaque manifest parsing rules are enforced without executing
  `setup.py`, using package caches, subprocesses, or network access.
- Add a failing network-primitive sentinel test proving local-only behavior.
- Preserve the exact T3c-0 omission literals and the T3b containment/reporting
  contract, including omission fields `path`, `reason`, and non-empty `detail`.
- Run the scoped unit tests plus the relevant repository QA gates.

## Required workflow after approval

1. Record the live task checklist before implementation; keep one phase active.
2. Run ADR-038 Qwen27 advisory refinement via
   `scripts/local-architect/run_analysis.py --profile med-high-refinement-v1`.
3. Produce the hash-bound route receipt with
   `scripts/local-agent/med_high_gate.py`.
4. If both sides resolve `GO_LOCAL`, allow exactly one
   `qwen3.6:35b-a3b` session through `run_med_high_task.py` (maximum 8 turns,
   300 seconds, zero repairs). Otherwise escalate to Claude/Codex with the
   complete ADR-038 evidence bundle. Never bypass or silently upgrade the route.
5. Apply three complete Reflection passes: contract, failure/containment
   boundaries, and deterministic coverage. Record Draft → Critique → Revise
   for every pass.
6. Run phase 2 code review through the required chain:
   `qwen3.6:27b-q4_K_M` → Gemma fallback → D14 final fallback. Record the
   exact `Code-solution review:` line and artifact.
7. Only after phase 2 PASS, reflection, tests, unit coverage certification, and
   owner final verification, synchronize the task ledger, plan, RRI artifact,
   and downstream blocker text; then mark T3c-1 done.

## Required report lines

Existing phase 1 evidence:

`Task-analysis review: d14 .agent/peer-task-review-antares-t3c-1-phase1-d14.json - PASS`

Required at closure:

`Code-solution review: <qwen3.6:27b-q4_K_M|gemma|d14> <artifact path> - <PASS|BLOCKED>`

## Stop condition

Execution has not started. Claude must stop here and obtain explicit user
approval before editing any implementation path, invoking the ADR-038 route,
or changing T3c-1 status.
