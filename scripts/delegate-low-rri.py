#!/usr/bin/env python3
"""Delegate a Low-RRI task packet to local Ollama/Gemma.

The script intentionally keeps Gemma sandboxed: it receives a packet and returns
tagged text with complete file contents. The orchestrating agent still validates,
builds the diff, applies it, reviews it, and verifies any patch.

Transport: streaming (`stream: true`).  Each NDJSON chunk resets the idle
timer (`--idle-timeout`, default 60 s).  A separate `--max-wall` cap (default
900 s) guards against runaway generation.  This lets the agent invoke the script
in the background (no 120 s Bash foreground limit) and be notified when it
completes naturally — rather than racing a blind wall-clock timeout against
Gemma's generation speed.

Invoke from an agent:
    scripts/delegate-low-rri.py packet.md --out result.json
The agent watches for completion (exit 0 → read result.json) or timeout (124).
"""

import argparse
import fnmatch
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request


DEFAULT_HOST = "http://localhost:11434"
DEFAULT_MODEL = "gemma4:12b-it-q4_K_M"
DEFAULT_IDLE_TIMEOUT_SECONDS = 60
DEFAULT_MAX_WALL_SECONDS = 900
# Context window large enough to hold any realistic delegation packet + contract.
DEFAULT_NUM_CTX = 16384
DEFAULT_NUM_PREDICT = 4096

STATUS_VALUES = {"PATCH": "patch", "NO_PATCH": "no_patch", "BLOCKED": "blocked"}
ACTION_VALUES = {"create", "modify", "delete"}
FILE_START_MARKER = "=== FILE START ==="
FILE_END_MARKER = "=== FILE END ==="
CONTENT_MARKER = "--- CONTENT ---"


class DelegationIdleTimeout(RuntimeError):
    def __init__(self, idle):
        super().__init__(f"Gemma idle timeout after {idle}s without a token")
        self.exit_code = 124


class DelegationWallTimeout(RuntimeError):
    def __init__(self, wall):
        super().__init__(f"Gemma wall timeout after {wall}s total")
        self.exit_code = 124


def parse_args():
    parser = argparse.ArgumentParser(
        description="Send a Low-RRI delegation packet to local Ollama/Gemma.",
    )
    parser.add_argument(
        "packet",
        nargs="?",
        help="Packet file to send. Reads stdin when omitted or set to '-'.",
    )
    parser.add_argument(
        "--host",
        default=os.environ.get("OLLAMA_HOST", DEFAULT_HOST),
        help=f"Ollama host; defaults to OLLAMA_HOST or {DEFAULT_HOST}.",
    )
    parser.add_argument(
        "--model",
        default=os.environ.get("DUBBRIDGE_LOW_RRI_MODEL", DEFAULT_MODEL),
        help=(
            "Local model name; defaults to DUBBRIDGE_LOW_RRI_MODEL or "
            f"{DEFAULT_MODEL}."
        ),
    )
    parser.add_argument(
        "--idle-timeout",
        type=int,
        dest="idle_timeout",
        default=int(
            os.environ.get(
                "DUBBRIDGE_LOW_RRI_IDLE_TIMEOUT_SECONDS",
                str(DEFAULT_IDLE_TIMEOUT_SECONDS),
            )
        ),
        help=(
            "Seconds without a new token before treating Gemma as stalled "
            f"(exit 124); default {DEFAULT_IDLE_TIMEOUT_SECONDS}."
        ),
    )
    parser.add_argument(
        "--max-wall",
        type=int,
        dest="max_wall",
        default=int(
            os.environ.get(
                "DUBBRIDGE_LOW_RRI_MAX_WALL_SECONDS",
                str(DEFAULT_MAX_WALL_SECONDS),
            )
        ),
        help=(
            f"Hard wall-time cap in seconds (exit 124); default {DEFAULT_MAX_WALL_SECONDS}."
        ),
    )
    parser.add_argument(
        "--num-ctx",
        type=int,
        dest="num_ctx",
        default=int(
            os.environ.get("DUBBRIDGE_LOW_RRI_NUM_CTX", str(DEFAULT_NUM_CTX))
        ),
        help=(
            f"Context window size for Ollama; default {DEFAULT_NUM_CTX}. "
            "Raise if the packet + tagged contract is truncated."
        ),
    )
    parser.add_argument(
        "--num-predict",
        type=int,
        dest="num_predict",
        default=int(
            os.environ.get("DUBBRIDGE_LOW_RRI_NUM_PREDICT", str(DEFAULT_NUM_PREDICT))
        ),
        help=(
            f"Max tokens Ollama may generate; default {DEFAULT_NUM_PREDICT}. "
            "Raise when the response is truncated before the final FILE END marker."
        ),
    )
    parser.add_argument(
        "--out",
        default=None,
        metavar="FILE",
        help=(
            "Write the validated result JSON atomically to FILE instead of stdout. "
            "Recommended for agent invocations: the agent polls/reads FILE on exit 0."
        ),
    )
    parser.add_argument(
        "--allow-path",
        action="append",
        default=[],
        metavar="PATH",
        help=(
            "Repo-relative in-scope path prefix or glob; repeatable. Any file "
            "Gemma returns outside this set is rejected. Required for --apply."
        ),
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help=(
            "Build a unified diff from the returned file contents (via git) and "
            "apply it with `git apply`. Requires --allow-path. The diff is "
            "constructed deterministically by this script, never by the model."
        ),
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        metavar="DIR",
        help="Repository root for scope checks and git apply; default cwd.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the Ollama request payload without sending it.",
    )
    return parser.parse_args()


