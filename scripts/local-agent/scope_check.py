#!/usr/bin/env python3
"""Read-only diff-scope inspection for disposable local-agent worktrees."""

from __future__ import annotations

from dataclasses import dataclass
import subprocess


@dataclass(frozen=True)
class ScopeCheckResult:
    """The scope result, including whether the worktree has any changes."""

    in_scope: bool
    offending_paths: list[str]
    has_diff: bool


def _git_paths(worktree_dir: str, *args: str) -> set[str]:
    result = subprocess.run(
        ["git", *args],
        cwd=worktree_dir,
        check=True,
        capture_output=True,
    )
    return {path.decode("utf-8") for path in result.stdout.split(b"\0") if path}


def _normalise_allowed_path(path: str) -> str:
    normalised = path.removeprefix("./").rstrip("/")
    if not normalised or normalised.startswith("/") or ".." in normalised.split("/"):
        raise ValueError(f"allowed path must be a non-empty repository-relative path: {path!r}")
    return normalised


def _is_allowed(path: str, allowed_paths: list[str]) -> bool:
    return any(path == allowed or path.startswith(f"{allowed}/") for allowed in allowed_paths)


# Build/dependency artifact directory names that are safe to exclude from
# git-ignored path checking. Any path component matching one of these names
# means the whole path is a build artifact, not a deliberate model write.
# Scoped only to the --ignored scan below (T7f): a model can still only
# evade the scope gate via a *tracked* or plain-untracked path matching one
# of these names, which continues to be checked exactly as before.
_ARTIFACT_DIR_NAMES = frozenset(
    (
        "target",
        "node_modules",
        "__pycache__",
        ".pytest_cache",
        "dist",
        "build",
        "coverage",
    )
)


def _is_artifact_path(path: str) -> bool:
    """Return True if any component of *path* is a build/dependency artifact dir."""
    return any(part in _ARTIFACT_DIR_NAMES for part in path.split("/"))


def check_scope(worktree_dir: str, allowed_paths: list[str]) -> ScopeCheckResult:
    """Return whether every changed path belongs to ``allowed_paths``.

    The base commit is the worktree's current ``HEAD``.  ``git diff`` captures
    tracked staged and unstaged changes; untracked files are included separately
    so a newly-created out-of-scope file cannot evade the post-run gate.

    Git-ignored paths are only filtered against known build/dependency
    artifact directories (T7f) -- e.g. ``target/`` or ``.pytest_cache/``
    populated by the model's own verification commands. Any other
    git-ignored path (e.g. a newly written ``.env``) is still checked like
    any other changed path, so a model cannot hide an out-of-scope write
    behind ``.gitignore``.
    """

    allowed = [_normalise_allowed_path(path) for path in allowed_paths]
    changed_paths = _git_paths(worktree_dir, "diff", "--name-only", "-z", "HEAD")
    changed_paths.update(
        _git_paths(worktree_dir, "ls-files", "--others", "--exclude-standard", "-z"),
    )
    ignored_paths = _git_paths(
        worktree_dir,
        "ls-files",
        "--others",
        "--ignored",
        "--exclude-standard",
        "-z",
    )
    changed_paths.update(path for path in ignored_paths if not _is_artifact_path(path))
    ordered_paths = sorted(changed_paths)
    offending_paths = [path for path in ordered_paths if not _is_allowed(path, allowed)]
    return ScopeCheckResult(
        in_scope=not offending_paths,
        offending_paths=offending_paths,
        has_diff=bool(ordered_paths),
    )
