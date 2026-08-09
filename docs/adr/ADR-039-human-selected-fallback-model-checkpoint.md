---
type: ADR
title: "ADR-039: Human-selected fallback model checkpoint"
status: Accepted
supersedes: ""
superseded_by: ""
---

# ADR-039: Human-selected Fallback Model Checkpoint

- **Status:** Accepted
- **Date:** 2026-08-09
- **Deciders:** DubBridge owner and platform-agent workflow maintainers
- **Scope:** agent workflow, reviewer fallback, and local-to-cloud handoff only;
  no application-runtime or product-architecture change
- **Amends:** ADR-034 reviewer fallback evidence, ADR-036 local-first cloud
  escalation, and ADR-038 Med-high cloud routing
- **Owner approval:** the owner authorized the bounded change in the active
  session with `autorizado`, after reviewing the summary of the checkpoint,
  receipt, D14/cloud separation, model matrix, documentation, scripts, and tests.

## Context

DubBridge already stops local implementation after its bounded attempt budget and
`peer-workflow-review.py` already returns `d14_required` when the reviewer chain
is unusable. Those stop conditions do not carry a machine-readable human model
selection. The approval card can recommend a cloud model, but a failure that
occurs later cannot prove whether a human confirmed that recommendation, changed
it, or preauthorized it before D14 or a cloud implementer started.

The aggregate change spans reviewer fallback, Low/Moderate/Med-high local exits,
policy, templates, and generated agent summaries. Its RRI is 89 (Very high), so
it must be delivered through the decomposed tasks in
`docs/tasks/fallback-model-checkpoint.md`; no single 89-point implementation is
permitted.

## Decision

### 1. Insert a fail-closed checkpoint before every cloud fallback

When a local reviewer or implementer reaches a terminal fallback condition, the
responsible script emits a `fallback-selection-v1` artifact before any D14 or
cloud execution begins. The artifact is bound to the exact input packet with a
SHA-256 digest and records the task, phase, trigger, fallback role, recommended
model, and recommended reasoning effort.

### 2. Support two explicit selection modes

- `human-select` is the interactive default. Without a complete selection, the
  artifact status is `awaiting_fallback_selection` and the process exits without
  authorizing the fallback.
- `preauthorized` is permitted only when the model, reasoning effort, and human
  selector were frozen in the approved task card or preflight. Missing fields
  fail closed.

In either mode, an authorized selection records the exact selected model,
reasoning effort, selector, packet digest, timestamp, and a deterministic receipt
digest. A materially changed packet invalidates the receipt.

### 3. Keep reviewer and implementation fallbacks distinct

- `d14` is a read-only, context-isolated reviewer/adjudicator. Its default remains
  Balanced; in the current Codex environment the recommendation is
  `gpt-5.6-terra` at `medium`.
- `cloud-implementer` authors code only after the local implementation route ends.
  Its recommendation follows the RRI/cause matrix in the workflow guide.

Selecting a model does not change the RRI, waive HITL approval, make D14 an
implementer, or allow a reviewer to self-review.

### 4. Treat the receipt as authorization evidence, not execution

The scripts do not spawn D14 or a cloud model. They stop with either an awaiting
checkpoint or an authorized receipt. The orchestrator must verify the receipt's
packet digest and then invoke exactly the selected model and effort. Any mismatch,
missing receipt, or stale packet remains blocked.

## Risk analysis

| Risk | Failure mode | Mitigation |
|---|---|---|
| Silent fallback | Orchestrator starts D14/cloud from an old `d14_required` or escalation artifact | New workflow requires a matching authorized receipt before execution |
| Stale approval | Packet changes after human selection | Receipt binds the canonical packet SHA-256; mismatch fails closed |
| Role confusion | D14 selection is treated as cloud implementation approval | Artifact records `role`; policy defines D14 as read-only |
| Cost escalation | Infrastructure outage promotes a task to Premium | Recommendation separates operational-only and capability/risk triggers |
| Automation deadlock | Unattended workflow always waits for a human | `preauthorized` mode is explicit and auditable |
| False identity assurance | Free-form selector field is mistaken for cryptographic identity | Receipt is tamper-evident, not identity-signed; external identity attestation remains out of scope |

## Consequences

### Positive

- Humans can change the concrete model and effort at the moment fallback is
  actually needed.
- Interactive and unattended workflows use the same artifact contract.
- Review fallback and implementation takeover remain auditable and role-correct.

### Negative

- Interactive fallback adds one pause and resume step.
- Every local exit path must preserve the same receipt contract.
- The receipt proves packet integrity and recorded selection, not the real-world
  identity of the selector.

## Alternatives considered

- **Always use the model in the original task card:** rejected because availability,
  task evidence, and cost constraints may change before fallback.
- **Let the orchestrator choose silently:** rejected because it provides no human
  adjustment point or durable authorization evidence.
- **Open a new full task approval:** rejected because fallback does not necessarily
  change scope or RRI; a focused model-selection checkpoint is sufficient unless
  the evidence changes the task's risk or boundaries.

## Related

- `docs/plan/fallback-model-checkpoint.md`
- `docs/tasks/fallback-model-checkpoint.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/RRI_POLICY.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md`
- `docs/adr/ADR-036-local-first-agentic-implementation-band.md`
- `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md`
