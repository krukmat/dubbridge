# Required Reasoning Index (RRI) Policy

> **Status:** Active. Adopted by `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` as the
> canonical method for complexity-and-risk scoring, model-tier selection, and
> autonomy-gate determination. `AGENT_WORKFLOW_GUIDE.md` is the highest authority;
> this file is the detailed procedure it delegates to.

## Purpose

RRI estimates how much reasoning, context, caution, and verification a task
requires before an AI agent may safely implement it.

RRI **determines the approval gate and evidence required** before an agent may
implement a task. For bands **RRI 26+**, the HITL approval checkpoint is
mandatory; what the band controls is what evidence the agent must bring to it.
For band **RRI 0–25**, the agent skips the full human approval presentation and
delegates execution to local Gemma through Ollama, then reviews, verifies, and
reports the result (see `docs/policies/HITL_AUTONOMY_POLICY.md` for the full rule).

## Formula

```
RRI = 100 × ((0.18·C + 0.12·F + 0.15·D + 0.15·T + 0.12·A + 0.12·K + 0.10·P + 0.06·X) / 5)
    + Penalties
```

Weight verification: 0.18 + 0.12 + 0.15 + 0.15 + 0.12 + 0.12 + 0.10 + 0.06 = **1.00** ✓

Each variable is scored **0–5**. The base term is therefore in **[0, 100]**.
Penalties push the score above 100.

## Variables

### How to obtain each variable

Objective variables must be **measured**, not estimated.
Subjective variables must be **judged using the anchor rubric** below so that
two independent agents score the same task to the same number.

| Var | Name | Nature | How to obtain |
|---|---|---|---|
| **C** | Cyclomatic complexity | Objective (proxy) | Estimate via `CC = E − N + 2P` (count: `if`, `else if`, `match` arm, `while`, `for`, `loop`, `?` branches, `&&`/`\|\|` in conditions). Or use `clippy::cognitive_complexity` as a proxy. |
| **F** | Files affected | **Objective** | `git diff --name-only <base>...HEAD` — count the files. |
| **D** | Domain complexity | Subjective — anchor rubric | Classify the task's target path/crate using the anchor table. |
| **T** | Test-coverage risk | Semi-objective | Check `cargo llvm-cov` output for the affected file/module. If no tests exist in the area, score high. |
| **A** | Task ambiguity | Subjective | Is there a task file with acceptance criteria + happy/edge examples (required by the workflow guide)? Score near 0. Vague tasks score 5. |
| **K** | Coupling / side effects | Subjective — anchor rubric | Classify using the anchor table. |
| **P** | Public API / security / data impact | Subjective — anchor rubric (ADR-anchored) | Classify using the anchor table. |
| **X** | Context size required | Subjective | How many files/modules must the agent hold in mind? |

### Scoring bands per variable

**C — Cyclomatic complexity**

| Score | CC range |
|---|---|
| 0 | 1–5 |
| 1 | 6–10 |
| 2 | 11–20 |
| 3 | 21–30 |
| 4 | 31–50 |
| 5 | 50+ |

**F — Files affected**

| Score | Files |
|---|---|
| 0 | 1 |
| 1 | 2 |
| 2 | 3–5 |
| 3 | 6–10 |
| 4 | 11–20 |
| 5 | 20+ |

**D — Domain complexity**

| Score | Domain |
|---|---|
| 0 | Documentation, naming, formatting |
| 1 | Simple logic, constants, copy |
| 2 | Normal business logic |
| 3 | Integrations, workflows, state management |
| 4 | Platform-specific core logic, async orchestration, agent orchestration, permissions |
| 5 | Security, authentication, compliance, financial or critical data logic |

**T — Test-coverage risk**

| Score | Test state |
|---|---|
| 0 | Strong specific tests exist for the area |
| 1 | Reasonable tests exist |
| 2 | Partial tests exist |
| 3 | Weak or fragile tests |
| 4 | No tests in the affected area |
| 5 | No tests and critical logic |

**A — Task ambiguity**

| Score | Ambiguity |
|---|---|
| 0 | Exact task with acceptance criteria and happy/edge examples |
| 1 | Mostly clear |
| 2 | Some missing details |
| 3 | Requires significant interpretation |
| 4 | Very open-ended |
| 5 | Vague ("improve this", "make it better") |

