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

_UNSAFE_ROOT_PREFIXES = (
    ".git/",
    ".cache/",
    "tmp/",
    "target/",
    "secrets/",
    "credentials/",
    "var/run/",
)
_UNSAFE_EXACT_NAMES = {
    ".env",
    "id_rsa",
    "id_ed25519",
}
_UNSAFE_SUFFIXES = (".pem", ".key")
_UNSAFE_CONTENT_MARKERS = (
    "-----BEGIN PRIVATE KEY-----",
    "AWS_SECRET_ACCESS_KEY=",
    "GITHUB_TOKEN=",
)


class ContextPolicyError(ValueError):
    """Raised when context cannot be exported safely."""


def build_artifacts(
    *,
    task_id: str,
    task_text: str,
    target: str,
    graph: GraphResult,
    expected_git_revision: str,
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Return a deterministic receipt and bounded capsule for a verified graph result."""
    task_id = _required_text(task_id, "task_id")
    task_text = _required_text(task_text, "task_text")
    expected_git_revision = _required_text(
        expected_git_revision, "expected_git_revision"
    )
    if target not in {"local", "cloud"}:
        raise ContextPolicyError("target must be 'local' or 'cloud'")
    _verify_freshness(graph=graph, expected_git_revision=expected_git_revision)

    denied = _CLOUD_DENY if target == "cloud" else _ALWAYS_DENY

    exported_fragments = [
        asdict(fragment)
        for fragment in graph.source_fragments
        if fragment.classification not in denied and not _fragment_is_unsafe(fragment)
    ]
    excluded_fragments = [
        {
            "path": fragment.path,
            "start_line": fragment.start_line,
            "end_line": fragment.end_line,
            "classification": fragment.classification,
            "reason": _fragment_exclusion_reason(fragment, denied),
        }
        for fragment in graph.source_fragments
        if fragment.classification in denied or _fragment_is_unsafe(fragment)
    ]

    relationships = [
        relationship
        for relationship in graph.relationships
        if str(relationship.get("classification", "task_local")) not in denied
        and not _relationship_is_unsafe(relationship)
    ]
    excluded_relationships = [
        relationship
        for relationship in graph.relationships
        if str(relationship.get("classification", "task_local")) in denied
        or _relationship_is_unsafe(relationship)
    ]

    metadata = _export_metadata(
        graph=graph,
        target=target,
        exported_fragments=exported_fragments,
        relationships=relationships,
    )

    capsule = {
        "schema": "context-capsule-v1",
        "task": {"id": task_id, "description": task_text},
        "repository": {
            "git_revision": graph.git_revision,
            "graph_revision": graph.graph_revision,
        },
        "target": target,
        "anchors": metadata["anchors"],
        "files": metadata["files"],
        "symbols": metadata["symbols"],
        "tests": metadata["tests"],
        "governance": metadata["governance"],
        "source_fragments": exported_fragments,
        "relationships": relationships,
        "boundaries": metadata["boundaries"],
    }

    capsule_hash = _sha256_json(capsule)
    receipt = {
        "schema": "context-receipt-v1",
        "task": {"id": task_id, "description": task_text},
        "repository": {
            "git_revision": graph.git_revision,
            "graph_revision": graph.graph_revision,
        },
        "target": target,
        "anchors": metadata["anchors"],
        "impact": {
            "files": metadata["files"],
            "symbols": metadata["symbols"],
            "tests": metadata["tests"],
        },
        "boundaries": metadata["boundaries"],
        "governance": metadata["governance"],
        "export": {
            "files": metadata["files"],
            "symbols": metadata["symbols"],
            "tests": metadata["tests"],
            "anchors": metadata["anchors"],
            "boundaries": metadata["boundaries"],
            "governance": metadata["governance"],
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


def build_expansion_artifacts(
    *,
    task_id: str,
    task_text: str,
    target: str,
    graph: GraphResult,
    expected_git_revision: str,
    base_receipt: dict[str, Any],
    reason: str,
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Build a bounded expansion linked to a verified prior receipt."""
    reason = _required_text(reason, "expansion reason")
    _validate_base_receipt(base_receipt)

    base_target = base_receipt.get("target")
    if base_target != target:
        raise ContextPolicyError(
            f"expansion target mismatch: base={base_target!r} requested={target!r}"
        )

    base_repo = base_receipt.get("repository", {})
    if base_repo.get("git_revision") != expected_git_revision:
        raise ContextPolicyError(
            "base receipt git revision does not match expected repository revision"
        )
    if base_repo.get("git_revision") != graph.git_revision:
        raise ContextPolicyError(
            "expansion graph git revision does not match base receipt"
        )
    if base_repo.get("graph_revision") != graph.graph_revision:
        raise ContextPolicyError(
            "expansion graph revision does not match base receipt"
        )

    receipt, capsule = build_artifacts(
        task_id=task_id,
        task_text=task_text,
        target=target,
        graph=graph,
        expected_git_revision=expected_git_revision,
    )

    base_export = base_receipt.get("export", {})
    current_export = receipt["export"]
    added = _export_delta(base_export, current_export)
    has_exclusions = bool(
        receipt["excluded"]["source_fragments"]
        or receipt["excluded"]["relationships"]
    )
    decision = "allow"
    if not added:
        decision = "deny"
    elif has_exclusions:
        decision = "reduce"

    previous_expansions = base_receipt.get("expansions", [])
    if not isinstance(previous_expansions, list):
        raise ContextPolicyError("base receipt expansions must be a list")

    receipt["expansions"] = [
        *previous_expansions,
        {
            "base_receipt_sha256": base_receipt["receipt_sha256"],
            "reason": reason,
            "decision": decision,
            "added": added,
        },
    ]
    receipt["receipt_sha256"] = _sha256_json(
        {k: v for k, v in receipt.items() if k != "receipt_sha256"}
    )
    return receipt, capsule


def _verify_freshness(*, graph: GraphResult, expected_git_revision: str) -> None:
    if graph.git_revision != expected_git_revision:
        raise ContextPolicyError(
            "stale graph rejected: "
            f"expected git revision {expected_git_revision}, "
            f"received {graph.git_revision}"
        )


def _fragment_is_unsafe(fragment: Any) -> bool:
    return _unsafe_path(fragment.path) or _unsafe_content(fragment.content)


def _fragment_exclusion_reason(fragment: Any, denied: set[str]) -> str:
    if fragment.classification in denied:
        return "classification denied for target"
    return "gateway defense-in-depth deny rule"


def _relationship_is_unsafe(relationship: dict[str, Any]) -> bool:
    return any(
        _unsafe_metadata_value(str(relationship.get(field, "")))
        for field in ("from", "to")
    )


def _export_metadata(
    *,
    graph: GraphResult,
    target: str,
    exported_fragments: list[dict[str, Any]],
    relationships: list[dict[str, Any]],
) -> dict[str, list[str]]:
    safe_files = [item for item in graph.files if not _unsafe_path(item)]
    safe_symbols = [
        item for item in graph.symbols if not _unsafe_metadata_value(item)
    ]
    safe_anchors = [
        item for item in graph.anchors if not _unsafe_metadata_value(item)
    ]
    safe_tests = [item for item in graph.tests if not _unsafe_metadata_value(item)]
    safe_boundaries = [
        item for item in graph.boundaries if not _unsafe_metadata_value(item)
    ]
    safe_governance = [
        item for item in graph.governance if not _unsafe_metadata_value(item)
    ]

    if target == "local":
        return {
            "anchors": safe_anchors,
            "files": safe_files,
            "symbols": safe_symbols,
            "tests": safe_tests,
            "boundaries": sorted(set(safe_boundaries)),
            "governance": safe_governance,
        }

    allowed_paths = {
        str(item["path"])
        for item in exported_fragments
        if item.get("classification") == "task_local"
    }
    relationship_endpoints = {
        str(relationship.get(field, ""))
        for relationship in relationships
        if relationship.get("classification", "task_local") == "task_local"
        for field in ("from", "to")
        if relationship.get(field)
    }

    files = [item for item in safe_files if item in allowed_paths]
    symbols = [item for item in safe_symbols if item in relationship_endpoints]
    anchors = [
        item for item in safe_anchors
        if item in relationship_endpoints or item in symbols
    ]
    tests = [
        item for item in safe_tests
        if _metadata_related(item, symbols, files)
    ]
    boundaries = [
        item for item in safe_boundaries
        if _metadata_related(item, symbols, files)
    ]
    governance = [
        item for item in safe_governance
        if _metadata_related(item, symbols, files)
    ]
    return {
        "anchors": anchors,
        "files": files,
        "symbols": symbols,
        "tests": tests,
        "boundaries": sorted(set(boundaries)),
        "governance": governance,
    }


def _metadata_related(
    value: str, symbols: list[str], files: list[str]
) -> bool:
    normalized = value.casefold()
    for symbol in symbols:
        leaf = _identifier_leaf(symbol)
        if leaf and leaf in normalized:
            return True
    for path in files:
        stem = Path(path).stem.casefold()
        parent = Path(path).parent.name.casefold()
        if stem and stem in normalized:
            return True
        if parent and parent in normalized:
            return True
    return False


def _identifier_leaf(value: str) -> str:
    normalized = value.replace("::", "/").replace("\\", "/")
    leaf = normalized.rsplit("/", 1)[-1].casefold()
    return leaf if len(leaf) >= 3 else ""


def _unsafe_path(value: str) -> bool:
    normalized = value.strip().replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    lowered = normalized.casefold()
    name = Path(normalized).name.casefold()
    if name in _UNSAFE_EXACT_NAMES or name.startswith(".env."):
        return True
    if any(lowered.startswith(prefix) for prefix in _UNSAFE_ROOT_PREFIXES):
        return True
    return lowered.endswith(_UNSAFE_SUFFIXES)


def _unsafe_content(value: str) -> bool:
    return any(marker in value for marker in _UNSAFE_CONTENT_MARKERS)


def _unsafe_metadata_value(value: str) -> bool:
    lowered = value.casefold()
    if _unsafe_path(value):
        return True
    return any(
        token in lowered
        for token in ("private_key", "secret_access_key", "github_token")
    )


def _export_delta(
    base_export: dict[str, Any], current_export: dict[str, Any]
) -> dict[str, list[Any]]:
    added: dict[str, list[Any]] = {}
    for field in ("files", "symbols", "tests", "anchors", "boundaries", "governance"):
        before = set(base_export.get(field, []))
        after = current_export.get(field, [])
        values = [item for item in after if item not in before]
        if values:
            added[field] = values

    before_fragments = {
        (
            item.get("path"),
            item.get("start_line"),
            item.get("end_line"),
            item.get("classification"),
        )
        for item in base_export.get("source_fragments", [])
        if isinstance(item, dict)
    }
    added_fragments = [
        item
        for item in current_export.get("source_fragments", [])
        if (
            item.get("path"),
            item.get("start_line"),
            item.get("end_line"),
            item.get("classification"),
        )
        not in before_fragments
    ]
    if added_fragments:
        added["source_fragments"] = added_fragments
    return added


def _validate_base_receipt(receipt: dict[str, Any]) -> None:
    if not isinstance(receipt, dict):
        raise ContextPolicyError("base receipt must be a JSON object")
    expected_hash = receipt.get("receipt_sha256")
    if not isinstance(expected_hash, str) or len(expected_hash) != 64:
        raise ContextPolicyError("base receipt is missing a valid receipt_sha256")
    unsigned = {k: v for k, v in receipt.items() if k != "receipt_sha256"}
    actual_hash = _sha256_json(unsigned)
    if actual_hash != expected_hash:
        raise ContextPolicyError("base receipt hash verification failed")
    if receipt.get("schema") != "context-receipt-v1":
        raise ContextPolicyError("unsupported base receipt schema")


def _load_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ContextPolicyError(f"unable to load {label} {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ContextPolicyError(f"{label} must be a JSON object")
    return value


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
    parser.add_argument("--expected-git-revision", required=True)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--base-receipt", type=Path)
    parser.add_argument("--expansion-reason")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        graph = JsonGraphBackend(args.backend_json).resolve()
        if bool(args.base_receipt) != bool(args.expansion_reason):
            raise ContextPolicyError(
                "--base-receipt and --expansion-reason must be supplied together"
            )

        if args.base_receipt:
            base_receipt = _load_json_object(args.base_receipt, "base receipt")
            receipt, capsule = build_expansion_artifacts(
                task_id=args.task_id,
                task_text=args.task,
                target=args.target,
                graph=graph,
                expected_git_revision=args.expected_git_revision,
                base_receipt=base_receipt,
                reason=args.expansion_reason,
            )
        else:
            receipt, capsule = build_artifacts(
                task_id=args.task_id,
                task_text=args.task,
                target=args.target,
                graph=graph,
                expected_git_revision=args.expected_git_revision,
            )
    except (BackendPayloadError, ContextPolicyError) as exc:
        print(f"context gateway rejected input: {exc}", file=os.sys.stderr)
        return 2

    _write_json(args.output_dir / "context-capsule.json", capsule)
    _write_json(args.output_dir / "context-receipt.json", receipt)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
