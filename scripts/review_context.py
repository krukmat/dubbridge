#!/usr/bin/env python3
"""Build a deterministic local-review packet with optional CKG impact context.

M3 changes only the packet producer for the existing local reviewer. The git diff
and supplied acceptance text are mandatory and never truncated to make room for
CKG context. Graph-selected source is local-only, optional, worktree-authoritative,
and subject to the existing LocalAgentBoundary semantics when allowed paths are
supplied.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Callable, Iterable

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
LOCAL_AGENT_DIR = os.path.join(SCRIPT_DIR, "local-agent")
sys.path.insert(0, SCRIPT_DIR)
sys.path.insert(0, LOCAL_AGENT_DIR)

import gemma_local  # noqa: E402
from ckg_adapter import (  # noqa: E402
    CKGAdapterError,
    CodebaseMemoryCLIAdapter,
    GraphCandidate,
)


SCHEMA = "review-context-v1"
DEFAULT_MAX_IMPACT_TOKENS = 6000
DEFAULT_PROMPT_RESERVE_TOKENS = 3072
DEFAULT_MAX_ENTRIES = 12
DEFAULT_MAX_SOURCE_BYTES = 128 * 1024

_DEFINITION_PATTERNS = (
    re.compile(r"\b(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    re.compile(r"\bdef\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    re.compile(r"\b(?:async\s+)?function\s+([A-Za-z_$][A-Za-z0-9_$]*)\b"),
    re.compile(r"\bclass\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    re.compile(r"\b(?:struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
)


@dataclass(frozen=True)
class SelectedImpact:
    path: str
    symbol: str | None
    label: str | None
    relation: str | None
    reason: str
    priority: int
    start_line: int | None
    end_line: int | None
    estimated_tokens: int


@dataclass(frozen=True)
class ScopeGap:
    path: str
    symbol: str | None
    reason: str
    detail: str


def extract_changed_paths(diff_text: str) -> list[str]:
    """Return stable repo-relative changed paths from a unified git diff."""
    paths: list[str] = []
    seen = set()
    for line in (diff_text or "").splitlines():
        value = None
        if line.startswith("diff --git a/"):
            match = re.match(r"diff --git a/(.+?) b/(.+)$", line)
            if match:
                value = match.group(2)
        elif line.startswith("+++ b/"):
            value = line[len("+++ b/") :].strip()
        if value and value != "/dev/null" and value not in seen:
            seen.add(value)
            paths.append(value)
    return paths


def extract_changed_symbols(diff_text: str, *, limit: int = 16) -> list[str]:
    """Extract lightweight definition anchors from changed lines/hunk context."""
    symbols: list[str] = []
    seen = set()
    for raw in (diff_text or "").splitlines():
        if raw.startswith(("+++", "---", "diff --git", "index ")):
            continue
        text = raw
        if raw.startswith("@@"):
            marker = raw.find("@@", 2)
            text = raw[marker + 2 :] if marker >= 0 else raw
        elif raw.startswith(("+", "-", " ")):
            text = raw[1:]
        else:
            continue
        for pattern in _DEFINITION_PATTERNS:
            match = pattern.search(text)
            if not match:
                continue
            symbol = match.group(1)
            if symbol not in seen:
                seen.add(symbol)
                symbols.append(symbol)
                if len(symbols) >= limit:
                    return symbols
    return symbols


def _candidate_priority(candidate: GraphCandidate, changed_paths: set[str]) -> tuple[int, str]:
    relation = (candidate.relation or "").upper()
    label = (candidate.label or "").lower()
    if candidate.path in changed_paths and candidate.start_line and candidate.end_line:
        return 0, "changed_symbol_context"
    if label == "test" or relation == "TESTS":
        return 1, "related_test"
    if relation in {"CALLED_BY", "CALLER", "CALLERS"}:
        return 2, "direct_caller"
    if label in {"struct", "enum", "trait", "type", "interface"} or relation in {
        "IMPLEMENTS",
        "REFERENCES",
    }:
        return 3, "related_type_or_reference"
    if relation in {"CALLS", "USES", "IMPORTS"}:
        return 4, "direct_dependency"
    return 5, candidate.reason or "graph_candidate"


def rank_impact_candidates(
    candidates: Iterable[GraphCandidate], changed_paths: Iterable[str]
) -> list[GraphCandidate]:
    changed = set(changed_paths)
    ranked = []
    for candidate in candidates:
        priority, reason = _candidate_priority(candidate, changed)
        # A whole-file changed-path candidate duplicates the authoritative diff
        # without adding structural context. Keep changed-path regions/symbols,
        # but discard the unbounded whole-file anchor itself.
        if candidate.path in changed and not (
            candidate.start_line and candidate.end_line
        ):
            continue
        ranked.append(
            GraphCandidate(
                path=candidate.path,
                symbol=candidate.symbol,
                label=candidate.label,
                relation=candidate.relation,
                start_line=candidate.start_line,
                end_line=candidate.end_line,
                priority=priority,
                reason=reason,
                graph_revision=candidate.graph_revision,
            )
        )
    ranked.sort(
        key=lambda item: (
            item.priority,
            item.path,
            item.symbol or "",
            item.start_line or 0,
            item.end_line or 0,
        )
    )
    result = []
    seen = set()
    for item in ranked:
        key = (item.path, item.symbol, item.start_line, item.end_line)
        if key not in seen:
            seen.add(key)
            result.append(item)
    return result


def _inside(candidate: str, root: str) -> bool:
    try:
        return os.path.commonpath((candidate, root)) == root
    except ValueError:
        return False


def read_current_source(
    worktree: str,
    candidate: GraphCandidate,
    *,
    max_source_bytes: int = DEFAULT_MAX_SOURCE_BYTES,
) -> str:
    """Read current worktree source after authority has already been checked."""
    root = os.path.realpath(os.path.abspath(worktree))
    if os.path.isabs(candidate.path):
        raise ValueError("absolute graph path rejected")
    normalized = os.path.normpath(candidate.path)
    if normalized in (".", "..") or normalized.startswith(".." + os.sep):
        raise ValueError("graph path escapes worktree")
    lexical = os.path.abspath(os.path.join(root, normalized))
    resolved = os.path.realpath(lexical)
    if not _inside(lexical, root) or not _inside(resolved, root):
        raise ValueError("graph path escapes worktree")
    try:
        fd = os.open(lexical, os.O_RDONLY | os.O_NOFOLLOW)
        with os.fdopen(fd, "r", encoding="utf-8", errors="replace") as handle:
            content = handle.read(max_source_bytes + 1)
    except OSError as exc:
        raise ValueError(f"source read rejected: {exc}") from exc
    if len(content.encode("utf-8")) > max_source_bytes:
        raise ValueError(f"source exceeds {max_source_bytes} byte review-context limit")
    if candidate.start_line and candidate.end_line and candidate.end_line >= candidate.start_line:
        lines = content.splitlines(keepends=True)
        start = max(0, candidate.start_line - 1)
        end = min(len(lines), candidate.end_line)
        if start < end:
            return "".join(lines[start:end])
    return content


def build_task_authorizer(worktree: str, allowed_paths: list[str]) -> Callable[[str], None]:
    """Reuse the existing local-agent boundary; do not implement a second matcher."""
    # Delayed import keeps the ordinary packet/test path lightweight and avoids
    # importing the full local runner when tests inject an authorizer directly.
    import boundary  # noqa: E402

    task_boundary = boundary.LocalAgentBoundary(worktree, allowed_paths, [])
    return task_boundary.check_path


def _review_retrieval_text(
    changed_paths: list[str], changed_symbols: list[str], acceptance_text: str
) -> str:
    parts = ["Review impact for changed paths:"]
    parts.extend(changed_paths)
    if changed_symbols:
        parts.append("Changed symbols:")
        parts.extend(f"`{symbol}`" for symbol in changed_symbols)
    if acceptance_text.strip():
        parts.append("Acceptance context:")
        parts.append(acceptance_text)
    return "\n".join(parts)


def _resolve_context_window() -> tuple[int, int]:
    model = os.environ.get("DUBBRIDGE_REVIEW_MODEL", gemma_local.DEFAULT_REVIEW_MODEL)
    default_ctx = gemma_local.MODEL_NUM_CTX_OVERRIDES.get(model, gemma_local.DEFAULT_NUM_CTX)
    default_predict = gemma_local.MODEL_NUM_PREDICT_OVERRIDES.get(
        model, gemma_local.DEFAULT_NUM_PREDICT
    )
    return (
        int(os.environ.get("DUBBRIDGE_REVIEW_NUM_CTX", str(default_ctx))),
        int(os.environ.get("DUBBRIDGE_REVIEW_NUM_PREDICT", str(default_predict))),
    )


def _mandatory_packet_prefix(task_id: str, acceptance_text: str, diff_text: str) -> str:
    acceptance = acceptance_text.strip() or "(not supplied)"
    return (
        f"# Local reviewer packet\n"
        f"task_id: {task_id or 'n/a'}\n\n"
        "## Acceptance context\n"
        f"{acceptance}\n\n"
        "## Authoritative diff\n"
        f"{diff_text.rstrip()}\n"
    )


def _render_source_block(candidate: GraphCandidate, content: str) -> str:
    line_hint = ""
    if candidate.start_line and candidate.end_line:
        line_hint = f" lines={candidate.start_line}-{candidate.end_line}"
    return (
        f"--- BEGIN REVIEW IMPACT {candidate.path}{line_hint} "
        f"reason={candidate.reason} ---\n"
        f"{content.rstrip()}\n"
        f"--- END REVIEW IMPACT {candidate.path} ---"
    )


def build_review_context(
    *,
    diff_text: str,
    acceptance_text: str = "",
    task_id: str = "",
    worktree: str = ".",
    allowed_paths: list[str] | None = None,
    adapter=None,
    authorizer: Callable[[str], None] | None = None,
    num_ctx: int | None = None,
    num_predict: int | None = None,
    max_impact_tokens: int = DEFAULT_MAX_IMPACT_TOKENS,
    prompt_reserve_tokens: int = DEFAULT_PROMPT_RESERVE_TOKENS,
    max_entries: int = DEFAULT_MAX_ENTRIES,
    enable_ckg: bool = True,
) -> tuple[str, dict]:
    """Return (review_packet, metadata). CKG failures degrade to mandatory content."""
    changed_paths = extract_changed_paths(diff_text)
    changed_symbols = extract_changed_symbols(diff_text)
    mandatory = _mandatory_packet_prefix(task_id, acceptance_text, diff_text)
    resolved_ctx, resolved_predict = _resolve_context_window()
    num_ctx = int(num_ctx if num_ctx is not None else resolved_ctx)
    num_predict = int(num_predict if num_predict is not None else resolved_predict)
    mandatory_tokens = gemma_local.estimate_text_tokens(mandatory)
    available_after_mandatory = max(
        0,
        num_ctx - num_predict - mandatory_tokens - int(prompt_reserve_tokens),
    )
    impact_budget = min(max(0, int(max_impact_tokens)), available_after_mandatory)

    metadata = {
        "schema": SCHEMA,
        "local_only": True,
        "status": "disabled" if not enable_ckg else "pending",
        "task_id": task_id or None,
        "changed_paths": changed_paths,
        "changed_symbols": changed_symbols,
        "coverage": "unknown",
        "fallback_reason": None,
        "scope_gaps": [],
        "selected": [],
        "budget": {
            "num_ctx": num_ctx,
            "num_predict": num_predict,
            "mandatory_tokens": mandatory_tokens,
            "prompt_reserve_tokens": int(prompt_reserve_tokens),
            "max_impact_tokens": int(max_impact_tokens),
            "impact_budget_tokens": impact_budget,
            "selected_impact_tokens": 0,
            "mandatory_over_budget": available_after_mandatory == 0
            and mandatory_tokens + num_predict + int(prompt_reserve_tokens) > num_ctx,
        },
    }

    if not enable_ckg or not changed_paths or impact_budget <= 0:
        if enable_ckg and changed_paths and impact_budget <= 0:
            metadata["status"] = "budget_exhausted"
            metadata["fallback_reason"] = "no reviewer context budget remains after mandatory material"
        return _assemble_packet(mandatory, metadata, []), metadata

    adapter = adapter or CodebaseMemoryCLIAdapter()
    try:
        available = getattr(adapter, "available", None)
        if callable(available) and not available():
            raise CKGAdapterError("CKG backend unavailable")
        discovery = adapter.discover(
            _review_retrieval_text(changed_paths, changed_symbols, acceptance_text),
            worktree,
            force_refresh=True,
        )
        coverage = adapter.coverage(discovery["project"], changed_paths)
        metadata["coverage"] = coverage.get("status", "unknown")
        if metadata["coverage"] != "verified":
            raise CKGAdapterError(
                f"CKG coverage is {metadata['coverage']}; review enrichment skipped"
            )
    except (CKGAdapterError, OSError, ValueError, RuntimeError) as exc:
        metadata["status"] = "fallback"
        metadata["fallback_reason"] = str(exc)
        return _assemble_packet(mandatory, metadata, []), metadata

    effective_allowed = list(allowed_paths or changed_paths)
    if authorizer is None:
        try:
            authorizer = build_task_authorizer(worktree, effective_allowed)
        except (OSError, ValueError, RuntimeError) as exc:
            metadata["status"] = "fallback"
            metadata["fallback_reason"] = f"review boundary unavailable: {exc}"
            return _assemble_packet(mandatory, metadata, []), metadata

    selected_blocks: list[str] = []
    selected_tokens = 0
    for candidate in rank_impact_candidates(discovery.get("candidates", []), changed_paths):
        if len(selected_blocks) >= max_entries:
            break
        try:
            authorizer(candidate.path)
        except Exception as exc:  # boundary implementations use task-specific exception classes
            gap = ScopeGap(
                path=candidate.path,
                symbol=candidate.symbol,
                reason="outside_review_allowed_paths",
                detail=str(exc),
            )
            metadata["scope_gaps"].append(asdict(gap))
            continue
        try:
            content = read_current_source(worktree, candidate)
        except (OSError, ValueError, RuntimeError) as exc:
            gap = ScopeGap(
                path=candidate.path,
                symbol=candidate.symbol,
                reason="source_unreadable",
                detail=str(exc),
            )
            metadata["scope_gaps"].append(asdict(gap))
            continue
        block = _render_source_block(candidate, content)
        tokens = gemma_local.estimate_text_tokens(block)
        if selected_tokens + tokens > impact_budget:
            continue
        selected_tokens += tokens
        selected_blocks.append(block)
        selected = SelectedImpact(
            path=candidate.path,
            symbol=candidate.symbol,
            label=candidate.label,
            relation=candidate.relation,
            reason=candidate.reason,
            priority=candidate.priority,
            start_line=candidate.start_line,
            end_line=candidate.end_line,
            estimated_tokens=tokens,
        )
        metadata["selected"].append(asdict(selected))

    metadata["budget"]["selected_impact_tokens"] = selected_tokens
    metadata["status"] = "enriched" if selected_blocks else "no_authorized_impact"
    return _assemble_packet(mandatory, metadata, selected_blocks), metadata


def _assemble_packet(mandatory: str, metadata: dict, blocks: list[str]) -> str:
    # Metadata contains decisions/hashes/paths only; source bodies live exclusively
    # in the local-only blocks below and are never serialized into metadata-out.
    metadata_text = json.dumps(metadata, ensure_ascii=False, sort_keys=True)
    sections = [mandatory.rstrip(), "## Local CKG impact metadata", metadata_text]
    if blocks:
        sections.extend(["## Local CKG impact source", "\n\n".join(blocks)])
    else:
        sections.extend(["## Local CKG impact source", "(none)"])
    return "\n\n".join(sections) + "\n"


def extract_task_section(text: str, task_id: str) -> str:
    """Return the matching Markdown heading section, or the full document if absent."""
    if not task_id:
        return text
    lines = text.splitlines()
    start = None
    level = None
    for index, line in enumerate(lines):
        match = re.match(r"^(#{1,6})\s+(.+)$", line)
        if match and task_id.lower() in match.group(2).lower():
            start = index
            level = len(match.group(1))
            break
    if start is None:
        return text
    end = len(lines)
    for index in range(start + 1, len(lines)):
        match = re.match(r"^(#{1,6})\s+", lines[index])
        if match and len(match.group(1)) <= level:
            end = index
            break
    return "\n".join(lines[start:end]).strip()


def _read_acceptance(args) -> str:
    if args.acceptance_text is not None:
        return args.acceptance_text
    if not args.acceptance_file:
        return ""
    text = Path(args.acceptance_file).read_text(encoding="utf-8")
    return extract_task_section(text, args.task_id or "")


def _write_metadata(path: str, metadata: dict) -> None:
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = target.with_suffix(target.suffix + ".tmp")
    temporary.write_text(
        json.dumps(metadata, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    os.replace(temporary, target)


def parse_args(argv=None):
    default_ctx, default_predict = _resolve_context_window()
    parser = argparse.ArgumentParser(
        description="Build a local-only CKG-enriched code-review packet."
    )
    parser.add_argument("diff", nargs="?", default="-", help="Diff file or '-' for stdin.")
    parser.add_argument("--worktree", default=".")
    parser.add_argument("--task-id", default="")
    acceptance = parser.add_mutually_exclusive_group()
    acceptance.add_argument("--acceptance-file")
    acceptance.add_argument("--acceptance-text")
    parser.add_argument("--allowed-path", action="append", default=[])
    parser.add_argument("--ckg-binary", default="codebase-memory-mcp")
    parser.add_argument("--num-ctx", type=int, default=default_ctx)
    parser.add_argument("--num-predict", type=int, default=default_predict)
    parser.add_argument(
        "--max-impact-tokens",
        type=int,
        default=int(
            os.environ.get(
                "DUBBRIDGE_REVIEW_CONTEXT_MAX_TOKENS", str(DEFAULT_MAX_IMPACT_TOKENS)
            )
        ),
    )
    parser.add_argument("--metadata-out")
    parser.add_argument("--no-ckg", action="store_true")
    return parser.parse_args(argv)


def main(argv=None) -> int:
    args = parse_args(argv)
    if args.diff == "-":
        diff_text = sys.stdin.read()
    else:
        diff_text = Path(args.diff).read_text(encoding="utf-8")
    acceptance_text = _read_acceptance(args)
    adapter = None if args.no_ckg else CodebaseMemoryCLIAdapter(binary=args.ckg_binary)
    packet, metadata = build_review_context(
        diff_text=diff_text,
        acceptance_text=acceptance_text,
        task_id=args.task_id,
        worktree=args.worktree,
        allowed_paths=args.allowed_path or None,
        adapter=adapter,
        num_ctx=args.num_ctx,
        num_predict=args.num_predict,
        max_impact_tokens=args.max_impact_tokens,
        enable_ckg=not args.no_ckg,
    )
    if args.metadata_out:
        _write_metadata(args.metadata_out, metadata)
    sys.stdout.write(packet)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
