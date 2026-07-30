"""Canonical path containment for Antares candidate/operand paths (T2b).

Validates that a path -- whether a command operand from `command_policy.py`
or a candidate path submitted via `submit_vulnerable_files` (T2a) -- resolves
strictly inside the read-only repository snapshot after symlink resolution.
This module performs no filesystem mutation and no command execution; it only
resolves and compares paths.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

_TERMINAL_STATE_SCRIPT = Path(__file__).with_name("terminal_state.py")
_TERMINAL_STATE_SPEC = importlib.util.spec_from_file_location(
    "antares_terminal_state", _TERMINAL_STATE_SCRIPT
)
if _TERMINAL_STATE_SPEC is None or _TERMINAL_STATE_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_TERMINAL_STATE_SCRIPT}")
_TERMINAL_STATE_MOD = importlib.util.module_from_spec(_TERMINAL_STATE_SPEC)
sys.modules[_TERMINAL_STATE_SPEC.name] = _TERMINAL_STATE_MOD
_TERMINAL_STATE_SPEC.loader.exec_module(_TERMINAL_STATE_MOD)

TerminalState = _TERMINAL_STATE_MOD.TerminalState
TerminalStateKind = _TERMINAL_STATE_MOD.TerminalStateKind


def _rejected(detail: str) -> TerminalState:
    return TerminalState(
        kind=TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE, detail=detail
    )


def resolve_within_snapshot(raw_path: str, snapshot_root: Path) -> Path | None:
    """Resolve `raw_path` against `snapshot_root` after symlink resolution.

    Returns the resolved absolute `Path` if it stays inside the snapshot,
    otherwise `None`. Rejects absolute operand paths and `..` traversal
    outright (EC-3) before touching the filesystem, then canonicalizes via
    `Path.resolve()` (which follows symlinks) and checks containment against
    the snapshot's own resolved root -- so a symlink whose target escapes the
    snapshot is rejected even if its own path looks contained (EC-3).
    """
    if raw_path.strip() == "":
        return None
    if Path(raw_path).is_absolute():
        return None
    if ".." in Path(raw_path).parts:
        return None

    resolved_root = snapshot_root.resolve()
    candidate = (resolved_root / raw_path).resolve()

    try:
        candidate.relative_to(resolved_root)
    except ValueError:
        return None
    return candidate


def check_path_containment(
    raw_paths: tuple[str, ...], snapshot_root: Path
) -> TerminalState:
    """Validate a tuple of candidate/operand paths against the snapshot.

    HP-2: every path that resolves inside the snapshot is returned (in
    input order) as `candidates` on a PATH_CONTAINMENT_VALID state.
    EC-3: an absolute path, a `..`-traversing path, or a symlink resolving
    outside the snapshot causes the whole batch to fail closed as
    PATH_REJECTED_CONTAINMENT_ESCAPE -- a partially-valid batch is never
    silently narrowed, since callers (T2c command execution, T2d candidate
    recording) must not guess which subset of an untrusted batch was
    intended.
    """
    if not raw_paths:
        return _rejected("No paths were supplied for containment validation.")

    validated: list[str] = []
    for raw_path in raw_paths:
        resolved = resolve_within_snapshot(raw_path, snapshot_root)
        if resolved is None:
            return _rejected(
                f"Path {raw_path!r} escapes the read-only snapshot root "
                f"{snapshot_root!s} after canonical resolution."
            )
        validated.append(raw_path)

    return TerminalState(
        kind=TerminalStateKind.PATH_CONTAINMENT_VALID, candidates=tuple(validated)
    )
