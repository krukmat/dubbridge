---
type: ADR
title: "ADR-036: Local-first agentic implementation band (RRI 26–40) and Apple Silicon local model stack"
status: Accepted
supersedes: ""
superseded_by: ""
---

# ADR-036: Local-first agentic implementation band (RRI 26–40) and Apple Silicon local model stack

- **Status:** Accepted
- **Date:** 2026-07-12
- **Deciders:** DubBridge platform team
- **Scope:** agent workflow / local delegation process (documentation-and-process
  category, same as ADR-033/ADR-034); no runtime or crate boundary changes
- **Closes:** the "Moderate-band (RRI 26–40) implementation always spends cloud
  turns even when the task is verifiable and low-ambiguity, and the local
  hardware (Apple M5, 32 GB) is underutilized as an implementation surface" gap

## Context

DubBridge routes work by RRI band. Today the local model surface is limited to:

- **Gemma Developer** (`scripts/delegate-low-rri.py`) — packet-based patch
  delegation for eligible Low (0–25) simple code patches. The model has no
  filesystem or tool access; it emits tagged full-file or before/after blocks.
- **Gemma Reviewer** (`scripts/gemma-code-review.py`) — read-only advisory
  review for Low/Moderate (0–40) development tasks (ADR-034).

Everything in the Moderate band (26–40) is implemented by a cloud agent
(Claude Code or Codex), even when the task has deterministic acceptance
criteria (`HP-#`/`EC-#` cases, `unit-v1` contract) and a full local
verification pipeline (`make qa-local`, coverage gate). The cost asymmetry is
inverted: the band where verification is cheapest and ambiguity lowest is the
band where cloud tokens are spent on token-heavy implementation, while the
local hardware idles.

Two things changed in 2026 that make a local implementation band viable:

1. **Open MoE coding models with small active-parameter counts.** On a base
   M5 (153.6 GB/s memory bandwidth), token generation is bandwidth-bound:
   a dense 27B at Q4 (~16.8 GB weights) decodes at roughly 7–9 tok/s —
   loadable, but impractical for agentic loops. A MoE with ~3–4B active
   parameters reads ~2 GB per token and decodes in the interactive range.
   Qwen3.6-35B-A3B (35B total / 3B active, Apache 2.0, agentic-coding-tuned,
   released 2026-04) and Gemma 4 26B A4B (25.2B total / 3.8B active,
   multimodal, QAT checkpoints, released 2026-04) both fit this profile.
2. **Tool-calling maturity in open models plus local OpenAI-compatible
   serving** (Ollama/MLX) makes a filesystem-capable local agent feasible,
   replacing the blind packet protocol for larger tasks.

Key evidence (primary sources, retrieved 2026-07-12):

- Qwen3.6-27B model card and blog: SWE-bench Verified 77.2, Terminal-Bench
  59.3; ~16.8 GB at Q4_K_M (dense; decode-bound on base M5).
- Qwen3.6-35B-A3B blog: ~3.8 pts below the 27B on SWE-bench, ~11 pts below on
  Terminal-Bench, 3–5× faster generation; ~21 GB at Q4.
- Gemma 4 model overview (ai.google.dev): family E2B/E4B/12B/26B-A4B/31B;
  official QAT q4_0 checkpoints; 26B A4B is 128-expert top-8 MoE, 256K ctx,
  image+video input, native function calling.
- Apple M5 (apple.com newsroom / support specs): 153.6 GB/s bandwidth,
  per-GPU-core Neural Accelerators (~4× peak GPU compute vs M4 → materially
  faster prefill under MLX).
- DiffusionGemma: no MLX runtime path (required drafter module absent from
  mlx-lm); llama.cpp issue #24529 reports far-below-expected Apple Silicon
  throughput; published ~1000 tok/s figures are datacenter-GPU-specific.

This decision is recorded as an ADR because it changes the advisory/authority
contract that every agent consumes (which roles may write code, in which band,
under which enforcement boundary), defines a new execution surface with
security constraints, and is intended to be **portable to future projects** as
a policy layer with per-project bindings.

## Decision

### 1. Local model stack is role-based, three models, MoE-first

| Role | Model (dubbridge binding) | Rationale |
|---|---|---|
| **Local implementer** | `Qwen3.6-35B-A3B` (4-bit, 32K operational ctx) | Best open agentic-coding quality per active parameter; interactive decode on base M5 |
| **Local reviewer / challenger / multimodal** | `gemma4:26b-a4b-it-qat` | Cross-family independence from the implementer; only local multimodal (mobile UI screenshot review); already the repo fallback |
| **Fast lane** | `gemma4:12b-mlx` | Retained as-is: small-diff review and triage while the dev stack is under load |

