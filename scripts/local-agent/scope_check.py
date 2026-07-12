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


def check_scope(worktree_dir: str, allowed_paths: list[str]) -> ScopeCheckResult:
    """Return whether every changed path belongs to ``allowed_paths``.

    The base commit is the worktree's current ``HEAD``.  ``git diff`` captures
    tracked staged and unstaged changes; untracked files are included separately
    so a newly-created out-of-scope file cannot evade the post-run gate.
    """

    allowed = [_normalise_allowed_path(path) for path in allowed_paths]
    changed_paths = _git_paths(worktree_dir, "diff", "--name-only", "-z", "HEAD")
    changed_paths.update(
        _git_paths(worktree_dir, "ls-files", "--others", "--exclude-standard", "-z"),
    )
    changed_paths.update(
        _git_paths(
            worktree_dir,
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "-z",
        ),
    )
    ordered_paths = sorted(changed_paths)
    offending_paths = [path for path in ordered_paths if not _is_allowed(path, allowed)]
    return ScopeCheckResult(
        in_scope=not offending_paths,
        offending_paths=offending_paths,
        has_diff=bool(ordered_paths),
    )
