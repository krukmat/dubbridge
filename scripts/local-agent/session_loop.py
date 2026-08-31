#!/usr/bin/env python3
"""Turn-by-turn state machine for the bounded local implementer."""

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys

import gemma_local
import scope_check
from context_provider import LegacyContextProvider
from runner_file_tools import ALLOWED_TOOL_NAMES, RunnerFileTools


class BoundaryViolation(RuntimeError):
    pass


class NullBoundary:
    def check_write(self, path):
        return None

    def check_path(self, path):
        return None

    def check_command(self, argv):
        return None

    def env_for_subprocess(self):
        return None


class ToolCall:
    def __init__(self, name, arguments):
        self.name = name
        self.arguments = arguments


class MalformedToolCall(ValueError):
    pass


FORBIDDEN_MODEL_TOOL_NAMES = frozenset(("read_file", "run_command"))
COMMAND_TIMEOUT_SECONDS = 120


def parse_tool_call(raw_message):
    tool_calls = raw_message.get("tool_calls")
    if not tool_calls:
        raise MalformedToolCall("no tool_calls in model response")
    call = tool_calls[0]
    function = call.get("function", {})
    name = function.get("name")
    if name in FORBIDDEN_MODEL_TOOL_NAMES:
        raise BoundaryViolation(f"model tool is operator-controlled: {name}")
    if name not in ALLOWED_TOOL_NAMES:
        raise MalformedToolCall(f"unknown tool name: {name!r}")
    raw_arguments = function.get("arguments", {})
    if isinstance(raw_arguments, str):
        try:
            arguments = json.loads(raw_arguments)
        except json.JSONDecodeError as exc:
            raise MalformedToolCall(f"invalid tool arguments JSON: {exc}") from exc
    elif isinstance(raw_arguments, dict):
        arguments = raw_arguments
    else:
        raise MalformedToolCall(
            "tool arguments must be a JSON object or JSON-encoded string, "
            f"got {type(raw_arguments).__name__}"
        )
    return ToolCall(name, arguments)


def require_argument(call, key):
    if key not in call.arguments:
        raise MalformedToolCall(f"{call.name!r} call missing required argument {key!r}")
    return call.arguments[key]


