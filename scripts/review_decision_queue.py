#!/usr/bin/env python3
"""Non-authoritative shadow queue for review decision models."""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
from pathlib import Path
from typing import Any

import review_decision as decision
import review_decision_shadow as shadow
import systemone_local_adapter as local_adapter

SCHEMA = "dubbridge-review-decision-queue-v1"
REPO = Path(__file__).resolve().parent.parent
DEFAULT_ROOT = REPO / ".agent" / "review-decision"


def _json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path}: expected JSON object")
    return value


def _write(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(tmp, path)


def request_sha256(request: dict[str, Any]) -> str:
    raw = json.dumps(request, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    return hashlib.sha256(raw).hexdigest()


def _slug(value: str) -> str:
    return re.sub(r"[^a-zA-Z0-9._-]+", "-", value).strip("-").lower() or "unknown"


def _rooted(root: Path, value: str) -> Path:
    path = Path(value)
    return path.resolve() if path.is_absolute() else (root / path).resolve()


def _review_path(value: str) -> Path:
    path = Path(value)
    return path.resolve() if path.is_absolute() else (REPO / path).resolve()


def _review_ref(value: str) -> str:
    path = Path(value).resolve()
    try:
        return path.relative_to(REPO).as_posix()
    except ValueError:
        return str(path)


def enqueue(*, content: str, phase: str, rri: int, task_id: str | None,
            review_artifact: str, metadata_path: str | None = None,
            root: str | Path = DEFAULT_ROOT,
            model: str = decision.DEFAULT_MODEL) -> dict[str, Any]:
    """Write a pending request only; never invokes a model."""
    root = Path(root).resolve()
    metadata = decision.read_json(metadata_path) if metadata_path else None
    state = decision.build_state(content=content, phase=phase, rri=rri,
                                 task_id=task_id or "unknown", metadata=metadata)
    request = decision.build_systemone_request(state=state, model=model)
    sha = request_sha256(request)
    stem = f"{_slug(task_id or 'unknown')}-{phase}-{state['content_sha256'][:16]}"
    manifest_path = root / "pending" / f"{stem}.manifest.json"
    request_rel = f"pending/{stem}.request.json"
    manifest = {
        "schema_version": SCHEMA,
        "status": "pending_local",
        "case_id": state["case_id"],
        "task_id": state["task_id"],
        "phase": phase,
        "rri": rri,
        "content_sha256": state["content_sha256"],
        "request_sha256": sha,
        "request_file": request_rel,
        "response_file": f"responses/{stem}.response.json",
        "normalized_file": f"completed/{stem}.normalized.json",
        "shadow_dataset": "shadow.jsonl",
        "review_artifact": _review_ref(review_artifact),
        "model": model,
        "created_at": dt.datetime.now(dt.timezone.utc).isoformat(),
    }
    if manifest_path.exists():
        old = _json(manifest_path)
        if old.get("case_id") != manifest["case_id"] or old.get("request_sha256") != sha:
            raise ValueError("existing shadow manifest conflicts with request")
        return {**old, "manifest_file": str(manifest_path)}
    _write(root / request_rel, request)
    _write(manifest_path, manifest)
    return {**manifest, "manifest_file": str(manifest_path)}


def _bound(manifest_path: str | Path) -> tuple[Path, dict[str, Any], dict[str, Any], Path]:
    path = Path(manifest_path).resolve()
    manifest = _json(path)
    if manifest.get("schema_version") != SCHEMA:
        raise ValueError("unsupported shadow manifest schema")
    root = path.parent.parent
    request = _json(_rooted(root, manifest["request_file"]))
    if request_sha256(request) != manifest.get("request_sha256"):
        raise ValueError("request hash does not match manifest")
    if request.get("state", {}).get("case_id") != manifest.get("case_id"):
        raise ValueError("request case_id does not match manifest")
    return path, manifest, request, root


def ingest(*, manifest_path: str | Path, response_path: str | Path,
           threshold: float = decision.DEFAULT_FAST_PATH_THRESHOLD) -> dict[str, Any]:
    path, manifest, request, root = _bound(manifest_path)
    envelope = _json(Path(response_path).resolve())
    if envelope.get("request_sha256") != manifest["request_sha256"]:
        raise ValueError("response envelope request_sha256 mismatch")
    raw = envelope.get("response")
    if not isinstance(raw, dict):
        raise ValueError("response envelope missing response object")
    normalized = decision.normalize_systemone_response(raw, request=request)
    result = {
        "case_id": manifest["case_id"],
        "request_sha256": manifest["request_sha256"],
        "decision": normalized,
        "fast_path": decision.fast_path_eligibility(
            decision=normalized, state=request["state"], threshold=threshold),
    }
    if isinstance(envelope.get("latency_ms"), (int, float)) and not isinstance(envelope.get("latency_ms"), bool):
        result["latency_ms"] = float(envelope["latency_ms"])
    _write(_rooted(root, manifest["normalized_file"]), result)
    _write(path, {**manifest, "status": "decision_ready"})
    return result


def pair(*, manifest_path: str | Path, expected_review: str | None = None) -> dict[str, Any]:
    path, manifest, request, root = _bound(manifest_path)
    normalized_path = _rooted(root, manifest["normalized_file"])
    review_path = _review_path(manifest["review_artifact"])
    if not normalized_path.exists() or not review_path.exists():
        raise FileNotFoundError("normalized decision or authoritative review missing")
    row = shadow.build_capture_row(request, _json(normalized_path), _json(review_path),
                                   manifest["review_artifact"], expected_review)
    dataset = _rooted(root, manifest["shadow_dataset"])
    dataset.parent.mkdir(parents=True, exist_ok=True)
    existing = [] if not dataset.exists() else [json.loads(x) for x in dataset.read_text().splitlines() if x.strip()]
    same = [x for x in existing if x.get("case_id") == row["case_id"]]
    if same and same[0] != row:
        raise ValueError("case_id already paired with different data")
    if not same:
        with dataset.open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(row, sort_keys=True) + "\n")
    _write(path, {**manifest, "status": "paired"})
    return {"case_id": row["case_id"], "dataset_file": str(dataset), "appended": not same}


def pending(root: str | Path = DEFAULT_ROOT) -> list[Path]:
    root = Path(root).resolve()
    results = []
    for path in sorted((root / "pending").glob("*.manifest.json")):
        try:
            manifest = _json(path)
            if not _rooted(root, manifest["normalized_file"]).exists():
                results.append(path)
        except (OSError, ValueError, KeyError, json.JSONDecodeError):
            continue
    return results


def run_local_batch(*, root: str | Path = DEFAULT_ROOT, limit: int | None = None,
                    endpoint: str = local_adapter.DEFAULT_ENDPOINT,
                    timeout: int = local_adapter.DEFAULT_TIMEOUT_SECONDS) -> dict[str, Any]:
    """Explicit local-only step. Everything else in this module is cloud-safe."""
    items = pending(root)
    if limit is not None:
        items = items[:limit]
    done = paired = 0
    failures = []
    for path in items:
        try:
            manifest_path, manifest, request, queue_root = _bound(path)
            envelope = local_adapter.invoke_systemone(request, endpoint=endpoint, timeout=timeout)
            response_path = _rooted(queue_root, manifest["response_file"])
            _write(response_path, envelope)
            ingest(manifest_path=manifest_path, response_path=response_path)
            done += 1
            review_path = _review_path(manifest["review_artifact"])
            if review_path.exists():
                review_data = _json(review_path)
                if shadow.extract_ground_truth(review_data, manifest["review_artifact"]) is not None:
                    pair(manifest_path=manifest_path)
                    paired += 1
        except Exception as exc:
            failures.append({"manifest": str(path), "error": str(exc)})
    return {"attempted": len(items), "completed": done, "paired": paired, "failures": failures}


def main() -> int:
    parser = argparse.ArgumentParser(description="DubBridge review-decision shadow queue")
    sub = parser.add_subparsers(dest="command", required=True)
    p = sub.add_parser("pending"); p.add_argument("--root", default=str(DEFAULT_ROOT))
    i = sub.add_parser("ingest"); i.add_argument("--manifest", required=True); i.add_argument("--response", required=True)
    q = sub.add_parser("pair"); q.add_argument("--manifest", required=True); q.add_argument("--expected-review", choices=["none", "local", "advanced", "human"])
    r = sub.add_parser("run-local"); r.add_argument("--root", default=str(DEFAULT_ROOT)); r.add_argument("--limit", type=int); r.add_argument("--endpoint", default=local_adapter.DEFAULT_ENDPOINT); r.add_argument("--timeout", type=int, default=local_adapter.DEFAULT_TIMEOUT_SECONDS)
    args = parser.parse_args()
    if args.command == "pending":
        items = [str(x) for x in pending(args.root)]; print(json.dumps({"count": len(items), "manifests": items}, indent=2)); return 0
    if args.command == "ingest":
        print(json.dumps(ingest(manifest_path=args.manifest, response_path=args.response), indent=2)); return 0
    if args.command == "pair":
        print(json.dumps(pair(manifest_path=args.manifest, expected_review=args.expected_review), indent=2)); return 0
    result = run_local_batch(root=args.root, limit=args.limit, endpoint=args.endpoint, timeout=args.timeout)
    print(json.dumps(result, indent=2)); return 0 if not result["failures"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
