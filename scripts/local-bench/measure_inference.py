#!/usr/bin/env python3
"""Measure decode/prefill throughput, TTFT, and peak memory for an Ollama model binding."""

import argparse
import datetime
import json
import os
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request

try:
    import psutil
except ImportError:
    psutil = None

DEFAULT_HOST = "http://localhost:11434"
DEFAULT_TIMEOUT_SECONDS = 120
DEFAULT_NUM_PREDICT = 64
MEMORY_POLL_INTERVAL_SECONDS = 0.5

SIZE_TARGET_TOKENS = {"8k": 8192, "16k": 16384, "32k": 32768}

# ~4 characters per token is a common English-text approximation; good enough
# for the +/-10% synthetic-prompt tolerance this script requires.
CHARS_PER_TOKEN = 4
FILLER_SENTENCE = (
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod "
    "tempor incididunt ut labore et dolore magna aliqua. "
)


def build_prompt(target_tokens):
    target_chars = target_tokens * CHARS_PER_TOKEN
    repeats = target_chars // len(FILLER_SENTENCE) + 1
    return (FILLER_SENTENCE * repeats)[:target_chars]


def normalize_host(host):
    # OLLAMA_HOST is commonly set as "host:port" with no scheme (Ollama's own
    # CLI accepts that); urllib requires an explicit scheme or every request
    # fails with "unknown url type".
    if "://" not in host:
        return f"http://{host}"
    return host


def endpoint(host, path):
    return f"{normalize_host(host).rstrip('/')}{path}"


def http_get_json(url, timeout):
    request = urllib.request.Request(url, method="GET")
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.loads(response.read().decode("utf-8"))


def http_post_json(url, payload, timeout):
    data = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.loads(response.read().decode("utf-8"))


def installed_model_names(host, timeout):
    tags = http_get_json(endpoint(host, "/api/tags"), timeout)
    return {item.get("name") for item in tags.get("models", [])}


def ensure_model_available(host, model, timeout):
    installed = installed_model_names(host, timeout)
    if model not in installed:
        available = ", ".join(sorted(n for n in installed if n)) or "<none>"
        raise RuntimeError(
            f"model {model!r} not found on {host}; available: {available}"
        )


OLLAMA_PROCESS_NAME_MARKERS = ("ollama", "llama-server")


def ollama_process_rss_bytes():
    if psutil is None:
        return None
    total = None
    for proc in psutil.process_iter(["name", "memory_info"]):
        try:
            name = (proc.info["name"] or "").lower()
            # The actual inference engine runs as a separate "llama-server"
            # child process; its RSS (tens of GB) dominates the parent
            # "Ollama"/"ollama serve" processes (a few MB) and was previously
            # excluded entirely, undercounting peak memory by orders of
            # magnitude.
            if not any(marker in name for marker in OLLAMA_PROCESS_NAME_MARKERS):
                continue
            rss = proc.info["memory_info"].rss
            total = rss if total is None else total + rss
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    return total


def ollama_size_vram_bytes(host, model, timeout):
    try:
        data = http_get_json(endpoint(host, "/api/ps"), timeout)
    except (urllib.error.URLError, TimeoutError, OSError):
        return None
    for item in data.get("models", []):
        if item.get("name") == model or item.get("model") == model:
            size_vram = item.get("size_vram")
            if isinstance(size_vram, int):
                return size_vram
    return None


def sample_peak_memory(host, model, timeout):
    if psutil is not None:
        return ollama_process_rss_bytes()
    return ollama_size_vram_bytes(host, model, timeout)


class MemoryPoller:
    def __init__(self, host, model, timeout):
        self._host = host
        self._model = model
        self._timeout = timeout
        self._peak = None
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, daemon=True)

    def _run(self):
        while not self._stop.is_set():
            sample = sample_peak_memory(self._host, self._model, self._timeout)
            if sample is not None and (self._peak is None or sample > self._peak):
                self._peak = sample
            self._stop.wait(MEMORY_POLL_INTERVAL_SECONDS)

    def __enter__(self):
        self._thread.start()
        return self

    def __exit__(self, *exc_info):
        self._stop.set()
        self._thread.join(timeout=MEMORY_POLL_INTERVAL_SECONDS * 4)

    @property
    def peak_bytes(self):
        return self._peak


