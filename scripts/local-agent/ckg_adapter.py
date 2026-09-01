from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
from dataclasses import dataclass, replace


MINIMUM_GRAPH_LABELS = frozenset(
    ("Module", "File", "Function", "Method", "Struct", "Enum", "Trait", "Test")
)
MINIMUM_GRAPH_RELATIONSHIPS = frozenset(
    ("CONTAINS", "IMPORTS", "USES", "CALLS", "IMPLEMENTS", "REFERENCES", "TESTS")
)
DEFAULT_TRACE_DEPTH = 1
DEFAULT_COMMAND_TIMEOUT_SECONDS = 20


@dataclass(frozen=True)
class GraphCandidate:
    path: str
    symbol: str | None = None
    label: str | None = None
    relation: str | None = None
    start_line: int | None = None
    end_line: int | None = None
    priority: int = 5
    reason: str = "text_candidate"
    graph_revision: str | None = None


def _safe_int(value):
    if isinstance(value, bool):
        return None
    if isinstance(value, int) and value > 0:
        return value
    if isinstance(value, str) and value.isdigit():
        parsed = int(value)
        return parsed if parsed > 0 else None
    return None


def _walk(value):
    if isinstance(value, dict):
        yield value
        for nested in value.values():
            yield from _walk(nested)
    elif isinstance(value, list):
        for nested in value:
            yield from _walk(nested)


def _extract_candidates(value, *, priority, reason):
    candidates = []
    seen = set()
    for item in _walk(value):
        path = item.get("path") or item.get("file_path") or item.get("file")
        if not isinstance(path, str) or not path:
            continue
        path = path.replace("\\", "/")
        symbol = item.get("name") or item.get("symbol") or item.get("qualified_name")
        label = item.get("label") or item.get("type") or item.get("kind")
        relation = item.get("relationship") or item.get("relation") or item.get("edge")
        start = _safe_int(
            item.get("start_line") or item.get("line_start") or item.get("line")
        )
        end = _safe_int(item.get("end_line") or item.get("line_end"))
        key = (path, symbol, start, end)
        if key in seen:
            continue
        seen.add(key)
        candidates.append(
            GraphCandidate(
                path=path,
                symbol=symbol if isinstance(symbol, str) else None,
                label=label if isinstance(label, str) else None,
                relation=relation if isinstance(relation, str) else None,
                start_line=start,
                end_line=end,
                priority=priority,
                reason=reason,
            )
        )
    return candidates


def extract_task_anchors(task_text):
    explicit_paths = []
    explicit_symbols = []
    for match in re.finditer(
        r"(?<![\w.-])([A-Za-z0-9_./-]+\.(?:rs|py|toml|json|yaml|yml|md|ts|tsx|js|jsx))(?![\w.-])",
        task_text or "",
    ):
        path = match.group(1)
        if path not in explicit_paths:
            explicit_paths.append(path)
    for match in re.finditer(r"`([A-Za-z_][A-Za-z0-9_:.-]*)`", task_text or ""):
        value = match.group(1)
        if "." not in value and value not in explicit_symbols:
            explicit_symbols.append(value)
    for match in re.finditer(
        r"\b([A-Za-z_][A-Za-z0-9_]*::[A-Za-z0-9_:]+)\b", task_text or ""
    ):
        value = match.group(1)
        if value not in explicit_symbols:
            explicit_symbols.append(value)
    return {"paths": explicit_paths, "symbols": explicit_symbols}


def rank_candidates(candidates, anchors):
    paths = set(anchors.get("paths", ()))
    symbols = set(anchors.get("symbols", ()))
    ranked = []
    for candidate in candidates:
        priority = candidate.priority
        reason = candidate.reason
        if candidate.path in paths or (candidate.symbol and candidate.symbol in symbols):
            priority, reason = 0, "explicit_task_anchor"
        elif candidate.label and candidate.label.lower() == "test":
            priority, reason = min(priority, 1), "task_test"
        elif candidate.relation and candidate.relation.upper() in {
            "CALLS",
            "USES",
            "IMPORTS",
            "IMPLEMENTS",
            "REFERENCES",
        }:
            priority, reason = min(priority, 2), "direct_dependency"
        elif candidate.relation and candidate.relation.upper() in {"CALLED_BY", "CALLER"}:
            priority, reason = min(priority, 3), "direct_caller"
        elif candidate.label and candidate.label in {"Struct", "Enum", "Trait"}:
            priority, reason = min(priority, 4), "adjacent_type"
        ranked.append(replace(candidate, priority=priority, reason=reason))
    ranked.sort(key=lambda c: (c.priority, c.path, c.symbol or "", c.start_line or 0))
    deduped = []
    seen = set()
    for candidate in ranked:
        key = (candidate.path, candidate.symbol, candidate.start_line, candidate.end_line)
        if key not in seen:
            seen.add(key)
            deduped.append(candidate)
    return deduped


class CKGAdapterError(RuntimeError):
    pass


def _unwrap_mcp_envelope(payload, tool):
    """Return the tool's JSON payload from CBM's --json MCP envelope."""
    if not isinstance(payload, dict):
        return payload
    if payload.get("isError") is True:
        messages = []
        for item in payload.get("content", []):
            if isinstance(item, dict) and isinstance(item.get("text"), str):
                messages.append(item["text"])
        detail = "; ".join(messages) or "unknown MCP error"
        raise CKGAdapterError(f"CBM {tool} returned an error: {detail}")
    content = payload.get("content")
    if not isinstance(content, list):
        return payload
    for item in content:
        if not isinstance(item, dict):
            continue
        text = item.get("text")
        if not isinstance(text, str):
            continue
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            continue
    return payload


