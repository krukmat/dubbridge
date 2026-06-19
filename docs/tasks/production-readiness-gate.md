---
type: TaskList
title: "Tasks: Production Readiness Gate"
status: closed
plan: docs/plan/production-readiness-gate.md
---
# Tasks: Production Readiness Gate

Governing plan: `docs/plan/production-readiness-gate.md`
Governing guides: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/RRI_POLICY.md`

## Status Legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

## Task PRG-T1 — Enforce production readiness via native tooling

**Status:** [x] Done (superseded the original regex-scanner design)
**Effort:** M
**Complexity:** Med-high

### Outcome

The standalone regex gate (`scripts/check-production-readiness.py`) and its test
were removed. Production readiness is now enforced by language-native AST tools
with no extra bespoke script (see the governing plan for the rationale):

- Rust panic/debug macros (`todo!`, `unimplemented!`, `dbg!`, `panic!`,
  `println!`, `eprintln!`) → Clippy restriction lints in `Cargo.toml`
  `[workspace.lints.clippy]`, with `clippy.toml` exempting test code. Enforced
  by the existing `make qa-lint` / CI clippy gate.
- New `unwrap()`/`expect()` in production Rust → diff ratchet
  (`check_runtime_risk_additions`) in `scripts/check-maintainability.py`.
- Mobile `any` / `console.log`/`debug` / `debugger` / `@ts-ignore` →
  `mobile/eslint.config.js` at error severity with `--max-warnings 0`, plus
  `tsc --noEmit` and Jest, via `make qa-mobile`.

### Wiring

- `make qa-mobile` added; `qa-production-readiness` removed.
- CI: `mobile` job (Node + `npm ci` + `make qa-mobile`) replaces the
  production-readiness job; Rust lints ride the existing `clippy` job.
- pre-push runs `make qa-mobile` when `mobile/node_modules` is present.

### Acceptance criteria

- [x] Clippy fails on newly added `todo!`/`unimplemented!`/`dbg!`/`panic!`/
      `println!`/`eprintln!` in production source; test code is exempt.
- [x] `make qa-mobile` fails on `any`/`console`/`debugger`/`@ts-ignore`; no
      warning tier (`--max-warnings 0`).
- [x] `check-maintainability.py` blocks newly added `unwrap()`/`expect()` in
      production Rust source while grandfathering existing call sites.
- [x] CI and pre-push run the mobile gate.
- [x] No `check-production-readiness*` files remain in the repo.

### Follow-up (not blocking)

- Expo's RC `react-hooks/set-state-in-effect` flags 6 intentional fetch-on-mount
  / prop-reset effects in `mobile/src/screens`. The focused gate does not enable
  it; revisit enabling the broader Expo ruleset once that rule stabilises.
