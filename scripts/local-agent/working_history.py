from __future__ import annotations

import hashlib
import json


def _sha256_text(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def compact_assistant_action(call):
    """Return a model-visible summary of a tool call without replaying source bodies."""
    name = call.name
    args = call.arguments
    if name == "write_file":
        path = args.get("path")
        content = args.get("content", "")
        return (
            f"ACTION: write_file\nPATH: {path}\n"
            f"GENERATED_BYTES: {len(content.encode('utf-8')) if isinstance(content, str) else 0}"
        )
    if name == "apply_patch":
        path = args.get("path")
        anchor = args.get("anchor", "")
        replacement = args.get("replacement", "")
        anchor_hash = _sha256_text(anchor) if isinstance(anchor, str) else "invalid"
        replacement_bytes = (
            len(replacement.encode("utf-8")) if isinstance(replacement, str) else 0
        )
        return (
            f"ACTION: apply_patch\nPATH: {path}\n"
            f"ANCHOR_SHA256: {anchor_hash}\nREPLACEMENT_BYTES: {replacement_bytes}"
        )
    if name == "finish":
        return "ACTION: finish"
    return f"ACTION: {name}"


def compact_tool_result(result, file_tools=None):
    """Compact a successful runner result while keeping the worktree authoritative."""
    tool = result.get("tool", "unknown")
    lines = [f"RESULT: {tool}", f"OK: {bool(result.get('ok'))}"]
    path = result.get("path")
    if path:
        lines.append(f"PATH: {path}")
    if "created" in result:
        lines.append(f"CREATED: {bool(result.get('created'))}")
    if path and result.get("ok") and file_tools is not None and tool in {
        "write_file",
        "apply_patch",
    }:
        try:
            current = file_tools.read_checked(path)
        except (OSError, ValueError, RuntimeError):
            current = None
        if isinstance(current, str):
            lines.append(f"CURRENT_SOURCE_SHA256: {_sha256_text(current)}")
            lines.append(f"CURRENT_SOURCE_BYTES: {len(current.encode('utf-8'))}")
    return "\n".join(lines)


def compact_history_event(call, result, file_tools=None):
    return {
        "assistant": compact_assistant_action(call),
        "user": compact_tool_result(result, file_tools=file_tools),
    }


def compact_json(value, max_chars=1200):
    """Bound arbitrary metadata for model-visible error/status messages."""
    text = json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    if len(text) <= max_chars:
        return text
    return text[: max_chars - 20] + "...[truncated]"