**K — Coupling / side effects**

| Score | Coupling |
|---|---|
| 0 | Pure function |
| 1 | Isolated class or module |
| 2 | Internal module with contained side effects |
| 3 | Database, API, filesystem, external service, or framework integration |
| 4 | Async behavior, events, queues, transactions, platform side effects |
| 5 | Distributed system behavior or critical external side effects |

**P — Public API / security / permissions / data impact**

| Score | Impact |
|---|---|
| 0 | No impact |
| 1 | Minor internal impact |
| 2 | Changes internal behavior |
| 3 | Changes internal API |
| 4 | Changes public API, permissions, ownership, data visibility, or persisted data |
| 5 | Security, authentication, authorization, data loss, compliance, or critical business risk |

**X — Context size required**

| Score | Scope |
|---|---|
| 0 | One function |
| 1 | One class or file |
| 2 | 2–5 files |
| 3 | One complete module |
| 4 | Several modules or crates |
| 5 | Multi-repository or global architecture context |

## DubBridge anchor rubric

Use this table to derive the **minimum floor** for D, P, and K when the task
touches these paths or crates. Score higher if the specific change within the
path warrants it; never score lower than the floor.

| Task touches | D floor | P floor | K floor | ADR anchor |
|---|---|---|---|---|
| `docs/**`, naming, formatting, `config/*.toml` (non-secret) | 0 | 0 | 0 | — |
| `config/*.toml` with env-wiring logic, `config/README.md` | 1 | 1 | 1 | ADR-026 |
| Internal crate business logic (`crates/qc`, `crates/media` builders, `crates/providers`, `crates/domain`) | 2 | 2 | 2 | — |
| `crates/db`, `crates/storage`, `crates/jobs`, `crates/connectors`, `crates/ingestion`, `crates/observability`, async orchestration, HTTP proxy logic | 3 | 3 | 3 | ADR-006, ADR-018 |
| `apps/gateway/src/auth/**`, token handling, session/cookie management, CSRF | 4 | 4 | 4 | ADR-024 |
| `crates/auth`, JWT boundary, principal propagation | 4 | 4 | 4 | ADR-023 |
| `crates/audit`, rights-ledger path (`crates/domain` rights types), `infra/migrations/**` | 4 | 5 | 4 | ADR-008, ADR-018 |
| Secrets, credential storage, authentication/authorization system boundary | 5 | 5 | 5 | ADR-023, ADR-024, ADR-025 |

## Penalties

Apply each penalty independently; they are additive.

| Condition | Penalty |
|---|---|
| Refactor and functional behavior change combined in the same task | +8 |
| Tests missing **and** public/security/data impact is high (P ≥ 4) | +10 |
| Cyclomatic complexity > 30 (C ≥ 4) **and** domain complexity ≥ 3 (D ≥ 3) | +10 |
| Task touches authn, authz, permissions, security, ownership, or sensitive data | +10 |
| Task is likely to affect more than 10 files (F ≥ 4) | +8 |
| An architecture or process/policy decision is required | +12 |
| No clear verification strategy exists | +15 |

## Bands, autonomy gates, and model tiers

The HITL approval requirement applies at every band **except RRI 0–25**, which
uses local Gemma delegation (see below). For all other bands, what the band
controls is the evidence and gates the agent must satisfy before and after that
approval.

Effort, capability, thinking, and gate are each derived **in parallel** from the RRI
band — never derive one output from another (e.g. do not infer capability from Effort).

| RRI band | Label | Effort | Capability (Codex) | Capability (Claude Code) | Thinking | Gate |
|---|---|---|---|---|---|---|
| **0–25** | Low | **S** | Local Gemma via Ollama | Local Gemma via Ollama | Off | **Local delegation:** do not present the full task for approval; delegate to local Gemma, validate and apply only an in-scope diff, review against requirements, verify, and report. |
| **26–40** | Moderate | **M** | Balanced | Balanced | Off | Confirm tests exist in the affected area. |
| **41–55** | Med-high | **L** | Balanced → Premium | Balanced → Premium | On | Plan + explicit acceptance criteria required before approval. |
| **56–70** | Complex | **L** | Premium | Premium | On | Plan first. Do not implement before producing and approving a clear plan. Human reviews the plan. |
| **71–85** | High | **XL** | Premium | Premium | On | Characterization tests + explicit acceptance criteria + human reviews the **diff** (not just the plan). |
| **86–100** | Very high | **XL** | Premium | Premium | On | Do not implement directly. Produce an ADR + risk analysis + decompose into subtasks. |
| **> 100** | Excessive | **XL** | Premium | Premium | On | Architecture/design work must happen first. Re-scope before any implementation. |

