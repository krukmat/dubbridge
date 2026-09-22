---
type: Plan
title: Human orchestrator knowledge transfer
status: artifacts delivered; human rehearsal pending
---

# Human orchestrator knowledge transfer

## Objective

Transfer day-to-day orchestration to Matias through one concise Spanish guide,
a safe command launcher, and three practical exercises. The canonical workflow
guide remains the source of truth; the KT material explains how to operate it.

## Design

- One operational entry point: `docs/playbooks/HUMAN_ORCHESTRATOR_KT.md`.
- `scripts/human-orchestrator.py` is a thin, dry-run-first adapter over existing
  scripts. It never infers approvals, routing, retries, or completion.
- Durable status stays in the task ledger; detailed machine evidence stays in
  `.agent/human-orchestrator-kt/` with a compact audit index.
- The legacy runbook path remains as a compatibility pointer.

Dependencies: workflow/HITL/RRI policies, existing orchestration scripts, and
the plan/ledger of each real task. No product architecture or roadmap changes.

## Completion

Artifacts and automated verification are complete. The remaining outcome is
human rehearsal of Low, Moderate, and interrupted-task scenarios. Each live
exercise uses the real task's RRI and approval gates.