def read_packet(path):
    if not path or path == "-":
        return sys.stdin.read()
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def endpoint(host, path):
    return f"{host.rstrip('/')}{path}"


def get_json(url, timeout):
    request = urllib.request.Request(url, method="GET")
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8"))
    except (TimeoutError, socket.timeout) as exc:
        raise DelegationIdleTimeout(timeout) from exc


def ensure_model_available(host, model, timeout):
    tags = get_json(endpoint(host, "/api/tags"), timeout)
    installed = {item.get("name") for item in tags.get("models", [])}
    if model not in installed:
        available = ", ".join(sorted(name for name in installed if name)) or "<none>"
        raise RuntimeError(
            f"local Ollama model {model!r} is not installed; available: {available}",
        )


def build_payload(model, packet, num_ctx, num_predict):
    return {
        "model": model,
        "stream": True,
        "think": False,
        "keep_alive": "10m",
        "options": {
            "temperature": 0.1,
            "num_predict": num_predict,
            "num_ctx": num_ctx,
        },
        "messages": [
            {
                "role": "system",
                "content": (
                    "You are a sandboxed local delegation model for DubBridge "
                    "Low-RRI tasks.\n"
                    "Return ONLY tagged text in this exact shape:\n"
                    "STATUS: PATCH|NO_PATCH|BLOCKED\n"
                    "SUMMARY: short summary\n"
                    "TEST: optional verification command\n"
                    "RISK: optional risk note\n"
                    "=== FILE START ===\n"
                    "PATH: relative/path.ext\n"
                    "ACTION: create|modify|delete\n"
                    "--- CONTENT ---\n"
                    "<COMPLETE final file contents>\n"
                    "=== FILE END ===\n"
                    "Rules: no JSON, no markdown fences, no unified diff, no "
                    "explanations, no extra text outside these sections. For "
                    "ACTION delete, emit empty content."
                ),
            },
            {
                "role": "user",
                "content": packet,
            },
        ],
    }


def validate_delegation_payload(payload):
    required = ["status", "summary", "files", "test_commands", "risk_notes"]
    missing = [key for key in required if key not in payload]
    if missing:
        raise RuntimeError(f"Gemma response is missing required keys: {missing}")
    if payload["status"] not in {"patch", "no_patch", "blocked"}:
        raise RuntimeError(f"Gemma response has invalid status: {payload['status']!r}")
    if not isinstance(payload["files"], list):
        raise RuntimeError("Gemma response files must be an array")
    for i, entry in enumerate(payload["files"]):
        if not isinstance(entry, dict):
            raise RuntimeError(f"Gemma response files[{i}] is not an object")
        for key in ("path", "action", "contents"):
            if key not in entry:
                raise RuntimeError(f"Gemma response files[{i}] missing {key!r}")
        if entry["action"] not in {"create", "modify", "delete"}:
            raise RuntimeError(
                f"Gemma response files[{i}] has invalid action: {entry['action']!r}")
        if not isinstance(entry["path"], str) or not entry["path"].strip():
            raise RuntimeError(f"Gemma response files[{i}] path must be a non-empty string")
        if not isinstance(entry["contents"], str):
            raise RuntimeError(f"Gemma response files[{i}] contents must be a string")
    if payload["status"] == "patch" and not payload["files"]:
        raise RuntimeError("Gemma response status is 'patch' but files is empty")
    if not isinstance(payload["test_commands"], list):
        raise RuntimeError("Gemma response test_commands must be an array")
    if not isinstance(payload["risk_notes"], list):
        raise RuntimeError("Gemma response risk_notes must be an array")