def measure_size(host, model, size_label, target_tokens, timeout):
    prompt = build_prompt(target_tokens)
    payload = {
        "model": model,
        "prompt": prompt,
        "stream": False,
        "options": {"num_predict": DEFAULT_NUM_PREDICT},
    }

    with MemoryPoller(host, model, timeout) as poller:
        response = http_post_json(endpoint(host, "/api/generate"), payload, timeout)

    prompt_eval_count = response.get("prompt_eval_count", 0)
    prompt_eval_duration_ns = response.get("prompt_eval_duration", 0)
    eval_count = response.get("eval_count", 0)
    eval_duration_ns = response.get("eval_duration", 0)

    prefill_tok_s = (
        prompt_eval_count / (prompt_eval_duration_ns / 1e9)
        if prompt_eval_duration_ns
        else 0.0
    )
    decode_tok_s = (
        eval_count / (eval_duration_ns / 1e9) if eval_duration_ns else 0.0
    )
    # Non-streaming /api/generate returns only after the full response is
    # built, so the first decoded token cannot precede prompt processing;
    # prompt_eval_duration is the closest available TTFT proxy.
    ttft_ms = prompt_eval_duration_ns / 1e6

    return {
        "size_label": size_label,
        "target_tokens": target_tokens,
        "prompt_eval_count": prompt_eval_count,
        "prompt_eval_duration_ns": prompt_eval_duration_ns,
        "prefill_tok_s": round(prefill_tok_s, 1),
        "ttft_ms": round(ttft_ms, 1),
        "eval_count": eval_count,
        "eval_duration_ns": eval_duration_ns,
        "decode_tok_s": round(decode_tok_s, 1),
        "peak_memory_bytes": poller.peak_bytes,
    }


def parse_sizes(raw):
    labels = [s.strip().lower() for s in raw.split(",") if s.strip()]
    unknown = [label for label in labels if label not in SIZE_TARGET_TOKENS]
    if unknown:
        raise RuntimeError(f"unknown size label(s): {', '.join(unknown)}")
    return labels


def write_json_atomic(payload, out_path):
    directory = os.path.dirname(os.path.abspath(out_path)) or "."
    fd, tmp_path = tempfile.mkstemp(dir=directory, suffix=".tmp")
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            json.dump(payload, f, indent=2, sort_keys=True)
            f.write("\n")
        os.replace(tmp_path, out_path)
    except BaseException:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)
        raise


def run(model, host, sizes, timeout):
    ensure_model_available(host, model, timeout)

    results = []
    errors = []
    for label in sizes:
        target_tokens = SIZE_TARGET_TOKENS[label]
        results.append(measure_size(host, model, label, target_tokens, timeout))

    return {
        "model": model,
        "host": host,
        "timestamp": datetime.datetime.now(datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        ),
        "results": results,
        "errors": errors,
    }


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="Measure Ollama model inference throughput and memory.",
    )
    parser.add_argument("--model", required=True, help="Ollama model tag to measure.")
    parser.add_argument(
        "--sizes",
        default="8k,16k,32k",
        help="Comma-separated prompt sizes to measure (8k,16k,32k).",
    )
    parser.add_argument("--out", required=True, help="Path to write the JSON artifact.")
    parser.add_argument(
        "--host",
        default=os.environ.get("OLLAMA_HOST", DEFAULT_HOST),
        help=f"Ollama host; defaults to OLLAMA_HOST or {DEFAULT_HOST}.",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=DEFAULT_TIMEOUT_SECONDS,
        help="Per-request timeout in seconds.",
    )
    return parser.parse_args(argv)


def main(argv=None):
    args = parse_args(argv)
    try:
        sizes = parse_sizes(args.sizes)
        payload = run(args.model, args.host, sizes, args.timeout)
    except (
        RuntimeError,
        urllib.error.URLError,
        TimeoutError,
        ConnectionError,
        OSError,
    ) as exc:
        print(f"measure_inference: {exc}", file=sys.stderr)
        return 1

    write_json_atomic(payload, args.out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
