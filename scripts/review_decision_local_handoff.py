#!/usr/bin/env python3
"""One-command local handoff for DubBridge review-decision shadow processing.

All orchestration is standard-library and testable without Ollama. The live
commands only probe the local Ollama API and, on explicit `run`, execute the
already-captured pending batch.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
from pathlib import Path
import urllib.error
import urllib.request
from typing import Any, Callable

import review_decision
import review_decision_queue

DEFAULT_HOST = os.environ.get("OLLAMA_HOST", "http://localhost:11434")
DEFAULT_TIMEOUT = 10


def normalize_host(host: str) -> str:
    value = host.strip()
    if not value:
        return "http://localhost:11434"
    if "://" not in value:
        value = "http://" + value
    return value.rstrip("/")


def _url(host: str, path: str) -> str:
    return normalize_host(host) + path


def get_json(url: str, *, timeout: int = DEFAULT_TIMEOUT,
             opener: Callable[..., Any] = urllib.request.urlopen) -> dict[str, Any]:
    req = urllib.request.Request(url, method="GET")
    try:
        with opener(req, timeout=timeout) as response:
            raw = response.read()
    except (urllib.error.URLError, TimeoutError, OSError) as exc:
        raise RuntimeError(f"local Ollama probe failed: {exc}") from exc
    try:
        value = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise RuntimeError("local Ollama probe returned invalid JSON") from exc
    if not isinstance(value, dict):
        raise RuntimeError("local Ollama probe returned non-object JSON")
    return value


def _model_names(tags: dict[str, Any]) -> set[str]:
    models = tags.get("models")
    if not isinstance(models, list):
        return set()
    names: set[str] = set()
    for item in models:
        if not isinstance(item, dict):
            continue
        for key in ("name", "model"):
            value = item.get(key)
            if isinstance(value, str) and value:
                names.add(value)
    return names


def preflight(*, root: str | Path = review_decision_queue.DEFAULT_ROOT,
              host: str = DEFAULT_HOST,
              model: str = review_decision.DEFAULT_MODEL,
              timeout: int = DEFAULT_TIMEOUT,
              opener: Callable[..., Any] = urllib.request.urlopen) -> dict[str, Any]:
    pending = review_decision_queue.pending(root)
    host = normalize_host(host)
    version = get_json(_url(host, "/api/version"), timeout=timeout, opener=opener)
    tags = get_json(_url(host, "/api/tags"), timeout=timeout, opener=opener)
    names = _model_names(tags)
    model_present = model in names
    return {
        "status": "ready" if pending and model_present else ("idle" if not pending else "blocked"),
        "ready": bool(pending) and model_present,
        "host": host,
        "ollama_version": version.get("version"),
        "model": model,
        "model_present": model_present,
        "pending": len(pending),
        "pending_manifests": [str(path) for path in pending],
        "remediation": None if model_present else f"ollama pull {model}",
    }


def run(*, root: str | Path = review_decision_queue.DEFAULT_ROOT,
        host: str = DEFAULT_HOST,
        model: str = review_decision.DEFAULT_MODEL,
        timeout: int = DEFAULT_TIMEOUT,
        limit: int | None = None,
        opener: Callable[..., Any] = urllib.request.urlopen) -> dict[str, Any]:
    host = normalize_host(host)
    check = preflight(root=root, host=host, model=model, timeout=timeout, opener=opener)
    if not check["ready"]:
        return {"preflight": check, "batch": None}
    batch = review_decision_queue.run_local_batch(
        root=root,
        limit=limit,
        endpoint=_url(host, "/v1/systemone"),
        timeout=max(timeout, 120),
    )
    return {"preflight": check, "batch": batch}


def _write_receipt(root: str | Path, result: dict[str, Any]) -> Path:
    out = Path(root).resolve() / "local-run-receipt.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema_version": "dubbridge-review-decision-local-run-v1",
        "created_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        **result,
    }
    tmp = out.with_name(out.name + ".tmp")
    tmp.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(tmp, out)
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description="Local handoff for review-decision shadow batches")
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("check", "run"):
        cmd = sub.add_parser(name)
        cmd.add_argument("--root", default=str(review_decision_queue.DEFAULT_ROOT))
        cmd.add_argument("--host", default=DEFAULT_HOST)
        cmd.add_argument("--model", default=review_decision.DEFAULT_MODEL)
        cmd.add_argument("--timeout", type=int, default=DEFAULT_TIMEOUT)
        if name == "run":
            cmd.add_argument("--limit", type=int)
    args = parser.parse_args()
    try:
        result = (
            preflight(root=args.root, host=args.host, model=args.model, timeout=args.timeout)
            if args.command == "check"
            else run(root=args.root, host=args.host, model=args.model,
                     timeout=args.timeout, limit=args.limit)
        )
    except RuntimeError as exc:
        print(json.dumps({"status": "blocked", "error": str(exc)}, indent=2))
        return 2

    receipt = _write_receipt(args.root, result)
    print(json.dumps({**result, "receipt": str(receipt)}, indent=2))
    if args.command == "check":
        return 0 if result["ready"] or result["status"] == "idle" else 2
    batch = result.get("batch")
    if batch is None:
        return 0 if result["preflight"]["status"] == "idle" else 2
    return 0 if not batch.get("failures") else 2


if __name__ == "__main__":
    raise SystemExit(main())
