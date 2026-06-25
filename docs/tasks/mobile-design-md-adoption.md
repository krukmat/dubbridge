---
type: TaskList
title: "Tasks: S-205 - Mobile DESIGN.md Adoption"
status: done
plan: docs/plan/mobile-design-md-adoption.md
---
# Tasks: S-205 - Mobile DESIGN.md Adoption

Plan: `docs/plan/mobile-design-md-adoption.md`.

RRI calculations were run on 2026-06-25 with `scripts/rri.py`. For Low-band
documentation and planning tasks, execute directly as the primary agent per
`docs/policies/HITL_AUTONOMY_POLICY.md`; local Gemma delegation is reserved for
eligible simple code patches only.

## Task summary

| ID | Title | RRI -> band | Effort | Status | Depends on |
|---|---|---:|---|---|---|
| S-205-T0 | ADR assessment + plan/ledger package | 22 -> Low | S | Done | - |
| S-205-T1 | Author root `DESIGN.md` from mobile design-system intent | 13 -> Low | S | Done | T0 |
| S-205-T2 | Add explicit `DESIGN.md` lint command | 18 -> Low | S | Done | T1 |
| S-205-T3 | Integrate `DESIGN.md` into mobile UI agent workflow | 28 -> Moderate | M | Done | T1, T2 |
| S-205-T4 | Audit playback mobile surfaces against `DESIGN.md` | 7 -> Low | S | Done | T1 |
| S-205-T5 | Closeout docs, roadmap, and optional follow-ups | 13 -> Low | S | Done | T2, T3, T4 |

---

## S-205-T0 - ADR assessment + plan/ledger package

- **Status:** Done
- **Effort:** S
- **RRI:** 22 -> Low (0-25)
- **Depends on:** -
- **Affected:** `docs/plan/mobile-design-md-adoption.md`,
  `docs/tasks/mobile-design-md-adoption.md`, `docs/plan/roadmap.md`

### Objective

Determine whether adopting Google Labs' `DESIGN.md` concept for DubBridge mobile
requires a new ADR or amendments to existing ADRs, then document the implementation
plan and task ledger.

### Inputs

- Google Labs `DESIGN.md` repository and format/spec/philosophy pages.
- Existing S-115/S-190/S-127 mobile design and playback plans.
- Related ADRs: ADR-029, ADR-032, ADR-033.
- Current mobile token and primitive files.

### Outputs

- S-205 plan and task ledger.
- ADR assessment recorded in the plan.
- Roadmap row for S-205.

### Acceptance criteria

- The plan states whether an ADR or ADR amendment is required.
- Related ADRs are named only if materially relevant.
- The plan records the conditions under which a later ADR would be needed.
- The task ledger decomposes adoption into ordered tasks with RRI, effort, inputs,
  outputs, acceptance criteria, and handoff prompts.
- The roadmap contains an S-205 row pointing to the new plan and ledger.

### Handoff prompt

Review the Google Labs `DESIGN.md` concept against DubBridge mobile's existing
S-115/S-190/S-127 design system. Decide whether ADR-029, ADR-032, or ADR-033 need
new amendments. Document the result in a new S-205 plan and task ledger, then add a
roadmap row.

### Completion notes

- Conclusion: no new ADR or ADR amendment is required for initial adoption because
  `DESIGN.md` is agent-facing design intent, not a runtime source, product-surface
  decision, playback-delivery boundary, or OKF vocabulary change.
- The plan records explicit triggers that would require a future ADR or amendment.

---

## S-205-T1 - Author root `DESIGN.md` from mobile design-system intent

- **Status:** Done
- **Effort:** S
- **RRI:** 13 -> Low (0-25)
- **Depends on:** T0
- **Affected:** `DESIGN.md`, `mobile/src/theme/tokens.ts`

### Objective

Create a root `DESIGN.md` that mirrors the existing mobile runtime tokens and
explains the DubBridge mobile visual identity for agents and humans.

### Inputs

- `mobile/src/theme/tokens.ts`
- `mobile/src/components/`
- `docs/plan/s-115-mobile-ux-foundation.md`
- `docs/plan/s-190-mobile-ux-usability-pass.md`
- Google Labs `DESIGN.md` format specification.

### Outputs

- New root `DESIGN.md` with YAML frontmatter and Markdown prose.

### Acceptance criteria

- Frontmatter includes mobile colors, typography, spacing, rounded values, and
  component token references where the spec supports them.
- Markdown sections follow the Google spec order where applicable.
- Prose clearly states that `mobile/src/theme/tokens.ts` remains the runtime source
  of truth.