def stream_chat(url, payload, idle_timeout, max_wall):
    """POST to /api/chat with stream:true, return the assembled content string.

    Reads NDJSON lines from the response.  Each received line resets the idle
    timer.  Raises DelegationIdleTimeout if no data arrives within `idle_timeout`
    seconds; raises DelegationWallTimeout if the total elapsed time exceeds
    `max_wall` seconds.
    """
    data = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    wall_start = time.monotonic()
    content_parts = []
    tokens_received = 0

    try:
        with urllib.request.urlopen(request, timeout=idle_timeout) as response:
            while True:
                elapsed = time.monotonic() - wall_start
                if elapsed > max_wall:
                    raise DelegationWallTimeout(max_wall)

                # Read one NDJSON line.  urlopen's timeout applies per read
                # attempt, acting as our idle-timeout: if Gemma stalls and
                # sends nothing for `idle_timeout` seconds, socket.timeout fires.
                try:
                    line = response.readline()
                except (TimeoutError, socket.timeout) as exc:
                    raise DelegationIdleTimeout(idle_timeout) from exc

                if not line:
                    break

                try:
                    chunk = json.loads(line.decode("utf-8"))
                except json.JSONDecodeError:
                    continue

                msg = chunk.get("message", {})
                fragment = msg.get("content", "")
                if fragment:
                    content_parts.append(fragment)
                    tokens_received += 1
                    print(
                        f"[delegate] tokens: {tokens_received} "
                        f"elapsed: {time.monotonic() - wall_start:.0f}s",
                        file=sys.stderr,
                    )

                if chunk.get("done"):
                    break

    except (TimeoutError, socket.timeout) as exc:
        raise DelegationIdleTimeout(idle_timeout) from exc

    return "".join(content_parts)


def parse_stream_content(content):
    if not isinstance(content, str):
        raise RuntimeError("stream produced no content string")
    content = content.replace("\r\n", "\n").replace("\r", "\n").strip()
    if not content:
        raise RuntimeError("invalid tagged response: empty content")
    payload = parse_tagged_response(content)
    validate_delegation_payload(payload)
    return payload


def parse_tagged_response(content):
    lines = content.split("\n")
    idx = 0
    status = None
    summary = None
    test_commands = []
    risk_notes = []
    files = []
    seen_paths = set()

    def skip_blank(i):
        while i < len(lines) and not lines[i].strip():
            i += 1
        return i

    idx = skip_blank(idx)
    while idx < len(lines):
        line = lines[idx]
        if line == FILE_START_MARKER:
            block, idx = parse_file_block(lines, idx)
            path = block["path"]
            if path in seen_paths:
                raise RuntimeError(f"invalid tagged response: duplicate path {path!r}")
            seen_paths.add(path)
            files.append(block)
            idx = skip_blank(idx)
            continue
        if line.startswith("STATUS: "):
            if status is not None:
                raise RuntimeError("invalid tagged response: duplicate STATUS header")
            raw_status = line[len("STATUS: "):].strip()
            if raw_status not in STATUS_VALUES:
                raise RuntimeError(f"invalid tagged response: unknown STATUS {raw_status!r}")
            status = STATUS_VALUES[raw_status]
        elif line.startswith("SUMMARY: "):
            if summary is not None:
                raise RuntimeError("invalid tagged response: duplicate SUMMARY header")
            summary = line[len("SUMMARY: "):].strip()
        elif line.startswith("TEST: "):
            test_commands.append(line[len("TEST: "):].strip())
        elif line.startswith("RISK: "):
            risk_notes.append(line[len("RISK: "):].strip())
        else:
            raise RuntimeError(
                f"invalid tagged response: unexpected text outside sections: {line!r}"
            )
        idx += 1
        idx = skip_blank(idx)

    if status is None:
        raise RuntimeError("invalid tagged response: missing STATUS header")
    if summary is None:
        raise RuntimeError("invalid tagged response: missing SUMMARY header")
    return {
        "status": status,
        "summary": summary,
        "files": files,
        "test_commands": [cmd for cmd in test_commands if cmd],
        "risk_notes": [note for note in risk_notes if note],
    }