### Model tier resolution

For RRI 26+, the capability labels above (Balanced / Premium) map to concrete model
IDs per the resolution table in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (Model
tier resolution). Resolve against official vendor documentation at
task-presentation time — do not rely on stale memory for "latest" or "best".

The Low band is special: it resolves to the local Ollama/Gemma delegation path,
not to a cloud vendor model recommendation. Use `OLLAMA_HOST` when set, otherwise
`http://localhost:11434`. Use `DUBBRIDGE_LOW_RRI_MODEL` when set, otherwise
`gemma4:12b-it-q4_K_M`.

Thinking mode: activate for Balanced→Premium and above when the task requires
multi-step reasoning that cannot be validated incrementally. Do **not** activate
for config edits, doc updates, or tasks where the strategy is fully pre-defined.

### Low RRI local delegation

For final **RRI 0–25**, the active agent remains the orchestrator and reviewer.
Gemma has no direct filesystem or shell authority; it returns **full file
contents** plus verification intent, and the caller (the script + orchestrating
agent) deterministically builds the diff and applies it. Gemma must not evaluate,
approve, or mark its own delegated work as complete; the delegating agent owns
that decision.

For the operational step-by-step handoff discipline, packet-shaping rules, and
scope-reduction guidance for local-model work, see
`docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`.

**Why file contents, not a diff:** small local models reliably write correct file
bodies but botch unified-diff framing — merged hunks, missing headers, wrong line
counts — especially across multiple files. The model returns each changed file in
full; the script constructs the diff with `git diff --no-index` (git owns all hunk
framing) and applies it with `git apply`. The failure-prone step never runs in the
model.

Use `scripts/delegate-low-rri.py` to communicate with Ollama. The wrapper avoids
shell-quoting failures, checks that the resolved model is installed, and uses the
`/api/chat` endpoint with **`stream: true`**. Each received token resets an
idle-timeout (default 60 s); a separate max-wall cap (default 900 s) guards
against runaway generation. This distinguishes a stalled Gemma (no tokens for 60 s
→ exit 124) from a slow but working one (tokens still arriving), making the
delegation reliable at any generation speed without imposing a blind wall-clock
timeout against total generation time.

**Always invoke the script in the background from an agent** so the agent's Bash
tool timeout (typically 120 s) does not abort a legitimately long generation:

```bash
# 1. Write the packet to a file (avoids shell-quoting issues with heredocs).
# 2. Delegate in the background; agent is notified on completion. Pass the
#    in-scope path set with --allow-path; --apply builds and applies the diff.
scripts/delegate-low-rri.py packet.md \
  --allow-path scripts/ --apply --out result.json
# Exit 0  → read result.json (includes the built unified_diff + apply_result)
# Exit 124 → Gemma stalled (idle) or hit the max-wall cap; escalate
# Exit 2   → Ollama unreachable
# Exit 1   → validation / out-of-scope path / git apply failure
```

`--allow-path` is **required** whenever files are written: any returned path
outside the declared prefixes/globs (or escaping the repo) is rejected before any
diff is built. Omit `--apply` to build the diff and inspect it without touching
the tree; the script still writes the diff into `result.json` under
`unified_diff`.

The wrapper resolves:

| Env var | Default | Purpose |
|---|---|---|
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama endpoint |
| `DUBBRIDGE_LOW_RRI_MODEL` | `gemma4:12b-it-q4_K_M` | Local model |
| `DUBBRIDGE_LOW_RRI_IDLE_TIMEOUT_SECONDS` | `60` | Seconds without a token = stall |
| `DUBBRIDGE_LOW_RRI_MAX_WALL_SECONDS` | `900` | Hard generation cap |
| `DUBBRIDGE_LOW_RRI_NUM_CTX` | `16384` | Context window for packet + schema |