- Do's and Don'ts prohibit new palettes, raw screen hex values, marketing-style
  heroes, nested cards, and engineering copy on user-facing surfaces.
- The document is written in English and scoped explicitly to DubBridge mobile.

### Handoff prompt

Create `DESIGN.md` at the repository root from the current mobile design system.
Mirror `mobile/src/theme/tokens.ts`, explain the S-115/S-190 "ink + teal" product
intent, and make the file useful to future agents without replacing runtime tokens.

### Completion notes

- Added root `DESIGN.md` using the Google Labs section order and alpha frontmatter
  schema.
- Mirrored the shipped mobile tokens for colors, spacing, rounded values, and
  typography without changing runtime ownership.
- Recorded explicit scope boundaries: mobile-only, agent-facing, and
  non-authoritative relative to `mobile/src/theme/tokens.ts`.
- Validation: `make qa-docs` passed; `npx -y @google/design.md lint DESIGN.md`
  reported `0` errors and only non-blocking unused-token warnings because the file
  mirrors the full runtime token set while the initial `components:` map stays
  intentionally compact.

---

## S-205-T2 - Add explicit `DESIGN.md` lint command

- **Status:** Done
- **Effort:** S
- **RRI:** 18 -> Low (0-25)
- **Depends on:** T1
- **Affected:** `Makefile`, `mobile/package.json`, `DESIGN.md`

### Objective

Add a repository command that validates `DESIGN.md` with Google's linter while
keeping the alpha external spec isolated from the main QA gate at first.

### Inputs

- Root `DESIGN.md`
- Google `@google/design.md` CLI behavior.
- Existing `make qa-docs` and `make qa-mobile` gate structure.

### Outputs

- A command such as `make qa-design` or `npm run design:lint`.
- Documentation in the task completion notes describing whether the command is
  advisory or blocking.

### Acceptance criteria

- The command runs the Google linter against root `DESIGN.md`.
- The command is easy for agents to discover.
- `qa-ci` is not widened unless the task explicitly records package stability and
  revisits the ADR assessment.
- Failure output is actionable enough for a future agent to fix broken references,
  section order, or contrast warnings.

### Handoff prompt

Add a lightweight `DESIGN.md` validation command using Google's CLI. Prefer a
separate repository target first; do not turn it into a hard global CI gate unless
the task explicitly rechecks the ADR and external-tooling risk.

### Completion notes

- Added `make qa-design` as the explicit repository entry point for validating
  root `DESIGN.md`.
- Kept the command outside `qa-docs` and `qa-ci` so the alpha external dependency
  remains opt-in rather than a hard global gate.
- Validation command: `npx -y @google/design.md lint DESIGN.md`.

---

## S-205-T3 - Integrate `DESIGN.md` into mobile UI agent workflow

- **Status:** Done
- **Effort:** M
- **RRI:** 28 -> Moderate (26-40)
- **Depends on:** T1, T2
- **Affected:** `AGENTS.md`, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `DESIGN.md`

### Objective

Update the agent workflow so future mobile UI tasks read `DESIGN.md` as part of
their mobile design context.

### Inputs

- Root `DESIGN.md`
- `AGENTS.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- S-115/S-190/S-127 plans.

### Outputs

- A workflow rule that names `DESIGN.md` as required context for mobile UI work.

### Acceptance criteria

- The workflow change is scoped to mobile UI/presentation tasks, not all backend
  or platform tasks.
- The change does not weaken the mandatory plan -> tasks -> RRI -> approval flow.
- The rule preserves `AGENT_WORKFLOW_GUIDE.md` as the workflow authority.
- The rule states that `DESIGN.md` guides visual intent while task files govern
  behavior, acceptance criteria, and verification.
- Because RRI is Moderate, execution requires the normal approval presentation
  before editing workflow files.

### Handoff prompt

Integrate root `DESIGN.md` into the mobile UI agent workflow. Keep the change
narrow: agents should read it before mobile presentation work, but it must not
override the existing workflow guide, task ledger, RRI gate, or runtime source of
truth in `tokens.ts`.

### Completion notes

- Added a mobile-UI-specific analysis rule to
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` requiring agents to read root
  `DESIGN.md` before planning or implementing mobile presentation work.
- Added a matching note to `AGENTS.md` so mobile UI task presentations include
  `DESIGN.md` in `Related documents` when it materially constrains the work.
- Kept the rule explicitly subordinate to the workflow guide, task files, and
  runtime token source of truth in `mobile/src/theme/tokens.ts`.

