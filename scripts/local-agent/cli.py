#!/usr/bin/env python3
"""Argument parsing and process entry point for the local agentic runner."""

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
from context_budget import derive_invocation_budget
from context_provider import CKGContextProvider, FallbackContextProvider, LegacyContextProvider
from ollama_lifecycle import unload_model
from prompt_builder import build_system_prompt
from rust_toolchain import build_default_boundary, build_default_formatter
from runner_file_tools import ALLOWED_TOOL_NAMES, RunnerFileTools
from session_loop import BoundaryViolation, MalformedToolCall, run_loop

_DEFAULT_MODEL_CONTEXT_TOKENS = 32768
_DEFAULT_GENERATION_TOKEN_BUDGET = 8192

_TOOL_CALLING_OUTPUT_FORMAT_TEXT = """\
You are the bounded implementer for one fully specified task. The operator has \
already selected the files, requirements, and acceptance commands. Do not \
explore the repository, choose a different design, or expand the task.

This session has a hard budget of {MAX_TOTAL_TURNS} turns total. Prioritize \
focused edits followed by `finish`.

The operator supplies the authorized source context needed for this task. It \
may contain selected regions rather than every file under `allowed_paths`. Use \
that supplied context directly. Do not infer that an omitted authorized file \
is necessary or safe to modify, and do not inspect the repository or run commands.

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
failure output plus refreshed authorized source context and get a bounded repair turn.

Typical workflow: inspect the supplied authorized source context, make the required \
focused edits with apply_patch or write_file, then call finish. The runner alone \
formats edited Rust files and runs the full operator-authored acceptance suite.

Call exactly one tool per turn. Only call finish once you believe the acceptance \
tests described in your task will pass.
"""


def build_tool_calling_system_prompt(num_ctx, num_predict):
    return build_system_prompt(
        role="local_developer",
        num_ctx=num_ctx,
        num_predict=num_predict,
        output_format_text=_TOOL_CALLING_OUTPUT_FORMAT_TEXT,
    )


TOOL_CALLING_SYSTEM_PROMPT = build_tool_calling_system_prompt(
    _DEFAULT_MODEL_CONTEXT_TOKENS,
    _DEFAULT_GENERATION_TOKEN_BUDGET,
)

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
                            "name": {"type": "string", "enum": list(ALLOWED_TOOL_NAMES)},
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
    resolved_model = gemma_local.ensure_model_available(host, model, idle_timeout)
    url = gemma_local.endpoint(host, "/api/chat")
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
            "options": {"num_predict": num_predict, "num_ctx": num_ctx},
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
        description="Run a bounded local-agent draft/test/repair loop over a task card."
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
        help="Local implementer model tag. When omitted, the task band selects the binding.",
    )
    parser.add_argument("--idle-timeout", type=int, default=180)
    parser.add_argument("--max-wall", type=int, default=1800)
    parser.add_argument("--num-ctx", type=int, default=default_num_ctx)
    parser.add_argument("--num-predict", type=int, default=default_num_predict)
    parser.add_argument(
        "--max-turns",
        type=int,
        default=None,
        help="Optional tighter session turn cap; may not exceed the band-resolved limit.",
    )
    parser.add_argument(
        "--context-provider",
        choices=("auto", "legacy"),
        default=os.environ.get("DUBBRIDGE_CONTEXT_PROVIDER", "auto"),
        help="Context source: auto tries local CKG then fails back to legacy preload.",
    )
    parser.add_argument(
        "--ckg-manifest",
        default=None,
        help="Optional ckg-context-manifest-v1 output path; defaults beside --out.",
    )
    fallback_selection.add_cli_arguments(parser)
    return parser.parse_args(argv)


def _context_provider_for(
    args,
    card,
    boundary,
    file_tools,
    runtime_prompt,
):
    legacy = LegacyContextProvider(card, file_tools)
    if args.context_provider == "legacy":
        return legacy, None
    budget = derive_invocation_budget(
        num_ctx=args.num_ctx,
        num_predict=args.num_predict,
        system_prompt=runtime_prompt,
        task_spec=card.spec,
        allowed_paths=card.allowed_paths,
        acceptance_tests=card.acceptance_tests,
    )
    manifest_path = args.ckg_manifest or f"{args.out}.ckg-context.json"
    ckg = CKGContextProvider(
        card=card,
        worktree_dir=args.worktree,
        boundary=boundary,
        file_tools=file_tools,
        retrieval_budget_tokens=budget.retrieval_budget_tokens,
        budget_details=budget.as_dict(),
        boundary_error=BoundaryViolation,
        manifest_path=manifest_path,
    )
    return FallbackContextProvider(ckg, legacy), budget


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
    if limits.required_model and args.model != limits.required_model:
        result = {
            "status": "model_substitution_rejected",
            "reason": (
                f"Med-high card requires model {limits.required_model!r}, got {args.model!r}."
            ),
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

    runtime_prompt = tool_calling_system_prompt
    if tool_calling_system_prompt == TOOL_CALLING_SYSTEM_PROMPT:
        runtime_prompt = build_tool_calling_system_prompt(args.num_ctx, args.num_predict)

    def checkpoint_fn(transcript, turn):
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
    owns_live_model = chat_fn is None
    chat_fn = chat_fn or build_live_chat_fn(
        args.host,
        args.model,
        args.idle_timeout,
        args.max_wall,
        num_predict=args.num_predict,
        num_ctx=args.num_ctx,
        max_total_turns=limits.max_total_turns,
    )
    test_runner = test_runner or build_default_test_runner(card, boundary)
    file_tools = RunnerFileTools(
        args.worktree, boundary, MalformedToolCall, BoundaryViolation
    )
    formatter_fn = build_default_formatter(card, boundary, file_tools)
    context_provider, invocation_budget = _context_provider_for(
        args, card, boundary, file_tools, runtime_prompt
    )
    result = None
    unload_requested = False
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
            context_provider=context_provider,
            resolve_effective_limits=resolve_effective_limits,
            max_malformed_bounces=max_malformed_bounces,
            tool_calling_system_prompt=runtime_prompt,
        )
    finally:
        file_tools.close()
        if owns_live_model:
            unload_requested = unload_model(args.host, args.model)
    session_end = datetime.datetime.now(datetime.timezone.utc)
    elapsed_s = (session_end - session_start).total_seconds()

    result["task_id"] = card.task_id
    result["finished_at"] = datetime.datetime.now(datetime.timezone.utc).strftime(
        "%Y-%m-%dT%H:%M:%SZ"
    )
    result["model_unload_requested"] = unload_requested
    if invocation_budget is not None:
        result["invocation_budget"] = invocation_budget.as_dict()
    if isinstance(context_provider, FallbackContextProvider) and context_provider.last_fallback_reason:
        result["context_provider_fallback"] = context_provider.last_fallback_reason

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
    gemma_local.append_audit_log(audit_record)
    for bundle in build_attempt_bundles(
        card, result, args.model, session_start, session_end
    ):
        gemma_local.append_audit_log(bundle)

    if fallback_exit_code is not None:
        return fallback_exit_code
    return 0 if result["status"] == "success" else 1
