#!/usr/bin/env python3
"""Turn-by-turn model-interaction state machine for the local agentic runner.

Extracted from run_local_task.py (LRPC-0b, Extract Module / Single
Responsibility) as the largest single cohesive block of that runner.
Behavior-preserving: no logic changed from the original module.
"""

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys

import gemma_local
import scope_check
from runner_file_tools import ALLOWED_TOOL_NAMES, RunnerFileTools


class BoundaryViolation(RuntimeError):
    pass


class NullBoundary:
    """Stub boundary: allows everything. T6b replaces this with real enforcement.

    The runner only depends on this two-method shape (`check_write`,
    `check_command`), so T6b can ship its own class satisfying the same
    interface without any change here.
    """

    def check_write(self, path):
        return None

    def check_command(self, argv):
        return None

    def env_for_subprocess(self):
        # None means "let subprocess.run inherit the caller's environment
        # unchanged" — NullBoundary allows everything, including full env
        # inheritance. A real boundary (T6b) returns a stripped mapping.
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
    # Real models (confirmed against qwen3.6:35b-a3b) naturally emit
    # `arguments` as a nested JSON object, not a JSON-encoded string, despite
    # the system prompt asking for a string — accept both rather than
    # bouncing every single call from a model that follows the far more
    # common native-object convention.
    if isinstance(raw_arguments, str):
        try:
            arguments = json.loads(raw_arguments)
        except json.JSONDecodeError as exc:
            raise MalformedToolCall(f"invalid tool arguments JSON: {exc}") from exc
    elif isinstance(raw_arguments, dict):
        arguments = raw_arguments
    else:
        raise MalformedToolCall(
            f"tool arguments must be a JSON object or JSON-encoded string, got {type(raw_arguments).__name__}"
        )
    return ToolCall(name, arguments)


def require_argument(call, key):
    # a valid tool name with a missing required argument is still malformed
    # model output — it must count against the bounce budget (EC-2), not
    # crash past it via an uncaught KeyError.
    if key not in call.arguments:
        raise MalformedToolCall(f"{call.name!r} call missing required argument {key!r}")
    return call.arguments[key]


def _run_command_with_timeout(argv, worktree_dir, boundary):
    # D14 finding: subprocess.run's own timeout handling only signals the
    # immediate child — a multi-process command like `cargo test` (compiler
    # + test-binary children of its own) can leave those grandchildren
    # orphaned and still running in the background after the timeout is
    # caught. Popen with start_new_session=True puts the whole command in
    # its own process group, so killpg on timeout reaches the entire tree,
    # not just the directly-spawned process.
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
        # T7d-fix: a well-typed argv (list[str], passed check_command) can
        # still fail to spawn — OSError covers a nonexistent/non-executable
        # binary (FileNotFoundError, PermissionError, ...); ValueError covers
        # an argv element Popen itself rejects before ever spawning (e.g. an
        # embedded NUL byte). Before this fix, either propagated uncaught and
        # crashed the whole benchmark batch, exactly like the TimeoutExpired
        # case below — report it the same way, as a structured failed-command
        # result, not a crash.
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
            pass  # already exited between the timeout firing and the kill
        # Found live: a real `cargo test` (first build of a crate) ran past
        # COMMAND_TIMEOUT_SECONDS — before this fix, TimeoutExpired escaped
        # uncaught here and crashed the whole benchmark process with a
        # traceback instead of a structured, recoverable tool result.
        stdout, stderr = process.communicate()  # reap now-dead process, drain pipes
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
    authorized_files = file_tools.preload_context(card.allowed_paths)
    context_blocks = []
    for entry in authorized_files:
        path = json.dumps(entry["path"], ensure_ascii=False)
        if entry["missing"]:
            context_blocks.append(f"--- {path} (missing; creation allowed) ---")
        else:
            context_blocks.append(
                f"--- BEGIN {path} ---\n{entry['content']}\n--- END {path} ---"
            )
    return "\n\n".join(context_blocks) if context_blocks else "(none)"


