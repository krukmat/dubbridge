# Tasks: Low-RRI Local Model Handoff

**Plan:** `docs/plan/low-rri-local-model-handoff.md`
**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `AGENTS.md`
**Related policy:** `docs/policies/RRI_POLICY.md`

## Status legend

- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
T0 -> T1 -> T2
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| T0 | Plan the handoff contract and file home | — | 13 | Low | S |
| T1 | Author the Low-RRI local handoff playbook | T0 | 13 | Low | S |
| T2 | Cross-link workflow and policy docs | T1 | 13 | Low | S |

## T0 — Plan the handoff contract and file home

- **Status:** [x] Done
- **Type:** Planning / docs
- **Objective:** Decide where the local-model handoff protocol lives and what it
  must cover.
- **Acceptance criteria:**
  - The file home is explicit.
  - The scope is limited to docs and cross-links.
  - The plan explains why the protocol is separate from HITL policy.

## T1 — Author the Low-RRI local handoff playbook

- **Status:** [x] Done
- **Type:** Docs
- **Objective:** Write the dedicated playbook for step-by-step local-model handoff.
- **Acceptance criteria:**
  - The playbook explains when to use local delegation.
  - It makes simple instructions and narrow scope mandatory.
  - It prefers pure development or mechanical tasks.
  - It defines review, failure, and retry discipline.

## T2 — Cross-link workflow and policy docs

- **Status:** [x] Done
- **Type:** Docs sync
- **Objective:** Add minimal references from the workflow guide and RRI policy to
  the new playbook.
- **Acceptance criteria:**
  - The workflow guide points readers to the new playbook for the operational
    handoff discipline.
  - The RRI policy points readers to the new playbook for the step-by-step
    Low-RRI protocol.
  - The links are minimal and do not rewrite large sections.
