# Tasks: Documentation Consistency Guardrails

Governing plan: `docs/plan/doc-consistency-guardrails.md`
Governing guides: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `AGENTS.md`
Related: `docs/adr/README.md`, `docs/audit/2026-05-31-roadmap-adr-architecture-consolidation.md`

## Status Legend
- [ ] Not started · [x] Done · [~] In progress · [!] Blocked

## Default model recommendation (per AGENTS.md)
- Codex: `GPT-5.2-Codex`
- Claude Code: `Claude Sonnet 4`

Build order: **G1 → G2 → G3 → G4**. G1 is doc-only (the contract). G2 is the script.
G3 wires enforcement once the script exists. G4 re-adds the index sync note last,
so it never references unbuilt tooling.

---

## Task G1 — Propagation contract in the guide

**Effort:** S · **Complexity:** Low · **Depends on:** nothing
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Convert the implicit "sync status docs" rule into an explicit ADR-change propagation
map + a Definition of Done, so an ADR change outside a task ledger still triggers
the right doc updates.

### Scope
- Add an "ADR change propagation" section to `AGENT_WORKFLOW_GUIDE.md` with the
  propagation table and DoD from the plan.
- Cross-link it from the per-task discipline list ("see ADR change propagation").

### Acceptance criteria
- The guide contains the four-row propagation table and the DoD checklist.
- The DoD names `make qa-docs` as the closing check (forward reference is acceptable
  here because G3 lands it before this slice is reported complete).
- No contradiction with `CLAUDE.md` / `AGENTS.md`.

