#!/usr/bin/env python3
"""Argument parsing and process entry point for the local agentic runner.

Extracted from run_local_task.py (LRPC-0b, Extract Module / Single
Responsibility). Behavior-preserving: no logic changed from the original
module.
"""

from __future__ import annotations

import argparse
import datetime
import json
import os

import fallback_selection
import gemma_local

from audit_record import (
    build_attempt_bundles,
    build_audit_record,
    build_moderate_fallback_checkpoint,
    build_terminal_attempt_packet,
)
from prompt_builder import build_system_prompt
from rust_toolchain import build_default_boundary, build_default_formatter
from runner_file_tools import ALLOWED_TOOL_NAMES, RunnerFileTools
from session_loop import BoundaryViolation, MalformedToolCall, run_loop

# Mirrors run_local_task.py's MODEL_CONTEXT_TOKENS / GENERATION_TOKEN_BUDGET
# module-level defaults. Duplicated here rather than imported to avoid the
# circular import (run_local_task.py already does `import cli`); both sides
# are the same ADR-036 local-implementer defaults, not two independent
# guesses.
_DEFAULT_MODEL_CONTEXT_TOKENS = 65536
_DEFAULT_GENERATION_TOKEN_BUDGET = 8192

# Output-format contract local to this script's tool-call transport: no
# canonical-doc source, so it is not sourced from prompt_anchors.py (LRPC-4,
# docs/tasks/local-role-prompt-canonicalization.md). Only the boundary clause
# above it (now supplied by build_system_prompt's "local_developer" anchor)
# has a canonical source.
_TOOL_CALLING_OUTPUT_FORMAT_TEXT = """\
You are the bounded implementer for one fully specified task. The operator has \
already selected the files, requirements, and acceptance commands. Do not \
explore the repository, choose a different design, or expand the task.

This session has a hard budget of {MAX_TOTAL_TURNS} turns total. Prioritize \
focused edits followed by `finish`.

The complete contents of every authorized existing file are included below. \
Use that supplied context directly. You do not inspect the repository or run \
commands.

You act only by responding with a single JSON object \
of the exact form:

{"tool_calls": [{"function": {"name": "<tool>", "arguments": {<tool-specific fields>}}}]}

"arguments" is a nested JSON object (not a string).

Respond with ONLY that JSON object — no prose before or after it, no markdown \
code fences.

Available tools:
- write_file: arguments {"path": "<repo-relative path>", "content": "<full file contents>"}. \
Creates a new file or overwrites an existing one with exactly the content you supply.
- apply_patch: arguments {"path": "<repo-relative path>", "anchor": "<exact existing text>", "replacement": "<replacement text>"}. \
Replaces exactly one occurrence of "anchor" (which must appear exactly once in the file) \
with "replacement". Use this for a focused edit to a large file instead of rewriting it \
whole. If the anchor is not unique, use more surrounding text from the supplied context.
- finish: arguments {}. Signals you believe the task is complete; this triggers the \
runner-controlled formatter and acceptance tests. If they fail, you receive the \
failure output plus refreshed authorized files and get a bounded repair turn.

Typical workflow: inspect the supplied authorized file contents, make the required \
focused edits with apply_patch or write_file, then call finish. The runner alone \
formats edited Rust files and runs the full operator-authored acceptance suite.

Call exactly one tool per turn. Only call finish once you believe the acceptance \
tests described in your task will pass.
"""

# Prepended to every card's own spec as the system message. The model is not
# given native tool-calling (see build_live_chat_fn's docstring) — it must be
# told, in plain text, the exact JSON shape parse_tool_call() expects, or it
# replies with ordinary prose and every turn is bounced as a malformed tool
# call.
#
# The boundary clause (allowed_paths / boundary_violation) is now sourced
# from prompt_anchors.ROLE_ANCHORS["local_developer"] via build_system_prompt
# (LRPC-4), not hardcoded — closing the drift risk this plan was opened to
# fix (docs/plan/local-role-prompt-canonicalization.md). Built once at
# import time against this module's own defaults, matching this constant's
# pre-existing module-level-string shape so every caller/test that reads
# TOOL_CALLING_SYSTEM_PROMPT as a plain string keeps working unchanged.
# PromptBudgetExceeded is intentionally not caught here: propagates uncaught
# to any import-time failure, matching prompt_builder's own fail-closed
# contract and LRPC-3's identical precedent for gemma-code-review.py.
TOOL_CALLING_SYSTEM_PROMPT = build_system_prompt(
    role="local_developer",
    num_ctx=_DEFAULT_MODEL_CONTEXT_TOKENS,
    num_predict=_DEFAULT_GENERATION_TOKEN_BUDGET,
    output_format_text=_TOOL_CALLING_OUTPUT_FORMAT_TEXT,
)

