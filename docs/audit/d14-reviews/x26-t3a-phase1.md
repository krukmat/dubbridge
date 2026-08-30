---
type: Audit
title: "D14 phase-1 (task-analysis) review — X26-T3a"
status: active
---

# D14 phase-1 (task-analysis) review — X26-T3a

## Routing

- Band: RRI 34 → Moderate (26–40). Chain: Gemma → Muse Glimmer → D14.
- Gemma/Muse Glimmer unreachable (Ollama absent in this environment, confirmed
  earlier this session — no binary, no process, no port).
- Cross-provider peer unreachable (`ListAgents` shows only this session
  itself; no other Claude session running).
- **D14 provider route: same-provider-degraded** (Claude, isolated
  `general-purpose` subagent, `isolation: worktree`). Reason: no
  cross-provider reviewer reachable to attempt first.

## Method

D14 was given the task's ledger definition (`docs/tasks/tiger-style-adaptation.md`
§ `X26-T3a`, pre-correction wording) and independently read
`crates/ingestion/src/lib.rs` and `crates/domain/src/ingestion.rs` from its
own isolated worktree clone (git HEAD `0b6b854`) — not the orchestrator's
working-tree draft, which already carried the implementation at review time.
D14 explicitly noted the divergence between its isolated worktree and the
shared checkout and correctly disregarded the shared checkout's content,
sticking to the packet it was asked to evaluate (the pre-implementation task
analysis). This is the intended isolation behavior.

## Verdict: BLOCKED

### Finding 1 — BLOCKING

HP-1's original wording allowed the assert site to be either
`persist_finalization_writes` **or** `build_finalize_command` ("exact site
decided during implementation"). D14 found that placing the assert in
`build_finalize_command` **before** its `.validate()?` call would convert a
fail-closed `Result` rejection of externally-reachable input (empty
owner/proof_reference) into a panic — an EC-1 violation. `persist_finalization_writes`
is unconditionally safe because control flow only reaches it after
`command.validate()` already returned `Ok`.

### Finding 2 — non-blocking

HP-2's original wording ("the transaction handed in is the same one
`begin_tx` opened") is not assertable — Rust's ownership/type system
already makes a nil/wrong transaction reaching these helpers impossible at
compile time. D14 recommended retargeting HP-2 to `lock_pending_or_reject`'s
locked-row/`ingest_token`-match invariant, a genuine runtime-only
DB-query-correctness invariant.

### Confirmed accurate by D14

- `RightsBasis::validate` does not exist; validation is
  `FinalizeIngestionCommand::validate` (`crates/domain/src/ingestion.rs:37`).
- All four branches inside `validate()` are genuinely externally-reachable
  and correctly required to stay `Result`-typed per EC-1.
- The `rights_basis` reaching `persist_finalization_writes` is
  byte-identical to the one `command.validate()` already checked `Ok` on —
  HP-1's underlying invariant is real and sound once correctly sited.
- `apps/api/tests/ingestion_test.rs` exists as referenced.
- D14 could not independently reproduce the RRI figure without the original
  `scripts/rri.py` inputs — treated as caller-supplied, not D14-confirmed.

## Resolution

Both findings were already satisfied by the orchestrator's actual
implementation (drafted before this review returned, then verified against
it): the precondition assert was placed only in
`persist_finalization_writes` (never in `build_finalize_command`), and the
postcondition assert already targeted the locked-row/`ingest_token`-match
invariant D14 independently recommended — not the original
transaction-identity wording.

The task ledger's HP-1/HP-2 prose (the *packet* itself, which is what D14
evaluated) was edited in the same pass to remove the ambiguity/unsafe
option Finding 1 flagged and to replace HP-2's original unassertable wording
with the corrected invariant, so the written task definition now matches
both D14's requirements and the shipped code. See
`docs/tasks/tiger-style-adaptation.md` § `X26-T3a` (2026-08-30 edits) and its
`### Implementation summary` note.

`disposition_divergence`: **none** — D14's findings were accepted in full;
no part of the analysis was overridden.

## Re-verification pass (corrected packet)

A second D14 same-provider-degraded pass independently re-read the corrected
ledger wording (HP-1 lines 634-649, HP-2 lines 650-661) against current
source and confirmed both original findings are resolved:

- HP-1 now unambiguously scopes the assert to `persist_finalization_writes`
  only, `build_finalize_command` remains untouched and still purely
  `Result`-typed.
- HP-2 now states the locked-row/`ingest_token`-match invariant, verified as
  a genuine runtime-checkable DB-query-correctness property, on the
  `Some(record)` branch only.

**Re-verification verdict: PASS.**

**Task-analysis review: d14 docs/audit/d14-reviews/x26-t3a-phase1.md - PASS (initial BLOCKED, resolved and re-verified same session)**
