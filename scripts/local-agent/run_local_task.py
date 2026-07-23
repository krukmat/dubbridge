#!/usr/bin/env python3
"""Agentic runner: drives a local model through a bounded draft/test/repair loop.

Boundary enforcement (file/command allow-listing, env stripping) is owned by
T6b's `boundary` module; this task only defines the interface it calls
against, so T6b can be implemented independently without touching this file.

Transport (host normalization, streaming chat, model resolution, atomic
result writing) is reused from `gemma_local.py` rather than duplicated, per
the plan's design decision to extend the delegate-low-rri.py lineage instead
of reinventing its transport layer.

KNOWN LIMITATION (T7f): `gemma_local`, `scope_check`, and `boundary` are
imported once, below, from this script's own directory -- not from the
disposable worktree a session edits. If a task card's `allowed_paths`
includes any of those three modules, `run_loop`'s `finish`-time scope_check
call (and the boundary checks during the session) still run against the
pre-session, unedited copy of that module, not the model's changes. A
session whose task is "fix a bug in scope_check.py/boundary.py/gemma_local.py
itself" can therefore get a misleading in_scope/boundary verdict. Verify such
a session's diff independently (run its own tests against the worktree copy)
rather than trusting this runner's own gate for that narrow case.
"""

import argparse
import datetime
import json
import os
import signal
import subprocess
import sys

# When this file is run directly (`python3 run_local_task.py ...`, the real
# CLI path), Python executes it as module `__main__`. boundary.py separately
# does `from run_local_task import BoundaryViolation`, which -- absent this
# line -- makes Python import this same file a *second* time under the name
# `run_local_task`, producing a second, distinct BoundaryViolation class.
# run_loop's `except BoundaryViolation` (bound to the __main__ copy) then
# does not match an instance raised via boundary.py's copy, so every real
# boundary violation escaped as an uncaught traceback instead of the clean
# {"status": "boundary_violation"} result the code is designed to return.
# Confirmed live: a real qwen3.6:35b-a3b session hit exactly this crash on a
# legitimate out-of-scope write attempt. Registering this module object
# under its on-disk name before any sibling module can trigger the second
# import makes both names resolve to the identical module, so both refer to
# the same class. Tests that `import run_local_task` directly (never as
# __main__) never exercised this path, which is why 100 passing tests missed
# it.
if __name__ == "__main__":
    sys.modules.setdefault("run_local_task", sys.modules[__name__])

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gemma_local

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import runner_workflow_gate
import scope_check
from runner_file_tools import ALLOWED_TOOL_NAMES, RunnerFileTools

MAX_REPAIR_ATTEMPTS = 2
# Independent of MAX_REPAIR_ATTEMPTS (only counts failed finish->test cycles)
# and MAX_MALFORMED_BOUNCES (only counts consecutive malformed calls): a
# model that keeps issuing valid, successful tool calls (e.g. read_file
# repeatedly) without ever calling finish has no other bound on total turns.
# Discovered live: a real qwen3.6:35b-a3b session ran well past 300 turns in
# a single card with neither counter ever tripping, burning ~14 minutes
# before being killed manually.
MAX_TOTAL_TURNS = 30
# Raised from 1 to 3 after a real qwen3.6:35b-a3b pilot run: the model
# demonstrably can produce a well-formed deeply-nested tool-call JSON
# (confirmed — a read_file call succeeded), but intermittently drops exactly
# one closing brace on the same schema. A budget of 1 killed every pilot
# session on two consecutive misses of an error the model itself recovers
# from given another turn. This is a tolerance/threshold change, not a
# change to the boundary/security model.
MAX_MALFORMED_BOUNCES = 3
DEFAULT_IDLE_TIMEOUT_SECONDS = 180
DEFAULT_MAX_WALL_SECONDS = 1800
COMMAND_TIMEOUT_SECONDS = 120
# Output-token budget per turn. read_file/write_file/apply_patch have no size
# cap (see runner_file_tools.py), so a turn can legitimately need to emit a
# large "content"/"replacement" string; 4096 is comfortably above any file in
# this workspace's largest source files while leaving room for the JSON
# envelope around it.
GENERATION_TOKEN_BUDGET = 8192
# qwen3.6:35b-a3b's advertised context window (`ollama show`) is 131072, but
# that full size measurably slows generation. Set explicitly to a smaller
# ceiling rather than trusting Ollama's server-side default (smaller still,
# and would silently truncate the model's view of a full-file read_file
# result on a long session) or the full advertised window (slow in practice
# for this workspace's task sizes).
MODEL_CONTEXT_TOKENS = 90112