# Passed as Ollama's `format` request field (constrained/structured-output
# decoding): confirmed via web research that small/medium local models
# reliably drop or miscount braces in free-form deeply-nested tool-call JSON
# (an Ollama-tracked qwen3 issue reports the identical symptom) — the fix
# documented for this class of problem is schema-constrained decoding, which
# makes malformed JSON impossible at the token level rather than merely less
# likely via prompt wording. Deliberately loose on "arguments" (an open
# object, not a per-tool shape): a stricter per-tool schema would need
# oneOf/if-then-else, which isn't reliably supported by every constrained-
# decoding backend, and per-tool argument validation (e.g. a missing "path")
# is already handled by parse_tool_call/require_argument regardless.
TOOL_CALL_JSON_SCHEMA = {
    "type": "object",
    "properties": {
        "tool_calls": {
            "type": "array",
            "minItems": 1,
            "maxItems": 1,
            "items": {
                "type": "object",
                "properties": {
                    "function": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "enum": list(ALLOWED_TOOL_NAMES),
                            },
                            "arguments": {"type": "object"},
                        },
                        "required": ["name", "arguments"],
                    },
                },
                "required": ["function"],
            },
        },
    },
    "required": ["tool_calls"],
}


def build_live_chat_fn(
    host,
    model,
    idle_timeout,
    max_wall,
    *,
    num_predict,
    num_ctx,
    max_total_turns,
):
    """Adapt gemma_local's single-shot stream_chat to this loop's per-turn chat_fn shape.

    Each call is one /api/chat turn with the full running message list; Ollama
    (like OpenAI-compatible chat APIs) is stateless per-request, so the whole
    transcript is resent every turn.

    This does NOT use Ollama's native tool-calling (`tools=[...]` request
    field / native `tool_calls` response field) — the system prompt must
    instruct the model to emit the `{"tool_calls": [...]}` JSON contract as
    plain text content instead. Native tool-calling support and reliability
    varies across locally-served models; the text-JSON contract is simpler to
    validate and matches what `parse_tool_call` below expects either way.
    """
    resolved_model = gemma_local.ensure_model_available(host, model, idle_timeout)
    url = gemma_local.endpoint(host, "/api/chat")
    # Mutable across calls: each chat_fn(messages) invocation is exactly one
    # run_loop turn, so counting calls here gives the live token-streaming
    # progress line a "turn N/MAX_TOTAL_TURNS" label -- without it, a full
    # write_file generation of a several-hundred-line file (confirmed live:
    # ~3 minutes at local-model throughput) prints only a token count that
    # resets to zero every turn, indistinguishable from a stalled process to
    # an operator watching stderr.
    turn_counter = {"n": 0}

    def chat_fn(messages):
        turn_counter["n"] += 1
        payload = {
            "model": resolved_model,
            "stream": True,
            "think": False,
            "format": TOOL_CALL_JSON_SCHEMA,
            "keep_alive": "10m",
            "messages": messages,
            "options": {
                # Confirmed live (S-140-T2b-i pilot, 2026-07-22): with no
                # explicit num_predict, Ollama's server-side default cut a
                # real apply_patch tool call mid-JSON (done_reason="length")
                # with ~14k tokens of file context in play — the model still
                # had a large "replacement" string left to emit. write_file
                # and apply_patch have no size cap in this tool contract, so the output
                # budget must comfortably exceed one full file's worth of
                # text, not just a short tool-call envelope.
                "num_predict": num_predict,
                "num_ctx": num_ctx,
            },
        }
        result = gemma_local.stream_chat(
            url,
            payload,
            idle_timeout,
            max_wall,
            progress_label=f"local-agent turn {turn_counter['n']}/{max_total_turns}",
        )
        content = gemma_local.stream_result_content(result)
        try:
            return json.loads(content)
        except json.JSONDecodeError as exc:
            # D14 finding: without the raw content, the transcript records only
            # the generic decode-error position, losing the actual model text
            # that triggered the bounce — the exact signal an unattended run
            # needs to diagnose why the model didn't follow the tool-call format.
            raise MalformedToolCall(
                f"non-JSON model response: {exc}; raw content: {content!r}"
            ) from exc

    return chat_fn


def load_card(card_path, task_card_cls):
    with open(card_path, encoding="utf-8") as f:
        data = json.load(f)
    return task_card_cls(
        task_id=data["task_id"],
        spec=data["spec"],
        acceptance_tests=data.get("acceptance_tests", []),
        allowed_paths=data.get("allowed_paths", []),
        rri=data.get("rri"),
        band=data.get("band"),
        capsule_hash=data.get("capsule_hash"),
    )


