---
type: ADR
title: "ADR-042: Push-review remediation controller and bounded escalation lifecycle"
status: Proposed
supersedes: ""
superseded_by: ""
---

# ADR-042: Push-review remediation controller and bounded escalation lifecycle

- **Status:** Proposed
- **Date:** 2026-08-24
- **Deciders:** DubBridge owner and agent-workflow maintainers
- **Scope:** X27 post-pipeline review, remediation planning, bounded local repair,
  frontier/human handoff, and report publication; no application-runtime or
  product-domain change
- **Amends if accepted:** ADR-034 reviewer reconciliation evidence and ADR-039
  fallback-selection application to push-review remediation handoffs

## Context

The X27 Gemma Push Reviewer baseline is deployed even though its plan, task
ledger, and roadmap still describe it as proposed. A 2026-08-24 implementation
audit found that the deployed behavior does not satisfy several of its own
contracts:

- the push-audit path performs one model generation but reports a successful
  one-pass quorum instead of the specified three independent passes;
- collected annotations, failed-log tails, and artifact content are incomplete
  or are not included in the model-visible evidence packet;
- missing or malformed model-proposed RRI inputs can be interpreted as zero or
  can terminate the whole audit instead of routing to manual review;
- ungrounded observations, blocked audits, non-Low findings, and failed local
  delegations can disappear from the durable daily follow-through path;
- a failed delegation is labeled `in_review` even when no patch exists;
- the self-hosted workflow can execute repository code from a reviewed SHA with
  write-capable credentials, and its publisher is not fully idempotent.

The desired outcome is broader than another reviewer prompt. The layer must be
able to evaluate evidence, turn supported findings into bounded remediation
plans, implement only when policy permits, and otherwise produce a complete
handoff to an authorized frontier agent or human. Those responsibilities cannot
all belong to the Gemma Push Reviewer itself without violating its advisory
authority boundary.

The aggregate remediation program scores RRI 96 (Very high): it combines agent
orchestration, secret-bearing CI evidence, self-hosted workflow trust, local
patch delegation, durable state, and human/cloud escalation. It must therefore
be approved as an architecture decision and delivered through the decomposed
tasks in `docs/tasks/gemma-push-reviewer-role.md`.

## Decision

This decision takes effect only if the ADR is accepted.

### 1. Split the layer into four authorities

- **Evaluator — Gemma Push Reviewer:** read-only, runs independent push-audit
  passes and reports evidence-bound findings. It does not score final RRI,
  approve a plan, author a patch, or close an item.
- **Remediation controller — deterministic repository code:** validates evidence,
  invokes `scripts/rri.py`, creates the remediation plan/work item, enforces the
  state machine, and selects only policy-allowed routes.
- **Implementer — Qwen Developer or an explicitly selected frontier agent:**
  authors a candidate patch within the approved packet and path boundary. It
  never reviews or accepts its own work.
- **Acceptor — primary agent or human owner:** owns approval where required,
  dispositions review evidence, and is the only authority that can accept or
  close a remediation item.

Using the same underlying model family in more than one role does not merge
these authorities. Each role keeps its own prompt, packet, output contract, and
audit record.

### 2. Evidence precedes inference

Before model evaluation, the controller collects run/job metadata, annotations,
budgeted failed-log tails, and an artifact manifest. All model-visible or
committed text is secret-redacted. The packet records completeness, truncation,
redaction, and collection failures per evidence class.

Missing required evidence cannot become `PASS`. It produces `evidence_partial`
or `blocked`, with a durable work item when follow-through is required. The model
must receive the evidence text needed to support a finding, not only local file
paths or counts.

### 3. Evaluation uses real independent passes and deterministic reconciliation

The default push audit runs three fresh-context model generations. At least two
usable passes are required; exactly two is degraded. Reconciliation is imported
from one shared deterministic implementation and records consensus,
pass-specific, severity/location disagreement, and likely-false-positive
buckets. `BLOCKED`, timeout, malformed output, or empty terminal content is a
failed pass, never a successful quorum.