# Prepended to every card's own spec as the system message. The model is not
# given native tool-calling (see build_live_chat_fn's docstring) — it must be
# told, in plain text, the exact JSON shape parse_tool_call() expects, or it
# replies with ordinary prose and every turn is bounced as a malformed tool
# call.
TOOL_CALLING_SYSTEM_PROMPT = """\
You are an autonomous coding agent working inside a disposable, isolated git \
worktree. Ordinary development commands (build, test, format, lint, etc.) \
are permitted — there is no fixed command allowlist. What determines success \
is the final scoped diff you produce and the operator-controlled acceptance \
tests run against it. You act only by responding with a single JSON object \
of the exact form:

{"tool_calls": [{"function": {"name": "<tool>", "arguments": {<tool-specific fields>}}}]}

"arguments" is a nested JSON object (not a string).

Respond with ONLY that JSON object — no prose before or after it, no markdown \
code fences.

Available tools:
- read_file: arguments {"path": "<repo-relative path>"}. Returns the full current \
contents of an existing file. There is no size limit — read the whole file you need \
to change.
- write_file: arguments {"path": "<repo-relative path>", "content": "<full file contents>"}. \
Creates a new file or overwrites an existing one with exactly the content you supply.
- apply_patch: arguments {"path": "<repo-relative path>", "anchor": "<exact existing text>", "replacement": "<replacement text>"}. \
Replaces exactly one occurrence of "anchor" (which must appear exactly once in the file) \
with "replacement". Use this for a focused edit to a large file instead of rewriting it \
whole. If the anchor is not unique, read the file and include more surrounding text so it is.
- run_command: arguments {"argv": ["<program>", "<arg1>", ...]}. Runs a command inside \
the worktree and returns its real exit code, stdout, and stderr.
- finish: arguments {}. Signals you believe the task is complete; this triggers the \
acceptance tests. If they fail, you will see the failure output and get another turn \
to fix it (bounded number of repair attempts).

Typical workflow: read_file the file(s) named in your task, make your edits with \
apply_patch (focused changes to large files) or write_file (new files or small full \
rewrites), run the acceptance command yourself with run_command to check, then call \
finish.

Call exactly one tool per turn. Only call finish once you believe the acceptance \
tests described in your task will pass.
"""

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


class TaskCard:
    def __init__(
        self, task_id, spec, acceptance_tests, allowed_paths, rri=None, band=None,
        capsule_hash=None,
    ):
        self.task_id = task_id
        self.spec = spec
        self.acceptance_tests = acceptance_tests
        self.allowed_paths = allowed_paths
        self.rri = rri
        self.band = band
        # T2 (docs/tasks/local-first-cloud-local-handoff.md): the T1 capsule
        # hash this card was issued against, so bundles emitted for this
        # session can reference it. None for cards produced before T1/T2
        # existed -- build_attempt_bundles skips emission when unset rather
        # than fabricating a hash, since an invented hash would validate
        # against T1's schema syntactically while being semantically false.
        self.capsule_hash = capsule_hash


class ToolCall:
    def __init__(self, name, arguments):
        self.name = name
        self.arguments = arguments


class MalformedToolCall(ValueError):
    pass


