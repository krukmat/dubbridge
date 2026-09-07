---
type: Audit
title: "Autonomous execution window 2026-09-06/07: closed leaves and 7h queue"
status: in-progress
slice: MVP0-P2P
parent: P2.T2
---

# Autonomous execution window (owner absent, ~7h, 2026-09-06 22:xx onward)

## Mandate

Owner instruction (2026-09-06, this session, model switched to Opus): act
autonomously as orchestrator, maximize local-dev delegation, actively split
tasks to RRI Low where a real seam allows it, use own judgment where a
Low split is not honest, and treat the owner's own RRI-band judgment as
replaceable by the orchestrator's where warranted. Cloud-only or
HITL-blocked tasks that do not block other work may be postponed to the
next contact (~7h). Token/time consumption should be watched — this
document exists so a resumed session (or the owner) can pick up the queue
without re-deriving state.

**Tension with `AGENT_WORKFLOW_GUIDE.md § Bounded cloud-implementation
priority` noted and resolved:** that subsection defaults every S-230/MVP0-P2P
code task to cloud, citing host memory saturation (~31/32 GB used at idle) as
its stated rationale. Direct measurement at session start
(`memory_pressure`, `vm_stat`, `ollama ps`) found **84% free / no models
loaded**, and later checks during heavy dual-model use (Qwen + Muse Glimmer
loaded together) still showed 74–84% free. The exception's own stated
premise does not hold on this host right now. Combined with the owner's
explicit override this session, local-first execution was used for every
Low-band leaf below. The exception is not being silently ignored — its
factual basis was checked and found absent at execution time, and this is
recorded rather than assumed.

## Closed this session (all RRI Low, full local pipeline, owner-delegated approval)

All four leaves below used the identical pattern: `scripts/rri.py` score →
phase-1 review (`muse-glimmer:30b-q4_K_M`, PASS required before delegation)
→ `scripts/delegate-low-rri.py --mode full-file` (`qwen3.8:27b-mlx`) →
independent verification (`cargo test`/`clippy`/`fmt`) → phase-2 review
(`muse-glimmer`) → ledger closure block → commit.

| Leaf | RRI | Commit | Notes |
|---|---:|---|---|
| `P2.T2a-i` | 16 Low | `40df270` | Crate bootstrap. First delegation attempt (`--mode full-file` on the **existing** root `Cargo.toml`) destructively regenerated it, dropping `resolver`, `edition`, license, all workspace deps, and the clippy lints block. Caught via `git diff`, reverted, one line applied directly. **Lesson generalized**: `full-file` is only safe for files that do not yet exist. |
| `P2.T2a-ii-1` | 12 Low | `e1e4dff` | `path.rs` — NFC normalize + validate, C0-frozen vectors. First attempt correct logic, one stray trailing `--- CONTENT ---` line (see below). |
| `P2.T2a-ii-2a` | 14 Low | `73dfb65` | `manifest.rs` — canonical JSON + SHA-256, matched fixture on first attempt, no repair needed. |
| `P2.T2a-ii-2b` | 14 Low | `5c03303` | `aad.rs` — AAD canonical JSON, reuses `manifest_sha256`. Same stray-marker defect as T2a-ii-1, fixed via the now-known pattern with no repair-delegation round-trip. |

All four have complete closure records (Reflection-equivalent Low-band
review evidence, behavioral coverage tables, owner-verification blocks) in
`docs/tasks/mvp0-p2p-p2-encrypted-publication.md`.

**Recurring defect class identified and now documented in memory**
(`feedback_full_file_appends_content_marker_echo`): Qwen in `--mode
full-file` intermittently echoes the wrapper's own tagged-block
`CONTENT_MARKER` literal (`--- CONTENT ---`, `scripts/delegate-low-rri.py:73`)
as a spurious trailing line after otherwise-correct file content. Confirmed
twice this session, both times pure formatting noise with zero logic impact.
Fix is a one-line scripted, asserted byte removal — do not spend a
repair-delegation round-trip on it if the file exceeds the 40-line
before-after anchor cap (it will predictably fail the same guard both times).

## P2.T2 remaining leaves — why each is not in the autonomous queue

