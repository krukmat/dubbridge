---
type: Evaluation
title: "Antares Phase B — comparative experiment (harness vs. antares-cli)"
status: complete
date: 2026-08-05
plan: docs/plan/antares-local-runtime-adoption.md
---

# Antares Phase B — Comparative Experiment

## Result

`Path A (antares tool query --stdin) completes. Path B (scripts/antares/harness.py::dispatch_tool_call) cannot consume real Antares wire-format output today — confirmed empirically, not by inference.`

This closes the Decision-points table row "Does the existing T2 harness work
against real Antares output?" in
`docs/plan/antares-local-runtime-adoption.md` with: **no**.

## Scope and boundary

- Read-only evaluation. No approval required per the plan's § "Approval
  boundary": "Phase B: no approval required. Read-only comparative
  evaluation that emits artifacts; it changes no tracked code."
- No file under `scripts/antares/*.py` was modified. No tracked production
  code was modified. This artifact and its evidence files are the only
  output.
- Snapshot: repository `HEAD` at `b8cd690bd451b585c675f13e89e2cd7396755bd1`,
  2026-08-05.

## Fixture

Same fixed fixture used for T1 R4/R5 closure
(`docs/evaluations/antares-runtime-preflight.md` "R4/R5 execution record"),
scoped to one tracked root for a direct, comparable single-invocation
result:

- Scope: `crates/` (tracked root)
- CWE ID: `CWE-20` (`Improper Input Validation`)
- Profile: `antares-local` (Ollama-backed, Element 2)

## Path A — `antares tool query --stdin`

### Command run

```bash
echo '{"target":"/Users/matias/dubbridge/crates","cwe_ids":["CWE-20"],"profile":"antares-local"}' \
  | antares tool query --stdin
```

### Result

- Exit code: `0`
- `incomplete_reason`: `null`
- `tool_call_count`: `15`
- `failed_tool_calls`: `4`
- `duration_seconds`: `37.90`
- `generation_errors`: `0`
- Finding: `Improper Input Validation`, `connectors/src/lib.rs`, `CWE-20`,
  `High` likelihood

This matches the shape of the earlier R4/R5 `crates/` run
(`api/src/ingestion_service.rs` finding on the combined `apps/`+`crates/`
scope; this run is `crates/`-only, hence a different single file). Full raw
JSON output preserved at
`/private/tmp/claude-501/-Users-matias-dubbridge/2ed3d913-6f35-4c41-a051-1ab24246b60a/scratchpad/phase-b-path-a-result.json`
(session-scoped scratch path, not part of this repository).

### What the CLI's output contains — and does not contain

The CLI's JSON result is a **finished summary**: `summary`, `findings`,
`metadata` (model/backend/profile config, reproducibility hash, full
model/profile spec). It does not expose the raw per-tool-call `<tool_call>`
wire-format stream to the caller at all — that parsing happens entirely
inside the CLI's own process (`agent/streaming.py`,
`agent/model_adapter.py`, confirmed by source inspection in the governing
plan's "Wire-format ground truth" section). There is nothing in this
output for `scripts/antares/harness.py` to consume even if it wanted to
intercept individual tool calls; the CLI's automation contract is
request-in / final-result-out, not a per-call hook.

## Path B — `scripts/antares/harness.py::dispatch_tool_call`

Three inputs were run against the unmodified harness, using the real
`crates/`-tree repository root as `snapshot_root` (Python 3.11,
`/opt/homebrew/bin/python3.11`, matching the module's own
`from __future__ import annotations` dataclass requirements — the system
`python3` 3.9 cannot load this module due to a `sys.modules` registration
order issue unrelated to this experiment).

### Case 1 — real Cisco wire format (`name`/`arguments`)

Per the plan's "Wire-format ground truth" section, Cisco's reference parser
reads tool name from `tool` or `name` and arguments from `args` or
`arguments`. This case sends exactly that shape, unwrapped (no
`<tool_call>` tags):

```json
{"name": "terminal", "arguments": {"argv": ["cat", "src/main.rs"]}}
```

Result:

```
TerminalState(kind=<TerminalStateKind.MALFORMED_TOOL_CALL: 'malformed_tool_call'>,
  detail="Tool-call payload is missing a string 'tool' field.")
```

