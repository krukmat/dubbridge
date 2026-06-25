"""Shared local Gemma/Ollama helpers for DubBridge agent roles.

Role-specific scripts own their prompt contract and parser. This module owns the
transport, common generation options, timeout behavior, packet IO, and atomic
result writing so developer and reviewer roles do not drift.
"""

import datetime
import json
import os
import pathlib
import re
import socket
import sys
import time
import urllib.request


DEFAULT_HOST = "http://localhost:11434"
DEFAULT_MODEL = "gemma4:26b-a4b-it-qat"
DEFAULT_IDLE_TIMEOUT_SECONDS = 60
DEFAULT_MAX_WALL_SECONDS = 900
DEFAULT_NUM_CTX = 16384
DEFAULT_NUM_PREDICT = 4096
DEFAULT_TEMPERATURE = 0.1
DEFAULT_THINK = False

TRUTHY_ENV_VALUES = {"1", "true", "TRUE", "yes", "YES", "on", "ON"}


class GemmaIdleTimeout(RuntimeError):
    def __init__(self, idle):
        super().__init__(f"Gemma idle timeout after {idle}s without a token")
        self.exit_code = 124


class GemmaWallTimeout(RuntimeError):
    def __init__(self, wall):
        super().__init__(f"Gemma wall timeout after {wall}s total")
        self.exit_code = 124


def bool_from_env(name, default=False):
    value = os.environ.get(name)
    if value is None:
        return default
    return value in TRUTHY_ENV_VALUES


def read_packet(path):
    if not path or path == "-":
        return sys.stdin.read()
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def endpoint(host, path):
    if host and "://" not in host:
        host = f"http://{host}"
    return f"{host.rstrip('/')}{path}"


def get_json(url, timeout):
    request = urllib.request.Request(url, method="GET")
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8"))
    except (TimeoutError, socket.timeout) as exc:
        raise GemmaIdleTimeout(timeout) from exc


def ensure_model_available(host, model, timeout):
    tags = get_json(endpoint(host, "/api/tags"), timeout)
    installed = {item.get("name") for item in tags.get("models", [])}
    if model not in installed:
        available = ", ".join(sorted(name for name in installed if name)) or "<none>"
        raise RuntimeError(
            f"local Ollama model {model!r} is not installed; available: {available}",
        )


def build_chat_payload(
    *,
    model,
    system_prompt,
    packet,
    num_ctx,
    num_predict,
    temperature,
    think,
):
    return {
        "model": model,
        "stream": True,
        "think": think,
        "keep_alive": "10m",
        "options": {
            "temperature": temperature,
            "num_predict": num_predict,
            "num_ctx": num_ctx,
        },
        "messages": [
            {
                "role": "system",
                "content": system_prompt,
            },
            {
                "role": "user",
                "content": packet,
            },
        ],
    }


def stream_chat(url, payload, idle_timeout, max_wall, progress_label="delegate"):
    """POST to /api/chat with stream:true, return the assembled content string."""
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
                    raise GemmaWallTimeout(max_wall)

                try:
                    line = response.readline()
                except (TimeoutError, socket.timeout) as exc:
                    raise GemmaIdleTimeout(idle_timeout) from exc

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
                        f"[{progress_label}] tokens: {tokens_received} "
                        f"elapsed: {time.monotonic() - wall_start:.0f}s",
                        file=sys.stderr,
                    )

                if chunk.get("done"):
                    if chunk.get("done_reason") == "length":
                        raise RuntimeError(
                            "response cut by token limit; output may be truncated"
                        )
                    break

    except (TimeoutError, socket.timeout) as exc:
        raise GemmaIdleTimeout(idle_timeout) from exc

    return "".join(content_parts)


def normalize_tagged_content(content, label):
    if not isinstance(content, str):
        raise RuntimeError(f"{label} stream produced no content string")
    content = content.replace("\r\n", "\n").replace("\r", "\n").strip()
    if not content:
        raise RuntimeError(f"invalid {label} response: empty content")
    return content


def next_nonempty_line(lines, idx, label, error_prefix):
    while idx < len(lines) and not lines[idx].strip():
        idx += 1
    if idx >= len(lines):
        raise RuntimeError(f"{error_prefix}: missing {label} line")
    return lines[idx], idx + 1


def parse_header_value(line, label, error_prefix):
    prefix = f"{label}: "
    if not line.startswith(prefix):
        raise RuntimeError(f"{error_prefix}: expected {prefix!r}")
    value = line[len(prefix):].strip()
    if not value:
        raise RuntimeError(f"{error_prefix}: empty {label} value")
    return value


_SECRET_PATTERN = re.compile(
    r'(api[_\-]?key|token|password|secret|credential)[^\s]*\s*[=:]\s*\S+',
    re.IGNORECASE,
)


def _redact(value):
    return _SECRET_PATTERN.sub(r'\1=***REDACTED***', value)


def append_audit_log(record, *, now=None):
    ts = now or datetime.datetime.utcnow()
    log_dir = pathlib.Path("logs/gemma-audit")
    log_dir.mkdir(parents=True, exist_ok=True)
    log_path = log_dir / ts.strftime("%Y-%m.jsonl")
    safe = {k: (_redact(v) if isinstance(v, str) else v) for k, v in record.items()}
    with open(log_path, "a", encoding="utf-8") as f:
        f.write(json.dumps(safe, sort_keys=True) + "\n")


def write_result(delegation, out_path):
    """Write JSON atomically: write to a temp file then rename."""
    tmp = out_path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(delegation, f, indent=2, sort_keys=True)
        f.write("\n")
    os.replace(tmp, out_path)
