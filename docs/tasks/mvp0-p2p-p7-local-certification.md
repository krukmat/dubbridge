---
type: TaskList
title: "Tasks: P7Local Local end-to-end P2P pre-certification"
status: planned
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p7-local-certification.md
behavioral_coverage_contract: behavior-v2
---

# P7Local — planning task ledger

**Status:** Planned. Documentation can be prepared before the full execution gate; no local product-flow PASS is implied.

**Purpose:** Pre-certify the composed MVP0-P2P owner/viewer flow in a reproducible local Android environment before P7 exact-artifact certification.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P7L.T0 | Freeze local certification profile, CU matrix and evidence manifest | planning | S | P5 implementation complete; P6 contract available | Planned; documentation may be prepared early |
| P7L.T1 | Happy-path owner-to-viewer local E2E | integration/e2e evidence | M | P5 PASS; P6.T2 PASS | Planned; blocked |
| P7L.T2 | Negative/lifecycle local controls | integration/e2e evidence | M | T1 PASS | Planned; blocked |
| P7L.T3 | Local verdict and downstream handoff | evidence/handoff | S | T2 evidence complete | Planned; blocked |

## Shared evidence rules

Every executed local certification task records the exact branch/commit, Android build identity, device/emulator profile, backend/runtime profile and redacted actor aliases. Evidence must distinguish UI observation from authoritative runtime/backend proof.

Visual automation may demonstrate navigation and state presentation, but cannot by itself prove authorization, ciphertext integrity, key handling, loopback-only delivery, tamper rejection, teardown or account isolation.

P7Local never upgrades another phase to PASS merely because the composite flow worked once. If evidence reveals a defect owned by P3-P6, stop the affected scenario and return it to the owning phase/task.

## P7L.T0 — Freeze local certification profile, CU matrix and evidence manifest

**Type:** planning

**Effort:** S (provisional)

**Depends on:** P5 implementation complete; P6 state/action contract available. Documentation may be prepared before P6 implementation, but execution inputs must be refreshed after P6 changes.

**Status:** Planned; preparation allowed, execution profile not frozen.

**Acceptance criteria:**

- Freeze the seven canonical P7Local use cases from the phase plan against the then-current P3-P6 interfaces.
- Record the local backend/runtime services, Android build command/profile and device/emulator identity required by the run.
- Define one evidence manifest tying all observations to one commit/build identity.
- Define which assertions are UI/component, backend/contract, runtime/integration or device/e2e evidence.
- Explicitly preserve review playback as a non-regression path and prohibit remote audience-media fallback.

**HP-P7L.T0-1:** An agent can execute the local certification without inventing missing actor, build, service or evidence assumptions.

**EC-P7L.T0-1:** A changed P5/P6 interface or build identity invalidates the frozen execution profile until it is refreshed; stale screenshots/logs are not carried forward as proof.

**Evidence to emit:** `docs/audit/` task decision/profile record plus the evidence-manifest template used by T1-T3.

## P7L.T1 — Happy-path owner-to-viewer local E2E

**Type:** integration/e2e evidence

**Effort:** M (provisional)

**Depends on:** P5 PASS; P6.T2 PASS; T0 execution profile frozen.

**Status:** Planned; blocked.

**Acceptance criteria:**

Execute the following ordered path on the recorded local profile:

`owner content -> eligible/Ready -> create invite -> viewer receives/enters invite -> claim -> start sync -> verified package -> Available -> Play -> local loopback playback`.

The run must show that action eligibility is derived from canonical backend/runtime facts, not optimistic UI state. Playback must use the P5 local session path and existing VideoPlayer seam without HTTP/S3 audience-media fallback.

**HP-P7L.T1-1:** The intended viewer reaches local playback from the authorized invitation using the verified ciphertext package.

**EC-P7L.T1-1:** Neither S-120 Ready alone, claim alone nor completed bytes without P4 verification may produce an effective `Available/can_play` state.

**Evidence to emit:** ordered CU result record, relevant backend/runtime/device logs, build identity, and UI evidence for meaningful product-state transitions.

## P7L.T2 — Negative/lifecycle local controls

**Type:** integration/e2e evidence

**Effort:** M (provisional)

**Depends on:** T1 PASS

**Status:** Planned; blocked.

**Acceptance criteria:**

Verify the composed product remains fail-closed for the required negative/lifecycle paths:

- expired/denied/foreign-viewer invitation;
- interrupted or incomplete sync;
- altered/missing package material or failed verification;
- unavailable/invalid playback key material or authenticated-decrypt failure;
- playback error and retry;
- stop/unmount/logout/account switch;
- attempted reuse of prior-account state;
- review playback non-regression.

Teardown must be deterministic and idempotent. No negative path may activate remote audience-media delivery as fallback.

**HP-P7L.T2-1:** Recoverable failures can be retried only after the owning session/resource teardown completes and the authoritative state still permits retry.

**EC-P7L.T2-1:** Tamper, authorization failure or account switch never leaves stale `Play`, stale loopback ownership, reusable transient key state or cross-account content authorization.

**Evidence to emit:** negative-control matrix with actual PASS/FAIL, selected runtime/backend/device logs, and any UI evidence needed to show stale actions/states are absent.

## P7L.T3 — Local verdict and downstream handoff

**Type:** evidence/handoff

**Effort:** S (provisional)

**Depends on:** T2 evidence complete (PASS or recorded failure)

**Status:** Planned; blocked.

**Acceptance criteria:**

- Produce a single local verdict tied to the T0 evidence manifest and exact commit/build identity.
- Record unresolved failures by owning phase/task rather than waiving them.
- Identify any P6.T3 visual/flow evidence that must be refreshed.
- Identify inputs that P7 must repeat or strengthen against the exact RC/deployment artifact.
- Explicitly state that P7Local PASS is not P7 PASS.

**HP-P7L.T3-1:** A future P7 operator can see exactly which behaviors were locally demonstrated and which exact-artifact/no-fallback controls still require P7 evidence.

**EC-P7L.T3-1:** Mixed-build, stale, inferred or screenshot-only evidence cannot produce a local PASS verdict.

**Evidence to emit:** P7Local verdict/handoff record under `docs/audit/`, linked to the manifest and scenario evidence.

## Agent handoff

Read `docs/plan/mvp0-p2p-p7-local-certification.md`, P5/P6 plans and ledgers, ADR-043 and ADR-044 before execution. Do not bypass P5/P6 PASS dependencies. Prepare T0 inputs early if useful, but only execute T1/T2 when their dependencies are authoritative. If a scenario exposes a product defect, return it to its owning phase rather than adding a local workaround.