---

## S-205-T4 - Audit playback mobile surfaces against `DESIGN.md`

- **Status:** Planned
- **Effort:** S
- **RRI:** 7 -> Low (0-25)
- **Depends on:** T1
- **Affected:** `docs/audit/mobile-design-md-playback-audit.md`, `DESIGN.md`

### Objective

Use the new `DESIGN.md` as a pilot review artifact for the current playback
surfaces before applying it broadly to future mobile UI work.

### Inputs

- `DESIGN.md`
- `mobile/src/screens/ReviewDetailScreen.tsx`
- `mobile/src/screens/AssetDetailScreen.tsx`
- `mobile/src/components/PlaybackStateView.tsx`
- `mobile/src/components/VideoPlayer.tsx`
- `mobile/artifacts/screenshots/18_asset_detail_playback.png`
- `mobile/artifacts/screenshots/19_review_detail_playback.png`

### Outputs

- `docs/audit/mobile-design-md-playback-audit.md` with findings and either:
  - "no code changes required"; or
  - a follow-up task list of specific low-risk mobile UI patches.

### Acceptance criteria

- The audit checks colors, typography, panel/card use, button hierarchy, state
  surfaces, error/empty/loading copy, long IDs, and action usability.
- The audit distinguishes design-intent drift from behavior/API concerns.
- Any recommended code change is small, testID-preserving, and explicitly scoped;
  broad redesign recommendations are rejected or deferred.
- No implementation changes are made in this task.

### Handoff prompt

Read `DESIGN.md` and audit the playback mobile surfaces against it. Produce a short
audit file with concrete findings and follow-ups. Do not change code in this task.

### Completion notes

- Added `docs/audit/mobile-design-md-playback-audit.md`.
- Result: no redesign required; the playback surfaces broadly match `DESIGN.md`.
- Recorded two narrow follow-ups:
  - dark playback overlays currently reuse `StateView` light-surface text colors,
    which weakens readability on the dark media shell;
  - adjacent summary metadata still exposes raw IDs and may need the same
    formatting/polish treatment used elsewhere in S-190.
- No implementation changes were made in this audit task.

---

## Future optional development patch - Playback surface polish

If S-205-T4 finds concrete UI drift that is safe to fix, create a separate
development task before editing code.

### Behavioral example set for that future development task

- **HP-1:** A review task with playable media shows a token-consistent playback
  panel, keeps Approve/Reject available, and preserves existing testIDs.
- **HP-2:** A finalized asset shows the Play affordance and inline playback area
  with token-consistent copy, spacing, and state rendering.
- **EC-1:** Playback denial or grant failure renders a token-consistent failure
  state while leaving decision or compliance actions usable.
- **EC-2:** Long asset/task IDs and localized timestamps do not overlap buttons,
  badges, or player controls.

---

## S-205-T5 - Closeout docs, roadmap, and optional follow-ups

- **Status:** Done
- **Effort:** S
- **RRI:** 13 -> Low (0-25)
- **Depends on:** T2, T3, T4
- **Affected:** `docs/plan/mobile-design-md-adoption.md`,
  `docs/tasks/mobile-design-md-adoption.md`, `docs/plan/roadmap.md`,
  `mobile/maestro/playback.yaml`

### Objective

Synchronize status documents and record whether S-205 remains documentation-only or
opens follow-up mobile UI patches.

### Inputs

- Completed T1-T4 outputs.
- Design lint result.
- Playback audit result.

### Outputs

- Updated plan/task statuses.
- Roadmap status updated.
- Follow-up tasks recorded if the audit finds actionable drift.

### Acceptance criteria

- `make qa-docs` passes.
- Any design lint command added by T2 has a recorded result.
- The roadmap row accurately states whether S-205 is done, planned, or blocked.
- If code follow-ups are needed, they are not hidden inside closeout; they are
  recorded as separate tasks with RRI and behavioral examples.

### Handoff prompt

Close out S-205 by syncing the plan, task ledger, and roadmap. Run documentation
verification and record the design lint result. If playback audit findings require
code, open separate tasks instead of folding implementation into closeout.

### Completion notes

- Updated the S-205 plan, task ledger, and roadmap row to reflect slice closure.
- Verified `make qa-design` and recorded the current result: `0` errors, only
  non-blocking unused-token warnings.
- Verified `make qa-docs` passes after all S-205 documentation and workflow
  updates.
- Kept the playback audit findings outside the closeout and recorded them as
  separate future development follow-ups.