Rejected, with re-entry conditions:

- **Qwen3.6-27B / Gemma 4 31B (large dense):** bandwidth-bound (~7–10 tok/s)
  on base M5; not practical as agentic implementers. Re-entry: higher-bandwidth
  hardware (M5 Pro/Max class) or an offline batch-analysis role.
- **DiffusionGemma:** no viable Apple Silicon runtime; speedup premise
  (arithmetic-intensity exploitation) does not hold on unified memory.
  Re-entry: official mlx-lm support **and** an infilling benchmark on Apple
  Silicon beating the primary implementer.

Advertised context windows (262K/256K) are **not** treated as usable; the
operational context is 32K until the pilot measures KV-cache cost on the real
machine (Qwen3.6's hybrid Gated-DeltaNet attention reduces KV size, but the
exact footprint is unverified).

### 2. RRI 26–40 gains a local agentic implementation path

The Moderate band keeps every existing gate and changes only who types the
code:

- **HITL approval (RRI 26+) is unchanged** — the human approves the task
  before any implementation, local or cloud (`HITL_AUTONOMY_POLICY.md`).
- **The orchestrator of record remains the primary agent** (cloud). It plans,
  authors the delegation contract, reviews, reflects, and closes. The local
  implementer produces the token-heavy artifact: the diff.
- **Gemma Reviewer (ADR-034) is unchanged** as the phase-2 reviewer for 0–40,
  subject to the pairing rule in §5.
- **Reflection passes (2 for Moderate) are owned by the orchestrator** and
  applied to the delegated diff, extending the existing Low-band rule
  ("the delegating agent applies the Reflection cycle to Gemma's output").
- **Repair budget: at most 2 local repair attempts**, each requiring new
  evidence (different failing output, not a blind retry). Then escalate.
- The Low band (0–25) keeps the existing packet protocol; migrating it to the
  agentic runner is a possible follow-up, not part of this decision.

### 3. Execution boundary for the local implementer (fail-closed)

The local agent runs inside an enforcement boundary, not on trust:

- **Isolated git worktree** per task; allowed-path set declared up front;
  out-of-scope writes are rejected by the wrapper exactly as the packet
  protocol does today.
- **No push, no merge, no deploy:** the agent process runs without push
  credentials; the worktree remote is not writable from the agent environment.
- **Environment stripped:** only `DUBBRIDGE_ENV=local` bindings; never the
  production descriptor (ADR-026 alignment); no secrets in the agent's env.
- **Network limited to the local model endpoint** (`OLLAMA_HOST`); the
  harness gets no other egress.
- **Destructive commands denied** (recursive delete, `git push`, migration
  apply against non-local databases) via wrapper command policy.
- **Test execution is permitted and expected** (`cargo test`, targeted
  `make` gates) inside the worktree.

### 4. Test-first delegation contract

The local implementer must never grade its own homework:

- The orchestrator authors (or explicitly approves) the failing `HP-#`/`EC-#`
  unit tests **before** delegation. The delegated stop condition is
  deterministic: the pre-agreed tests plus the standard gates pass.
- This generalizes the existing async-test lesson in
  `LOW_RRI_LOCAL_MODEL_HANDOFF.md` (pre-design the test, don't let the small
  model invent control flow) from packets to agentic sessions.
- Tasks without a deterministically verifiable acceptance signal are not
  eligible for local implementation, regardless of RRI.

### 5. Reviewer-pairing rule (cross-family independence)

Implementer and phase-2 reviewer must come from **different model families**:

| Implementer | Phase-2 reviewer |
|---|---|
| Cloud primary agent (status quo) | Gemma Reviewer (unchanged) |
| Qwen3.6-35B-A3B (this ADR) | Gemma Reviewer (unchanged) |
| Gemma 4 26B A4B (memory contingency, §6) | Qwen review pass, else D14 |

D14 (context-isolated subagent, ADR-034) remains the universal fallback. This
rule is what makes the memory contingency safe: if Gemma becomes the
implementer, Gemma cannot also be the reviewer.

### 6. Memory-residency rule (32 GB constraint)

- **At most one large model resident at a time.** Sequence:
  implement → unload (`keep_alive 0`) → run heavy verification → reload only
  if a repair attempt is needed. Only the 12B fast lane may stay resident.
