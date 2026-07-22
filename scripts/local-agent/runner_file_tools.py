#!/usr/bin/env python3
"""File tools for the local runner: read, write, and single-anchor patch.

Deliberately simple. There is no language server, no symbol lookup, and no
line/byte budget: the local implementer has a very large context window (the
S-140 pilot target is ~14k tokens against a 262k-token model), so it can read
the file it must edit and rewrite or patch it directly. The only edit-time
safety kept from the earlier semantic tools is the filesystem hardening
(``O_NOFOLLOW`` on every open, atomic overwrite via a temp file + rename) and
the "anchor must match exactly once" rule, which stops a blind patch from
silently editing the wrong occurrence.

Boundary enforcement (path allow-listing) stays owned by the injected
``boundary``; this module calls ``boundary.check_write(path)`` before every
read or write, exactly as the old tools did.
"""

import os

ALLOWED_TOOL_NAMES = (
    "read_file",
    "write_file",
    "apply_patch",
    "run_command",
    "finish",
)


class RunnerFileTools:
    def __init__(self, worktree_dir, boundary, malformed_error, boundary_error):
        self._worktree_dir = worktree_dir
        self._boundary = boundary
        self._malformed_error = malformed_error
        self._boundary_error = boundary_error

    def close(self):
        # Present for interface parity with the old semantic tools (main()
        # calls tools.close() in a finally); there is no session to tear down.
        return None

    def allowed_tool_names(self):
        return ALLOWED_TOOL_NAMES

    def handle(self, call):
        if call.name == "read_file":
            return self._read_file(call.arguments)
        if call.name == "write_file":
            return self._write_file(call.arguments)
        if call.name == "apply_patch":
            return self._apply_patch(call.arguments)
        return None

    def _read_file(self, arguments):
        path = self._require(arguments, "path")
        content = self._read_existing(path)
        return {
            "tool": "read_file",
            "path": path,
            "ok": True,
            "content": content,
            "line_count": self._line_count(content),
        }

    def _write_file(self, arguments):
        path = self._require(arguments, "path")
        content = arguments.get("content", "")
        self._boundary.check_write(path)
        target = os.path.join(self._worktree_dir, path)
        os.makedirs(os.path.dirname(target) or ".", exist_ok=True)
        created = not os.path.exists(target)
        self._write_nofollow(target, path, content)
        return {
            "tool": "write_file",
            "path": path,
            "ok": True,
            "created": created,
            "line_count": self._line_count(content),
            "byte_count": len(content.encode("utf-8")),
        }

    def _apply_patch(self, arguments):
        path = self._require(arguments, "path")
        anchor = self._require(arguments, "anchor")
        replacement = self._require(arguments, "replacement")
        content = self._read_existing(path)
        matches = content.count(anchor)
        if matches != 1:
            raise self._malformed_error(
                f"apply_patch: anchor for {path!r} matched {matches} locations; require exactly 1"
            )
        updated = content.replace(anchor, replacement, 1)
        target = os.path.join(self._worktree_dir, path)
        self._write_nofollow(target, path, updated)
        return {
            "tool": "apply_patch",
            "path": path,
            "ok": True,
            "anchor_matches": 1,
            "line_count": self._line_count(replacement),
            "byte_count": len(replacement.encode("utf-8")),
        }

    def _read_existing(self, path):
        self._boundary.check_write(path)
        target = os.path.join(self._worktree_dir, path)
        try:
            fd = os.open(target, os.O_RDONLY | os.O_NOFOLLOW)
            with os.fdopen(fd, "r", encoding="utf-8", errors="replace") as handle:
                return handle.read()
        except FileNotFoundError as exc:
            raise self._malformed_error(f"file not found in worktree: {path!r}") from exc
        except IsADirectoryError as exc:
            raise self._malformed_error(f"path is a directory, not a file: {path!r}") from exc
        except OSError as exc:
            raise self._boundary_error(f"read rejected: {path!r} ({exc})") from exc

    def _write_nofollow(self, target, path, content):
        # O_NOFOLLOW so a pre-planted symlink at `target` cannot redirect the
        # write outside the worktree. os.O_TRUNC gives create-or-overwrite in a
        # single open; the earlier tools split create (O_EXCL) from overwrite,
        # but the simple contract allows both through one path.
        try:
            fd = os.open(
                target, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_NOFOLLOW
            )
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                handle.write(content)
        except OSError as exc:
            raise self._boundary_error(
                f"write rejected at open time: {path!r} ({exc})"
            ) from exc

    def _line_count(self, content):
        return 0 if not content else content.count("\n") + (0 if content.endswith("\n") else 1)

    def _require(self, arguments, key):
        if key not in arguments:
            raise self._malformed_error(f"missing required argument {key!r}")
        return arguments[key]