One-pass mode remains an explicit diagnostic/cost escape and must be labeled
`single_pass_no_quorum`; it cannot claim the normal anti-drift guarantee.

### 4. Every actionable finding becomes a validated remediation work item

The controller creates a deterministic item keyed by
`push-<short-sha>-F###`. A work item contains:

- source run and evidence references;
- grounded path/line and reconciliation class;
- proposed fix direction, bounded acceptance criteria, `HP-#`/`EC-#` examples,
  allowed paths, verification intent, and stop conditions;
- validated RRI inputs, their provenance/confidence, the complete canonical
  `scripts/rri.py --json` result, and active penalties;
- route, current state, attempt history, review receipts, approval/fallback
  receipts, and final disposition.

Missing, non-numeric, out-of-range, or unsupported RRI inputs never default to
zero. The item routes to `needs_planning` or `awaiting_human` until a validated
input set exists. Model proposals remain advisory and cannot overwrite measured
inputs or path floors.

### 5. Durable lifecycle is explicit and fail-closed

The controller persists one idempotent work-item artifact per finding or whole-
audit blocker. The minimum state vocabulary is:

```text
observed -> grounded -> planned -> scored
scored -> phase1_review | awaiting_approval | awaiting_human
phase1_review -> local_dispatch -> patch_ready -> phase2_review
awaiting_approval -> fallback_selection -> frontier_ready
patch_ready | frontier_ready -> awaiting_acceptance -> closed
any non-terminal state -> blocked | changes_requested
```

`in_review` is valid only when a patch exists. A failed/no-patch dispatch uses
`blocked` or `needs_retry`. `closed` and `dismissed` require a disposition,
actor, timestamp, and evidence reference. Daily reports are projections of this
durable state; rolling to a new day never deletes an unresolved item.

### 6. Local implementation is bounded and independently reviewed

Only canonical pure-Low, narrow code/test items may enter local implementation.
Before each delegation packet, the Low-band phase-1 reviewer chain must return
`PASS`. The controller may perform the initial Qwen Developer attempt plus one
evidence-backed repair for a diagnosable failure. A materially changed repair
packet receives a new phase-1 review. Hard scope, editorial, security, penalty,
or high-impact refusals are not repairable.

The output is a patch artifact in an isolated/disposable worktree; the push
workflow does not commit it directly to the reviewed branch. Phase-2 independent
review, verification, coverage evidence, and primary-agent/human acceptance are
still required before application or closure.

### 7. Frontier and human routes are handoffs, not silent escalation

RRI 26+ items receive an approval-ready draft packet but remain
`awaiting_approval`; no implementation starts without the normal HITL checkpoint.
When a frontier implementer or D14 reviewer is required, the controller emits a
`fallback-selection-v1` checkpoint bound to the exact packet, using ADR-039.

The unattended workflow may continue only with complete, matching
`preauthorized` fields. Otherwise it stops at `awaiting_fallback_selection` and
surfaces the item to a human. A selected frontier implementer receives only the
approved packet and may not expand scope. D14 remains read-only and cannot be
used as an implementer.

### 8. Workflow execution and publication use separate trust boundaries

The write-capable publisher executes only trusted default-branch controller
code. A reviewed commit SHA is treated as input data, never as executable
workflow code. Read-only audit/patch generation is separated from publication;
the publisher uses least privilege, a branch allow-list, SHA-bound artifacts,
idempotent item/report keys, concurrency control, and non-fast-forward recovery.

No pull request or arbitrary branch push may execute untrusted repository code
on the self-hosted runner with write credentials. Actions are pinned to immutable
revisions when the implementation task lands.

## Risk analysis

