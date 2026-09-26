---
type: Audit
title: "Local-execution routing evidence review — 2026-09-14"
status: complete
description: "Post-ADR-045 inventory of assessable Devstral authorship, forensic incident attribution, and evaluation of the gpt-oss reviewer profile plus Ollama runtime optimizers."
slice: local-agent-packet-hardening
---

# Local-Execution Routing Evidence Review — 2026-09-14

## Question

Does the repository's real execution history support expanding whole-task
local implementation beyond the currently authorized Low, Moderate, and
conditional RRI 41–45 routes?

## Method and evidence boundary

The inventory begins at the ADR-045 merge (`ffd7b88`, 2026-09-07
15:10:41 +02:00). The primary runtime index is
`logs/gemma-audit/2026-09.jsonl`, filtered to `role ==
"local-implementer"` after that merge time. Each in-scope row was checked
against its `.agent/` terminal result and the relevant task ledger.

The audit JSONL is append-only by the emitting code, but the two relevant
rows are unsigned. The `.agent/` cards and transcripts are also ignored
working artifacts rather than immutable committed evidence. Conclusions are
therefore limited to facts corroborated across the available records; this
report does not claim cryptographic provenance for historical runs.

Reproduction query:

```bash
jq -r 'select(.role=="local-implementer" and (.ts // "") >= "2026-09-07T13:10:41Z") | [.ts,.task_id,(.rri|tostring),(.band//""),(.model//""),(.outcome//""),(.attempts|tostring),(.verification_results.final_acceptance_passed|tostring)] | @tsv' logs/gemma-audit/2026-09.jsonl
```

## Post-migration Devstral corpus

| Invocation | Actual task/band | Model | Runtime outcome | Model turns / tests | Assessable authorship? |
|---|---|---|---|---|---|
| `P2.T2c-r1`, 2026-09-07 20:27:55Z | RRI 40 Moderate | `devstral-small-2:24b-instruct-2512-q4_K_M` | `TRANSPORT_ERROR` | 0 / none | No |
| `P2.T2c-r1`, 2026-09-07 20:43:19Z | RRI 40 Moderate | `devstral-small-2:24b-instruct-2512-q4_K_M` | `TRANSPORT_ERROR` | 0 / none | No |

Sources:

- `logs/gemma-audit/2026-09.jsonl` rows 82–83;
- `.agent/local-agent-p2-t2c-r1.json`;
- `.agent/local-agent-p2-t2c-r1-attempt2.json`;
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md` § `P2.T2c-r`.

Both records report zero total turns, zero repairs, zero tests, and no source
diff. They evaluate neither Devstral's code generation nor its ability to
repair a failing implementation. They only show that the local route failed
to obtain a response in those two invocations.

The terminal reason says `Gemma idle timeout` even though the recorded
implementer is Devstral. This is not evidence that Gemma was invoked: the
shared transport raises a class whose error text is hard-coded to “Gemma”
(`scripts/gemma_local.py::GemmaIdleTimeout`). It is an observability defect
and another reason not to infer the failing component from prose labels.

### Corpus conclusion

There are **zero assessable post-migration Devstral authoring outcomes** in
the available runtime log: no success, no model-authored defect, and no
model-authored patch that reached verification. The dataset is insufficient
to estimate Devstral quality in Moderate and contains no actual Devstral
execution in RRI 41–45.

## Why `P2.T3c-S1b` is not Devstral/Med-high evidence

The four repair cards under `.agent/p2-t3c/` declare RRI 25 Low. The three
terminal local-implementer audit rows name `qwen3.8:27b-mlx`:

| Leaf invocation | Audit model/band | Terminal evidence | What it can show |
|---|---|---|---|
| `P2.T3c-S1b-source-repair` | Qwen, Low | `OUT_OF_SCOPE` | scope/provenance behavior |
| `P2.T3c-S1b-validation-repair` | Qwen, Low | `BUDGET_EXHAUSTED` after invalid acceptance command | packet/verification contract behavior |
| `P2.T3c-S1b-validate-package-repair` | Qwen, Low | `BUDGET_EXHAUSTED` after formatting failure and a full-file rewrite | recovery/content-preservation risk |
| `P2.T3c-S1b-tree-compare-repair` | model absent from checkpoint | `in_progress`, one malformed event | one structured-output failure only |

The parent `P2.T3c-S1b` was RRI 55 Med-high, but it routed to cloud after its
decomposition decision. A Low repair leaf does not become a Med-high model
run because its parent had a higher RRI. Parent routing, executable-leaf
routing, and actual implementer identity must be recorded separately.

## Incident findings

### I1 — Prose executed as a command: confirmed packet/schema defect

The validation-repair card contains three English statements in
`acceptance_tests`. The transcript shows:

1. Qwen emitted an `apply_patch` call;
2. the patch applied successfully;
3. `finish` succeeded;
4. scope was in-bounds;
5. `rustfmt` passed;
6. the runner attempted argv beginning with `Validation` and received
   `Errno 2`.

This failure belongs to the packet/schema layer, not to model authorship.
`parse_acceptance_commands` validates quoting and shell composition but has
no semantic distinction between a criterion and an executable command.

Required design consequence: replace the ambiguous field for new cards with
separate typed `acceptance_criteria` and structured `verification_commands`.

### I2 — Allowed path reported out-of-scope: provenance gap confirmed,
scope matcher defect unproven

The current `s1b-source-repair-card.json` includes
`crates/p2p/src/lib.rs`; its transcript reports the same path as offending.
Running the present normalization/matching functions against that card accepts
the path.

Filesystem timestamps provide the missing explanation boundary:

```text
transcript  2026-09-13T05:16:49+0200
card        2026-09-13T05:17:07+0200
```

The card was modified 18 seconds after the transcript. Because neither file
contains a shared card hash, the exact `allowed_paths` loaded by the run cannot
be recovered. The transcript may reflect an earlier card, but that cannot be
proven from the artifacts.

Required design consequence: bind the loaded card hash and session-start
worktree state to every checkpoint/result. Do not modify `scope_check.py` on
the assumption that path matching failed.

### I3 — Long-anchor malformed JSON: one transport event confirmed,
exhaustion unproven

`s1b-tree-compare-repair-transcript.json` contains one
`malformed_tool_call` caused by `json.loads` rejecting an unterminated model
response while a long Rust `apply_patch` anchor was being emitted. Its status
is `in_progress`; it has no finish time, terminal reason, response length,
Ollama completion reason, or subsequent bounce.

Required design consequence: distinguish full-response decode from argument
decode and capture the completion/budget facts needed to decide whether the
cause was truncation, escaping, schema enforcement, or another transport
failure.

### I4 — Generic `write_file` fallback would increase risk

In `s1b-validate-package-repair-transcript.json`, the focused validation patch
applied and the edited source formatted. `cargo fmt --check` then reported a
formatting difference in `lib.rs`. On the repair turn, Qwen rewrote the full
file and omitted the existing `mod nonce_tracker;` export. The session ended
before another test could expose the regression.

That transcript does not establish that `write_file` is always unsafe. It
does establish that it is not a sound generic fallback for malformed long-
anchor JSON: it expands the output payload and can destroy unrelated valid
content.

Required design consequence: recovery must be cause-specific and prefer a
smaller, idempotent edit representation.

## Gaps in the evidence system

The forensic review had to join mutable cards, partial transcripts, an
unsigned audit log, filenames, filesystem timestamps, and prose closure
records. The current execution-normalization seam is a useful base, but the
historical/direct runner path does not yet guarantee:

- a shared execution-session ID across all artifacts;
- parent and executable-leaf identity/band;
- card schema version and SHA-256;
- top-level model identity in every checkpoint and terminal result;
- attested start revision and dirty-state snapshot;
- a session-owned before/after change set;
- distinct failure cause versus terminal consequence;
- response completion reason and size for malformed output;
- attribution of the failure owner;
- a queryable link from model authorship through final verification and
  fallback.

Without these fields, a routing decision can overcount transport failures as
model failures and Low decomposition leaves as higher-band executions.

## `gpt-oss:20b` Complex-review profile

The active RRI 56+ binding uses `num_ctx=49152`, `num_predict=8192`, and
`think=high`. Real repository evidence does not validate that profile:

| Review sequence | Profile | Outcome |
|---|---|---|
| `P2.T3c-Integ`, high attempt 1 | 49,152 context; 8,192 output; high | `done_reason: length`; empty content |
| `P2.T3c-Integ`, high attempt 2 | 49,152 context; 16,384 output; high | `done_reason: length`; empty content |
| `P2.T4b`, high attempt 1 | 49,152 context; 8,192 output; high | `done_reason: length`; 8,192 thinking tokens; empty content |
| `P2.T4b`, high reduced retry | 16,384 context; 1,024 output; high | `done_reason: length`; 1,024 thinking tokens; empty content |
| `P2.T3c-Integ`, completed attempt | 49,152 context; 10,240 output; medium | parsed `PASS`; `done_reason: stop`; 4,015 output tokens |

Sources are the `P2.T3c-Integ` closure record and its request/response under
`.agent/p2-t3c-integ/`, plus
`docs/audit/mvp0-p2p-p2-t4b-retrospective-review.json`. The successful attempt
followed an Ollama restart, so this is not a controlled proof that reasoning
level alone caused the difference. It is, however, repeated direct evidence
that high reasoning is liable to spend the entire generation budget before
emitting the required structured verdict, with no completed high verdict in
the reviewed corpus.

Disposition: keep `gpt-oss:20b` as the RRI 56+ primary reviewer, but recommend
`num_ctx=49152`, `num_predict=10240`, and `think=medium`. The canonical guide
and RRI policy still say high/8192; changing that governance binding requires
explicit approval and is tracked separately rather than silently applied.

## Flash Attention and KV-cache `q8_0`

Read-only host inspection on 2026-09-14 found Ollama 0.34.0 configured with:

```text
OLLAMA_FLASH_ATTENTION=1
OLLAMA_KV_CACHE_TYPE=q8_0
OLLAMA_MAX_LOADED_MODELS=1
```

Ollama's current documentation says Flash Attention reduces memory use as
context grows; KV-cache quantization is available when Flash Attention is
enabled; and `q8_0` uses approximately half the KV-cache memory of `f16` with
a comparatively small quality impact. This matches the local workload: the
active Devstral implementer is configured for 128K context and Complex
`gpt-oss` reviews request about 49K. Limiting residency to one model is also
appropriate for the 32 GB host when the active model artifacts are 13–18 GB.

No model was loaded during inspection (`ollama ps` was empty), so this audit
proves requested host configuration, not effective per-session application.
The normalized result should record the effective KV type, Flash Attention
state, context allocation, loaded model, and capacity failure evidence.

The optimization boundary is explicit:

- it can improve memory fit and reduce long-context attention cost;
- it may reduce resource pressure, but cannot retroactively reclassify a
  transport timeout as a model-quality result;
- it cannot distinguish prose from commands, bind a card hash, fix scope
  attribution, or make malformed tool JSON valid;
- it does not stop `think=high` from consuming `num_predict` on hidden
  reasoning before a verdict is emitted.

Therefore retain Flash Attention plus `q8_0` as baseline capacity settings,
but grant them no routing-band credit.

## Not-installed model candidate

The strongest fit for a future replacement consideration is
`qwen3-coder:30b`: its published 30B-A3B form uses 3.3B active parameters,
the Ollama artifact is approximately 19 GB, its advertised context is 256K,
and it is explicitly aimed at agentic coding and tool use. It plausibly fits
the 32 GB host with one-model residency.

This is a fit assessment, not a repository capability finding. The model is
not installed, and Devstral has no assessable post-migration authoring result.
The defensible sequence is to repair ordinary-run evidence first, then allow
Qwen3-Coder to be considered on an already-approved bounded coding task; no
synthetic pilot or vendor-claim promotion is created.

## Routing conclusion

The evidence supports no expansion of whole-task local authorship today:

- keep Low and Moderate local-first;
- keep RRI 41–45 local only after the existing `GO_LOCAL` gate;
- keep RRI 46–55 local-decomposition-only;
- keep RRI 56+ local for architecture/review roles, not implementation;
- keep cloud as the last-resort author of inseparable above-Low residue.

This conclusion does not assert that Devstral is incapable. It says its
post-migration implementation capability is currently **unmeasured** by the
available evidence. Normal work can supply that evidence once task cards,
session provenance, runtime optimization state, and failure attribution are
trustworthy. Flash Attention and KV `q8_0` remain enabled as capacity
optimizers, not evidence of authoring capability. Separately, the Complex
`gpt-oss` reviewer profile should move from high to medium reasoning after the
required policy approval.

## Related

- `docs/plan/local-agent-packet-hardening.md`
- `docs/tasks/local-agent-packet-hardening.md`
- `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md`
- `docs/adr/ADR-045-devstral-local-implementer-binding.md`
- `docs/adr/ADR-047-software-factory-execution-normalization-seam.md`
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
- `docs/audit/mvp0-p2p-p2-t4b-retrospective-review.json`
