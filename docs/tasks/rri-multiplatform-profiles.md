# Tasks: RRI Multi-Platform Profiles

**Plan:** `docs/plan/rri-multiplatform-profiles.md`
**Scope:** Refactor `scripts/rri.py` to make C measurement and the anchor rubric
platform-selectable via a `PlatformProfile` (Strategy + Registry). No change to the
formula, weights, penalties, or bands.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
T0 → T1 → {T2, T3} → T4 → {T5, T6}
```

---

## T0 — Create plan + task ledger

- **Status:** [x] Done — 2026-06-09
- **Effort:** S
- **Complexity:** Low
- **Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4.6`
- **RRI:** ~8 (Low) — base: D0 F1(0.12)+X2(0.06) scaled → ~8; penalties: 0
- **Depends on:** — (first task)
- **Objective:** Create `docs/plan/rri-multiplatform-profiles.md` and this ledger as
  the crash-safe progress record for the multi-platform refactor.
- **Context:** Continuation of `rri-calculator-script`. The plan/ledger pair must
  exist before any code edit so progress is recoverable and each task is approvable.
- **Related documents:** `docs/plan/rri-calculator-script.md` (format reference),
  `docs/policies/RRI_POLICY.md`, `AGENTS.md` (task contract).
- **Inputs:** approved plan (conversation), style of existing plan/task files.
- **Outputs:**
  - `docs/plan/rri-multiplatform-profiles.md`
  - `docs/tasks/rri-multiplatform-profiles.md` (this file)
- **Acceptance criteria:**
  - Both files exist and reflect T0–T6 with dependencies, effort, RRI, and
    acceptance criteria.
  - Plan records all closed decisions (scope, selection, rubrics, platforms,
    backward compatibility, no external config).
  - `make qa-docs` passes.
- **Execution summary (2026-06-09):**
  - Created plan + ledger; closed-decisions table and Mermaid dependency graph included.

---

## T1 — `PlatformProfile`/`RubricRow` + registry + `detect_platform`

- **Status:** [x] Done — 2026-06-09
- **Effort:** M
- **Complexity:** Medium
- **Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4.6`
- **RRI:** to be computed via `scripts/rri.py` at presentation (est. Low-Moderate)
- **Depends on:** T0
- **Objective:** Introduce the `PlatformProfile` and `RubricRow` dataclasses, the
  `PROFILES` registry, and `detect_platform(start_dir=".")`.
- **Context:** The structural backbone of the refactor. T2 (measurers) and T3
  (rubrics) plug into the profile fields this task defines.
- **Related documents:** `docs/plan/rri-multiplatform-profiles.md` § "The pattern".
- **Inputs:** current `scripts/rri.py` (`RUBRIC` tuples, imports).
- **Outputs:** amended `scripts/rri.py` with dataclasses, registry skeleton (measurer
  + rubric fields filled by T2/T3), and `detect_platform`.
- **Acceptance criteria:**
  - `PlatformProfile` (name, markers, source_suffixes, measure_cc, rubric) and
    `RubricRow` (glob, d, p, k, adr, label) defined.
  - `detect_platform` walks up from `start_dir`, returns the first profile whose
    marker is found; dubbridge marker (`docs/policies/RRI_POLICY.md`) checked before
    generic rust (`Cargo.toml`); no match → `generic` profile.
  - `PROFILES` registry keyed by name with rust/python/go/rn/dubbridge/generic.
  - Module imports cleanly; existing tests unaffected.
- **Pseudocode:**
  ```python
  def detect_platform(start_dir="."):
      d = Path(start_dir).resolve()
      for cur in [d, *d.parents]:
          for prof in DETECTION_ORDER:          # dubbridge before rust
              if any((cur / m).exists() for m in prof.markers):
                  return prof
      return PROFILES["generic"]
  ```

---

## T2 — Per-platform C measurers (clippy, gocyclo, eslint; radon refactor)

- **Status:** [x] Done — 2026-06-09
- **Effort:** M
- **Complexity:** Medium
- **Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4.6`
- **RRI:** to be computed via `scripts/rri.py` (est. Moderate)
- **Depends on:** T1
- **Objective:** Implement `measure_cc_clippy`, `measure_cc_gocyclo`,
  `measure_cc_eslint`, refactor `measure_cc_radon`, all with signature
  `(paths) -> (raw_cc | None, evidence)`; add `_filter_existing(paths, suffixes)`.
