---
type: Plan
title: "Plan: Antares local runtime adoption (official CLI + local inference shim)"
status: proposed
slice: antares-security-specialist-advisor
---

# Plan: Antares Local Runtime Adoption

## Objective

Get a real, working local Antares invocation path by adopting Cisco's own
official `antares-cli` reference implementation against a small local
inference shim, instead of continuing to hand-build a wire-format parser
against a model output shape that was never observed
(`scripts/antares/tool_call_parser.py`, T2a docstring). Define what stays
outside the tracked repository (personal, reusable tooling) versus what
becomes tracked, governed follow-on work inside `scripts/antares/*`.

This plan does not itself authorize any edit to a tracked path. It is
pre-implementation context per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` Step 2
and is written under the "drafting plans... no code execution" allowance in
`docs/policies/HITL_AUTONOMY_POLICY.md`. Any edit to `scripts/antares/*`
still requires its own RRI computation, task card, and — per its band —
explicit approval before implementation.

## Background

`docs/tasks/antares-security-specialist-advisor.md` T1 (runtime preflight)
recorded a `BLOCKED` technical result on 2026-07-29 (gated Hugging Face
access, `401 GatedRepo`) with an owner waiver to proceed. Its documented
recovery plan (R1-R5) marked R1 (human HF access) "not agent-solvable."

During this session:

- R1 was resolved: gated access to `fdtn-ai/antares-1b` was confirmed
  (`401` -> `200`) using a user-supplied, since-superseded Hugging Face
  token. **The token that was used is exposed in this conversation's
  history and must be treated as compromised regardless of later
  revocation.**
- R2 was resolved: a disposable Python 3.12 venv
  (`.antares-runtime/venv/`, gitignored) with `torch 2.13.0`,
  `transformers 5.14.1`, `safetensors 0.8.0`, `accelerate 1.14.0`,
  `huggingface_hub 1.26.0` confirmed Apple Silicon MPS availability
  (`torch.backends.mps.is_available() -> True`).
- R3 substantially advanced: the pinned revision
  (`10417eb35641b32e7141157db19c76eb545193b6`) was downloaded to
  `.antares-runtime/model/` (3.4G) and a SHA-256 manifest of the 8
  top-level artifact files was generated
  (`.antares-runtime/manifest-sha256.txt`). No externally published
  second-source digest has been cross-checked yet — this remains a minor
  open gap, not a blocker.
- An unplanned but decisive discovery: the model repository ships Cisco's
  own official reference CLI at `assets/antares-cli.zip`
  (`.antares-runtime/antares-cli-reference/` once unzipped, gitignored).
  It is a complete, tested, Apache-2.0-licensed Python package
  (`pyproject.toml`: `name = "antares-cli"`, console script
  `antares = "antares_cli.main:app"`), not a fragment.

### Wire-format ground truth (falsifies the current internal-schema docstring)

`scripts/antares/tool_call_parser.py` documents an internal normalized
schema, `{"tool": <name>, "payload": {...}}`, and says explicitly that "no
observed Antares transcript exists to pin an exact wire shape" and that
"whatever harness invokes Antares for real (T2c) is responsible for
translating the model's actual `<tool_call>` output into this internal
schema." That translation was never written. Cisco's reference parser is
that ground truth:

- `agent/streaming.py::_tool_call_arguments()` reads `args` or `arguments`
  — never `payload`.
- Tool name comes from `tool` or `name` (`agent/streaming.py`), normalized
  via `re.sub(r"[\s-]+", "_", tool_name.strip().lower())`.
- Wrapping tags are `<tool_call>`, `<done>`, `<answer>`
  (`_WRAPPED_OPEN_TAGS`), plus `<think>` framing stripped before parsing.
- The real tool set is `terminal` (argv-based; `bash` is an accepted
  alias — `agent/contracts.py`), `read_file`, `submit_vulnerable_files`
  (argument `ranked_files`, aliases `files`/`file_paths`), and
  `submit_no_vulnerability_found` (`agent/model_adapter.py:179-244`, the
  literal tool schema sent to the model).
- The full prompt contract — `<tool_call>{"name": ..., "arguments": ...}</tool_call>`
  — is built once in `agent/model_adapter.py::_build_antares_investigation_prompt()`.

### Why "adopt the CLI" beats "fix the parser"

`antares-cli` is not just a parser reference. It already includes the full
agent loop (`agent/loop.py`), tool execution sandbox (`tools/shell_exec.py`,
snapshot isolation described in its `README.md`), CWE selection
(bundled MITRE CWE 4.20), and — critically — a stable **JSON automation
interface**: `antares tool query --stdin` / `antares tool sweep --stdin`
take a JSON request on stdin and print a JSON result on stdout (exit `0`
completed, `1` completed-with-findings-and-`--fail-on-findings`, `2`
invocation/operational failure). Driving scans through that interface means
`scripts/antares/*` never has to parse a raw `<tool_call>` stream itself —
Cisco owns and maintains that parsing.

Licensing is clean for unmodified use: CLI source is Apache-2.0
(`LICENSE`); the only third-party content is a bundled MITRE CWE snapshot
under MITRE's free-use-with-attribution terms, already reproduced in the
package's own `THIRD_PARTY_NOTICES.md`. No obligation beyond preserving
that notice file, which ships inside the package unmodified.

## Design decisions

1. **Do not fork or hand-edit `antares_cli`.** Use it as an unmodified
   dependency. Lower maintenance burden, and Cisco's implementation is
   already tested (`tests/unit/*`, 25+ files including
   `test_streaming_parser.py`, `test_agent_contracts.py`,
   `test_read_only_commands_allowed.py`/`_blocked.py`).
2. **Local inference is served by Ollama, not a hand-built shim.**
   `antares_cli.inference.backend.InferenceBackend` is an open abstract
   interface (`stream_generate(messages) -> Iterator[str]`), but
   `core/runtime.py::_resolve_backend` only wires `"remote"`
   (`RemoteInferenceBackend`, an OpenAI-compatible HTTP client —
   `inference/remote.py`). An earlier iteration of this plan stood up a
   small FastAPI/uvicorn shim to speak that same `POST /v1/completions`
   contract. That shim was deleted once verification showed Ollama already
   exposes a native, OpenAI-compatible `POST /v1/completions` SSE endpoint
   matching `RemoteInferenceBackend`'s wire contract exactly — zero custom
   code needed. The model is served as a GGUF conversion of the pinned
   `antares-1b` checkpoint: `convert_hf_to_gguf.py` (`ggml-org/llama.cpp`,
   upstream PR #13550 "Granite Four") explicitly registers
   `GraniteMoeHybridForCausalLM`; because every one of antares-1b's 40
   layers is typed `"attention"` (zero real Mamba/SSM layers), conversion
   falls back to a standard, already-supported `GRANITE_MOE`/`GRANITE` GGUF
   architecture rather than a novel path. `ollama create antares-1b -f
   Modelfile` imports the resulting GGUF as `antares-1b:latest`.
3. **No chat-template logic needed, regardless of backend.**
   `RemoteInferenceBackend._completions_generate`
   (`inference/remote.py:209-229`) renders the full prompt client-side via
   `antares_cli.inference.granite.apply_granite_chat_template(messages)`
   and sends it as a single `"prompt"` string to `/v1/completions`. Ollama
   only needs to tokenize that already-templated string and generate —
   exactly what raw (non-chat) `/v1/completions` means, and exactly why the
   CLI's own `README.md` warns "chat completions are not equivalent." This
   holds identically whether the backend is the retired shim or Ollama.
4. **Governance boundary: satisfied by explicit task revision, not
   avoided.** T1's recovery plan forbids substituting vLLM/SGLang/Ollama/
   Docker Model Runner for the authorized Transformers+PyTorch route
   without "an explicit task revision with new provenance and RRI"
   (`docs/tasks/antares-security-specialist-advisor.md`, R1-R5 table and
   surrounding prose). An earlier version of this decision claimed the
   FastAPI+Transformers+MPS shim avoided triggering that clause by staying
   off Ollama. That claim no longer holds: this plan now runs on Ollama.
   The clause is satisfied instead by
   `docs/tasks/antares-security-specialist-advisor.md` § "T1 task revision
   (2026-08-05): Ollama/GGUF runtime proposal", approved 2026-08-05, which
   recorded the new provenance and computed RRI 34 (Moderate) before this
   plan was updated to reflect the Ollama-backed implementation as adopted.
   The original R2 Transformers+PyTorch route remains documented as a valid
   alternative; it is not superseded, only no longer the sole authorized path.
5. **Repository boundary.** The GGUF model, Ollama import, and CLI
   install/profile are personal, generically reusable local host state with
   nothing dubbridge-specific in them (Ollama would serve any GGUF-
   compatible model behind an OpenAI-completions endpoint; the CLI is
   Cisco's own, unmodified). They are not added to this repository and
   carry no tracked-repo governance obligation. Only the `scripts/antares/*`
   consumer side (Element 3 below) is tracked and governed.

## Architecture

```mermaid
flowchart TB
    subgraph personal["Personal local tooling (outside dubbridge, ungoverned)"]
        Ollama["Ollama\nantares-1b:latest (GGUF)\nPOST /v1/completions (SSE)\nnative OpenAI-compatible endpoint"]
        CLI["antares-cli (unmodified)\nuv tool install .\n~/.antares/profiles.toml"]
        CLI -- "OpenAI-compatible\ncompletions request" --> Ollama
    end

    subgraph tracked["dubbridge repo (tracked, governed)"]
        Harness["scripts/antares/harness.py\n(reconciled — Element 3)"]
        Parser["tool_call_parser.py /\nterminal_state.py\n(scope narrows or retires\nfor live-invocation path)"]
    end

    Harness -- "subprocess:\nantares tool query/sweep --stdin\n(JSON in, JSON out)" --> CLI
    Harness -.->|"replay-fixture /\nsynthetic tests only"| Parser
```

## Element 1 — Ollama-served GGUF runtime (personal tooling, not tracked) — implemented

- The pinned `fdtn-ai/antares-1b` checkpoint (revision
  `10417eb35641b32e7141157db19c76eb545193b6`, manifest-verified per R3) was
  converted to GGUF via `convert_hf_to_gguf.py` from a shallow clone of
  `ggml-org/llama.cpp`: 3.67GB bf16, 363 tensors. Source inspection of
  `conversion/granite.py` confirmed `GraniteMoeHybridForCausalLM` is
  explicitly registered and — because antares-1b has zero real Mamba/SSM
  layers — falls back to a standard Granite GGUF architecture, not a novel
  conversion path.
- Imported into Ollama: `Modelfile` (`FROM ./antares-1b.gguf`) +
  `ollama create antares-1b -f Modelfile` → `antares-1b:latest`, confirmed
  via `ollama list`.
- Ollama's native `POST /v1/completions` SSE endpoint matches
  `RemoteInferenceBackend`'s expected wire contract
  (`data: {"choices":[{"text": "<delta>"}]}\n\n`, terminated by
  `data: [DONE]`) with zero custom translation code — this is what replaced
  the earlier FastAPI/uvicorn shim, which has been deleted.
- Lives in the user's own tool space (`~/tools/antares-local-infer/`), not
  in this repository.

## Element 2 — Official `antares-cli`, unmodified (personal tooling, not tracked) — implemented

- Installed: `uv tool install .` from `.antares-runtime/antares-cli-reference/`.
- Configured `~/.antares/profiles.toml`:
  ```toml
  [profiles.antares-local]
  display_name = "Antares 1B (local via Ollama)"
  model = "antares-1b"
  backend = "remote"
  endpoint = "http://127.0.0.1:11434/v1/completions"
  context_window = 16384
  remote_timeout_seconds = 300

  [profiles.antares-local.generation]
  max_tokens = 4096
  temperature = 0.3
  top_p = 1.0
  frequency_penalty = 0.3
  stop_tokens = ["<|end_of_text|>", "<|start_of_role|>"]
  use_completions_api = true
  ```
  (values mirror `inference/defaults.py`; profile name is `antares-local`,
  endpoint is Ollama's default port `11434` — not the earlier plan draft's
  `local-antares`/`8000`, which named the now-deleted shim).
- Validated: `antares models list` correctly shows the `antares-local`
  profile. A live `antares tool query --stdin` run against
  `/Users/matias/dubbridge/crates/auth` with `CWE-287` completed end-to-end
  in ~11.8s with 0 generation errors and returned a genuine finding
  (`Improper Authentication`, `src/issuer.rs`). This is ad hoc validation,
  not T1's required fixed R4/R5 representative fixture.

## Element 3 — `scripts/antares/*` reconciliation (complete 2026-08-05 — Subtasks A, B, C all closed)

**Decision (Subtask A, `docs/audit/antares-t4-element3-rri.md` § Subtask A,
RRI 26 Moderate, approved 2026-08-05): retire the harness's live-invocation
role; adopt `antares tool query --stdin` / `antares tool sweep --stdin` as
direct CLI subprocess calls; retain T2a–T2e as the synthetic-fixture/
replay-test path only.**

Justification, grounded in evidence rather than the prior either/or framing:

- Phase B (`docs/evaluations/antares-phase-b-comparison.md`) empirically
  confirmed `harness.py::dispatch_tool_call` rejects real Antares
  wire-format output (3/3 real-shaped inputs -> `MALFORMED_TOOL_CALL`); no
  translation layer exists anywhere in `scripts/antares/*`.
- T2a's own 2026-07-29 closure record already documented the same gap: the
  translation layer it assigned to T2c was never built (T2c split into
  T2c-1 subprocess lifecycle and T2c-2 resource budgets, neither of which
  does wire-format translation).
- `antares tool query --stdin` is already proven working end-to-end via
  direct CLI invocation (T1 R4/R5,
  `docs/evaluations/antares-runtime-preflight.md`), so no new subprocess
  plumbing needs to be built inside the harness — the CLI already is that
  layer. Building a second one duplicates functionality the CLI provides
  natively, with no evidence motivating the duplication.
- `tool_call_parser.py` / `terminal_state.py` are **not deleted**: they
  keep serving `replay_fixtures.py` / `harness_test.py` as the synthetic
  fixture/replay-test path, unchanged, per the disposition already
  recorded for T2a–T2e.

**Implementation (Subtask B, RRI 50 Med-high, approved and closed
2026-08-05):** `docs/audit/antares-t4-element3-rri.md` § Subtask B — closure
record. `scripts/antares/harness.py` gained `dispatch_via_cli`, an
argv-only `subprocess.Popen` dispatch to `antares tool query --stdin` /
`antares tool sweep --stdin`, plus `cli_terminal_state_to_artifact` to map
its result into the existing `Artifact` schema. `terminal_state.py` gained
4 additive `TerminalStateKind` members (`CLI_EXECUTION_COMPLETE`,
`CLI_BINARY_UNAVAILABLE`, `CLI_EXECUTION_FAILED`, `CLI_OUTPUT_MALFORMED`);
`artifact_schema.py` gained a `T2CLI_KINDS` category. `dispatch_tool_call`/
`tool_call_parser.py`/`terminal_state.py`'s existing members are unchanged
and still serve `replay_fixtures.py`/`harness_test.py` as the retained
synthetic-fixture/replay-test path (Subtask C's disposition) — `harness.py`
now has two independent entrypoints, and the CLI-subprocess one is the
actual live-invocation path per this section's decision. The initial RRI 43
scope (`harness.py` only) was found incomplete mid-implementation — EC-1/EC-2
require new terminal-state kinds, which cannot exist without touching
`terminal_state.py`, a closed `Enum` — so the task was stopped before any
code was written, rescored to 50 with the corrected touch set, and
re-presented for its own approval before implementation resumed.

## What's needed beyond the two elements

1. ~~**Close T1's R4/R5.**~~ **Done 2026-08-05.** Ran T1's fixed
   representative fixture through the real pipeline (Element 1 + 2) as two
   `apps/`+`crates/`-scoped invocations and recorded cold-start latency, RSS,
   swap, and terminal results into
   `docs/evaluations/antares-runtime-preflight.md` /
   `antares-runtime-preflight.json` (`r4_r5_ollama_run_2026_08_05`).
   Result: `PASS` on all five thresholds (per-command latency supported by
   indirect evidence — the CLI does not emit per-tool-call timings; see the
   evidence file's caveat). Full record also cross-referenced from
   `docs/tasks/antares-security-specialist-advisor.md` § "T1 R4/R5 execution
   record — Ollama runtime (2026-08-05)".
2. **Credential hygiene (open, owner-owned).** Revoke/rotate the Hugging
   Face token pasted in this session's chat history — already the user's
   stated intent, not yet confirmed done. Any future token must be
   supplied via environment variable, never pasted in a conversation.
3. **Provenance gap.** The SHA-256 manifest in
   `.antares-runtime/manifest-sha256.txt` only proves the downloaded bytes
   are internally consistent; there is no externally published
   second-source digest to cross-check transport integrity against.
   Low priority, not a blocker to using the artifact locally.
4. **Lifecycle script.** A small launcher (start the shim in the
   background, healthcheck `GET`/`POST` before invoking `antares`, stop
   on exit) — again personal tooling, not a tracked deliverable.
5. **Model choice for routine use.** Default to `antares-1b` (already
   downloaded and MPS-verified) over `antares-350m`; revisit only if
   latency on routine scans becomes a problem.

## Orchestration and cross-plan dependencies

This plan and `docs/plan/antares-security-specialist-advisor.md` are not
independent tracks. This section is the single place where their sequencing
is reconciled; neither plan's own task order is authoritative for the other.

### Blocking finding: the wire-format translation layer was never written

Verified 2026-08-05 by direct inspection of `scripts/antares/*.py`:

- No module handles `<tool_call>` tags, the `name`/`arguments` fields, or a
  model output stream. The only occurrence of "arguments" in the package is
  inside `tool_call_parser.py`'s own docstring, describing the layer that
  was supposed to exist.
- The harness entrypoint is `harness.py::dispatch_tool_call(raw_json,
  session)` -> `parse_tool_call(raw_json)`, which already expects the
  internal `{"tool": ..., "payload": ...}` schema.
- `replay_fixtures.py::_msg()` builds exactly that internal schema, so the
  entire T2e corpus — and therefore every test that exercised the composed
  harness — validated against the assumed schema, never against real model
  output.
- T2a assigned the translation layer to T2c
  (`tool_call_parser.py` docstring). T2c was decomposed into T2c-1 (single
  subprocess lifecycle and isolation) and T2c-2 (aggregate budgets and
  teardown). Neither subtask's objective includes wire-format translation,
  and neither implemented it.

T2a's own Reflection Pass 1 flagged the discrepancy and Pass 2 resolved it
by reasoning that the observed `{"function": {"name": ..., "arguments":
...}}` shape was the local runner's generic function-calling envelope rather
than Antares' own protocol. That reasoning is now falsified: Cisco's
reference implementation reads `args`/`arguments` and `tool`/`name` inside
`<tool_call>` tags (`agent/streaming.py`, `agent/model_adapter.py`), so
`payload` was never the real key.

**Consequence for sequencing:** the five completed Med-high T2 subtasks are
not defective, but they are unvalidated against live model output *by
construction*, and the one field the internal schema pins is wrong. Any
further work layered on the T2 invocation path inherits that exposure until
either a translation layer is written or Element 3 replaces the path
outright.

### Dependency graph across both plans

```mermaid
flowchart TB
    subgraph runtime["Runtime track (this plan, personal/ungoverned)"]
        E1["Element 1\nlocal inference shim"] --> E2["Element 2\nofficial antares-cli\n+ profile"]
        E2 --> R45["T1 R4/R5 closure\nfixture run + resource metrics"]
    end

    subgraph packet["Packet track (slice plan, governed)"]
        T3done["T3a, T3b, T3c-0, T3c-1, T3c-2\n[x] Done"] --> T3d["T3d\n[ ] Open"]
    end

    subgraph invoke["Invocation track (resolved 2026-08-05)"]
        T2["T2a-T2e [x] Done\nretained as test-only path"]
        DEC["Decision: RESOLVED 2026-08-05\nadopt Cisco CLI, retire harness\nlive-invocation role"]
        T2 --> DEC
        E3["Element 3 / Subtask B [x] Done\ndispatch_via_cli implemented,\nnew CLI_* terminal-state kinds"]
        DEC --> E3
    end

    R45 --> EXP["Comparative experiment\nsame fixture, both paths"]
    T2 --> EXP
    EXP --> DEC
    R45 --> T4["T4 pilot\ncalibration + observe-only"]
    T3d --> T4
    DEC --> T4
    T4 --> T5["T5 promote / narrow / retire"]
```

Two things the graph makes explicit that neither plan stated alone:

1. **T4 has two broken prerequisites, not one.** It needs a working runtime
   (T1 was owner-waived, not solved — R4/R5 remain open) *and* a resolved
   invocation path. Continuing the packet track alone never unblocks it.
2. **The packet track is invocation-independent.** Computing a deterministic
   dependency/manifest closure over a snapshot is required whether Antares is
   driven by our own harness or by `antares tool query --stdin`. T3c-1,
   T3c-2, and T3d (**all done 2026-08-06** — T3 fully closed, all of
   T3a-T3d `[x] Done`) therefore survived the Element 3 decision intact and
   were not at risk from it. `packet_schema.py` and `cwe_watchlist.py`
   likewise.

### Proposed sequence

| Phase | Work | Why here | Gate |
|---|---|---|---|
| A | Elements 1 + 2 | Zero governance cost (personal tooling, outside the repo); closes the only hard-blocked prerequisite with no progress; produces the first live model contact in the slice's history | none — no approval required |
| B | Comparative experiment: same fixture through the existing harness and through `antares tool query --stdin` | Converts the Element 3 decision from speculation into measurement. The existing harness needs a translation layer first, which is itself the measurement of what adopting the CLI would save | none — read-only evaluation, artifacts only |
| C | T3c-1 (**done 2026-08-05**) -> T3c-2 (**done 2026-08-06**) -> T3d (**done 2026-08-06**) — **Phase C complete, T3 fully closed** | Invocation-independent; already scored, phase-1 reviewed, and handoff-ready. Ran in parallel with A/B without colliding | explicit approval per task, all granted and closed (T3c-1 RRI 55 Med-high; T3c-2 RRI 49 Med-high; T3d RRI 51 Med-high, owner-verified 2026-08-06) |
| D | Element 3 — **complete 2026-08-05** (Subtask A: adopt CLI, retire harness live-invocation role, T2a-T2e narrowed to test-only; Subtask B: `dispatch_via_cli` implemented in `harness.py`; Subtask C: T2a-T2e disposition docs synced) | Decomposed under the RRI 58 Complex pre-decomposition gate (`docs/audit/antares-t4-element3-rri.md`); Subtask A closed the architecture decision, Subtask B implemented it, Subtask C synced T2a-T2e disposition docs | none — all three subtasks closed |
| E | T4 pilot, then T5 | Unblocked only once A, C, and D have all landed | existing slice gates |

**Recommended ordering:** A before C, even though C is more execution-ready.
Each additional Med-high layer built on the T2 invocation path increases the
cost of a Phase D decision that retires it. The cost of inverting the order
is roughly one session of delay; the cost of not inverting it is continued
accumulation on an unvalidated base.

**Honest counter-argument:** C survives Phase D regardless, so running A and
C in parallel loses nothing structurally. Prefer parallel execution if slice
momentum matters more than sequencing cleanliness.

### Decision points

| Decision | Resolved by | Currently blocked on |
|---|---|---|
| Does the existing T2 harness work against real Antares output? | **Resolved 2026-08-05 — No.** `docs/evaluations/antares-phase-b-comparison.md`: three real-wire-format inputs (`name`/`arguments`, tag-wrapped) were rejected as `MALFORMED_TOOL_CALL` by the unmodified harness; the internal `tool`/`payload` schema it does accept is validated only against its own synthetic fixtures. No translation layer exists anywhere in `scripts/antares/*`. | — closed |
| Adopt Cisco's CLI (Element 3) or keep the own-harness path? | **Resolved 2026-08-05 — adopt the CLI, implemented.** Subtask A (RRI 26 Moderate, approved) decided the route; Subtask B (RRI 50 Med-high, approved) implemented `dispatch_via_cli` in `harness.py`. `docs/audit/antares-t4-element3-rri.md` §§ Subtask A / Subtask B — closure records. | — closed |
| Do `tool_call_parser.py` / `terminal_state.py` narrow to test-only scope, retire, or stay? | **Resolved 2026-08-05 — narrow to test-only.** Both modules keep serving `replay_fixtures.py` / `harness_test.py` unchanged; no live-invocation role. Same Subtask A decision. | — closed |
| Is local `antares-1b` fast enough on this host for routine use? | T1 R5 metrics (cold start, latency, peak RSS, swap growth) | Elements 1 + 2 |

### Approval boundary

- **Elements 1 + 2 (Phase A):** no approval required. Personal, reusable
  tooling outside this repository, nothing tracked, no governed path touched.
- **Phase B:** no approval required. Read-only comparative evaluation that
  emits artifacts; it changes no tracked code.
- **T3c-1 and successors (Phase C) — complete:** explicit human approval per
  task, per their RRI bands, all granted. T3c-1 approved 2026-08-05
  ("aprobado"), `[x] Done`. T3c-2 approved 2026-08-06 ("aprobado"),
  `[x] Done` (RRI 49 Med-high; ADR-038 routed `GO_LOCAL`, the bounded local
  session hit `budget_exhausted`, escalated to and implemented by cloud). T3d
  approved 2026-08-06 ("aprobado"), `[x] Done (owner-verified, 2026-08-06)`
  (RRI 51 Med-high; ADR-038 routed `GO_LOCAL`, the bounded local session
  again hit `budget_exhausted`, escalated to and implemented by cloud;
  phase-2 review `PASS`, 0 findings). This closes Phase C's task chain and
  T3 as a whole (all of T3a-T3d `[x] Done`); T4's dependency on T3 is now
  resolved, leaving T4 gated only on T1's separate runtime prerequisites.
- **Element 3 (Phase D) — complete 2026-08-05:** decomposed under RRI 58
  Complex (`docs/audit/antares-t4-element3-rri.md`). Subtask A (route
  decision, RRI 26) approved and closed. Subtask B (implementation, RRI 50
  after a mid-implementation scope correction from an initial RRI 43)
  approved and closed — `dispatch_via_cli` implemented in `harness.py`.
  Subtask C (T2a-T2e disposition doc sync, RRI 18) approved and closed. All
  three subtasks closed; Element 3 requires no further governed work.

## Open decisions

- Exact target location for the personal shim + CLI install (not this
  repo) — user's call, not blocking any dubbridge-tracked work.
- Whether `tool_call_parser.py`/`terminal_state.py` narrow to test-only
  scope or are retired outright once Element 3 lands — defer to that
  task's own Reflection/review, not decided here.

## Related documents

- `docs/tasks/antares-security-specialist-advisor.md` (T1 recovery plan
  R1-R5; T2a wire-format docstring this plan falsifies)
- `docs/plan/antares-security-specialist-advisor.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/audit/antares-t3c-1-rri.md` (RRI precedent for `arch_decision`
  penalty on `scripts/antares/*` changes)

## Sources

- `assets/antares-cli.zip` inside `fdtn-ai/antares-1b` revision
  `10417eb35641b32e7141157db19c76eb545193b6` (Hugging Face, gated repo).
- Local extraction: `.antares-runtime/antares-cli-reference/` (gitignored;
  `LICENSE`, `THIRD_PARTY_NOTICES.md`, `pyproject.toml`, `README.md`,
  `src/antares_cli/agent/{contracts,streaming,model_adapter,loop}.py`,
  `src/antares_cli/inference/{backend,remote,defaults,profiles}.py` read
  in full during this session.
