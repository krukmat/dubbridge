---
type: TaskList
title: "Tasks: RRI Calculator Script"
status: closed
plan: docs/plan/rri-calculator-script.md
---
# Tasks: RRI Calculator Script

**Plan:** `docs/plan/rri-calculator-script.md`
**Scope:** Create `scripts/rri.py` (+ tests), amend two docs to mandate its use, and
add a CI gate (T4) so the calculator can't silently regress.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
T0 → T1 → T2 → T3 → T4
```

---

## T0 — Create plan + task ledger

- **Status:** [x] Done — 2026-06-08
- **Effort:** S
- **RRI:** 5 (Low)
- **Depends on:** —
- **Outputs:** `docs/plan/rri-calculator-script.md`, `docs/tasks/rri-calculator-script.md`

---

## T1 — Write `scripts/rri.py` + `scripts/rri_test.py`

- **Status:** [x] Done — 2026-06-08
- **Effort:** M
- **RRI (to be recomputed by the script once it exists — manual estimate):** ~25 (Low/Moderate boundary)
  - C=2, F=1 (2 files), D=1, T=2, A=0, K=1, P=0, X=2
- **Thinking:** Off
- **Model:** Claude Sonnet 4.6
- **Depends on:** T0
- **Objective:** Implement the full RRI engine in Python: F measurement, anchor-rubric
  floor derivation, formula, penalty auto-detection, band/crosswalk resolution,
  decomposition-trigger detection, markdown + JSON output. Add unit tests with known
  vectors.
- **Inputs:**
  - `docs/policies/RRI_POLICY.md` — formula, weights, per-variable bands, anchor
    rubric, penalty table, bands crosswalk, reporting format
  - Interface + design from `docs/plan/rri-calculator-script.md`
- **Outputs:**
  - `scripts/rri.py` (executable)
  - `scripts/rri_test.py`
- **Acceptance criteria:**
  - Args: `--T --A --X` (required); C via `--cc <raw>` **or** `--C <0-5>` (mutually
    exclusive, at least one required); floor-enforced `--D --K --P`; path args
    `--touches` (repeatable) / `--base` / `--F`; penalty flags; `--low-confidence`;
    `--json`.
  - F: counts `--touches` paths if given, else `git diff --name-only <base>...HEAD`,
    else requires `--F`; maps count → 0–5 per the F band table.
  - C: if `--cc` given, maps raw CC → score via the policy table
    (1–5→0, 6–10→1, 11–20→2, 21–30→3, 31–50→4, 50+→5); else uses `--C`. Passing both
    `--cc` and `--C` → exit≠0.
  - Anchor rubric encoded; most-specific glob wins; D/K/P raised to floor when below;
    raise reported in evidence. Content-dependent rows emit an advisory note.
  - Formula weights exactly: 0.18·C + 0.12·F + 0.15·D + 0.15·T + 0.12·A + 0.12·K +
    0.10·P + 0.06·X, /5, ×100, rounded to nearest int.
  - Auto-penalties: `many_files` (F≥4), `complex_and_domain` (C≥4∧D≥3),
    `no_tests_high_impact` (T≥4∧P≥4), `auth_security` (rubric auth/secrets/audit match).
    Manual: `refactor_and_behavior`, `arch_decision`, `no_verification`, `auth_security`.
    All de-duped to one application each.
  - 7 bands resolved with label, Effort, Codex tier, Claude tier, thinking, gate.
  - Decomposition triggers detected: RRI>70, base>100, F≥4∧K≥3, C≥4∧D≥3,
    refactor_and_behavior active, T≥4∧P≥4.
  - Scores out of 0–5 → exit≠0 with a clear message; unknown penalty key → exit≠0
    listing valid keys.
  - Markdown output matches `RRI_POLICY.md` reporting format; `--json` emits all
    variables, base, penalties, final, band, Effort, tiers, thinking, gate, triggers.
  - `python3 scripts/rri_test.py` (or `-m unittest`) passes.
- **Happy paths:**
  - HP-1: `--touches a.rs b.rs --C 2 --T 3 --A 0 --X 2 --D 2 --K 2 --P 2` → F from 2
    paths, full markdown table, correct RRI + band.
  - HP-2: `--touches crates/auth/src/lib.rs --D 1 --P 1 --K 1 …` → D/P/K raised to 4,
    `auth_security` auto-applied, raises reported.
  - HP-3: F≥4 (≥11 touched paths) → `many_files` auto-applied (+8), shown in penalties.
  - HP-4: `--json` → valid JSON with every required key.
- **Edge cases:**
  - EC-1: `--C 6` → exit≠0, range message.
  - EC-2: `--penalty bogus` → exit≠0, lists valid keys.
  - EC-3: No `--touches`, git unavailable, no `--F` → exit≠0, clear message.
  - EC-4: Agent `--D 4` but rubric floor is 2 → D kept at 4 (floor only raises).
  - EC-5: `--cc 24` → C score 3; `--cc 50` → 4; `--cc 51` → 5 (boundary mapping).
  - EC-6: Both `--cc` and `--C` passed → exit≠0; neither passed → exit≠0.
- **Unit test vectors (rri_test.py):**
  - all-0 → 0; all-5 → 100; T1-vector C1F1D1T2A0K1P0X2 → 20.
  - CC mapping: `--cc 5`→0, `--cc 10`→1, `--cc 20`→2, `--cc 30`→3, `--cc 50`→4,
    `--cc 51`→5 (each boundary).
  - floor raise: `crates/auth` + `--D 1` → D=4.
  - auto-penalty: synthetic F=4 → many_files; C4∧D3 → complex_and_domain.
  - band boundaries: inputs landing on 25/26, 55/56, 70/71, 100/>100.
  - decomposition: C4∧D3 → trigger reported.
- **Completion record (2026-06-08):**
  - Created `scripts/rri.py` (stdlib-only) and `scripts/rri_test.py`, both executable.
  - 29 tests pass via `python3 scripts/rri_test.py` (CC/F mapping boundaries, base
    formula vectors all-0→0 / all-5→100 / T1→20, rubric floor-raise + specificity,
    4 auto + manual penalties with de-dup, 7 band boundaries, decomposition triggers,
    low-confidence bump, CLI exit codes EC-1/2/3/6).
  - E2E verified: T1 self-scores RRI 27 → Moderate → Effort M (matches the corrected
    estimate); `crates/auth` example raises D/K/P 1→4, auto-applies auth_security +
    no_tests_high_impact, lands band High with decomposition triggered.
  - Deviation: none. `--touches` is per-path repeatable (argparse `append`), as specified.

---

## T2 — Add "Script automation" section to `docs/policies/RRI_POLICY.md`

- **Status:** [x] Done — 2026-06-08
- **Effort:** S
- **RRI:** ~8 (Low)
- **Thinking:** Off · **Model:** Claude Sonnet 4.6
- **Depends on:** T1 (script interface must be final)
- **Objective:** Document `scripts/rri.py` as the canonical calculator: invocation,
  which variables the script decides vs. the agent supplies, `--touches` usage at
  task-presentation time, and the mandate to run it instead of computing by hand.
- **Inputs:** `docs/policies/RRI_POLICY.md`; finalized `scripts/rri.py`
- **Outputs:** Amended `docs/policies/RRI_POLICY.md`
- **Acceptance criteria:**
  - New "Script automation" section before "## Related".
  - Documents CLI usage, the script-vs-agent split table, an example invocation, and
    the floor/penalty auto-derivation behavior.
  - States: "Agents must run `python3 scripts/rri.py` instead of computing the
    formula, floors, or penalties manually."
  - `make qa-docs` passes.

---

## T3 — Mandate the script in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`