def parse_tool_call(raw_message):
    tool_calls = raw_message.get("tool_calls")
    if not tool_calls:
        raise MalformedToolCall("no tool_calls in model response")
    call = tool_calls[0]
    function = call.get("function", {})
    name = function.get("name")
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
    if call.name == "run_command":
        argv = require_argument(call, "argv")
        # T7d-fix: a real qwen3.6:35b-a3b session sent argv as a single raw
        # string (e.g. "cd mobile && npm install") instead of a list. Popen
        # would treat the whole string as a literal executable name and raise
        # an uncaught FileNotFoundError, crashing the entire benchmark batch
        # (MC-01, see run_stage1_benchmark.py's per-card isolation fix for the
        # same incident). Reject the wrong type here, before it ever reaches
        # check_command/Popen, as ordinary malformed model output.
        if not isinstance(argv, list) or not all(isinstance(item, str) for item in argv):
            raise MalformedToolCall(f"run_command: argv must be a list of strings, got {argv!r}")
        boundary.check_command(argv)
        return _run_command_with_timeout(argv, worktree_dir, boundary)

    if call.name == "finish":
        return {"tool": "finish", "ok": True}

    raise MalformedToolCall(f"unhandled tool name: {call.name!r}")


def run_acceptance_tests(test_runner, worktree_dir):
    return test_runner(worktree_dir)


def run_loop(
    card,
    chat_fn,
    test_runner,
    worktree_dir,
    boundary,
    file_tools,
    organization_gate_fn,
    checkpoint_fn=None,
):
    """checkpoint_fn(transcript, total_turns), if given, is called after every
    turn that continues the loop. A session killed between turns (e.g. an
    operator interrupting a slow local-model generation) previously left no
    artifact at all -- gemma_local.write_result() only ran once, after this
    function returned. Checkpointing lets the caller persist the
    transcript-so-far each turn instead, so an interrupted run still leaves
    diagnostic evidence and any already-applied worktree diff stays visible."""
    transcript = []
    repair_attempt = 0
    malformed_bounces = 0
    total_turns = 0

    messages = [
        {
            "role": "system",
            "content": TOOL_CALLING_SYSTEM_PROMPT + "\n\nTask:\n" + card.spec,
        }
    ]

    while True:
        total_turns += 1
        if total_turns > MAX_TOTAL_TURNS:
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
            if malformed_bounces > MAX_MALFORMED_BOUNCES:
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
            if malformed_bounces > MAX_MALFORMED_BOUNCES:
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
            f"[local-agent] turn {total_turns}/{MAX_TOTAL_TURNS} -> {call.name}",
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

            test_result = run_acceptance_tests(test_runner, worktree_dir)
            transcript.append({"event": "test_result", "result": test_result})

            if test_result["passed"]:
                organization_result = organization_gate_fn(worktree_dir)
                transcript.append(
                    {"event": "organization_gate", "result": organization_result}
                )
                if organization_result.get("status") != "pass":
                    return {
                        "status": "organization_violation",
                        "reason": organization_result.get("status", "organization_violation"),
                        "organization_gate": organization_result,
                        "transcript": transcript,
                    }
                return {
                    "status": "success",
                    "organization_gate": organization_result,
                    "transcript": transcript,
                }

            if repair_attempt >= MAX_REPAIR_ATTEMPTS:
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
                    "content": f"Tests failed: {test_result['output']}. Repair attempt {repair_attempt}.",
                }
            )
            if checkpoint_fn is not None:
                checkpoint_fn(transcript, total_turns)
            continue

        # read_file / write_file / run_command: report the real tool result
        # back into the conversation so the model has actual new information
        # to act on next turn (file content, write confirmation, command
        # output) instead of an unchanged message list that gives it no
        # signal that its last action already happened.
        messages.append(
            {"role": "user", "content": f"Tool result: {json.dumps(result)}"}
        )
        if checkpoint_fn is not None:
            checkpoint_fn(transcript, total_turns)
        continue