def _run_command_with_timeout(argv, worktree_dir, boundary):
    try:
        process = subprocess.Popen(
            argv,
            cwd=worktree_dir,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=boundary.env_for_subprocess(),
            start_new_session=True,
        )
    except (OSError, ValueError) as exc:
        return {
            "tool": "run_command",
            "argv": argv,
            "ok": False,
            "returncode": None,
            "stdout": "",
            "stderr": f"command failed to start: {exc}",
        }
    try:
        stdout, stderr = process.communicate(timeout=COMMAND_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(os.getpgid(process.pid), signal.SIGKILL)
        except ProcessLookupError:
            pass
        stdout, stderr = process.communicate()
        return {
            "tool": "run_command",
            "argv": argv,
            "ok": False,
            "returncode": None,
            "stdout": stdout or "",
            "stderr": f"command timed out after {COMMAND_TIMEOUT_SECONDS}s",
        }
    return {
        "tool": "run_command",
        "argv": argv,
        "ok": process.returncode == 0,
        "returncode": process.returncode,
        "stdout": stdout,
        "stderr": stderr,
    }


def apply_tool_call(call, worktree_dir, boundary, file_tools=None):
    if file_tools is None:
        file_tools = RunnerFileTools(
            worktree_dir, boundary, MalformedToolCall, BoundaryViolation
        )
    file_result = file_tools.handle(call)
    if file_result is not None:
        return file_result
    if call.name == "finish":
        return {"tool": "finish", "ok": True}
    raise MalformedToolCall(f"unhandled tool name: {call.name!r}")


def run_acceptance_tests(test_runner, worktree_dir):
    return test_runner(worktree_dir)


def render_authorized_context(card, file_tools):
    """Legacy compatibility helper; new runtime code uses ContextProvider."""
    return LegacyContextProvider(card, file_tools).render_initial()


def build_initial_system_message(
    card,
    file_tools,
    max_total_turns,
    tool_calling_system_prompt,
    context_provider=None,
):
    provider = context_provider or LegacyContextProvider(card, file_tools)
    return (
        tool_calling_system_prompt.replace("{MAX_TOTAL_TURNS}", str(max_total_turns))
        + "\n\nTask specification:\n"
        + card.spec
        + "\n\nAllowed paths (complete capability list):\n"
        + json.dumps(card.allowed_paths, ensure_ascii=False, indent=2)
        + "\n\nRunner-controlled acceptance commands (not model tools):\n"
        + json.dumps(card.acceptance_tests, ensure_ascii=False, indent=2)
        + "\n\nAuthorized source context:\n"
        + provider.render_initial()
    )


def _attach_manifest(result, provider):
    manifest = provider.manifest() if provider is not None else None
    if manifest is not None:
        result["ckg_context_manifest"] = manifest
    return result


def run_loop(
    card,
    chat_fn,
    test_runner,
    worktree_dir,
    boundary,
    file_tools,
    checkpoint_fn=None,
    limits=None,
    formatter_fn=None,
    context_provider=None,
    *,
    resolve_effective_limits,
    max_malformed_bounces,
    tool_calling_system_prompt,
):
    limits = limits or resolve_effective_limits(card)
    max_total_turns = limits.max_total_turns
    max_repair_attempts = limits.max_repair_attempts
    provider = context_provider or LegacyContextProvider(card, file_tools)
    transcript = []
    repair_attempt = 0
    malformed_bounces = 0
    total_turns = 0

    messages = [
        {
            "role": "system",
            "content": build_initial_system_message(
                card,
                file_tools,
                max_total_turns,
                tool_calling_system_prompt,
                context_provider=provider,
            ),
        },
        {"role": "user", "content": "Begin."},
    ]

    while True:
        total_turns += 1
        if total_turns > max_total_turns:
            transcript.append(
                {"event": "turn_budget_exhausted", "total_turns": total_turns - 1}
            )
            return _attach_manifest(
                {
                    "status": "budget_exhausted",
                    "reason": "total_turns_exhausted",
                    "transcript": transcript,
                },
                provider,
            )
        try:
            response = chat_fn(messages)
        except (
            gemma_local.GemmaIdleTimeout,
            gemma_local.GemmaWallTimeout,
            RuntimeError,
        ) as exc:
            transcript.append({"event": "transport_error", "error": str(exc)})
            return _attach_manifest(
                {"status": "transport_error", "reason": str(exc), "transcript": transcript},
                provider,
            )
        except MalformedToolCall as exc:
            malformed_bounces += 1
            transcript.append({"event": "malformed_tool_call", "error": str(exc)})
            if malformed_bounces > max_malformed_bounces:
                return _attach_manifest(
                    {
                        "status": "aborted",
                        "reason": "malformed_tool_call_repeated",
                        "transcript": transcript,
                    },
                    provider,
                )
            messages.append(
                {"role": "user", "content": f"Malformed tool call: {exc}. Retry."}
            )
            if checkpoint_fn is not None:
                checkpoint_fn(transcript, total_turns)
            continue

        transcript.append({"role": "assistant", "raw": response})
        messages.append({"role": "assistant", "content": json.dumps(response)})
        try:
            call = parse_tool_call(response)
            result = apply_tool_call(call, worktree_dir, boundary, file_tools)
        except MalformedToolCall as exc:
            malformed_bounces += 1
            transcript.append({"event": "malformed_tool_call", "error": str(exc)})
            if malformed_bounces > max_malformed_bounces:
                return _attach_manifest(
                    {
                        "status": "aborted",
                        "reason": "malformed_tool_call_repeated",
                        "transcript": transcript,
                    },
                    provider,
                )
            messages.append(
                {"role": "user", "content": f"Malformed tool call: {exc}. Retry."}
            )
            if checkpoint_fn is not None:
                checkpoint_fn(transcript, total_turns)
            continue
        except BoundaryViolation as exc:
            transcript.append({"event": "boundary_violation", "error": str(exc)})
            return _attach_manifest(
                {"status": "boundary_violation", "reason": str(exc), "transcript": transcript},
                provider,
            )

        transcript.append({"event": "tool_result", "result": result})
        malformed_bounces = 0
        print(
            f"[local-agent] turn {total_turns}/{max_total_turns} -> {call.name}",
            file=sys.stderr,
        )

        if call.name == "finish":
            scope_result = scope_check.check_scope(worktree_dir, card.allowed_paths)
            transcript.append(
                {
                    "event": "scope_check",
                    "in_scope": scope_result.in_scope,
                    "offending_paths": scope_result.offending_paths,
                    "has_diff": scope_result.has_diff,
                }
            )
            if not scope_result.in_scope:
                return _attach_manifest(
                    {
                        "status": "out_of_scope",
                        "reason": "diff touches paths outside allowed_paths",
                        "offending_paths": scope_result.offending_paths,
                        "transcript": transcript,
                    },
                    provider,
                )

            if formatter_fn is not None:
                format_result = formatter_fn(worktree_dir)
                transcript.append({"event": "format_result", "result": format_result})
                if not format_result["passed"]:
                    if repair_attempt >= max_repair_attempts:
                        return _attach_manifest(
                            {
                                "status": "budget_exhausted",
                                "reason": "repair_attempts_exhausted",
                                "attempts": repair_attempt,
                                "transcript": transcript,
                            },
                            provider,
                        )
                    repair_attempt += 1
                    messages.append(
                        {
                            "role": "user",
                            "content": (
                                f"Runner formatting failed: {format_result['output']}. "
                                f"Repair attempt {repair_attempt}.\n\n"
                                "Current authorized source context:\n"
                                + provider.render_refresh("formatter_failure")
                            ),
                        }
                    )
                    if checkpoint_fn is not None:
                        checkpoint_fn(transcript, total_turns)
                    continue

            test_result = run_acceptance_tests(test_runner, worktree_dir)
            transcript.append({"event": "test_result", "result": test_result})
            if test_result["passed"]:
                return _attach_manifest(
                    {"status": "success", "transcript": transcript}, provider
                )
            if repair_attempt >= max_repair_attempts:
                return _attach_manifest(
                    {
                        "status": "budget_exhausted",
                        "reason": "repair_attempts_exhausted",
                        "attempts": repair_attempt,
                        "transcript": transcript,
                    },
                    provider,
                )
            repair_attempt += 1
            messages.append(
                {
                    "role": "user",
                    "content": (
                        f"Acceptance failed: {test_result['output']}. "
                        f"Repair attempt {repair_attempt}.\n\n"
                        "Current authorized source context:\n"
                        + provider.render_refresh("acceptance_failure")
                    ),
                }
            )
            if checkpoint_fn is not None:
                checkpoint_fn(transcript, total_turns)
            continue

        messages.append(
            {"role": "user", "content": f"Tool result: {json.dumps(result)}"}
        )
        if checkpoint_fn is not None:
            checkpoint_fn(transcript, total_turns)
