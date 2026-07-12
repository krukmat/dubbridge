#!/usr/bin/env python3
"""Sample decode throughput/memory/swap at a fixed interval during a dev-stack contention soak."""

import argparse
import datetime
import json
import os
import statistics
import sys
import tempfile
import time
import urllib.error
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import measure_inference

try:
    import psutil
except ImportError:
    psutil = None

DEFAULT_HOST = "http://localhost:11434"
DEFAULT_DURATION_SECONDS = 3600
DEFAULT_INTERVAL_SECONDS = 30
DEFAULT_TIMEOUT_SECONDS = 30
DEFAULT_NUM_PREDICT = 16
PROBE_PROMPT = "Reply with exactly one word: OK"


def normalize_host(host):
    # OLLAMA_HOST is commonly set as "host:port" with no scheme (Ollama's own
    # CLI accepts that); urllib requires an explicit scheme or every request
    # fails with "unknown url type".
    if "://" not in host:
        return f"http://{host}"
    return host


def endpoint(host, path):
    return f"{normalize_host(host).rstrip('/')}{path}"


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


def swap_used_bytes():
    if psutil is None:
        return None
    return psutil.swap_memory().used


def take_sample(host, model, elapsed_s, timeout):
    payload = {
        "model": model,
        "prompt": PROBE_PROMPT,
        "stream": False,
        "options": {"num_predict": DEFAULT_NUM_PREDICT},
    }
    try:
        response = http_post_json(endpoint(host, "/api/generate"), payload, timeout)
    except (urllib.error.URLError, TimeoutError, ConnectionError, OSError) as exc:
        return {
            "elapsed_s": elapsed_s,
            "decode_tok_s": None,
            "peak_memory_bytes": None,
            "swap_used_bytes": swap_used_bytes(),
            "sample_ok": False,
            "error": str(exc),
        }

    eval_count = response.get("eval_count", 0)
    eval_duration_ns = response.get("eval_duration", 0)
    decode_tok_s = eval_count / (eval_duration_ns / 1e9) if eval_duration_ns else 0.0

    return {
        "elapsed_s": elapsed_s,
        "decode_tok_s": round(decode_tok_s, 1),
        # reuse T2's memory strategy so soak numbers are directly comparable
        # to the point-in-time measure_inference.py readings
        "peak_memory_bytes": measure_inference.sample_peak_memory(host, model, timeout),
        "swap_used_bytes": swap_used_bytes(),
        "sample_ok": True,
    }


def summarize(samples):
    ok_samples = [s for s in samples if s["sample_ok"]]
    decode_values = [s["decode_tok_s"] for s in ok_samples]
    memory_values = [s["peak_memory_bytes"] for s in ok_samples if s["peak_memory_bytes"] is not None]
    swap_values = [s["swap_used_bytes"] for s in ok_samples if s["swap_used_bytes"] is not None]

    return {
        "min_decode_tok_s": min(decode_values) if decode_values else None,
        "median_decode_tok_s": statistics.median(decode_values) if decode_values else None,
        "peak_swap_used_bytes": max(swap_values) if swap_values else None,
        "peak_memory_bytes": max(memory_values) if memory_values else None,
        "failed_sample_count": len(samples) - len(ok_samples),
        "total_samples": len(samples),
    }


def run(model, host, duration_seconds, interval_seconds, timeout):
    samples = []
    started_at = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    start = time.monotonic()

    while True:
        tick_start = time.monotonic()
        elapsed_s = round(tick_start - start)
        samples.append(take_sample(host, model, elapsed_s, timeout))

        if tick_start - start >= duration_seconds:
            break
        # subtract the sample's own cost so ticks stay aligned to the interval
        remaining = interval_seconds - (time.monotonic() - tick_start)
        if remaining > 0:
            time.sleep(remaining)
        if time.monotonic() - start >= duration_seconds:
            break

    return {
        "model": model,
        "host": host,
        "duration_seconds": duration_seconds,
        "interval_seconds": interval_seconds,
        "started_at": started_at,
        "samples": samples,
        "summary": summarize(samples),
        # placeholder only: macOS has no stdlib/psutil throttle signal;
        # a future heuristic could read the decode_tok_s series for collapse
        "throttle_detected": None,
    }


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


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="Sample Ollama decode throughput/memory/swap during a dev-stack contention soak.",
    )
    parser.add_argument("--model", required=True, help="Ollama model tag to probe.")
    parser.add_argument(
        "--duration-seconds",
        type=int,
        default=DEFAULT_DURATION_SECONDS,
        help="Total soak duration in seconds.",
    )
    parser.add_argument(
        "--interval-seconds",
        type=int,
        default=DEFAULT_INTERVAL_SECONDS,
        help="Seconds between samples.",
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
        help="Per-sample generate call timeout in seconds.",
    )
    return parser.parse_args(argv)


def main(argv=None):
    args = parse_args(argv)
    payload = run(
        args.model, args.host, args.duration_seconds, args.interval_seconds, args.timeout
    )
    write_json_atomic(payload, args.out)
    return 1 if payload["summary"]["failed_sample_count"] else 0


if __name__ == "__main__":
    sys.exit(main())