class CodebaseMemoryCLIAdapter:
    """One-shot CBM adapter. The model never receives this tool surface."""

    def __init__(
        self,
        binary="codebase-memory-mcp",
        timeout=DEFAULT_COMMAND_TIMEOUT_SECONDS,
        runner=None,
    ):
        self.binary = binary
        self.timeout = timeout
        self._runner = runner or subprocess.run
        self._projects_by_root = {}

    @property
    def backend_name(self):
        return "codebase-memory-mcp"

    def available(self):
        return shutil.which(self.binary) is not None

    def _call(self, tool, arguments=None):
        # CBM documents JSON-on-stdin for one-shot CLI tools. --json returns
        # the full MCP envelope, so unwrap its JSON text payload before the
        # rest of DubBridge consumes it.
        argv = [self.binary, "cli", "--json", tool]
        stdin = None if arguments is None else json.dumps(arguments, ensure_ascii=False)
        try:
            completed = self._runner(
                argv,
                input=stdin,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=self.timeout,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as exc:
            raise CKGAdapterError(f"CBM command failed: {exc}") from exc
        if completed.returncode != 0:
            raise CKGAdapterError(
                f"CBM {tool} exited {completed.returncode}: {completed.stderr.strip()}"
            )
        try:
            payload = json.loads(completed.stdout)
        except json.JSONDecodeError as exc:
            raise CKGAdapterError(f"CBM {tool} returned non-JSON output") from exc
        return _unwrap_mcp_envelope(payload, tool)

    @staticmethod
    def _worktree_root(worktree_dir):
        # Each linked worktree is a distinct source state. Index the exact
        # checkout instead of redirecting to the shared/common Git directory;
        # otherwise graph relationships can come from another branch.
        return os.path.realpath(os.path.abspath(worktree_dir))

    def _project_for_root(self, root):
        payload = self._call("list_projects")
        root_real = os.path.realpath(root)
        for item in _walk(payload):
            path = item.get("path") or item.get("repo_path") or item.get("root")
            name = item.get("name") or item.get("project")
            if (
                isinstance(path, str)
                and isinstance(name, str)
                and os.path.realpath(path) == root_real
            ):
                return name
        return None

    @staticmethod
    def _project_from_index_payload(payload):
        for item in _walk(payload):
            for key in ("project", "project_name", "name"):
                value = item.get(key)
                if isinstance(value, str) and value:
                    return value
        return None

    def ensure_project(self, worktree_dir, *, force_refresh=False):
        root = self._worktree_root(worktree_dir)
        cached = self._projects_by_root.get(root)
        if cached and not force_refresh:
            return cached

        # CBM indexing is incremental. Refreshing the exact disposable worktree
        # keeps graph topology aligned with edits before a repair-context pass.
        payload = self._call("index_repository", {"repo_path": root})
        project = self._project_from_index_payload(payload) or self._project_for_root(root)
        if not project:
            raise CKGAdapterError(
                "CBM indexed worktree but project could not be resolved"
            )
        self._projects_by_root[root] = project
        return project

    def coverage(self, project, paths):
        if not paths:
            return {"status": "unknown", "raw": {}}
        payload = self._call(
            "check_index_coverage", {"project": project, "paths": list(paths)}
        )
        statuses = []
        for item in _walk(payload):
            value = item.get("status")
            if isinstance(value, str):
                statuses.append(value.lower())
        safe_statuses = {"no_recorded_issue", "covered", "complete", "verified", "fresh"}
        if statuses:
            status = "verified" if all(value in safe_statuses for value in statuses) else "partial"
        else:
            text = json.dumps(payload, ensure_ascii=False).lower()
            status = (
                "partial"
                if any(
                    term in text
                    for term in (
                        "stale",
                        "partial",
                        "skipped",
                        "excluded",
                        "unknown",
                        "pending",
                        "gap",
                    )
                )
                else "verified"
            )
        return {"status": status, "raw": payload}

    def discover(self, task_text, worktree_dir, *, force_refresh=False):
        project = self.ensure_project(worktree_dir, force_refresh=force_refresh)
        anchors = extract_task_anchors(task_text)
        candidates = []
        for path in anchors["paths"]:
            candidates.append(
                GraphCandidate(path=path, priority=0, reason="explicit_task_anchor")
            )
        for symbol in anchors["symbols"][:6]:
            payload = self._call(
                "search_graph",
                {"project": project, "name_pattern": re.escape(symbol), "limit": 10},
            )
            candidates.extend(
                _extract_candidates(payload, priority=0, reason="explicit_task_anchor")
            )
            try:
                trace = self._call(
                    "trace_path",
                    {
                        "project": project,
                        "function_name": symbol,
                        "direction": "both",
                        "max_depth": DEFAULT_TRACE_DEPTH,
                    },
                )
                candidates.extend(
                    _extract_candidates(trace, priority=2, reason="direct_dependency")
                )
            except CKGAdapterError:
                pass
        if not candidates and task_text.strip():
            # Prefer CBM's text/BM25 query as the low-priority anchor fallback.
            # Semantic retrieval remains optional rather than authoritative.
            payload = self._call(
                "search_graph",
                {"project": project, "query": task_text[:1200], "limit": 10},
            )
            candidates.extend(
                _extract_candidates(payload, priority=5, reason="text_candidate")
            )
        return {
            "project": project,
            "anchors": anchors,
            "candidates": rank_candidates(candidates, anchors),
        }