def parse_args(argv, *, default_num_ctx, default_num_predict):
    parser = argparse.ArgumentParser(
        description="Run a bounded local-agent draft/test/repair loop over a task card.",
    )
    parser.add_argument("--card", required=True, help="Path to the task card JSON.")
    parser.add_argument("--worktree", required=True, help="Path to the isolated worktree.")
    parser.add_argument("--out", required=True, help="Path to write the transcript artifact.")
    parser.add_argument(
        "--host",
        default=os.environ.get("OLLAMA_HOST", gemma_local.DEFAULT_HOST),
        help=f"Ollama host; defaults to OLLAMA_HOST or {gemma_local.DEFAULT_HOST}.",
    )
    parser.add_argument(
        "--model",
        default=os.environ.get("DUBBRIDGE_LOCAL_AGENT_MODEL"),
        help=(
            "Local implementer model tag. When omitted, the task band selects "
            "the ADR-036 binding after the card is loaded."
        ),
    )
    parser.add_argument(
        "--idle-timeout",
        type=int,
        default=180,
        help="Seconds without a new token before treating the model as stalled.",
    )
    parser.add_argument(
        "--max-wall",
        type=int,
        default=1800,
        help="Maximum wall-clock seconds for a single chat turn.",
    )
    parser.add_argument(
        "--num-ctx",
        type=int,
        default=default_num_ctx,
        help=f"Ollama context window per turn; defaults to {default_num_ctx}.",
    )
    parser.add_argument(
        "--num-predict",
        type=int,
        default=default_num_predict,
        help=(
            "Ollama generation-token budget per turn; defaults to "
            f"{default_num_predict}."
        ),
    )
    parser.add_argument(
        "--max-turns",
        type=int,
        default=None,
        help=(
            "Optional tighter session turn cap. It may not exceed the "
            "band-resolved limit."
        ),
    )
    fallback_selection.add_cli_arguments(parser)
    return parser.parse_args(argv)