def build_live_chat_fn(host, model, idle_timeout, max_wall):
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
                # on the turn right after a full-file read_file put ~14k
                # tokens of context in play — the model still had a large
                # "replacement" string left to emit. read_file/write_file
                # have no size cap in this tool contract, so the output
                # budget must comfortably exceed one full file's worth of
                # text, not just a short tool-call envelope.
                "num_predict": GENERATION_TOKEN_BUDGET,
                "num_ctx": MODEL_CONTEXT_TOKENS,
            },
        }
        result = gemma_local.stream_chat(
            url,
            payload,
            idle_timeout,
            max_wall,
            progress_label=f"local-agent turn {turn_counter['n']}/{MAX_TOTAL_TURNS}",
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


def load_card(card_path):
    with open(card_path, encoding="utf-8") as f:
        data = json.load(f)
    return TaskCard(
        task_id=data["task_id"],
        spec=data["spec"],
        acceptance_tests=data.get("acceptance_tests", []),
        allowed_paths=data.get("allowed_paths", []),
        rri=data.get("rri"),
        band=data.get("band"),
        capsule_hash=data.get("capsule_hash"),
    )


def parse_args(argv=None):
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
        default=os.environ.get("DUBBRIDGE_LOCAL_AGENT_MODEL", "qwen3.6:35b-a3b"),
        help="Local implementer model tag (ADR-036 binding).",
    )
    parser.add_argument(
        "--idle-timeout",
        type=int,
        default=DEFAULT_IDLE_TIMEOUT_SECONDS,
        help="Seconds without a new token before treating the model as stalled.",
    )
    parser.add_argument(
        "--max-wall",
        type=int,
        default=DEFAULT_MAX_WALL_SECONDS,
        help="Maximum wall-clock seconds for a single chat turn.",
    )
    return parser.parse_args(argv)


def build_default_boundary(worktree_root):
    # Deferred import: boundary.py imports this module (for BoundaryViolation),
    # so importing it at module load time here would create a circular import.
    import boundary

    return boundary.LocalAgentBoundary(worktree_root)


def build_default_test_runner(card):
    """CLI fallback: turn card.acceptance_tests (a list of shell command
    strings) into the test_runner run_loop calls at finish. Mirrors
    run_stage1_benchmark.make_test_runner.

    Before this existed, main()'s CLI path left test_runner=None -- unlike
    chat_fn and boundary, which both have `x or build_x(...)` fallbacks -- so
    every real `python3 run_local_task.py --card ...` session that reached
    finish crashed with `TypeError: 'NoneType' object is not callable` inside
    run_acceptance_tests. The whole local-first delegation channel could never
    close a task from the CLI; it only ever worked when an injected caller
    (the unit tests, the benchmark harness) supplied its own test_runner.
    Injected callers still override this and are unaffected.

    An empty acceptance_tests list means "no acceptance gate": finish passes
    rather than crashing, so a card with nothing to verify still closes
    cleanly instead of reintroducing the same NoneType failure by another name.
    """

    def test_runner(worktree_dir):
        outputs = []
        for cmd in card.acceptance_tests:
            try:
                completed = subprocess.run(
                    cmd,
                    shell=True,
                    cwd=worktree_dir,
                    capture_output=True,
                    text=True,
                    timeout=COMMAND_TIMEOUT_SECONDS,
                )
            except subprocess.TimeoutExpired:
                # A hung acceptance command must surface as a failed test, not
                # an uncaught exception that would crash the runner at finish --
                # exactly the failure class this fallback exists to remove.
                outputs.append(f"$ {cmd}\n[TIMEOUT after {COMMAND_TIMEOUT_SECONDS}s]")
                return {"passed": False, "output": "\n\n".join(outputs)}
            outputs.append(
                f"$ {cmd}\n(exit {completed.returncode})\n{completed.stdout}\n{completed.stderr}"
            )
            if completed.returncode != 0:
                return {"passed": False, "output": "\n\n".join(outputs)}
        return {"passed": True, "output": "\n\n".join(outputs)}

    return test_runner