- **Status:** [x] Done — 2026-06-08
- **Effort:** S
- **RRI:** ~12 (Low)
- **Thinking:** Off · **Model:** Claude Sonnet 4.6
- **Depends on:** T2
- **Objective:** Add a paragraph to the RRI section (~lines 248–251) mandating the
  script call and copy-paste of its output into the task presentation.
- **Inputs:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (RRI section ~227–251);
  amended `RRI_POLICY.md`
- **Outputs:** Amended `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- **Acceptance criteria:**
  - Concrete call shown:
    `python3 scripts/rri.py --touches <paths> --C <n> --T <n> --A <n> --X <n> [--D --K --P] [--penalty …]`
  - Clarifies: F and D/P/K floors come from the script; C and T are measured by the
    agent first (clippy / llvm-cov) then passed in.
  - Existing RRI prose preserved (additive change).
  - `make qa-docs` passes.

---

## T4 — Add `qa-rri` Makefile target

- **Status:** [x] Done — 2026-06-08
- **Effort:** S
- **RRI:** ~10 (Low)
- **Thinking:** Off · **Model:** Claude Sonnet 4.6
- **Depends on:** T1 (tests must exist), T3 (doc mandate landed)
- **Objective:** Add a `qa-rri` target running `python3 scripts/rri_test.py` and wire
  it into `qa-ci` so the calculator can't silently regress.
- **Inputs:** `Makefile` (current targets); `scripts/rri_test.py` (T1 output)
- **Outputs:** Amended `Makefile`
- **Acceptance criteria:**
  - `qa-rri` target added; `make qa-rri` runs the tests and fails on a broken script.
  - `qa-ci` depends on `qa-rri`.
  - `qa-rri` added to the `.PHONY` line.
  - Scope decision honored: wired into `qa-ci`, **not** `qa-docs` (it is a tooling
    test, not a doc-consistency check).
  - `make qa-rri` passes against the T1 script.

---

## Agent handoff prompt (delegation-ready)

> Implement T1 → T4 from `docs/tasks/rri-calculator-script.md` in the
> `dubbridge` repo. T0 is complete. Governing docs:
> `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (highest authority),
> `docs/policies/RRI_POLICY.md` (source of truth: formula, anchor rubric, penalties,
> bands crosswalk, reporting format).
> T1: `scripts/rri.py` (Python, zero-dep stdlib only) — F from `--touches` or git
> diff; anchor-rubric D/P/K floor auto-raise; exact formula; 4 auto + 3 manual
> penalties (de-duped); 7-band crosswalk output; decomposition-trigger detection;
> markdown + `--json`. Plus `scripts/rri_test.py` with the known vectors listed in T1.
> T2: "Script automation" section in `RRI_POLICY.md`.
> T3: mandate the script in the RRI section of `AGENT_WORKFLOW_GUIDE.md`.
> Run `python3 scripts/rri_test.py` and `make qa-docs` before reporting done.
> Present each task for explicit approval before editing; mark progress in this ledger.
