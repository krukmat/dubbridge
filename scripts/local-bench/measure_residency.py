#!/usr/bin/env python3
"""Measure Ollama model load/unload/reload cycle timings (ADR-036 residency rule)."""

import argparse
import datetime
import json
import os
import sys
import tempfile
import time
import urllib.error
import urllib.request

DEFAULT_HOST = "http://localhost:11434"
DEFAULT_TIMEOUT_SECONDS = 120
RESIDENT_KEEP_ALIVE = "5m"
UNLOAD_KEEP_ALIVE = 0
UNLOAD_POLL_INTERVAL_SECONDS = 0.5
UNLOAD_POLL_MAX_SECONDS = 30
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


def resident_model_names(host, timeout):
    data = http_get_json(endpoint(host, "/api/ps"), timeout)
    return {item.get("name") for item in data.get("models", [])}


def generate(host, model, keep_alive, timeout):
    payload = {
        "model": model,
        "prompt": PROBE_PROMPT,
        "stream": False,
        "keep_alive": keep_alive,
        "options": {"num_predict": 8},
    }
    return http_post_json(endpoint(host, "/api/generate"), payload, timeout)


def wait_for_unload(host, model, timeout):
    # Ollama's keep_alive=0 unload is asynchronous from the caller's point of
    # view; poll /api/ps until the model actually leaves residency instead of
    # assuming the unload request completing means the unload finished.
    deadline = time.monotonic() + UNLOAD_POLL_MAX_SECONDS
    while time.monotonic() < deadline:
        if model not in resident_model_names(host, timeout):
            return True
        time.sleep(UNLOAD_POLL_INTERVAL_SECONDS)
    return False


def timed_generate(host, model, keep_alive, timeout):
    start = time.monotonic()
    response = generate(host, model, keep_alive, timeout)
    elapsed_s = time.monotonic() - start
    return elapsed_s, response


def cycle_model(host, model, timeout):
    phases = {}

    cold_load_s, _ = timed_generate(host, model, RESIDENT_KEEP_ALIVE, timeout)
    phases["cold_load_s"] = round(cold_load_s, 3)

    unload_start = time.monotonic()
    generate(host, model, UNLOAD_KEEP_ALIVE, timeout)
    unloaded = wait_for_unload(host, model, timeout)
    phases["unload_s"] = round(time.monotonic() - unload_start, 3)
    phases["unload_confirmed"] = unloaded

    reload_s, _ = timed_generate(host, model, RESIDENT_KEEP_ALIVE, timeout)
    phases["reload_s"] = round(reload_s, 3)

    phases["total_cycle_s"] = round(
        phases["cold_load_s"] + phases["unload_s"] + phases["reload_s"], 3
    )
    return phases


def run(models, host, timeout):
    results = []
    errors = []

    for index, model in enumerate(models):
        try:
            phases = cycle_model(host, model, timeout)
            results.append({"model": model, **phases, "failed": False})
        except (
            RuntimeError,
            urllib.error.URLError,
            TimeoutError,
            ConnectionError,
            OSError,
        ) as exc:
            results.append({"model": model, "failed": True, "error": str(exc)})
            errors.append(f"{model}: {exc}")
            # EC-1: a failed model must not stay resident; best-effort unload
            # before moving on so later models aren't starved of memory.
            try:
                generate(host, model, UNLOAD_KEEP_ALIVE, timeout)
            except (urllib.error.URLError, TimeoutError, ConnectionError, OSError):
                pass

    return {
        "host": host,
        "timestamp": datetime.datetime.now(datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        ),
        "results": results,
        "errors": errors,
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


def parse_models(raw):
    models = [m.strip() for m in raw.split(",") if m.strip()]
    if not models:
        raise RuntimeError("no models supplied")
    return models


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="Measure Ollama model load/unload/reload cycle timings.",
    )
    parser.add_argument(
        "--models", required=True, help="Comma-separated Ollama model tags."
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
        models = parse_models(args.models)
    except RuntimeError as exc:
        print(f"measure_residency: {exc}", file=sys.stderr)
        return 1

    payload = run(models, args.host, args.timeout)
    write_json_atomic(payload, args.out)
    return 1 if payload["errors"] else 0


if __name__ == "__main__":
    sys.exit(main())