- **Context:** The Strategy implementations. Each must degrade to `(None, reason)`
  when its tool is absent so the script never hard-fails for a missing toolchain.
- **Related documents:** `RRI_POLICY.md § Measuring C` (per-language commands).
- **Inputs:** current `measure_cc_radon` (parser to keep), `cc_to_score`.
- **Outputs:** four measurer functions + `_filter_existing`; wired into the registry.
- **Acceptance criteria:**
  - Each measurer filters by its `source_suffixes` and on-disk existence via
    `_filter_existing`.
  - Each returns `(None, reason)` on `FileNotFoundError` (tool absent) or no files.
  - clippy uses `--message-format=json`; gocyclo parses the leading number; eslint
    uses `complexity` rule + `--format json`; radon parser unchanged.
  - Max CC across functions is returned (policy: highest CC wins).
- **Pseudocode:**
  ```python
  def _filter_existing(paths, suffixes):
      return [p for p in paths
              if Path(p).suffix in suffixes and Path(p).exists()]
  ```

---

## T3 — Generic + DubBridge rubrics; parametrize `match_rubric`

- **Status:** [x] Done — 2026-06-09
- **Effort:** M
- **Complexity:** Medium
- **Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4.6`
- **RRI:** to be computed via `scripts/rri.py` (est. Moderate)
- **Depends on:** T1
- **Objective:** Define `_GENERIC_RUBRIC` (cross-language convention) and
  `_DUBBRIDGE_RUBRIC` (current ADR rows); change `match_rubric` / `first_matching_row`
  to take the active profile's rubric as a parameter.
- **Context:** Decouples the floor logic from the single global `RUBRIC`. The
  raise-never-lower rule, `matched_auth`, and advisories stay byte-for-byte identical.
- **Related documents:** `RRI_POLICY.md § DubBridge anchor rubric` (rows to preserve).
- **Inputs:** current `RUBRIC` + `match_rubric` + `first_matching_row`.
- **Outputs:** two rubric constants; parametrized matchers.
- **Acceptance criteria:**
  - `_DUBBRIDGE_RUBRIC` reproduces every current row (path, D, P, K, ADR, label).
  - `_GENERIC_RUBRIC` raises floors for `**/auth/**`, `**/security/**`,
    `**/migrations/**`, `**/crypto/**`; zeroes `docs/**`, `**/test*/**`.
  - `match_rubric(paths, rubric)` and `first_matching_row(path, rubric)` accept the
    rubric; floor/advisory/matched_auth logic unchanged.

---

## T4 — `evaluate(profile=…)` wiring + CLI `--platform` + render

- **Status:** [x] Done — 2026-06-09
- **Effort:** M
- **Complexity:** Medium
- **Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4.6`
- **RRI:** to be computed via `scripts/rri.py` (est. Moderate)
- **Depends on:** T2, T3
- **Objective:** Thread the active profile through `evaluate`; add `--platform`
  (default `auto`); add a Platform line to markdown and a `platform` field to JSON.
- **Context:** The integration seam. Must stay backward-compatible: `profile=None`
  auto-detects, resolving to dubbridge here so existing tests do not regress.
- **Related documents:** `docs/plan/rri-multiplatform-profiles.md` § T4.
- **Inputs:** `evaluate`, `build_parser`, `render_markdown`, `render_json`.
- **Outputs:** amended `evaluate`/CLI/renderers.
- **Acceptance criteria:**
  - `evaluate(..., profile=None)` auto-detects; `auto_cc` calls
    `profile.measure_cc`; rubric matching uses `profile.rubric`.
  - `--platform {rust,python,go,rn,dubbridge,auto}` resolves to a registry profile;
    `auto` triggers detection.
  - Markdown shows `**Platform:** <name> (...)`; JSON has a `platform` key.
  - Existing `evaluate(...)` callers without `profile` behave identically.

---

## T5 — Tests + regression

- **Status:** [x] Done — 2026-06-09 (61 tests green; 37 original + 24 new)
- **Effort:** M
- **Complexity:** Medium
- **Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4.6`
- **RRI:** to be computed via `scripts/rri.py` (est. Moderate)
- **Depends on:** T4
- **Objective:** Add unit tests for detection, measurer graceful-skip, generic
  rubric, and `--platform` override; confirm the 37 existing tests stay green.
- **Context:** The verification gate. Tests must not require clippy/gocyclo/eslint to
  be installed (they assert the `(None, reason)` skip path).
- **Related documents:** current `scripts/rri_test.py`.
- **Inputs:** T1–T4 code.
- **Outputs:** amended `scripts/rri_test.py`.
- **Acceptance criteria:**
  - `detect_platform` returns the right profile for temp fixtures with
    `Cargo.toml` / `go.mod` / `package.json` / `pyproject.toml`.
  - Each `measure_cc_*` returns `(None, reason)` when its tool is absent.
  - `--platform rust` forces the profile regardless of cwd.
  - Generic rubric `**/auth/**` → high floors; dubbridge rubric keeps ADR floors.
  - `python3 scripts/rri_test.py` — all tests (37 existing + new) green.
  - `make qa-rri` passes.

---

## T6 — Document "Platform profiles" in `RRI_POLICY.md`

- **Status:** [x] Done — 2026-06-09
- **Effort:** S
- **Complexity:** Low
- **Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4.6`
- **RRI:** to be computed via `scripts/rri.py` (est. Low)
- **Depends on:** T4
- **Objective:** Add a "Platform profiles" section: platform→measurer→marker table,
  `--platform auto` default note, link from § "Measuring C" to the registry.
- **Context:** The policy is the source of truth; it must document the new capability
  so agents know auto-detection and the override exist.
- **Related documents:** `docs/policies/RRI_POLICY.md § Measuring C` (link target).
- **Inputs:** `RRI_POLICY.md`.
- **Outputs:** amended `RRI_POLICY.md`.
- **Acceptance criteria:**
  - New section lists rust/python/go/rn/dubbridge with measurer + marker.
  - States `--platform auto` is the default and how detection works.
  - `make qa-docs` passes.

---

## Agent handoff prompt (delegation-ready)

> Implement T1 → T6 from `docs/tasks/rri-multiplatform-profiles.md` in the
> `dubbridge` repo. T0 is complete. Governing docs:
> `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (highest authority),
> `docs/policies/RRI_POLICY.md` (source of truth), `docs/plan/rri-multiplatform-profiles.md`.
> Refactor `scripts/rri.py`: `PlatformProfile`/`RubricRow` + `PROFILES` registry +
> `detect_platform` (T1); per-platform C measurers clippy/gocyclo/eslint + radon
> refactor + `_filter_existing`, each `(paths)->(cc|None, evidence)` with graceful
> tool-absent skip (T2); `_GENERIC_RUBRIC` + `_DUBBRIDGE_RUBRIC` + parametrized
> `match_rubric` (T3); `evaluate(profile=None)` auto-detect, CLI `--platform`,
> Platform render line (T4). Add tests without requiring external tools, keep the 37
> existing green (T5). Document "Platform profiles" in `RRI_POLICY.md` (T6).
> Run `python3 scripts/rri_test.py`, `make qa-rri`, and `make qa-docs` before
> reporting done. Present each task for approval before editing; mark progress here.