Gemma's response content must be JSON with full file contents (never a diff):

```json
{
  "status": "patch",
  "summary": "<short implementation summary>",
  "files": [
    {"path": "<repo-relative path>",
     "action": "create | modify | delete",
     "contents": "<COMPLETE final file contents>"}
  ],
  "test_commands": ["<command>"],
  "risk_notes": ["<note>"]
}
```

The script then enforces the allowed-path scope, builds the unified diff with git,
and (with `--apply`) runs `git apply --check` followed by `git apply`, recording
the diff under `unified_diff` and the outcome under `apply_result` in the result
JSON. The orchestrator must still personally evaluate the applied result against
all task requirements and acceptance criteria, and run the required checks — this
evaluation is performed by the delegating agent, not by Gemma. If requirements are
missed or checks fail, the orchestrator may run one bounded repair request through
Gemma with the same allowed paths and the failure evidence. A second failure, an
out-of-scope path, invalid JSON, unavailable Ollama/model, or a post-application
RRI above 25 must escalate to the normal human-gated workflow. If the delegation
times out (exit 124), report it explicitly as `Gemma timeout (idle|wall)` in the
final task summary.

## Decomposition triggers

Split a task into subtasks before implementing if **any** of the following apply:

- RRI > 70, or base RRI > 100 (before penalties).
- F ≥ 4 **and** K ≥ 3 — large change surface with high coupling; isolate each seam.
- C ≥ 4 **and** D ≥ 3 — the +10 penalty activates; separate complex logic into a
  testable unit first.
- The +8 penalty is active (refactor + behavior change combined) — always separate
  refactor from functional change into distinct tasks/commits.
- T ≥ 4 **and** P ≥ 4 (no tests + high impact) — first subtask must be
  characterization tests; implementation is the second subtask.

**Split target:** divide until each subtask scores RRI ≤ 55 with A ∈ {0, 1}
(own acceptance criteria + happy/edge examples per the workflow guide).

## Reporting format

Before every implementation, compute the RRI as a table. For RRI 26+, present it
in the task approval packet. For RRI 0–25, include it in the local delegation
packet and final report instead of presenting the full task for approval.

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0–5 | How obtained (CC formula / clippy / estimate) | High / Medium / Low |
| F files | 0–5 | `git diff` count | High |
| D domain | 0–5 | Anchor rubric row | High / Medium |
| T coverage | 0–5 | llvm-cov output or "no tests found" | High / Medium |
| A ambiguity | 0–5 | Task file has/lacks criteria + examples | High |
| K coupling | 0–5 | Anchor rubric row | High / Medium |
| P impact | 0–5 | Anchor rubric row + ADR cited | High |
| X context | 0–5 | Files/modules required | Medium |

Then state:
- **Base value:** `100 × (Σ / 5) = ___`
- **Penalties applied:** list each triggered penalty and its value.
- **Final RRI:** base + penalties = ___ → band ___ → tier ___ / thinking ___.
- **Gates for this band:** list the gates that apply.

Low-confidence scores on D, P, or K are themselves a signal: treat the variable
as one step higher when confidence is Low.

## Script automation

**Agents must run `python3 scripts/rri.py` instead of computing the formula,
floors, or penalties manually.** The script is the canonical RRI calculator.

### What the script decides vs. what the agent supplies

| Decided by `scripts/rri.py` (objective / derivable) | Supplied by the agent (irreducible judgment) |
|---|---|
| F score — counts `--touches` paths or `git diff`, maps to 0–5 | **C** — agent measures raw CC (radon/mccabe/clippy/gocyclo/eslint), passes as `--cc <raw>` (or `--auto-cc` to let the script measure it per platform) |
| C score — maps raw CC to 0–5 via the policy CC table | **T** — agent measures via `cargo llvm-cov`, passes as `--T` |
| D / P / K floors — derived from the anchor rubric; raises agent input, never lowers | **A** — task ambiguity (has acceptance criteria + examples?) |
| `many_files`, `complex_and_domain`, `no_tests_high_impact`, `auth_security` penalties | **X** — context size required |
| Band, Effort (S/M/L/XL), tiers (Economy/Balanced/Premium), thinking, gate | **D / P / K above the floor** + 3 intent penalties: `refactor_and_behavior`, `arch_decision`, `no_verification` |
| Decomposition-trigger detection | — |