- macOS caps GPU-wired memory below total RAM (default ≈ 65–75% on 32 GB
  ≈ 21–24 GB). Qwen3.6-35B-A3B at ~20–22 GB weights + KV sits at that limit.
  **Contingency:** if the pilot shows wired-limit pressure or swap under the
  full dev stack (Docker Postgres/Redis/MinIO + cargo + IDE), the primary
  implementer binding drops to `gemma4:26b-a4b-it-qat` (~14–16 GB) and the
  pairing rule in §5 flips the reviewer. This is a binding change, not a
  policy change.
- The reviewability budget derivation (`scripts/check-review-budget.py`)
  becomes **per-role**: the implementer's budget derives from its own
  ctx/predict envelope, not from the reviewer's.

### 7. Escalation packet contract (no context re-purchase)

Escalation to cloud (after 2 failed local repairs, or on boundary violation)
ships a packet extending the ADR-034 audit record: task spec + RRI table,
plan, allowed paths, full diff, commands executed with output, test results,
and a summary of each failed attempt. **The cloud agent starts from the
packet and does not re-explore the repository from scratch** — repeated
repository re-ingestion is the dominant avoidable cloud cost driver.

### 8. Operating modes (cost policy)

| | Economy | Balanced (default) | High-confidence |
|---|---|---|---|
| Local implements | RRI 0–40 | RRI 0–40 | RRI 0–25 |
| Cloud implements | only after 2 local failures | RRI 41+ | RRI 26+ |
| Review | deterministic gates + Gemma | band routing (status quo) | Claude **and** Codex in parallel |

High-confidence is **forced**, regardless of mode, for: security/auth
surfaces, rights-gate and consent invariants (ADR-008/ADR-028/ADR-030),
schema migrations, release cuts, and ADR-level decisions. Review duplication
(two reviewers on one change) is permitted only in high-confidence mode.

### 9. Policy/binding split (portability to future projects)

Everything above the line is **project-agnostic policy**: role definitions
(implementer / reviewer / fast lane), band routing, repair and escalation
budgets, execution boundary, test-first contract, pairing rule, residency
rule, operating modes. **Bindings** are per-project configuration: concrete
model IDs, endpoint, context/predict envelopes, allowed-path conventions,
verification commands. In dubbridge the bindings live in the existing
`DUBBRIDGE_*` / `OLLAMA_HOST` environment surface; a future project adopts
the policy by supplying its own binding set, not by editing the policy.

### 10. Staged adoption with promotion gates and rollback triggers

Adoption is a two-stage pilot; this ADR does **not** flip the band by itself.

**Stage 1 — measure (no policy change):** install the stack; measure on the
real machine: decode tok/s, prefill tok/s at 8K/16K/32K, peak memory with and
without the dev stack, load/unload cycle cost, 1-hour thermal soak; run a
15–20-task benchmark drawn from real repo history (Rust API tasks, mobile
Jest/RN tasks, a CI failure, docs task) through the agentic runner.

**Stage 2 — pilot:** 5 real RRI 26–40 tasks with normal HITL approval.

**Promotion gate (band flips to local-first only if all hold):**
≥ 75% task success without escalation; ≤ 2 repair attempts average; zero
scope violations; zero boundary violations (push/secret/path); end-to-end
wall-clock ≤ 2× the cloud equivalent; measured cloud-token reduction vs the
recorded baseline.

**Rollback triggers (after adoption):** escalation rate > 40% over a rolling
20-task window, any boundary violation, or sustained swap/thermal degradation
→ the band reverts to cloud implementation; the stack remains for review
roles. Reverting is a one-line mode/binding change.

## Consequences

### Positive

- Cloud spend in the Moderate band shifts from implementation (token-heavy)
  to orchestration and review (token-light); the escalation packet removes
  repeated repo re-ingestion cost in all bands.
- The 32 GB M5 becomes a productive implementation surface instead of a
  review-only surface.
- Cross-family implementer/reviewer pairing strengthens review independence
  over today's single-family (Gemma reviews Gemma-adjacent) arrangement.
- Gemma 4 26B A4B adds the first local **multimodal** review capability
  (Maestro screenshot analysis for `mobile/`) as an optional follow-up.
- The policy/binding split makes the workflow reusable across future projects
  without re-deriving the routing, budgets, or boundary rules.
- HITL, RRI gating, ADR-034 audit/review contracts, and the 41+ cloud band
  are unchanged — the blast radius is confined to who implements 26–40.

### Negative / cost

- End-to-end latency: local decode plus model load/unload cycles per repair
  iteration (~15–20 GB reload from SSD each time) can make local tasks slower
  than cloud even when they succeed; the ≤2× wall-clock gate exists for this.
