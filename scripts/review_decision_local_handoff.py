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
import systemone_local_adapter

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



def smoke(*, root: str | Path = review_decision_queue.DEFAULT_ROOT,
          host: str = DEFAULT_HOST,
          model: str = review_decision.DEFAULT_MODEL,
          timeout: int = DEFAULT_TIMEOUT) -> dict[str, Any]:
    """Run one isolated live System One request without touching shadow metrics."""
    host = normalize_host(host)
    check = preflight(root=root, host=host, model=model, timeout=timeout)
    if not check["model_present"]:
        return {"status": "blocked", "preflight": check, "smoke": None}

    state = review_decision.build_state(
        content=(
            "Synthetic DubBridge review smoke case. The change only updates a "
            "comment. Tests and contracts pass. There is no security, migration, "
            "architecture, dependency, infrastructure, or scope impact."
        ),
        phase="code",
        rri=12,
        task_id="REVIEW-DECISION-LOCAL-SMOKE",
        metadata={
            "checks": {"tests": "pass", "contracts": "pass"},
            "deterministic": {
                "security_sensitive": False,
                "migration_change": False,
                "architecture_change": False,
                "dependency_sensitive": False,
            },
            "synthetic": True,
        },
    )
    request = review_decision.build_systemone_request(
        state=state,
        model=model,
        keep_alive=0,
    )
    envelope = systemone_local_adapter.invoke_systemone(
        request,
        endpoint=_url(host, "/v1/systemone"),
        timeout=max(timeout, 120),
    )
    if envelope.get("request_sha256") != review_decision_queue.request_sha256(request):
        raise RuntimeError("smoke response request hash mismatch")
    raw = envelope.get("response")
    if not isinstance(raw, dict):
        raise RuntimeError("smoke response missing response object")
    normalized = review_decision.normalize_systemone_response(raw, request=request)
    expected = set(review_decision.QUESTION_SCHEMA)
    actual = set(normalized.get("answers", {}))
    if actual != expected:
        raise RuntimeError(
            f"smoke response answer set mismatch: expected={sorted(expected)} actual={sorted(actual)}"
        )

    answers = normalized["answers"]
    semantic_observation = {
        "risk": answers["risk"]["choice"],
        "evidence_complete": answers["evidence_complete"]["value"],
        "scope": answers["scope"]["choice"],
        "failure_domain": answers["failure_domain"]["choice"],
        "suggested_review": answers["suggested_review"]["choice"],
    }
    result = {
        "status": "pass",
        "transport_schema_valid": True,
        "model": normalized["model"],
        "request_sha256": envelope["request_sha256"],
        "latency_ms": envelope.get("latency_ms"),
        "answer_count": len(actual),
        "answers": semantic_observation,
        "metrics_excluded": True,
        "keep_alive": 0,
    }
    out = Path(root).resolve() / "local-smoke-receipt.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema_version": "dubbridge-review-decision-local-smoke-v1",
        "created_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "preflight": check,
        "smoke": result,
    }
    tmp = out.with_name(out.name + ".tmp")
    tmp.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(tmp, out)
    return {**result, "receipt": str(out)}


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
    for name in ("check", "run", "smoke"):
        cmd = sub.add_parser(name)
        cmd.add_argument("--root", default=str(review_decision_queue.DEFAULT_ROOT))
        cmd.add_argument("--host", default=DEFAULT_HOST)
        cmd.add_argument("--model", default=review_decision.DEFAULT_MODEL)
        cmd.add_argument("--timeout", type=int, default=DEFAULT_TIMEOUT)
        if name == "run":
            cmd.add_argument("--limit", type=int)
    args = parser.parse_args()
    try:
        if args.command == "check":
            result = preflight(root=args.root, host=args.host, model=args.model, timeout=args.timeout)
        elif args.command == "smoke":
            result = smoke(root=args.root, host=args.host, model=args.model, timeout=args.timeout)
        else:
            result = run(root=args.root, host=args.host, model=args.model,
                         timeout=args.timeout, limit=args.limit)
    except (RuntimeError, ValueError) as exc:
        print(json.dumps({"status": "blocked", "error": str(exc)}, indent=2))
        return 2

    if args.command == "smoke":
        print(json.dumps(result, indent=2))
        return 0 if result.get("status") == "pass" else 2

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