def build_audit_record(card, result, model, elapsed_s):
    # Derived entirely from the transcript run_loop already produced — no
    # new capture logic, only aggregation, so this stays in lockstep with
    # whatever event shapes T6a/T6b already emit instead of duplicating them.
    transcript = result.get("transcript", [])
    test_events = [e for e in transcript if e.get("event") == "test_result"]
    command_events = [
        e["result"] for e in transcript
        if e.get("event") == "tool_result" and e["result"].get("tool") == "run_command"
    ]
    edit_events = [
        e["result"] for e in transcript
        if e.get("event") == "tool_result" and e["result"].get("tool") in ("write_file", "apply_patch")
    ]
    boundary_violations = [e for e in transcript if e.get("event") == "boundary_violation"]
    scope_check_events = [e for e in transcript if e.get("event") == "scope_check"]
    scope_check_result = scope_check_events[-1] if scope_check_events else None
    organization_events = [
        e["result"] for e in transcript
        if e.get("event") == "organization_gate"
    ]
    organization_result = (
        result.get("organization_gate")
        or (organization_events[-1] if organization_events else None)
    )
    acceptance_results = [e["result"]["passed"] for e in test_events]
    verification_results = {
        "acceptance_tests": acceptance_results,
        "final_acceptance_passed": acceptance_results[-1] if acceptance_results else None,
        "scope_in_scope": scope_check_result["in_scope"] if scope_check_result else None,
        "organization_status": (
            organization_result.get("status") if organization_result else None
        ),
    }
    validation_errors = []
    if result["status"] == "success":
        if scope_check_result is None or not scope_check_result["in_scope"]:
            validation_errors.append("scope_gate_not_passed")
        if organization_result is None or organization_result.get("status") != "pass":
            validation_errors.append("organization_gate_not_passed")
        if verification_results["final_acceptance_passed"] is not True:
            validation_errors.append("acceptance_tests_not_passed")
    signed = result["status"] == "success" and not validation_errors
    signature = {
        "status": "signed" if signed else "unsigned",
        "signer": "local-implementer" if signed else None,
        "reason": (
            "all_mandatory_gates_passed"
            if signed
            else (
                validation_errors[0]
                if validation_errors
                else result["status"]
            )
        ),
    }

    return {
        "ts": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z"),
        "role": "local-implementer",
        "outcome": result["status"].upper(),
        "model": model,
        "task_id": card.task_id,
        "rri": card.rri,
        "band": card.band,
        "attempts": len(test_events),
        "commands": [c["argv"] for c in command_events],
        "edit_metrics": [
            {
                "tool": e["tool"],
                "path": e["path"],
                "line_count": e.get("line_count"),
                "byte_count": e.get("byte_count"),
                "anchor_matches": e.get("anchor_matches"),
            }
            for e in edit_events
        ],
        "test_results": [e["result"]["passed"] for e in test_events],
        "boundary_violations": len(boundary_violations),
        "organization_gate": organization_result,
        "scope_check": {
            "in_scope": scope_check_result["in_scope"],
            "offending_paths": scope_check_result["offending_paths"],
        } if scope_check_result else None,
        "verification_results": verification_results,
        "audit_validation": {
            "valid": not validation_errors,
            "errors": validation_errors,
        },
        "signature": signature,
        "escalated": result["status"] != "success",
        "elapsed_s": round(elapsed_s, 3),
    }