def build_initial_system_message(card, file_tools, max_total_turns, tool_calling_system_prompt):
    return (
        tool_calling_system_prompt.replace(
            "{MAX_TOTAL_TURNS}", str(max_total_turns)
        )
        + "\n\nTask specification:\n"
        + card.spec
        + "\n\nAllowed paths (complete capability list):\n"
        + json.dumps(card.allowed_paths, ensure_ascii=False, indent=2)
        + "\n\nRunner-controlled acceptance commands (not model tools):\n"
        + json.dumps(card.acceptance_tests, ensure_ascii=False, indent=2)
        + "\n\nAuthorized file context:\n"
        + render_authorized_context(card, file_tools)
    )


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
    *,
    resolve_effective_limits,
    max_malformed_bounces,
    tool_calling_system_prompt,
):
    """checkpoint_fn(transcript, total_turns), if given, is called after every
    turn that continues the loop. A session killed between turns (e.g. an
    operator interrupting a slow local-model generation) previously left no
    artifact at all -- gemma_local.write_result() only ran once, after this
    function returned. Checkpointing lets the caller persist the
    transcript-so-far each turn instead, so an interrupted run still leaves
    diagnostic evidence and any already-applied worktree diff stays visible.

    limits: an EffectiveLimits (ADR-038 T3), or None to use resolve_effective_limits(card)
    unchanged -- every pre-T3 caller (including every existing test in this
    file) that does not pass `limits` keeps byte-for-byte identical behavior.

    resolve_effective_limits, max_malformed_bounces, and
    tool_calling_system_prompt are supplied by the caller (run_local_task.py)
    rather than imported here, to avoid a circular import between this module
    and run_local_task.py (which re-exports this function as its own public
    surface)."""
    limits = limits or resolve_effective_limits(card)
    max_total_turns = limits.max_total_turns
    max_repair_attempts = limits.max_repair_attempts
    transcript = []
    repair_attempt = 0
    malformed_bounces = 0
    total_turns = 0

    messages = [
        {
            "role": "system",
            "content": build_initial_system_message(
                card, file_tools, max_total_turns, tool_calling_system_prompt
            ),
        },
        # Some locally-served backends (confirmed: the MLX runtime behind
        # the band-resolved DUBBRIDGE_LOCAL_AGENT_MODEL binding)
        # reject a system-only /api/chat request with "no user query found
        # in messages" (HTTP 500) instead of degrading gracefully. GGUF
        # backends tolerate a system-only first turn, so this kickoff
        # message is added unconditionally rather than branching per model.
        {
            "role": "user",
            "content": "Begin.",
        },
    ]

    while True:
        total_turns += 1
        if total_turns > max_total_turns:
            transcript.append({"event": "turn_budget_exhausted", "total_turns": total_turns - 1})
            return {
                "status": "budget_exhausted",
                "reason": "total_turns_exhausted",
                "transcript": transcript,
            }
        try:
            response = chat_fn(messages)
        except (
            gemma_local.GemmaIdleTimeout,
            gemma_local.GemmaWallTimeout,
            RuntimeError,
        ) as exc:
            transcript.append({"event": "transport_error", "error": str(exc)})
            return {
                "status": "transport_error",
                "reason": str(exc),
                "transcript": transcript,
            }
        except MalformedToolCall as exc:
            # chat_fn itself can raise this (e.g. build_live_chat_fn's JSON
            # parse of a non-JSON model response) — before this fix, that
            # exception escaped uncaught here, since this try only handled
            # transport errors, crashing main() with no transcript written.
            # The model producing non-JSON prose instead of a tool call is
            # exactly the same "model produced garbage" case as the second
            # try below, so it shares the same bounce budget and retry
            # message rather than a separate, undocumented failure mode.
            malformed_bounces += 1
            transcript.append({"event": "malformed_tool_call", "error": str(exc)})
            if malformed_bounces > max_malformed_bounces:
                return {
                    "status": "aborted",
                    "reason": "malformed_tool_call_repeated",
                    "transcript": transcript,
                }
            messages.append(
                {"role": "user", "content": f"Malformed tool call: {exc}. Retry."}
            )
            if checkpoint_fn is not None:
                checkpoint_fn(transcript, total_turns)
            continue
        transcript.append({"role": "assistant", "raw": response})
        # Structural bug found live (both qwen3.6:35b-a3b and
        # gemma4:26b-a4b-it-qat got stuck calling read_file on the same path
        # dozens of times): this appends the model's own turn to `transcript`
        # (an internal log) but, before this fix, NOTHING was ever appended
        # to `messages` for a successful non-finish call — the next
        # chat_fn(messages) call resent the identical conversation, so the
        # model had no memory of having already acted and repeated the same
        # first action. Mocked tests never caught this because
        # ChatSequencer advances through a scripted list regardless of what
        # `messages` contains, so no test ever depended on `messages`
        # actually growing turn over turn.
        messages.append({"role": "assistant", "content": json.dumps(response)})

        try:
            call = parse_tool_call(response)
            result = apply_tool_call(call, worktree_dir, boundary, file_tools)
        except MalformedToolCall as exc:
            # covers both parse_tool_call (unparseable response) and
            # apply_tool_call (valid tool name, missing/invalid arguments) —
            # both are "the model produced garbage", so both share one bounce
            # budget rather than the boundary violating it via a second,
            # narrower try/except that used to let this escape uncaught.
            malformed_bounces += 1
            transcript.append({"event": "malformed_tool_call", "error": str(exc)})
            if malformed_bounces > max_malformed_bounces:
                return {
                    "status": "aborted",
                    "reason": "malformed_tool_call_repeated",
                    "transcript": transcript,
                }
            messages.append(
                {"role": "user", "content": f"Malformed tool call: {exc}. Retry."}
            )
            if checkpoint_fn is not None:
                checkpoint_fn(transcript, total_turns)
            continue
        except BoundaryViolation as exc:
            transcript.append({"event": "boundary_violation", "error": str(exc)})
            return {
                "status": "boundary_violation",
                "reason": str(exc),
                "transcript": transcript,
            }

        transcript.append({"event": "tool_result", "result": result})
        # a valid call resets the malformed-bounce counter: the budget guards
        # against consecutive garbage, not a single earlier hiccup in an
        # otherwise-recovering session.
        malformed_bounces = 0
        # Printed once the tool call is parsed (not during generation, since
        # the tool isn't known until the model finishes choosing it) — lets
        # an operator watching stderr tell "still generating turn 4/30" apart
        # from "turn 4/30 resolved to write_file, now running acceptance
        # tests", instead of only ever seeing a per-turn token counter reset
        # to zero with no sense of overall progress.
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
                # A scope violation is a different failure class than a failing
                # acceptance test: it never reaches run_acceptance_tests, never
                # consumes a repair attempt, and is not retryable — the model
                # already produced a diff outside the card's allowed_paths, and
                # giving it another turn to "fix" that is out of scope for what
                # repair_attempt exists to bound (test failures, not boundary
                # violations of the task's own contract).
                return {
                    "status": "out_of_scope",
                    "reason": "diff touches paths outside allowed_paths",
                    "offending_paths": scope_result.offending_paths,
                    "transcript": transcript,
                }

            if formatter_fn is not None:
                format_result = formatter_fn(worktree_dir)
                transcript.append({"event": "format_result", "result": format_result})
                if not format_result["passed"]:
                    if repair_attempt >= max_repair_attempts:
                        return {
                            "status": "budget_exhausted",
                            "reason": "repair_attempts_exhausted",
                            "attempts": repair_attempt,
                            "transcript": transcript,
                        }
                    repair_attempt += 1
                    messages.append(
                        {
                            "role": "user",
                            "content": (
                                f"Runner formatting failed: {format_result['output']}. "
                                f"Repair attempt {repair_attempt}.\n\n"
                                "Current authorized files:\n"
                                + render_authorized_context(card, file_tools)
                            ),
                        }
                    )
                    if checkpoint_fn is not None:
                        checkpoint_fn(transcript, total_turns)
                    continue

            test_result = run_acceptance_tests(test_runner, worktree_dir)
            transcript.append({"event": "test_result", "result": test_result})

            if test_result["passed"]:
                # The local DEV boundary ends here. Code organization, review,
                # coverage, and closure belong to later workflow phases owned by
                # the orchestrator; they must not rewrite a passing DEV result.
                return {"status": "success", "transcript": transcript}

            if repair_attempt >= max_repair_attempts:
                return {
                    "status": "budget_exhausted",
                    "reason": "repair_attempts_exhausted",
                    "attempts": repair_attempt,
                    "transcript": transcript,
                }

            repair_attempt += 1
            messages.append(
                {
                    "role": "user",
                    "content": (
                        f"Acceptance failed: {test_result['output']}. "
                        f"Repair attempt {repair_attempt}.\n\n"
                        "Current authorized files:\n"
                        + render_authorized_context(card, file_tools)
                    ),
                }
            )
            if checkpoint_fn is not None:
                checkpoint_fn(transcript, total_turns)
            continue

        # Report edit confirmation so the next turn knows the prior mutation
        # completed. File refreshes are injected automatically after a failed
        # finish; the model has no read or command tool of its own.
        messages.append(
            {"role": "user", "content": f"Tool result: {json.dumps(result)}"}
        )
        if checkpoint_fn is not None:
            checkpoint_fn(transcript, total_turns)
        continue
