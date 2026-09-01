#!/usr/bin/env python3
"""Generate bounded agent context from a local code-graph result."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import tempfile
from dataclasses import asdict
from pathlib import Path
from typing import Any

from backend import BackendPayloadError, GraphResult, JsonGraphBackend

_ALWAYS_DENY = {"secret", "runtime_data"}
_CLOUD_DENY = _ALWAYS_DENY | {"cross_boundary", "global_architecture"}


class ContextPolicyError(ValueError):
    """Raised when context cannot be exported safely."""


def build_artifacts(
    *, task_id: str, task_text: str, target: str, graph: GraphResult
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Return a deterministic receipt and bounded capsule for the requested target."""
    task_id = _required_text(task_id, "task_id")
    task_text = _required_text(task_text, "task_text")
    if target not in {"local", "cloud"}:
        raise ContextPolicyError("target must be 'local' or 'cloud'")

    denied = _CLOUD_DENY if target == "cloud" else _ALWAYS_DENY
    exported_fragments = [
        asdict(fragment)
        for fragment in graph.source_fragments
        if fragment.classification not in denied
    ]
    excluded_fragments = [
        {
            "path": fragment.path,
            "start_line": fragment.start_line,
            "end_line": fragment.end_line,
            "classification": fragment.classification,
            "reason": "classification denied for target",
        }
        for fragment in graph.source_fragments
        if fragment.classification in denied
    ]

    relationships = [
        relationship
        for relationship in graph.relationships
        if str(relationship.get("classification", "task_local")) not in denied
    ]
    excluded_relationships = [
        relationship
        for relationship in graph.relationships
        if str(relationship.get("classification", "task_local")) in denied
    ]

    capsule = {
        "schema": "context-capsule-v1",
        "task": {"id": task_id, "description": task_text},
        "repository": {
            "git_revision": graph.git_revision,
            "graph_revision": graph.graph_revision,
        },
        "target": target,
        "anchors": list(graph.anchors),
        "files": list(graph.files),
        "symbols": list(graph.symbols),
        "tests": list(graph.tests),
        "governance": list(graph.governance),
        "source_fragments": exported_fragments,
    }
    if target == "local":
        capsule["relationships"] = relationships
        capsule["boundaries"] = list(graph.boundaries)
    else:
        capsule["relationships"] = relationships
        capsule["boundaries"] = _cloud_safe_boundaries(graph.boundaries)

    capsule_hash = _sha256_json(capsule)
    receipt = {
        "schema": "context-receipt-v1",
        "task": {"id": task_id, "description": task_text},
        "repository": {
            "git_revision": graph.git_revision,
            "graph_revision": graph.graph_revision,
        },
        "target": target,
        "anchors": list(graph.anchors),
        "impact": {
            "files": list(graph.files),
            "symbols": list(graph.symbols),
            "tests": list(graph.tests),
        },
        "boundaries": list(graph.boundaries),
        "governance": list(graph.governance),
        "export": {
            "files": list(graph.files),
            "symbols": list(graph.symbols),
            "source_fragments": [
                {
                    "path": item["path"],
                    "start_line": item["start_line"],
                    "end_line": item["end_line"],
                    "classification": item["classification"],
                }
                for item in exported_fragments
            ],
        },
        "excluded": {
            "source_fragments": excluded_fragments,
            "relationships": excluded_relationships,
        },
        "expansions": [],
        "capsule_sha256": capsule_hash,
    }
    receipt["receipt_sha256"] = _sha256_json(receipt)
    return receipt, capsule


def _cloud_safe_boundaries(boundaries: tuple[str, ...]) -> list[str]:
    """Expose boundary labels, not topology, to preserve constraints without traversal."""
    return sorted(set(boundaries))


def _sha256_json(value: dict[str, Any]) -> str:
    encoded = json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _required_text(value: str, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ContextPolicyError(f"{field} must be a non-empty string")
    return value.strip()


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(
        prefix=f".{path.name}.", suffix=".tmp", dir=str(path.parent), text=True
    )
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            json.dump(payload, handle, indent=2, sort_keys=True, ensure_ascii=False)
            handle.write("\n")
        os.replace(temporary, path)
    except Exception:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a local or cloud-bounded context receipt/capsule"
    )
    parser.add_argument("--task-id", required=True)
    parser.add_argument("--task", required=True, help="task description")
    parser.add_argument("--backend-json", required=True, type=Path)
    parser.add_argument("--target", required=True, choices=("local", "cloud"))
    parser.add_argument("--output-dir", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        graph = JsonGraphBackend(args.backend_json).resolve()
        receipt, capsule = build_artifacts(
            task_id=args.task_id,
            task_text=args.task,
            target=args.target,
            graph=graph,
        )
    except (BackendPayloadError, ContextPolicyError) as exc:
        print(f"context gateway rejected input: {exc}", file=os.sys.stderr)
        return 2

    _write_json(args.output_dir / "context-capsule.json", capsule)
    _write_json(args.output_dir / "context-receipt.json", receipt)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