### Invocation

**At task-presentation time** (before any code is written; diff is empty):

```bash
python3 scripts/rri.py \
  --touches <path1> --touches <path2> \
  --cc <raw-cyclomatic>  \
  --D <0-5> --K <0-5> --P <0-5> \
  --T <0-5> --A <0-5> --X <0-5> \
  [--penalty refactor_and_behavior] \
  [--penalty arch_decision] \
  [--penalty no_verification]
```

`--touches` feeds both the F file count and the anchor-rubric floor derivation.
Repeat it once per affected path. The script raises D/P/K to their rubric floors
automatically and reports any raise in the evidence column.

**Post-implementation** (diff is available; omit `--touches`):

```bash
python3 scripts/rri.py --cc <raw> --D <0-5> --K <0-5> --P <0-5> \
  --T <0-5> --A <0-5> --X <0-5>
# F measured automatically from git diff --name-only <base>...HEAD
```

Use `--F <0-5>` only when git is unavailable (e.g., sandbox agent with no repo).
Use `--C <0-5>` only when the raw CC value is unavailable (pre-computed score).

**JSON output** (for tooling or CI):

```bash
python3 scripts/rri.py ... --json
```

### Measuring C (raw cyclomatic complexity)

- **Python:** `python3 -m mccabe --min 1 <file>` or `radon cc -s <file>`
- **Rust:** `cargo clippy -- -W clippy::cognitive_complexity` or count branch points manually with the CC formula in this policy.
- Take the highest CC value across all functions that will be created or materially changed.
- **Automated:** pass `--auto-cc` instead of `--cc <raw>` to let the script run the
  detected platform's measurer for you (see [Platform profiles](#platform-profiles)).
  If the tool is unavailable, the script falls back to `C=0` marked **Low**
  confidence — never a silent wrong value.

### Platform profiles

The script is **portable across language ecosystems.** A *platform profile* bundles
two platform-specific concerns; everything else (formula, weights, penalties, bands)
is universal and identical on every platform.

| Profile | Marker file | C measurer (`--auto-cc`) | Anchor rubric |
|---|---|---|---|
| `dubbridge` | `docs/policies/RRI_POLICY.md` | `cargo clippy` (cognitive_complexity) | DubBridge ADR-anchored rubric |
| `rust` | `Cargo.toml` | `cargo clippy` (cognitive_complexity) | generic convention rubric |
| `go` | `go.mod` | `gocyclo` | generic convention rubric |
| `rn` | `package.json` | `eslint` (`complexity` rule) | generic convention rubric |
| `python` | `pyproject.toml` / `setup.py` | `radon cc` | generic convention rubric |
| `generic` | _(none detected)_ | — (agent supplies `--cc`/`--C`) | empty (agent judgment) |

**Selection.** `--platform auto` (the default) walks up from the working directory
and picks the first profile whose marker file exists. The `dubbridge` marker is
checked before generic `rust`, so this repo never degrades to the generic Rust
rubric. Override with `--platform {rust,go,rn,python,dubbridge,generic}`.

**Generic rubric** (used by the non-DubBridge profiles) raises D/P/K floors by
directory convention: `**/auth/**`, `**/security/**`, `**/crypto/**` → 4/4/4;
`**/migrations/**` → 4/5/4; `**/db/**`, `**/api/**`, `**/services/**` → 3/3/3;
`docs/**`, `**/test*/**` → 0/0/0. It cites no ADRs (the `—` ADR column). Each
project's own critical paths should eventually graduate to a dedicated profile like
`dubbridge` when ADR anchoring is warranted.

### Copy the output into the task presentation

Run the script, then paste its markdown output directly into the task presentation
block. Do not reformat or recompute. The script output **is** the RRI report.

## Related

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — highest authority; adopts this policy
- `docs/policies/HITL_AUTONOMY_POLICY.md` — approval requirements and local delegation rule
- `docs/tasks/rri-integration.md` — integration task ledger
- `scripts/rri.py` — canonical calculator
- `scripts/rri_test.py` — unit tests (run via `make qa-rri`)
