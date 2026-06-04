# Plan: RRI Integration — Required Reasoning Index adoption

**Roadmap position:** Cross-cutting process improvement. Not a product slice.
Governs agent scoring, model selection, and autonomy gates for all future tasks.

**Implementation status (2026-06-04):**
- `T0` complete — plan + task ledger created.
- `T1` complete — `docs/policies/RRI_POLICY.md` created with formula, anchor rubric, bands, and gates.
- `T2` complete — `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` amended: RRI adoption section added, CC table subsumed into `C` variable, tier mapping driven by RRI band, Step 3 presentation block updated.

## Objective

Adopt the **Required Reasoning Index (RRI)** as the canonical method for scoring
complexity and risk in the DubBridge agentic workflow, **extending** (not
replacing) the single-axis cyclomatic-complexity scoring that already lives in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`.

The RRI adds the risk dimensions (test-coverage risk, domain complexity, coupling,
public/security/data impact, ambiguity) that the current Effort/Complexity labels
capture only implicitly. It makes the scoring repeatable across agents via a
repo-specific anchor rubric tied to the existing ADR set (ADR-008, ADR-023,
ADR-024, ADR-026).

## Decisions closed (scope locked)

| Decision | Choice | Rationale |
|---|---|---|
| Scope | Phase 0 only (T0→T1→T2) | Lightweight start; Phase 2 (tooling) deferred |
| Policy location | `docs/policies/RRI_POLICY.md` (A2) | Mirrors HITL_AUTONOMY_POLICY.md pattern; keeps the workflow guide from bloating |
| ADR for the adoption | None — inline adoption note in the guide | RRI is a workflow/process policy, not a runtime architecture decision |
| `AGENTS.md` / `CLAUDE.md` | Not touched | Workflow guide overrides both "without exception" on scoring; no propagation needed |
| CI / pre-push hooks | Not touched | Phase 2 concern; Phase 0 is pure documentation |

## Affected files

| File | Change | Task |
|---|---|---|
| `docs/plan/rri-integration.md` | Created (this file) | T0 |
| `docs/tasks/rri-integration.md` | Created (ledger) | T0 |
| `docs/policies/RRI_POLICY.md` | Created — canonical formula, rubric, bands, gates | T1 |
| `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` | Amended — new RRI section + adoption note | T2 |

## Module / document dependencies

```text
T0  (this ledger) ──► T1  (RRI_POLICY.md)
                           │
                           └──► T2  (amend AGENT_WORKFLOW_GUIDE.md → references RRI_POLICY.md)
```

## Design decisions

- **RRI is an extension, not a replacement.** The cyclomatic-complexity variable
  `C` in the formula maps directly to the CC formula already in the workflow guide.
  The model-tier mapping (Economy / Balanced / Premium + thinking) is preserved;
  it is now driven by the RRI band instead of the single CC range.

- **Anchor rubric is repo-specific.** The subjective variables (D, A, K, P, X) are
  scored against a rubric that names DubBridge's own crates and ADRs. This makes
  two independent agents score the same task to the same number.

- **Objective variables are measured, not guessed.** `F` (files affected) comes
  from `git diff`; `T` (coverage risk) from `cargo llvm-cov`; `C` from the
  CC formula or `clippy::cognitive_complexity`. Subjective variables are judged
  using the anchor rubric.

- **Advisory-first philosophy.** The RRI governs what evidence the agent must
  bring to the (always-mandatory) HITL approval checkpoint. It does not add new
  approval gates — it specifies per-band what those gates require.

## Related documents

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — highest-authority source (adopts RRI in T2)
- `docs/policies/HITL_AUTONOMY_POLICY.md` — structural pattern for T1
- `docs/policies/RRI_POLICY.md` — created in T1
- `docs/plan/roadmap.md` — no change required (RRI is process, not a product slice)
