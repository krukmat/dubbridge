#!/usr/bin/env python3
"""Bounded file primitives for the local runner.

This module owns checked source reads plus the small write/patch tool surface.
Context selection is owned by ``ContextProvider``; ``preload_context()`` remains
only as the behavior-compatible implementation used by ``LegacyContextProvider``
and as the source fallback when graph-guided retrieval is unavailable.

Filesystem hardening stays here (``O_NOFOLLOW`` on every open, atomic overwrite
via a temp file + rename) together with the "anchor must match exactly once"
rule. Boundary enforcement remains owned by the injected ``boundary`` and every
read or mutation passes through that existing capability check.
"""

import os

ALLOWED_TOOL_NAMES = ("write_file", "apply_patch", "finish")


class RunnerFileTools:
    def __init__(self, worktree_dir, boundary, malformed_error, boundary_error):
        self._worktree_dir = worktree_dir
        self._boundary = boundary
        self._malformed_error = malformed_error
        self._boundary_error = boundary_error
        self._edited_paths = set()

    def close(self):
        # Present for interface parity with the old semantic tools (main()
        # calls tools.close() in a finally); there is no session to tear down.
        return None

    def allowed_tool_names(self):
        return ALLOWED_TOOL_NAMES

    @property
    def edited_paths(self):
        return tuple(sorted(self._edited_paths))

    def read_checked(self, path):
        return self._read_existing(path)

    def preload_context(self, allowed_paths):
        """Return complete contents for every file authorized by the card.

        This is the legacy/fallback context path. Existing directory
        capabilities are expanded recursively without following directory
        symlinks. Missing exact paths are represented so the model knows it may
        create them. Every resulting file still passes through the boundary
        before it is opened.
        """
        entries = []
        for path in allowed_paths:
            self._boundary.check_write(path)
            target = os.path.join(self._worktree_dir, path)
            if not os.path.exists(target):
                entries.append({"path": path, "missing": True, "content": None})
                continue
            if os.path.isdir(target):
                for root, dirs, files in os.walk(target, followlinks=False):
                    dirs[:] = [
                        name
                        for name in sorted(dirs)
                        if not os.path.islink(os.path.join(root, name))
                    ]
                    for name in sorted(files):
                        absolute = os.path.join(root, name)
                        if os.path.islink(absolute):
                            continue
                        relative = os.path.relpath(absolute, self._worktree_dir)
                        entries.append(
                            {
                                "path": relative,
                                "missing": False,
                                "content": self._read_existing(relative),
                            }
                        )
                continue
            entries.append(
                {
                    "path": path,
                    "missing": False,
                    "content": self._read_existing(path),
                }
            )
        return entries

    def handle(self, call):
        if call.name == "write_file":
            return self._write_file(call.arguments)
        if call.name == "apply_patch":
            return self._apply_patch(call.arguments)
        return None

    def _write_file(self, arguments):
        path = self._require(arguments, "path")
        content = arguments.get("content", "")
        self._boundary.check_write(path)
        target = os.path.join(self._worktree_dir, path)
        os.makedirs(os.path.dirname(target) or ".", exist_ok=True)
        created = not os.path.exists(target)
        self._write_nofollow(target, path, content)
        self._edited_paths.add(os.path.normpath(path))
        return {
            "tool": "write_file",
            "path": path,
            "ok": True,
            "created": created,
        }

    def _apply_patch(self, arguments):
        path = self._require(arguments, "path")
        anchor = self._require(arguments, "anchor")
        replacement = arguments.get("replacement", "")
        if not anchor:
            raise self._malformed_error("apply_patch anchor must be non-empty")

        original = self._read_existing(path)
        occurrences = original.count(anchor)
        if occurrences != 1:
            raise self._malformed_error(
                f"apply_patch anchor must match exactly once in {path!r}; "
                f"matched {occurrences} times"
            )
        updated = original.replace(anchor, replacement, 1)
        target = os.path.join(self._worktree_dir, path)
        self._write_nofollow(target, path, updated)
        self._edited_paths.add(os.path.normpath(path))
        return {"tool": "apply_patch", "path": path, "ok": True}

    def _read_existing(self, path):
        self._boundary.check_write(path)
        target = os.path.join(self._worktree_dir, path)
        if not os.path.exists(target):
            raise self._malformed_error(f"path does not exist: {path!r}")
        if os.path.isdir(target):
            raise self._malformed_error(f"path is a directory: {path!r}")
        try:
            fd = os.open(target, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        except OSError as exc:
            raise self._boundary_error(f"refused to open {path!r}: {exc}") from exc
        try:
            with os.fdopen(fd, "r", encoding="utf-8") as handle:
                return handle.read()
        except UnicodeDecodeError as exc:
            raise self._boundary_error(f"refused non-UTF-8 source file {path!r}") from exc

    def _write_nofollow(self, target, path, content):
        if not isinstance(content, str):
            raise self._malformed_error(f"content must be a string for {path!r}")
        if os.path.lexists(target) and os.path.islink(target):
            raise self._boundary_error(f"refused symlink write: {path!r}")
        directory = os.path.dirname(target) or self._worktree_dir
        basename = os.path.basename(target)
        temp_name = f".{basename}.local-agent-tmp-{os.getpid()}"
        temp_path = os.path.join(directory, temp_name)
        fd = None
        try:
            fd = os.open(
                temp_path,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0),
                0o600,
            )
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                fd = None
                handle.write(content)
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temp_path, target)
        except OSError as exc:
            raise self._boundary_error(f"refused to write {path!r}: {exc}") from exc
        finally:
            if fd is not None:
                os.close(fd)
            try:
                if os.path.exists(temp_path):
                    os.unlink(temp_path)
            except OSError:
                pass

    def _require(self, arguments, key):
        value = arguments.get(key)
        if not isinstance(value, str):
            raise self._malformed_error(f"{key} must be a string")
        return value
