#!/usr/bin/env python3
"""Backend-neutral graph result contract for DubBridge code intelligence tooling."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

_ALLOWED_CLASSIFICATIONS = {
    "task_local",
    "cross_boundary",
    "global_architecture",
    "secret",
    "runtime_data",
}


class BackendPayloadError(ValueError):
    """Raised when graph backend data is unsafe or incomplete."""


@dataclass(frozen=True)
class SourceFragment:
    path: str
    start_line: int
    end_line: int
    content: str
    classification: str = "task_local"

    @classmethod
    def from_mapping(cls, value: dict[str, Any]) -> "SourceFragment":
        try:
            path = _non_empty_string(value["path"], "source fragment path")
            start_line = int(value["start_line"])
            end_line = int(value["end_line"])
            content = str(value["content"])
            classification = str(value.get("classification", "task_local"))
        except (KeyError, TypeError, ValueError) as exc:
            raise BackendPayloadError(f"invalid source fragment: {value!r}") from exc

        if start_line < 1 or end_line < start_line:
            raise BackendPayloadError(
                f"invalid source fragment line range for {path}: {start_line}-{end_line}"
            )
        if classification not in _ALLOWED_CLASSIFICATIONS:
            raise BackendPayloadError(
                f"unsupported source fragment classification: {classification}"
            )
        return cls(path, start_line, end_line, content, classification)


@dataclass(frozen=True)
class GraphResult:
    git_revision: str
    graph_revision: str
    anchors: tuple[str, ...]
    files: tuple[str, ...]
    symbols: tuple[str, ...]
    relationships: tuple[dict[str, Any], ...]
    tests: tuple[str, ...]
    boundaries: tuple[str, ...]
    governance: tuple[str, ...]
    source_fragments: tuple[SourceFragment, ...]

    @classmethod
    def from_mapping(cls, value: dict[str, Any]) -> "GraphResult":
        if not isinstance(value, dict):
            raise BackendPayloadError("graph payload must be a JSON object")

        git_revision = _non_empty_string(value.get("git_revision"), "git_revision")
        graph_revision = _non_empty_string(
            value.get("graph_revision"), "graph_revision"
        )
        relationships = value.get("relationships", [])
        if not isinstance(relationships, list) or not all(
            isinstance(item, dict) for item in relationships
        ):
            raise BackendPayloadError("relationships must be a list of objects")

        raw_fragments = value.get("source_fragments", [])
        if not isinstance(raw_fragments, list):
            raise BackendPayloadError("source_fragments must be a list")

        return cls(
            git_revision=git_revision,
            graph_revision=graph_revision,
            anchors=_string_tuple(value.get("anchors", []), "anchors"),
            files=_string_tuple(value.get("files", []), "files"),
            symbols=_string_tuple(value.get("symbols", []), "symbols"),
            relationships=tuple(relationships),
            tests=_string_tuple(value.get("tests", []), "tests"),
            boundaries=_string_tuple(value.get("boundaries", []), "boundaries"),
            governance=_string_tuple(value.get("governance", []), "governance"),
            source_fragments=tuple(
                SourceFragment.from_mapping(item) for item in raw_fragments
            ),
        )


class JsonGraphBackend:
    """Load graph results from a deterministic JSON interchange payload."""

    def __init__(self, payload_path: Path):
        self.payload_path = payload_path

    def resolve(self) -> GraphResult:
        try:
            with self.payload_path.open("r", encoding="utf-8") as handle:
                payload = json.load(handle)
        except (OSError, json.JSONDecodeError) as exc:
            raise BackendPayloadError(
                f"unable to load graph payload {self.payload_path}: {exc}"
            ) from exc
        return GraphResult.from_mapping(payload)


def _non_empty_string(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise BackendPayloadError(f"{field} must be a non-empty string")
    return value.strip()


def _string_tuple(value: Any, field: str) -> tuple[str, ...]:
    if not isinstance(value, list) or not all(
        isinstance(item, str) and item.strip() for item in value
    ):
        raise BackendPayloadError(f"{field} must be a list of non-empty strings")
    return tuple(dict.fromkeys(item.strip() for item in value))