The harness's parser requires the key literally named `tool`; it does not
accept `name` as a synonym. Rejected before ever reaching argv validation.

### Case 2 — internal schema (`tool`/`payload`) — for contrast

```json
{"tool": "terminal", "payload": {"argv": ["cat", "src/main.rs"]}}
```

Result: parses successfully and proceeds past `tool_call_parser` and
`command_policy` into the sandbox execution layer, where it is rejected for
an unrelated, pre-existing platform reason
(`SANDBOX_RUNTIME_UNAVAILABLE`: `RLIMIT_CPU`/`RLIMIT_AS`/`RLIMIT_NPROC`
unavailable on this host — a macOS resource-limit gap, not a wire-format
issue). This confirms the harness's parser layer works correctly against
the schema it was actually built for; the failure in Case 1 is specifically
a wire-format mismatch, not a general harness malfunction.

### Case 3 — full real wire format, tag-wrapped

```
<tool_call>{"name": "terminal", "arguments": {"argv": ["cat", "src/main.rs"]}}</tool_call>
```

Result:

```
TerminalState(kind=<TerminalStateKind.MALFORMED_TOOL_CALL: 'malformed_tool_call'>,
  detail='Tool-call payload is not valid JSON: Expecting value: line 1 column 1 (char 0)')
```

`parse_tool_call` calls `json.loads` directly on the raw string; it has no
tag-stripping step for `<tool_call>`/`<done>`/`<answer>`/`<think>` framing
at all. Fails at the JSON-parsing stage, before field-name matters.

## Comparison summary

| | Path A (`antares tool query --stdin`) | Path B (`harness.dispatch_tool_call`) |
|---|---|---|
| Completes against real Antares invocation | Yes — exit 0, genuine finding | No — every real-wire-format shape tested is rejected |
| Consumes real `<tool_call>` wire format | N/A — parsing is internal to the CLI, never exposed to the caller | No — requires `{"tool": ..., "payload": ...}`, has no tag-stripping, does not accept `name`/`arguments` synonyms |
| Schema actually implemented | Cisco's real model-facing contract (`agent/model_adapter.py`, `agent/streaming.py`) | An internal schema invented for T2, validated only against `replay_fixtures.py`'s own synthetic fixtures (same schema, circular) |
| Gap to close Path B | None — already works | A translation layer (tag-stripping + `name`/`args` normalization) would need to be written; no such layer exists anywhere in `scripts/antares/*` today (confirmed: no module references `<tool_call>`, `name`, or `args`/`arguments` outside this experiment's ad hoc test) |

## Explicit statement (acceptance criterion)

**The harness path cannot consume real Antares wire-format output today.**
This is evidence, not inference: three concrete real-shaped inputs were run
against the unmodified `dispatch_tool_call` and all three were rejected as
`MALFORMED_TOOL_CALL` before reaching any business logic. Case 2 rules out
"the harness is just broken" as an alternative explanation — the same
function correctly processes its own internal schema and fails later, for
an unrelated platform reason.

This artifact does not fabricate or add a translation layer. Per the
handoff's stop condition, doing so is explicitly out of scope for Phase B.

## What this does not decide

- Whether Element 3 adopts the CLI subprocess path outright or writes a
  translation layer for the existing harness — that is Phase D's decision,
  informed by this evidence.
- The disposition of `tool_call_parser.py` / `terminal_state.py` /
  T2a–T2e — Phase D's own scope decision, not implicit here.
- Nothing about `SANDBOX_RUNTIME_UNAVAILABLE` (Case 2) is in scope to fix;
  it is reported only as evidence that Case 1/3's rejection is
  wire-format-specific, not a general harness failure on this host.

## Related documents

- `docs/plan/antares-local-runtime-adoption.md` — governing plan, Element 3,
  "Blocking finding" section, Decision-points table
- `docs/evaluations/antares-runtime-preflight.md` — R4/R5 execution record
  (Path A's command and fixture provenance)
- `docs/tasks/handoff-antares-phase-b-2026-08-05.md` — this task's handoff
- `docs/tasks/handoff-antares-element3-2026-08-05.md` — the blocked
  successor task this evidence unblocks