def parse_file_block(lines, start_idx):
    idx = start_idx
    if lines[idx] != FILE_START_MARKER:
        raise RuntimeError("invalid tagged response: missing file start marker")
    idx += 1
    path_line, idx = _next_nonempty_line(lines, idx, "PATH")
    action_line, idx = _next_nonempty_line(lines, idx, "ACTION")
    content_line, idx = _next_nonempty_line(lines, idx, CONTENT_MARKER)

    path = _parse_header_value(path_line, "PATH")
    action = _parse_header_value(action_line, "ACTION")
    if action not in ACTION_VALUES:
        raise RuntimeError(f"invalid tagged response: unknown ACTION {action!r}")
    if content_line != CONTENT_MARKER:
        raise RuntimeError("invalid tagged response: missing content marker")

    content_lines = []
    while idx < len(lines) and lines[idx] != FILE_END_MARKER:
        content_lines.append(lines[idx])
        idx += 1
    if idx >= len(lines):
        raise RuntimeError("invalid tagged response: missing file end marker")
    idx += 1
    contents = "\n".join(content_lines)
    if action == "delete" and contents:
        raise RuntimeError("invalid tagged response: delete action requires empty content")
    return {
        "path": path,
        "action": action,
        "contents": contents,
    }, idx


def _next_nonempty_line(lines, idx, label):
    while idx < len(lines) and not lines[idx].strip():
        idx += 1
    if idx >= len(lines):
        raise RuntimeError(f"invalid tagged response: missing {label} line")
    return lines[idx], idx + 1


def _parse_header_value(line, label):
    prefix = f"{label}: "
    if not line.startswith(prefix):
        raise RuntimeError(f"invalid tagged response: expected {prefix!r}")
    value = line[len(prefix):].strip()
    if not value:
        raise RuntimeError(f"invalid tagged response: empty {label} value")
    return value


def write_result(delegation, out_path):
    """Write JSON atomically: write to a temp file then rename."""
    tmp = out_path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(delegation, f, indent=2, sort_keys=True)
        f.write("\n")
    os.replace(tmp, out_path)


# --- Caller-side patch construction (the work the small model can't do) ---------
# Gemma returns full file contents; the deterministic steps below — scope
# enforcement, diff construction, and `git apply` — run in the caller, never the
# model. This is the part the workflow guide assigns to the orchestrating agent.

def _normalize(path):
    """Collapse a repo-relative path, rejecting absolute or parent-escaping paths."""
    norm = os.path.normpath(path)
    if os.path.isabs(norm) or norm.startswith(".."):
        raise RuntimeError(f"path escapes the repository: {path!r}")
    return norm


def enforce_scope(files, allowed):
    """Reject any file whose path is not within an allowed glob/prefix.

    `allowed` is a list of repo-relative path prefixes or globs. An empty list
    means 'no scope restriction declared' and is rejected outright: a Low-RRI
    delegation must always run against an explicit allowed-path set so the model
    cannot touch files outside the task.
    """
    if not allowed:
        raise RuntimeError(
            "no --allow-path given; Low-RRI delegation requires an explicit "
            "in-scope path set before any file is written")
    norm_allowed = [_normalize(a) for a in allowed]
    for entry in files:
        path = _normalize(entry["path"])
        ok = any(path == a or path.startswith(a.rstrip("/") + "/")
                 or fnmatch.fnmatch(path, a) for a in norm_allowed)
        if not ok:
            raise RuntimeError(
                f"out-of-scope path {entry['path']!r}; allowed: {allowed}")


def validate_file_actions(files, repo_root):
    for entry in files:
        rel = _normalize(entry["path"])
        target = os.path.join(repo_root, rel)
        action = entry["action"]
        exists = os.path.exists(target)
        if action == "create" and exists:
            raise RuntimeError(
                f"invalid file action: create targets existing path {entry['path']!r}"
            )
        if action == "modify" and not exists:
            raise RuntimeError(
                f"invalid file action: modify targets missing path {entry['path']!r}"
            )
        if action == "delete" and not exists:
            raise RuntimeError(
                f"invalid file action: delete targets missing path {entry['path']!r}"
            )


