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
from dataclasses import dataclass
from typing import Optional


DEFAULT_HOST = "http://localhost:11434"
DEFAULT_MODEL = "gemma4:26b-a4b-it-qat"
DEFAULT_FALLBACK_MODEL = "gemma4:26b-a4b-it-qat"
DEFAULT_IDLE_TIMEOUT_SECONDS = 180
DEFAULT_MAX_WALL_SECONDS = 900
DEFAULT_NUM_CTX = 131072
DEFAULT_NUM_PREDICT = 4096
MODEL_NUM_PREDICT_OVERRIDES = {
    "gemma4:26b-a4b-it-qat": 8192,
}
DEFAULT_TEMPERATURE = 0.1
DEFAULT_THINK = False

TRUTHY_ENV_VALUES = {"1", "true", "TRUE", "yes", "YES", "on", "ON"}


@dataclass(frozen=True)
class StreamUsage:
    response_tokens: Optional[int] = None
    prompt_tokens: Optional[int] = None
    done_reason: Optional[str] = None


@dataclass(frozen=True)
class StreamChatResult:
    content: str
    usage: StreamUsage


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


def _installed_model_names(host, timeout):
    tags = get_json(endpoint(host, "/api/tags"), timeout)
    return {item.get("name") for item in tags.get("models", [])}


def _format_available_models(installed):
    return ", ".join(sorted(name for name in installed if name)) or "<none>"


def ensure_model_available(host, model, timeout):
    installed = _installed_model_names(host, timeout)
    if model not in installed:
        available = _format_available_models(installed)
        raise RuntimeError(
            f"local Ollama model {model!r} is not installed; available: {available}",
        )
    return model


def resolve_model_with_fallback(host, model, timeout, fallback_model=None):
    installed = _installed_model_names(host, timeout)
    if model in installed:
        return model
    if fallback_model and fallback_model in installed:
        print(
            f"[gemma] model {model!r} is not installed; "
            f"falling back to {fallback_model!r}",
            file=sys.stderr,
        )
        return fallback_model
    available = _format_available_models(installed)
    if fallback_model:
        raise RuntimeError(
            f"local Ollama model {model!r} is not installed and fallback "
            f"{fallback_model!r} is not installed; available: {available}",
        )
    raise RuntimeError(
        f"local Ollama model {model!r} is not installed; available: {available}",
    )


def default_fallback_model_for(*override_env_names):
    if any(os.environ.get(name) for name in override_env_names):
        return None
    return DEFAULT_FALLBACK_MODEL


def resolve_num_predict(model, num_predict):
    """Return the effective generation budget for a specific model.

    Callers that pass a non-default value keep that explicit override. The
    shared 4096-token default stays unchanged for models without an override.
    """
    if num_predict != DEFAULT_NUM_PREDICT:
        return num_predict
    return MODEL_NUM_PREDICT_OVERRIDES.get(model, DEFAULT_NUM_PREDICT)


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
    effective_num_predict = resolve_num_predict(model, num_predict)
    return {
        "model": model,
        "stream": True,
        "think": think,
        "keep_alive": "10m",
        "options": {
            "temperature": temperature,
            "num_predict": effective_num_predict,
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


def _coerce_usage_int(value):
    if isinstance(value, bool):
        return None
    if isinstance(value, int):
        return value
    return None


def estimate_text_tokens(text):
    """Return a deterministic local token estimate for comparative telemetry only.

    This heuristic is intentionally simple and stable across runs. It is useful
    for local audit comparisons, but it is not a billing-grade substitute for a
    model/runtime-reported token count.
    """
    if not text:
        return 0
    return max(1, (len(text.encode("utf-8")) + 3) // 4)


def estimate_payload_tokens(payload):
    serialized = json.dumps(
        payload,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    )
    return estimate_text_tokens(serialized)


def stream_result_content(result):
    if isinstance(result, StreamChatResult):
        return result.content
    return result


def stream_result_usage(result):
    if isinstance(result, StreamChatResult):
        return result.usage
    return StreamUsage()


def sum_measured_tokens(values):
    values = list(values)
    if not values:
        return None
    if any(value is None for value in values):
        return None
    return sum(values)


def stream_chat(url, payload, idle_timeout, max_wall, progress_label="delegate"):
    """POST to /api/chat with stream:true, return content plus usage metadata."""
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
    usage = StreamUsage()

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
                    usage = StreamUsage(
                        response_tokens=_coerce_usage_int(chunk.get("eval_count")),
                        prompt_tokens=_coerce_usage_int(chunk.get("prompt_eval_count")),
                        done_reason=chunk.get("done_reason"),
                    )
                    if chunk.get("done_reason") == "length":
                        raise RuntimeError(
                            "response cut by token limit; output may be truncated"
                        )
                    break

    except (TimeoutError, socket.timeout) as exc:
        raise GemmaIdleTimeout(idle_timeout) from exc

    return StreamChatResult(content="".join(content_parts), usage=usage)


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


def _redact_recursive(value):
    # D14 finding (T6c): the original top-level-only redaction never
    # recursed into list/dict fields. That was invisible while every caller
    # only ever put already-redacted prompt strings in the record, but T6c's
    # "commands" field is a list of argv lists — model-controlled content
    # that can carry a credential-shaped string straight into the persisted
    # log unredacted. Every audit-log caller goes through this one function,
    # so fixing it here covers all roles, not just the new one.
    if isinstance(value, str):
        return _redact(value)
    if isinstance(value, list):
        return [_redact_recursive(item) for item in value]
    if isinstance(value, dict):
        return {k: _redact_recursive(v) for k, v in value.items()}
    return value


def append_audit_log(record, *, now=None):
    ts = now or datetime.datetime.utcnow()
    log_dir = pathlib.Path("logs/gemma-audit")
    log_dir.mkdir(parents=True, exist_ok=True)
    log_path = log_dir / ts.strftime("%Y-%m.jsonl")
    safe = {k: _redact_recursive(v) for k, v in record.items()}
    with open(log_path, "a", encoding="utf-8") as f:
        f.write(json.dumps(safe, sort_keys=True) + "\n")


def write_result(delegation, out_path):
    """Write JSON atomically: write to a temp file then rename."""
    tmp = out_path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(delegation, f, indent=2, sort_keys=True)
        f.write("\n")
    os.replace(tmp, out_path)