| Risk | Failure mode | Mitigation |
|---|---|---|
| Reviewer self-authority | The evaluator plans, implements, and accepts its own finding | Four-role split; deterministic controller; independent phase-1/phase-2 review; acceptor-only close |
| False certainty | A single or blocked model pass is reported as quorum/PASS | Real N-pass accounting; two-pass minimum; typed single-pass diagnostic state |
| Evidence blindness | Findings are inferred without failed logs/annotations/artifacts | Evidence completeness matrix; model-visible redacted text; fail-closed partial state |
| Secret disclosure | CI output enters prompts or committed reports unredacted | Shared redaction before persistence/model use; security-focused unit fixtures; partial-state fallback |
| RRI under-scoring | Missing/invalid model hints default to zero | Typed validation, provenance/confidence, measured inputs, manual route on uncertainty |
| Lost work | Report rows disappear at daily rollover | Per-item durable state and idempotent daily projection until explicit disposition |
| Patch confusion | `in_review` is set when no patch exists | State invariant requiring a patch digest/path before `patch_ready` or `in_review` |
| Unauthorized cloud work | CI silently invokes a frontier model | HITL gate plus packet-bound ADR-039 receipt; fail closed without selection |
| Untrusted runner execution | Feature-branch code runs with write token | Trusted-base controller, reviewed SHA as data, read/write job separation and least privilege |
| Duplicate/racing publication | Reruns create duplicate rows or lose a main push race | Deterministic keys, concurrency group, fetch/rebase/retry policy, idempotency tests |
| Automation noise | Low-value observations flood the human queue | Grounding, reconciliation class, severity/RRI thresholds, deduplication, explicit dismiss disposition |

## Consequences

### Positive

- X27 can progress from evidence to a bounded plan and, for pure Low work, to a
  reviewed patch artifact without weakening approval or acceptance boundaries.
- Moderate+ and failed local work reach a complete frontier/human packet instead
  of ending as an ephemeral report row.
- Quorum, evidence completeness, RRI provenance, attempts, approvals, and final
  disposition become reconstructable per finding.
- Self-hosted execution no longer requires trusting the reviewed commit's code
  with write credentials.

### Negative / cost

- Normal evaluation costs three model generations plus any phase-1/phase-2
  review calls.
- Durable work-item state and idempotent publication add schema and migration
  work to a previously report-only path.
- Frontier fallback can pause unattended processing until a human selects or
  preauthorizes the exact model and effort.
- Existing X27 completion evidence must be revalidated; historical `[x] Done`
  records cannot be treated as current closure evidence.

### Neutral

- The layer remains advisory with respect to product acceptance and deployment.
- The primary GitHub pipeline remains authoritative; push review is a
  post-pipeline remediation workflow, not a merge gate.
- This ADR does not alter application crates, product APIs, media processing, or
  governance-domain invariants.

## Alternatives considered

- **Give Gemma Push Reviewer end-to-end authority:** rejected; it would evaluate,
  implement, and accept its own work and would bypass the repository's reviewer
  and HITL contracts.
- **Keep reports only and rely on daily manual transcription:** rejected; live
  evidence shows unresolved findings disappear without durable ownership.
- **Send every finding directly to a frontier model:** rejected; it ignores RRI,
  cost, approval, least-authority, and fallback-selection rules.
- **Allow direct commits for pure Low patches:** rejected; patch generation and
  acceptance/publication remain separate trust boundaries.
- **Treat missing RRI inputs as zero:** rejected; uncertainty must raise review
  requirements, not reduce them.
- **Run the reviewed SHA's workflow code on a write-capable self-hosted runner:**
  rejected; the SHA is evidence, not a trusted executable control plane.

## Related

- `docs/plan/gemma-push-reviewer-role.md`
- `docs/tasks/gemma-push-reviewer-role.md`
- `docs/plan/roadmap.md` § X27
- `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md`
- `docs/adr/ADR-039-human-selected-fallback-model-checkpoint.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md`
