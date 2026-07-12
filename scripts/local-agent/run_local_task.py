#!/usr/bin/env python3
"""Agentic runner: drives a local model through a bounded draft/test/repair loop.

Boundary enforcement (file/command allow-listing, env stripping) is owned by
T6b's `boundary` module; this task only defines the interface it calls
against, so T6b can be implemented independently without touching this file.

Transport (host normalization, streaming chat, model resolution, atomic
result writing) is reused from `gemma_local.py` rather than duplicated, per
the plan's design decision to extend the delegate-low-rri.py lineage instead
of reinventing its transport layer.
"""

import argparse
import datetime
import json
import os
import signal
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gemma_local

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scope_check

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
- read_file: arguments {"path": "<repo-relative path>"}. Returns the current contents of \
a file already in the worktree. Use this before write_file when you need to see or edit \
existing code — you cannot see repo contents any other way.
- write_file: arguments {"path": "<repo-relative path>", "content": "<full file contents>"}. \
Writes the complete file content (not a diff/patch) to the given path inside the worktree.
- run_command: arguments {"argv": ["<program>", "<arg1>", ...]}. Runs a command inside \
the worktree and returns its real exit code, stdout, and stderr.
- finish: arguments {}. Signals you believe the task is complete; this triggers the \
acceptance tests. If they fail, you will see the failure output and get another turn \
to fix it (bounded number of repair attempts).

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
                                "enum": ["read_file", "write_file", "run_command", "finish"],
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
    def __init__(self, task_id, spec, acceptance_tests, allowed_paths):
        self.task_id = task_id
        self.spec = spec
        self.acceptance_tests = acceptance_tests
        self.allowed_paths = allowed_paths


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
    if name not in ("write_file", "run_command", "read_file", "finish"):
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
    process = subprocess.Popen(
        argv,
        cwd=worktree_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=boundary.env_for_subprocess(),
        start_new_session=True,
    )
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


def apply_tool_call(call, worktree_dir, boundary):
    if call.name == "write_file":
        path = require_argument(call, "path")
        boundary.check_write(path)
        target = os.path.join(worktree_dir, path)
        os.makedirs(os.path.dirname(target) or ".", exist_ok=True)
        # D14 finding: check_write() resolving the path and this open() are
        # two separate steps — a symlink swapped in the gap between them
        # would previously be followed unconditionally by open(). O_NOFOLLOW
        # makes the final path component's open atomic against exactly that
        # race: if it's a symlink at open() time (original target or
        # swapped), the kernel rejects it instead of following it.
        try:
            fd = os.open(target, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_NOFOLLOW)
            with os.fdopen(fd, "w", encoding="utf-8") as f:
                f.write(call.arguments.get("content", ""))
        except OSError as exc:
            raise BoundaryViolation(f"write rejected at open time: {path!r} ({exc})") from exc
        return {"tool": "write_file", "path": path, "ok": True}

    if call.name == "read_file":
        path = require_argument(call, "path")
        # check_write()'s jail/symlink resolution is the same validation a
        # read needs (path stays inside the worktree, no traversal, no
        # symlink escape); its name is write-oriented but the check itself
        # is intent-agnostic, so reusing it here avoids a near-duplicate
        # boundary method for what is otherwise identical logic.
        boundary.check_write(path)
        target = os.path.join(worktree_dir, path)
        try:
            with open(target, "r", encoding="utf-8", errors="replace") as f:
                content = f.read()
        except FileNotFoundError as exc:
            raise MalformedToolCall(f"read_file: no such file in worktree: {path!r}") from exc
        except OSError as exc:
            raise BoundaryViolation(f"read rejected: {path!r} ({exc})") from exc
        return {"tool": "read_file", "path": path, "ok": True, "content": content}

    if call.name == "run_command":
        argv = require_argument(call, "argv")
        boundary.check_command(argv)
        return _run_command_with_timeout(argv, worktree_dir, boundary)

    if call.name == "finish":
        return {"tool": "finish", "ok": True}

    raise MalformedToolCall(f"unhandled tool name: {call.name!r}")


def run_acceptance_tests(test_runner, worktree_dir):
    return test_runner(worktree_dir)


def run_loop(card, chat_fn, test_runner, worktree_dir, boundary):
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
            result = apply_tool_call(call, worktree_dir, boundary)
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
                return {"status": "success", "transcript": transcript}

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
            continue

        # read_file / write_file / run_command: report the real tool result
        # back into the conversation so the model has actual new information
        # to act on next turn (file content, write confirmation, command
        # output) instead of an unchanged message list that gives it no
        # signal that its last action already happened.
        messages.append(
            {"role": "user", "content": f"Tool result: {json.dumps(result)}"}
        )
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

    def chat_fn(messages):
        payload = {
            "model": resolved_model,
            "stream": True,
            "think": False,
            "format": TOOL_CALL_JSON_SCHEMA,
            "keep_alive": "10m",
            "messages": messages,
        }
        result = gemma_local.stream_chat(
            url, payload, idle_timeout, max_wall, progress_label="local-agent"
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
    boundary_violations = [e for e in transcript if e.get("event") == "boundary_violation"]
    scope_check_events = [e for e in transcript if e.get("event") == "scope_check"]
    scope_check_result = scope_check_events[-1] if scope_check_events else None

    return {
        "ts": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z"),
        "role": "local-implementer",
        "outcome": result["status"].upper(),
        "model": model,
        "task_id": card.task_id,
        "rri": None,
        "band": None,
        "attempts": len(test_events),
        "commands": [c["argv"] for c in command_events],
        "test_results": [e["result"]["passed"] for e in test_events],
        "boundary_violations": len(boundary_violations),
        "scope_check": {
            "in_scope": scope_check_result["in_scope"],
            "offending_paths": scope_check_result["offending_paths"],
        } if scope_check_result else None,
        "escalated": result["status"] != "success",
        "elapsed_s": round(elapsed_s, 3),
    }


def main(argv=None, chat_fn=None, test_runner=None, boundary=None):
    args = parse_args(argv)
    card = load_card(args.card)
    boundary = boundary or build_default_boundary(args.worktree)
    chat_fn = chat_fn or build_live_chat_fn(
        args.host, args.model, args.idle_timeout, args.max_wall
    )

    session_start = datetime.datetime.now(datetime.timezone.utc)
    result = run_loop(card, chat_fn, test_runner, args.worktree, boundary)
    elapsed_s = (datetime.datetime.now(datetime.timezone.utc) - session_start).total_seconds()

    result["task_id"] = card.task_id
    result["finished_at"] = datetime.datetime.now(datetime.timezone.utc).strftime(
        "%Y-%m-%dT%H:%M:%SZ"
    )
    gemma_local.write_result(result, args.out)

    # Emitted for every exit path (success, aborted, budget_exhausted,
    # boundary_violation, transport_error) — audit visibility must not
    # depend on how the session ended.
    gemma_local.append_audit_log(
        build_audit_record(card, result, args.model, elapsed_s)
    )

    return 0 if result["status"] == "success" else 1


if __name__ == "__main__":
    sys.exit(main())
