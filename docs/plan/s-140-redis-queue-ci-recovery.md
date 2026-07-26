---
type: Plan
title: "Plan: S-140 Redis queue CI recovery"
status: done
---
# Plan: S-140 Redis queue CI recovery

> **Status:** Done - incident analyzed and recovered locally on 2026-07-26; remote rerun still pending outside this worktree.
> **Tasks ledger:** `docs/tasks/bug-s140-redis-queue-ci-recovery.md`
> **Source incident:** GitHub Actions run `30202703817` (`ci`, head SHA `a8e1281`)

## Objective

Restore `main` CI after the `S-140-T3c-i` Redis-backed queue landing by fixing
the maintainability regression in `crates/jobs/src/lib.rs` and resolving the
accompanying `cargo-deny` failure without weakening queue behavior or dependency
policy.

## Why this follow-up exists

The 2026-07-26 `ci` run for `a8e1281` failed in two jobs:

- `maintainability` (`89795325567`) failed immediately on the push range.
- `deny` (`89795325516`) failed in `Run dependency policy gate`.

Local triage reproduced the maintainability failure exactly with:

```bash
python3 scripts/check-maintainability.py --base 8ad36d4
```

That command reports:

- `crates/jobs/src/lib.rs: line repeated 9 times in added code; budget is 8: #[async_trait::async_trait]`

Local dependency-policy triage narrowed the `deny` failure to the same Redis
queue dependency delta introduced by `S-140-T3c-i`:

- `apalis-redis = 0.7.4`
- `redis = 0.32.7`
- `tokio-test = 0.4.5`
- `futures-timer = 3.0.4`

`cargo deny check bans licenses sources` passes locally, and offline advisory
inspection does not show an obvious advisory or yanked status for those newly
introduced versions. The exact failing `cargo deny check` reason still needs to
be captured in a writable, network-capable local run during implementation.

## Affected files

- `crates/jobs/src/lib.rs`
- `crates/jobs/Cargo.toml`
- `Cargo.toml`
- `Cargo.lock`
- `apps/api/Cargo.toml`
- `.github/workflows/ci.yml`
- `docs/daily/2026-07-26.md`

## Design decisions

1. Treat this as a semantics-preserving CI recovery task, not as new queue
   feature work.
2. Fix the maintainability violation by reducing repeated boilerplate or
   restructuring trait/impl boundaries; do not change queue namespaces,
   persistence contracts, or enqueue semantics just to satisfy the gate.
3. Resolve `cargo-deny` by making the dependency graph policy-compliant; do not
   weaken `deny.toml`, add advisory ignores, or bypass the gate.
4. Keep this bug outside the critical-path S-140 delivery tasks, but preserve
   traceability back to `S-140-T3c-i` because the incident was introduced there.

## Decomposition decision

The original combined recovery scope planned at `RRI 42 -> Med-high`, which
would force the ADR-038 architect-refined route as a single bundled change.
Review on 2026-07-26 shows the task can be split into three narrower slices
that fit the local-dev lane better:

1. **Maintainability-only fix**
   - Planning RRI: `33 -> Moderate`
   - Scope: `crates/jobs/src/lib.rs`
   - Route: local-first Moderate lane
2. **Dependency-policy (`deny`) triage + fix**
   - Planning RRI: `38 -> Moderate`
   - Scope: `Cargo.toml`, `Cargo.lock`, `crates/jobs/Cargo.toml`,
     `apps/api/Cargo.toml`
   - Route: local-first Moderate lane
3. **Verification + status sync**
   - Planning RRI: `7 -> Low`
   - Scope: daily / task / optional S-140 doc sync
   - Route: Low-band docs/status pass

This split keeps each coding slice inside the local-first Moderate band instead
of forcing a single Med-high package. Execution-time RRI still must be
recomputed before each slice is presented or delegated.

Because the owner asked to place part of `T3` on a local agent where
possible, the verification tail is further split operationally:

- `T3a` - a **conditional** Low-band micro-slice that may go to **Gemma**
  only if post-fix verification reveals an eligible simple code patch.
- `T3b` - the required docs/status sync pass, which stays with the
  **primary/orchestrator** because it is not an eligible Gemma delegation
  target under the repository workflow.

## Module dependencies

- `crates/jobs` owns the queue abstractions and the repeated
  `#[async_trait::async_trait]` additions that tripped the maintainability gate.
- Workspace manifests and `Cargo.lock` own the Redis queue dependency graph that
  the `deny` job evaluates.
- `.github/workflows/ci.yml` is evidence, not the primary fix target, unless the
  local repro proves the gate wiring itself is wrong.

## Execution route

The split changes the preferred execution route:

- `T1` and `T2` are planned as **Moderate** and fit the repo's local-first
  implementation path.
- `T3` remains a **Low** closure lane, but with mixed ownership:
  - `T3a` -> Gemma only if it is a true simple code patch and stays `RRI 0-25`
  - `T3b` -> primary/orchestrator direct for docs/status sync

For the coding slices, the repo workflow still requires:

- RRI recomputation at presentation time
- explicit acceptance criteria review before approval
- local-first Moderate execution as the default implementation route

If recomputation pushes either coding slice back to `41+`, it must be promoted
back into the Med-high route before implementation.

For the verification tail:

- if the candidate `T3a` work is docs-only or recomputes above `25`, do not
  delegate it to Gemma; keep it with the primary agent or re-present it under
  the correct band.
- `T3b` does not use Gemma delegation; it is the orchestrator-owned closure
  pass that records evidence and synchronizes status artifacts.