| Leaf | RRI | Verdict | Reason |
|---|---:|---|---|
| `T2b` | 33 Moderate | **Owner decision needed, not queued** | Requires resolving how the leaf reads the S-120 prepared-HLS package through `StorageAdapter` (canonical key layout, ADR-006) — this is unresolved design, not a complexity/mechanical gap. Freezing that interface myself would be inventing an architecture decision the workflow guide explicitly reserves for a human checkpoint. Does not block `T2c`/`T2d` (both depend only on `T2a`, already done). |
| `T2c` | 41 Med-high | **Hard-excluded from local, cloud/HITL required** | AES-256-GCM + nonce-invariant implementation. Crypto/key-material logic is an ADR-038 §6 hard exclusion from `GO_LOCAL` regardless of Muse Glimmer's advisory recommendation. Requires the full Med-high approval card + explicit owner sign-off before any implementation route (local or cloud). |
| `T2d` | 41 Med-high | **Hard-excluded from local, cloud/HITL required** | Versioned KEK wrap/unwrap + zeroization — same crypto/key-custody hard exclusion as `T2c`. Depends on `T2c`. |
| `T2e`–`T2g` | not yet scored | **Blocked transitively** | All depend on `T2c`/`T2d` (crypto) or `T2b` (storage decision). |

**Net effect:** `P2.T2`'s manifest/path/digest/AAD contract sub-branch is
now **fully closed** (`T2a-i`, `T2a-ii-1`, `T2a-ii-2a`, `T2a-ii-2b` — 4/4).
The remaining `T2b` (design decision) and `T2c`–`T2g` (crypto chain) cannot
proceed without the owner. This is not a token-budget shortcut — it is the
policy's own hard exclusion plus a genuine unresolved design boundary.

## Candidates evaluated for the next autonomous leaf (not yet executed)

Scored but not started, in order of consideration, all inside `P2.T4`
(O4 dispatcher/reconciler — chosen because `T4a` explicitly declares "pure
recovery decision kernel", i.e. no IO, no crypto, over frozen D3/O4
semantics already in the C0 freeze and this file's own `HP-T4-1`/`HP-T4-2`/
`EC-T4-1`/`EC-T4-2`):

| Leaf | RRI (preliminary) | Note |
|---|---:|---|
| `P2.T4a` | 27 Moderate | Pure state-transition kernel (claim/lease/timeout → retry/wait/mark-failed decision), no IO. Above Low; a further split would fragment a single decision-table invariant the same way the ledger already forbids for crypto — judged not honestly splittable further. Candidate for the Moderate local-first route (`run_local_task.py`, nemotron) if execution resumes autonomously, since the host is not memory-constrained right now. |
| `P2.T3a` | 30 Moderate | Node/TS Availability Node bootstrap. Same Moderate local-first candidacy; different toolchain (no Rust `cargo` verification loop — would need `npm run build`/`test` equivalents established first in `apps/availability-node`, which does not exist yet). |
| `P2.T6a` | 48 Med-high | Migration extending `audit_events` for P2 correlation. **Not a queue candidate** — schema/migration changes are an independent ADR-038 §6 hard exclusion (schema/migrations/release cuts), separately from the D/K score. |

Neither `T4a` nor `T3a` was started this pass — pausing here to deliver this
analysis was judged higher-value than opening a fifth ad-hoc leaf on a new
script path (`run_local_task.py`, unused this session) without checkpointing
state first, given the owner explicitly asked for a written plan of the
absence-window queue before further execution.

## Recommended queue for the remaining ~7h absence window

1. **`P2.T4a`** (RRI 27 Moderate) — attempt via `run_local_task.py`
   (Moderate local-first default), 2 evidence-backed repair attempts per
   policy, Gemma/Muse Glimmer phase-1+phase-2 review (26–55 chain: Gemma
   primary, Muse Glimmer fallback). On 2/2 exhaustion, decompose into
   Low-band subtasks per the standard post-repair-budget route before any
   cloud escalation — do not jump to cloud on first failure.
2. **`P2.T3a`** (RRI 30 Moderate) — same route, contingent on establishing
   a minimal `apps/availability-node/package.json`+build/test loop first
   (that scaffolding step itself may score Low and could be attempted
   before the full T3a leaf).
3. Re-score `T2e` once `T2d` unblocks (will not happen autonomously — flagged
   for owner attention, not queued).
4. **Do not attempt** `T2b`, `T2c`, `T2d`, `T2g`, `T6a`, or any other
   crypto/migration/schema leaf without the owner present, regardless of
   remaining time budget.

## Token/time notes for the resumed session

This session closed 4 leaves end-to-end (score → review → delegate →
verify → review → document → commit) in roughly 90 minutes of wall clock,
dominated by local-model latency (60–250s per Ollama call), not orchestrator
token spend. If continuing autonomously, prefer the same pattern: build a
fully decision-resolved packet (no ambiguity left for the model to invent),
one phase-1 review call, one delegation call, one phase-2 review call, then
move on — avoid re-exploring context already established in this document
or in the closed leaves' commit messages.
