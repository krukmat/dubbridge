---
type: Audit
title: "D14 phase-2 (code-solution) review — X26-T3a"
status: active
---

# D14 phase-2 (code-solution) review — X26-T3a

## Routing

Band: RRI 34 → Moderate (26–40). Chain: Gemma → Muse Glimmer → D14; Gemma/
Muse Glimmer unreachable (Ollama absent), no cross-provider peer reachable
(`ListAgents`). **D14 provider route: same-provider-degraded** (Claude,
isolated `general-purpose` subagent, `isolation: worktree`).

## Scope

Reviewed the diff at `crates/ingestion/src/lib.rs` (two `assert!`s added:
postcondition in `lock_pending_or_reject`, precondition in
`persist_finalization_writes`) against this worktree's actual source,
independently tracing the call graph rather than trusting the diff comments.

## Verdict: PASS

1. **Precondition placement (control flow):** confirmed sound —
   `finalize_ingestion_core` only reaches `persist_finalization_writes`
   after `build_finalize_command`'s internal `command.validate()` already
   returned `Ok` (early `?`-return on failure).
2. **Postcondition scope:** confirmed — assert sits only inside the
   `Some(record)` arm; the `None` branch's existing duplicate/not-found
   `Result` logic is untouched.
3. **No `Result`-branch deletion/weakening:** confirmed —
   `crates/domain/src/ingestion.rs::FinalizeIngestionCommand::validate` is
   unmodified; the diff's only hunks in `crates/ingestion/src/lib.rs` are
   pure insertions (the `Some(record) => Ok((tx, record))` one-liner became
   a block, not a logic change).
4. **Comments present:** confirmed on both asserts, each citing the
   invariant and X26-T3a/Tiger Style D1.
5. **Logical soundness (independent call-graph trace):**
   - Postcondition: `record` comes directly from `lock_for_finalize`'s own
     `ingest_token`-keyed `WHERE` clause — the assert can only fire on a
     genuine query/mapping defect, never on legitimate traffic.
   - Precondition: `rights_basis` passed into `persist_finalization_writes`
     is the same unmutated `pending.rights_basis` that `command.validate()`
     already checked `Ok` on inside `build_finalize_command`, cloned within
     the same synchronous span with no intervening mutation — sound.

No BLOCKING or MEDIUM findings. One disclosed non-blocking observation
(already recorded in the task ledger's own Open follow-up): the diff's
assert logic could not be exercised against a live-Postgres rollback path
in this environment; verified instead by independent static call-graph
trace above.

`disposition_divergence`: **none**.

**Code-solution review: d14 docs/audit/d14-reviews/x26-t3a-phase2.md - PASS**