def main(
    argv=None,
    chat_fn=None,
    test_runner=None,
    boundary=None,
    *,
    task_card_cls,
    resolve_effective_limits,
    build_default_test_runner,
    max_malformed_bounces,
    tool_calling_system_prompt=TOOL_CALLING_SYSTEM_PROMPT,
    default_num_ctx,
    default_num_predict,
    default_local_agent_model,
):
    args = parse_args(
        argv, default_num_ctx=default_num_ctx, default_num_predict=default_num_predict
    )
    card = load_card(args.card, task_card_cls)
    boundary = boundary or build_default_boundary(args.worktree, card)
    limits = resolve_effective_limits(card)
    if args.model is None:
        args.model = limits.required_model or default_local_agent_model(card)
    if not limits.local_execution_allowed:
        result = {
            "status": "local_execution_rejected",
            "reason": "RRI 46-55 Med-high cards are cloud-only for whole-task execution.",
            "transcript": [],
            "task_id": card.task_id,
            "finished_at": datetime.datetime.now(datetime.timezone.utc).strftime(
                "%Y-%m-%dT%H:%M:%SZ"
            ),
        }
        gemma_local.write_result(result, args.out)
        gemma_local.append_audit_log(
            build_audit_record(card, result, args.model, 0.0, effective_limits=limits)
        )
        return 1
    for flag, value in (("--num-ctx", args.num_ctx), ("--num-predict", args.num_predict)):
        if value <= 0:
            raise ValueError(f"{flag} must be greater than zero")
    if args.max_turns is not None:
        if args.max_turns <= 0:
            raise ValueError("--max-turns must be greater than zero")
        if args.max_turns > limits.max_total_turns:
            raise ValueError(
                "--max-turns may tighten but not exceed the band-resolved "
                f"limit ({limits.max_total_turns})"
            )
        limits.max_total_turns = args.max_turns

    # ADR-038 T3 EC-2: a Med-high card must run under the exact required
    # model -- no silent substitution. This is a routing-evidence check, not
    # a capability check: --model defaults to the same
    # nemotron-3.5-lightning:30b-a3b-q4_K_M tag, so this only ever fires when
    # a caller explicitly overrides --model for a card the gate (T2) already
    # routed to Med-high local implementation.
    if limits.required_model and args.model != limits.required_model:
        result = {
            "status": "model_substitution_rejected",
            "reason": (
                f"Med-high card requires model {limits.required_model!r}, "
                f"got {args.model!r}."
            ),
            "transcript": [],
            "task_id": card.task_id,
            "finished_at": datetime.datetime.now(datetime.timezone.utc).strftime(
                "%Y-%m-%dT%H:%M:%SZ"
            ),
        }
        gemma_local.write_result(result, args.out)
        audit_record = build_audit_record(
            card, result, args.model, 0.0, effective_limits=limits
        )
        gemma_local.append_audit_log(audit_record)
        return 1

    def checkpoint_fn(transcript, turn):
        # Overwritten by the real terminal write_result() call below once
        # run_loop returns. If the process is killed mid-session instead
        # (SIGINT/SIGKILL during a slow local-model turn), this is what's
        # left on disk -- previously nothing was, since write_result() only
        # ran once, after run_loop returned; an interrupted run left zero
        # trace in --out and zero rows in the audit log.
        gemma_local.write_result(
            {
                "status": "in_progress",
                "task_id": card.task_id,
                "turn": turn,
                "max_turns": limits.max_total_turns,
                "effective_limits": limits.as_dict(),
                "transcript": transcript,
            },
            args.out,
        )

    session_start = datetime.datetime.now(datetime.timezone.utc)
    chat_fn = chat_fn or build_live_chat_fn(
        args.host,
        args.model,
        args.idle_timeout,
        args.max_wall,
        num_predict=args.num_predict,
        num_ctx=args.num_ctx,
        max_total_turns=limits.max_total_turns,
    )
    # The missing fallback (chat_fn and boundary above both had theirs): the
    # CLI never injects test_runner, so without this it stayed None and every
    # finish crashed with TypeError. See build_default_test_runner's docstring.
    test_runner = test_runner or build_default_test_runner(card, boundary)
    file_tools = RunnerFileTools(
        args.worktree, boundary, MalformedToolCall, BoundaryViolation
    )
    formatter_fn = build_default_formatter(card, boundary, file_tools)
    try:
        result = run_loop(
            card,
            chat_fn,
            test_runner,
            args.worktree,
            boundary,
            file_tools,
            checkpoint_fn=checkpoint_fn,
            limits=limits,
            formatter_fn=formatter_fn,
            resolve_effective_limits=resolve_effective_limits,
            max_malformed_bounces=max_malformed_bounces,
            tool_calling_system_prompt=tool_calling_system_prompt,
        )
    finally:
        file_tools.close()
    session_end = datetime.datetime.now(datetime.timezone.utc)
    elapsed_s = (session_end - session_start).total_seconds()

    result["task_id"] = card.task_id
    result["finished_at"] = datetime.datetime.now(datetime.timezone.utc).strftime(
        "%Y-%m-%dT%H:%M:%SZ"
    )
    audit_record = build_audit_record(
        card, result, args.model, elapsed_s, effective_limits=limits
    )
    if result["status"] == "success" and not audit_record["audit_validation"]["valid"]:
        result["status"] = "audit_invalid"
        result["reason"] = ";".join(audit_record["audit_validation"]["errors"])
        audit_record = build_audit_record(
            card, result, args.model, elapsed_s, effective_limits=limits
        )

    fallback_exit_code = None
    try:
        checkpoint = build_moderate_fallback_checkpoint(
            args, card, result, args.model, limits
        )
        if checkpoint is not None:
            terminal_packet = build_terminal_attempt_packet(
                card, result, args.model, limits
            )
            selection_artifact = (
                args.fallback_selection_artifact
                or fallback_selection.default_checkpoint_path(args.out)
            )
            fallback_selection.write_checkpoint(checkpoint, selection_artifact)
            result["terminal_attempt_packet"] = terminal_packet
            result["fallback_selection_artifact"] = selection_artifact
            result["fallback_selection"] = checkpoint
            fallback_exit_code = (
                fallback_selection.HUMAN_SELECTION_EXIT_CODE
                if checkpoint["status"] == fallback_selection.AWAITING_STATUS
                else 1
            )
    except fallback_selection.FallbackSelectionError as exc:
        result["fallback_selection_error"] = str(exc)
        fallback_exit_code = 2

    gemma_local.write_result(result, args.out)

    # Emitted for every exit path (success, aborted, budget_exhausted,
    # boundary_violation, transport_error) — audit visibility must not
    # depend on how the session ended.
    gemma_local.append_audit_log(audit_record)

    # T2: additive alongside the ADR-034 audit record above -- one T1
    # attempt bundle per repair attempt, appended to the same audit-log
    # sink (append_audit_log is generic over the record shape it appends).
    for bundle in build_attempt_bundles(card, result, args.model, session_start, session_end):
        gemma_local.append_audit_log(bundle)

    if fallback_exit_code is not None:
        return fallback_exit_code
    return 0 if result["status"] == "success" else 1
