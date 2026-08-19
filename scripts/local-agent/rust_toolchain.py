#!/usr/bin/env python3
"""Rust-specific formatter/boundary wiring for the local agentic runner.

Extracted from run_local_task.py (LRPC-0b, Extract Module / Single
Responsibility) as the part of that runner most likely to need per-language
extension later -- kept out of the turn-by-turn session loop for that reason.
Behavior-preserving: no logic changed from the original module.
"""

from __future__ import annotations

import os
import re
import tempfile


def build_default_boundary(worktree_root, card):
    # Deferred import: boundary.py imports run_local_task (for
    # BoundaryViolation), so importing it at module load time here would
    # create a circular import.
    import boundary

    return boundary.LocalAgentBoundary(
        worktree_root,
        card.allowed_paths,
        card.acceptance_argvs,
    )


def _rust_edition_for_path(worktree_root, relative_path):
    """Resolve the nearest explicit Cargo edition, then the workspace default."""

    def manifest_edition(manifest_path, workspace=False):
        try:
            with open(manifest_path, encoding="utf-8") as handle:
                lines = handle.readlines()
        except OSError:
            return None
        wanted_section = "workspace.package" if workspace else "package"
        active = False
        for line in lines:
            section_match = re.match(r"\s*\[([^]]+)\]\s*(?:#.*)?$", line)
            if section_match:
                active = section_match.group(1).strip() == wanted_section
                continue
            if active:
                edition_match = re.match(
                    r"\s*edition\s*=\s*['\"]([^'\"]+)['\"]", line
                )
                if edition_match:
                    return edition_match.group(1)
        return None

    root_manifest = os.path.join(worktree_root, "Cargo.toml")
    fallback = (
        manifest_edition(root_manifest, workspace=True)
        or manifest_edition(root_manifest)
        or "2021"
    )
    current = os.path.dirname(os.path.join(worktree_root, relative_path))
    while os.path.commonpath((current, worktree_root)) == worktree_root:
        edition = manifest_edition(os.path.join(current, "Cargo.toml"))
        if edition:
            return edition
        if current == worktree_root:
            break
        current = os.path.dirname(current)
    return fallback


def build_default_formatter(card, boundary, file_tools):
    """Format only edited Rust paths, via isolated copies outside the worktree."""

    # Deferred import: avoids a module-load-time circular import with
    # run_local_task (which imports this module for the re-export facade and
    # supplies _run_command_with_timeout via the session_loop module).
    from session_loop import _run_command_with_timeout

    def formatter(worktree_dir):
        rust_paths = [path for path in file_tools.edited_paths if path.endswith(".rs")]
        if not rust_paths:
            return {"passed": True, "formatted_paths": [], "output": "no edited Rust files"}

        formatted_paths = []
        outputs = []
        with tempfile.TemporaryDirectory(prefix="dubbridge-local-format-") as temp_dir:
            for index, path in enumerate(rust_paths):
                source = file_tools.read_checked(path)
                temp_path = os.path.join(temp_dir, f"{index}-{os.path.basename(path)}")
                with open(temp_path, "w", encoding="utf-8") as handle:
                    handle.write(source)
                edition = _rust_edition_for_path(worktree_dir, path)
                argv = [
                    "rustfmt",
                    "--edition",
                    edition,
                    "--config",
                    "skip_children=true",
                    temp_path,
                ]
                result = _run_command_with_timeout(argv, worktree_dir, boundary)
                outputs.append(
                    f"$ rustfmt --edition {edition} --config skip_children=true {path}\n"
                    f"(exit {result['returncode']})\n{result['stdout']}\n{result['stderr']}"
                )
                if not result["ok"]:
                    return {
                        "passed": False,
                        "formatted_paths": formatted_paths,
                        "output": "\n\n".join(outputs),
                    }
                with open(temp_path, encoding="utf-8") as handle:
                    formatted = handle.read()
                file_tools.replace_from_runner(path, formatted)
                formatted_paths.append(path)

        return {
            "passed": True,
            "formatted_paths": formatted_paths,
            "output": "\n\n".join(outputs),
        }

    return formatter