def build_attempt_bundles(card, result, model, session_start, session_end):
    """T2 (docs/tasks/local-first-cloud-local-handoff.md): one T1 attempt
    bundle per repair attempt in this session, read-only over the same
    transcript build_audit_record already aggregates -- no new capture
    logic, no change to run_loop's control flow or build_audit_record's
    output.

    Segmentation: run_loop's transcript is one flat list across the whole
    session; each attempt ends at its `test_result` event (finish -> tests
    run -> pass or repair). Splitting on that event boundary turns the flat
    list into per-attempt slices without touching run_loop itself. A trailing
    slice with no closing test_result (e.g. the turn budget or a boundary
    violation cut the session off before finish) has no attempt to report,
    per EC-1 -- no partial/malformed bundle is emitted for it.

    Timestamps: run_loop's transcript events carry no per-event timestamps,
    so there is no ground truth for exactly when each attempt started/ended
    within the session. Qwen27 phase-1 review (T2) flagged an earlier version
    that called datetime.now() once per attempt *after* run_loop had already
    returned -- every bundle got a near-identical wall-clock timestamp from
    bundle-generation time, not from when the attempt actually happened, which
    is worse than useless for audit purposes. Rather than fabricate false
    per-attempt precision, every bundle's start_ts/end_ts is bounded by the
    caller-supplied session_start/session_end (the same window build_audit_record's
    elapsed_s is computed from) -- honest about the granularity actually
    available instead of inventing timestamps the data doesn't support.

    Returns [] when the card carries no capsule_hash (session predates T1/T2
    adoption): a bundle without a real capsule hash cannot pass T1's
    known_capsule_hashes check, so emitting one would only be discarded
    downstream.
    """
    if not card.capsule_hash:
        return []

    transcript = result.get("transcript", [])
    test_event_count = sum(1 for e in transcript if e.get("event") == "test_result")
    bundles = []
    segment = []
    tests_seen = 0
    for event in transcript:
        segment.append(event)
        if event.get("event") != "test_result":
            continue
        tests_seen += 1
        test_result = event["result"]
        edit_events = [
            e["result"] for e in segment
            if e.get("event") == "tool_result" and e["result"].get("tool") in ("write_file", "apply_patch")
        ]
        is_last_test_event = tests_seen == test_event_count
        if test_result["passed"]:
            outcome = "success"
        elif is_last_test_event:
            # Qwen27 phase-1 review (T2): this used to check
            # `result["status"] == "budget_exhausted"` specifically, so a
            # failing last attempt under any other terminal status (e.g.
            # boundary_violation, transport_error, aborted) fell through to
            # "repair-needed" -- implying another attempt was coming, which
            # is false once the session has actually ended. Any failing final
            # attempt is escalated regardless of which terminal status ended
            # the session; only a non-final failing attempt (more repair
            # turns follow within the same session) is "repair-needed".
            outcome = "escalated"
        else:
            outcome = "repair-needed"
        bundles.append(
            {
                "capsule_hash": card.capsule_hash,
                "implementer_id": "qwen35",
                "model_tag": model,
                "start_ts": session_start.isoformat().replace("+00:00", "Z"),
                "end_ts": session_end.isoformat().replace("+00:00", "Z"),
                "diff_ref": [
                    {"tool": e["tool"], "path": e["path"]} for e in edit_events
                ],
                "test_results": test_result,
                "review_verdict": "pending",
                "outcome": outcome,
            }
        )
        segment = []
    return bundles


def main(
    argv=None,
    chat_fn=None,
    test_runner=None,
    boundary=None,
    organization_gate_fn=None,
):
    args = parse_args(argv)
    card = load_card(args.card)
    boundary = boundary or build_default_boundary(args.worktree)
    organization_gate_fn = (
        organization_gate_fn or runner_workflow_gate.run_organization_gate
    )

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
                "max_turns": MAX_TOTAL_TURNS,
                "transcript": transcript,
            },
            args.out,
        )

    session_start = datetime.datetime.now(datetime.timezone.utc)
    chat_fn = chat_fn or build_live_chat_fn(
        args.host, args.model, args.idle_timeout, args.max_wall
    )
    # The missing fallback (chat_fn and boundary above both had theirs): the
    # CLI never injects test_runner, so without this it stayed None and every
    # finish crashed with TypeError. See build_default_test_runner's docstring.
    test_runner = test_runner or build_default_test_runner(card)
    file_tools = RunnerFileTools(
        args.worktree, boundary, MalformedToolCall, BoundaryViolation
    )
    try:
        result = run_loop(
            card,
            chat_fn,
            test_runner,
            args.worktree,
            boundary,
            file_tools,
            organization_gate_fn,
            checkpoint_fn=checkpoint_fn,
        )
    finally:
        file_tools.close()
    session_end = datetime.datetime.now(datetime.timezone.utc)
    elapsed_s = (session_end - session_start).total_seconds()

    result["task_id"] = card.task_id
    result["finished_at"] = datetime.datetime.now(datetime.timezone.utc).strftime(
        "%Y-%m-%dT%H:%M:%SZ"
    )
    audit_record = build_audit_record(card, result, args.model, elapsed_s)
    if result["status"] == "success" and not audit_record["audit_validation"]["valid"]:
        result["status"] = "audit_invalid"
        result["reason"] = ";".join(audit_record["audit_validation"]["errors"])
        audit_record = build_audit_record(card, result, args.model, elapsed_s)
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

    return 0 if result["status"] == "success" else 1


if __name__ == "__main__":
    sys.exit(main())