- The 35B-A3B binding operates at the Metal wired-memory ceiling on 32 GB;
  the contingency demotion to Gemma 26B A4B costs implementation quality.
- A new execution surface (agentic runner + boundary enforcement) must be
  built, tested, and audited — it is more wrapper code than the packet
  protocol it complements.
- Two large local models to version, quantize, and manage instead of one
  family; benchmark and pilot consume roughly two weeks of calendar effort.
- Prefill throughput on repeated large prompts is the unproven variable on
  base M5; if KV reuse across agent turns is poor, agent loops may be
  prefill-bound regardless of decode speed.

### Neutral

- The Low band (0–25) packet protocol continues unchanged.
- The stack recommendation is hardware-specific (base M5, 32 GB); different
  hardware re-runs Stage 1, not the policy discussion.
- Model choices are bindings with re-entry conditions, so newer models change
  configuration, not this ADR.

## Open questions the pilot must answer

1. Does Qwen3.6-35B-A3B fit under the default Metal wired limit with 32K ctx
   while Docker + cargo run, or does the contingency binding fire?
2. Prefill tok/s at 8K/16K/32K on base M5 under MLX, and whether Ollama/MLX
   KV reuse across agent turns is effective in practice.
3. Actual KV-cache footprint of Qwen3.6's hybrid linear attention at 32K.
4. Harness choice: thin bespoke wrapper (extending the `delegate-low-rri.py`
   lineage) vs an existing OpenAI-compatible coding harness. Whichever is
   piloted must write ADR-034 audit records and enforce §3; a harness that
   cannot is disqualified regardless of convenience.
5. Availability and quality of the `qwen3.6:35b-a3b` MLX 4-bit build in the
   Ollama library (assumed from the 27B listing; unverified).

## Alternatives considered

- **Status quo (26–40 stays cloud-implemented):** rejected as the default —
  it prices verifiable, low-ambiguity work at cloud implementation rates and
  leaves local hardware as a review-only surface. It remains the automatic
  rollback state.
- **Qwen3.6-27B dense as primary local implementer** (original hypothesis):
  rejected — bandwidth-bound decode (~7–9 tok/s) on base M5 makes agentic
  loops impractical; its quality edge (SWE-bench +3.8, Terminal-Bench +11) is
  reachable via escalation instead.
- **DiffusionGemma as an infilling/boilerplate specialist:** rejected — no
  MLX runtime path, poor llama.cpp Apple Silicon throughput, unproven tool
  use; its published speedups are datacenter-GPU-specific. Re-entry
  conditions recorded in §1.
- **Single-family local stack (Qwen-only or Gemma-only):** rejected — breaks
  implementer/reviewer independence (§5) and loses either agentic-coding
  quality (Gemma-only) or multimodal review (Qwen-only).
- **Extending local implementation to RRI 41+:** rejected — the Med-high band
  exists precisely because ambiguity, blast radius, or verification burden is
  high; those are the properties that make local small-model implementation
  unsafe. Escalation pressure, not band creep, is the correct response.
- **Off-the-shelf agentic harness without audit/boundary integration:**
  rejected — speed of adoption does not compensate for losing the ADR-034
  audit trail and the §3 enforcement boundary; both are non-negotiable
  acceptance requirements for whichever harness is chosen.
- **Treating advertised 262K/256K contexts as usable working context:**
  rejected — KV-cache memory at those lengths exceeds the machine's headroom;
  32K operational context with escalation for genuinely long-context work.
- **No ADR, guide-only change** (the RRI-adoption precedent): rejected — this
  decision defines a new execution surface with a security boundary, changes
  which roles may write code per band, and is explicitly intended for reuse
  in future projects; ADR-033/ADR-034 set the precedent for indexing durable
  process contracts.

## Related

- `docs/plan/adr036-local-first-pilot.md` — Stage 1/Stage 2 pilot plan for §10
- `docs/tasks/adr036-local-first-pilot.md` — granular, executor-tier-routed task ledger
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — band routing, Reflection, review gates
- `docs/policies/HITL_AUTONOMY_POLICY.md` — approval gates (unchanged by this ADR)
- `docs/policies/RRI_POLICY.md` — RRI formula and bands
- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md` — packet protocol (retained for 0–25)
- `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md` — audit log and reviewer contract this ADR extends
- `docs/adr/ADR-026-layered-fail-closed-configuration-and-environment-separation.md` — environment separation the execution boundary aligns with
- `docs/evaluations/large-file-delegation-2026-06-21.md` — failure evidence motivating the test-first and boundary rules