def build_diff(files, repo_root):
    """Build a single unified diff from full file contents using `git diff --no-index`.

    For each file, write the proposed contents to a temp file and diff it against
    the current on-disk version (or /dev/null for creates). git owns all hunk
    framing, so the result always applies cleanly — sidestepping the model's
    diff-formatting weakness entirely. Returns the assembled diff text.
    """
    parts = []
    for entry in files:
        rel = _normalize(entry["path"])
        target = os.path.join(repo_root, rel)
        action = entry["action"]

        if action == "delete":
            old = target if os.path.exists(target) else os.devnull
            new = os.devnull
            cleanup = None
        else:
            old = target if os.path.exists(target) else os.devnull
            tmp = tempfile.NamedTemporaryFile(
                mode="w", encoding="utf-8", suffix=".new", delete=False)
            tmp.write(entry["contents"])
            tmp.close()
            new = tmp.name
            cleanup = tmp.name

        try:
            out = subprocess.run(
                ["git", "diff", "--no-index", "--no-color",
                 f"--src-prefix=a/{os.path.dirname(rel)}/".replace("//", "/"),
                 old, new],
                capture_output=True, text=True, cwd=repo_root)
        finally:
            if cleanup:
                os.unlink(cleanup)

        # git diff --no-index uses exit 1 to mean "files differ" (not an error).
        diff = _relabel_diff(out.stdout, old, new, rel, action)
        if diff:
            parts.append(diff)
    return "\n".join(parts)


def _relabel_diff(diff_text, old, new, rel, action):
    """Rewrite git's temp-file paths in a --no-index diff to real a/ b/ repo paths."""
    if not diff_text.strip():
        return ""
    lines = diff_text.splitlines()
    out = []
    for line in lines:
        if line.startswith("diff --git"):
            out.append(f"diff --git a/{rel} b/{rel}")
        elif line.startswith("--- "):
            out.append("--- /dev/null" if old == os.devnull else f"--- a/{rel}")
        elif line.startswith("+++ "):
            out.append("+++ /dev/null" if new == os.devnull else f"+++ b/{rel}")
        else:
            out.append(line)
    return "\n".join(out) + "\n"


def apply_diff(diff_text, repo_root):
    """Validate then apply a unified diff with git. Raises on a check failure."""
    if not diff_text.strip():
        return "empty diff; nothing to apply"
    with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", suffix=".patch", delete=False) as f:
        f.write(diff_text)
        patch = f.name
    try:
        check = subprocess.run(["git", "apply", "--check", patch],
                               capture_output=True, text=True, cwd=repo_root)
        if check.returncode != 0:
            raise RuntimeError(f"git apply --check failed: {check.stderr.strip()}")
        applied = subprocess.run(["git", "apply", patch],
                                 capture_output=True, text=True, cwd=repo_root)
        if applied.returncode != 0:
            raise RuntimeError(f"git apply failed: {applied.stderr.strip()}")
    finally:
        os.unlink(patch)
    return "applied"


def main():
    args = parse_args()
    packet = read_packet(args.packet).strip()
    if not packet:
        raise RuntimeError("delegation packet is empty")

    payload = build_payload(args.model, packet, args.num_ctx, args.num_predict)
    if args.dry_run:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    if args.apply and not args.allow_path:
        raise RuntimeError("--apply requires at least one --allow-path")

    ensure_model_available(args.host, args.model, args.idle_timeout)
    content = stream_chat(
        endpoint(args.host, "/api/chat"),
        payload,
        idle_timeout=args.idle_timeout,
        max_wall=args.max_wall,
    )
    delegation = parse_stream_content(content)

    # Caller-side, deterministic: enforce scope, validate file actions, build the
    # diff with git, and apply it. The model never frames a diff; this is the
    # orchestrator's responsibility.
    if args.allow_path and delegation["status"] == "patch":
        enforce_scope(delegation["files"], args.allow_path)
        validate_file_actions(delegation["files"], args.repo_root)
        diff = build_diff(delegation["files"], args.repo_root)
        delegation["unified_diff"] = diff
        print(f"[delegate] built diff: {len(diff.splitlines())} lines from "
              f"{len(delegation['files'])} file(s)", file=sys.stderr)
        if args.apply:
            outcome = apply_diff(diff, args.repo_root)
            delegation["apply_result"] = outcome
            print(f"[delegate] {outcome}", file=sys.stderr)

    result_json = json.dumps(delegation, indent=2, sort_keys=True)
    if args.out:
        write_result(delegation, args.out)
        print(f"[delegate] result written to {args.out}", file=sys.stderr)
    else:
        print(result_json)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (DelegationIdleTimeout, DelegationWallTimeout) as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(124)
    except urllib.error.URLError as exc:
        print(f"Gemma/Ollama request failed: {exc}", file=sys.stderr)
        raise SystemExit(2)
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