### Files affected
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`

### Status: [x] DONE — 2026-05-31

Files affected:
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — new section "ADR change propagation":
  propagation table (6 rows), deletion rule, DoD checklist, what the contract
  does/does not guarantee; cross-link from per-task discipline bullet.

---

## Task G2 — `check-doc-consistency.sh` + `make qa-docs`

**Effort:** M · **Complexity:** Medium · **Depends on:** nothing (parallel to G1)
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
A deterministic check that fails on canonical-doc drift.

### Scope — five checks
1. **Index↔file status parity** — leading status token in each `docs/adr/ADR-*.md`
   `- **Status:**` line must equal the leading token in its `docs/adr/README.md`
   index row (parenthetical scope notes ignored).
2. **Index completeness** — every `docs/adr/ADR-*.md` file has an index row, and
   every index row points to an existing file. (Catches the *creation* case: a new
   ADR file with no index row; and the *deletion* case: an orphaned index row.)
3. **Dangling references in docs** — every `ADR-0\d\d` token in the canonical doc
   set (`docs/adr/`, `docs/architecture.md`, `docs/plan/`, `docs/tasks/`,
   `README.md`) resolves to an existing ADR file.
4. **Dangling references in code/migrations** — every `ADR-0\d\d` token in
   `crates/**/*.rs`, `apps/**/*.rs`, and `infra/migrations/**/*.sql` resolves to an
   existing ADR file. (This is the critical check for *deletion / renumbering*: the
   repo cites ADRs in source and SQL comments.)
5. **Superseded successor** — any `Superseded by ADR-YYY` requires `ADR-YYY*` to
   exist.
- Collect all violations; exit non-zero with a readable report. Add a `qa-docs`
  Makefile target invoking the script.

### Acceptance criteria
- Against the current tree (post index-fix), `make qa-docs` exits `0`.
- Synthetic drift, each validated manually then reverted (not committed):
  - flipping one index status → non-zero naming the ADR (modification/status case);
  - adding an ADR file with no index row → non-zero (creation case);
  - removing an ADR file still cited in a `.rs`/`.sql` comment → non-zero naming the
    citing file (deletion case);
  - a `Superseded by ADR-999` with no such file → non-zero.
- Script is `set -euo pipefail`, repo-root-relative, no external deps beyond
  grep/awk/sed/bash.

### Explicit non-goal
- The script does **not** verify that prose semantically matches an ADR whose
  *decision* changed (content drift). That is covered by the Layer 1 propagation
  contract + human review, per the plan's "what this contract does and does not
  guarantee".

### Files affected
- `scripts/check-doc-consistency.sh` (new)
- `Makefile` (`qa-docs` target)
- `docs/plan/stream-recording-ingest.md` (remove the dangling legacy ADR citation so the
  current tree satisfies the new guardrail)

### Status: [x] DONE — 2026-05-31

Files affected:
- `scripts/check-doc-consistency.sh` — lines 1-235: repo-root-relative bash checker
  with aggregated violations for status parity, index completeness, dangling refs
  in docs, dangling refs in Rust/SQL, and superseded-successor existence.
- `Makefile` — lines 1, 31-32: `.PHONY` update and new `qa-docs` target invoking the
  checker.
- `docs/plan/stream-recording-ingest.md` — lines 59-60: removed the stale legacy ADR
  citation so `make qa-docs` passes on the current tree.

Validation:
- `make qa-docs` exits `0` on the current tree.
- Synthetic drift cases validated manually and reverted:
  - index status flip on `ADR-006` → non-zero with status mismatch report;
  - synthetic unindexed ADR file → non-zero;
  - temporary removal of `ADR-021` → non-zero including
    `crates/domain/src/artifact.rs:15`;
  - `Superseded by ADR-999` in `ADR-024` → non-zero with missing successor report.

---

## Task G3 — Enforcement: pre-push + CI

**Effort:** M · **Complexity:** Medium · **Depends on:** G2
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Make the check blocking locally and in CI, mirroring the Rust QA gates.

### Scope
- `.githooks/pre-push`: add a `DOCS_CHANGED` detector (paths: `docs/adr/`,
  `docs/architecture.md`, `docs/plan/`, `docs/tasks/`, `README.md`,
  `docs/playbooks/`) that runs `make qa-docs` even when no Rust files changed.
- `.github/workflows/ci.yml`: add a blocking `qa-docs` step/job.
- `Makefile`: include `qa-docs` in the `qa-ci` aggregate.

### Acceptance criteria
- A docs-only change that introduces drift fails `pre-push` and CI.
- A docs-only change with no drift passes; a Rust-only change still runs the Rust
  gates unchanged (no regression to existing behavior).
- `qa-ci` runs `qa-docs`.

### Files affected
- `.githooks/pre-push`
- `.github/workflows/ci.yml`
- `Makefile`

### Status: [x] DONE — 2026-05-31

Files affected:
- `.githooks/pre-push` — lines 44-96: added `DOCS_CHANGED`, docs-path detection,
  docs-only `make qa-docs` execution, and preserved the existing Rust/deny path.
- `.github/workflows/ci.yml` — lines 9-16: added a blocking `qa-docs` job for
  push/pull_request runs.
- `Makefile` — line 34: added `qa-docs` to the `qa-ci` aggregate.

Validation:
- `bash -n .githooks/pre-push`
- `make qa-docs`
- Hook logic validated with mocked `git`/`make`/`cargo` scenarios:
  - no-impact diff → skips QA;
  - docs-only clean diff → runs `make qa-docs`;
  - docs-only drift diff → fails through `make qa-docs`;
  - Rust-only diff → runs `make qa-local` unchanged.

---

## Task G4 — Re-add the index sync note

**Effort:** S · **Complexity:** Low · **Depends on:** G1, G2
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Document the new guardrail at the point of use, now that it exists (avoiding the
dangling forward reference that was correctly reverted during planning).

### Scope
- Add a "Keeping this index in sync" note to `docs/adr/README.md` pointing to the
  guide's propagation contract and `make qa-docs`.

### Acceptance criteria
- The note references only artifacts that now exist (G1 section + G2 target).
- `make qa-docs` still passes after the edit.

### Files affected
- `docs/adr/README.md`

### Status: [x] DONE — 2026-05-31

Files affected:
- `docs/adr/README.md` — lines 16-18: added the "keep this index synchronized"
  note pointing to the workflow guide's ADR propagation contract and `make qa-docs`.

Validation:
- `make qa-docs`

---

## Agent handoff prompt (for delegation)

```
You are implementing Documentation Consistency Guardrails for DubBridge.

Repo: /Users/matiasleandrokruk/Documents/dubbridge
Plan: docs/plan/doc-consistency-guardrails.md
Tasks: docs/tasks/doc-consistency-guardrails.md

Work one approved task at a time in order: G1 -> G4. After each task:
1. For G2/G3, run `make qa-docs` (and confirm Rust gates still behave for G3).
2. Mark the task [x] and record files/lines affected.
3. Report a summary and WAIT for approval before the next task.

Hard invariants:
- The check is consistency-only: no spell/style/markdown-lint scope.
- Status parity compares the LEADING status token; parenthetical scope notes are
  allowed in the index and must not cause false failures.
- The dangling-reference scan covers BOTH docs AND code/migration comments
  (`.rs`, `.sql`) — this is what makes deletion/renumbering safe.
- Accepted ADRs are superseded/deprecated, never deleted (Layer 1 rule + global
  "ask before deleting"); the script enforces reference integrity, not intent.
- Content/decision drift is a documented non-goal (semantic, not machine-checkable).
- Do not auto-fix drift; report and fail. Humans/agents fix in the same edit.
- Mirror local pre-push and CI gates (guide rule). Do NOT weaken existing Rust gates.
- Do NOT commit if any test/gate is broken. User-facing comms in Spanish; docs/code
  in English.
```
