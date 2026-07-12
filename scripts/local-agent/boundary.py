#!/usr/bin/env python3
"""Fail-closed execution boundary for the local agentic runner (ADR-036 §3).

Implements the `check_write(path)` / `check_command(argv)` interface that
`run_local_task.py`'s `NullBoundary` stubbed out. Any rejection raises
`run_local_task.BoundaryViolation` so the runner's existing abort path (added
in T6a) handles it without change.

T7b-3: containment no longer depends on an *allowlist* of which commands run.
`check_command` no longer restricts by executable/argument/shell-composition/
path vocabulary the way the old ALLOWED_COMMAND_PREFIXES allowlist did —
arbitrary development commands are permitted, with the worktree as `cwd`
(T6a) and the stripped environment (`env_for_subprocess`, unchanged below)
as the primary containment mechanisms. This is a narrower guarantee than the
old allowlist: a permitted command can still read files or make network
calls the process itself has access to (worktree `cwd` is not a filesystem
sandbox, and the stripped env only removes credentials from what the command
*sees*, it does not stop the command from reaching outside the worktree on
its own). What post-run diff-scope validation (T7c-a/b2/b3) *does* guarantee
is on the write side: no diff touching a path outside `allowed_paths` is
ever copied to the primary checkout, regardless of which commands produced
it. That write-side scope check, not a command allowlist, is what this task
relies on for filesystem writes — it does not claim to contain arbitrary
read/network side effects.

A small denylist (`DENIED_COMMAND_PREFIXES`) is retained as defense-in-depth
for the highest-severity, hardest-to-undo actions specifically: `git push`
(irreversible exfiltration to a remote), `docker` (escapes the worktree jail
model entirely via containers/volumes), and `rm -rf` (irreversible
destructive deletion). This is deliberately narrow — a denylist of the three
worst cases, not the allowlist this task removes — and does not change the
task's core containment model: everything not on this short list still
passes unconditionally, including `curl`, `python3 -c ...`, `sh -c ...`, and
any other command the old allowlist would have rejected.
"""

import os
import shlex
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from run_local_task import BoundaryViolation

DENIED_COMMAND_PREFIXES = (
    ("git", "push"),
    ("rm", "-rf"),
    ("docker",),
)


def _matches_prefix(argv, prefix):
    return len(argv) >= len(prefix) and tuple(argv[: len(prefix)]) == prefix


def _tokenize_argv_element(element):
    try:
        return shlex.split(element)
    except ValueError:
        # unbalanced quotes etc.: fail closed by treating the raw string as
        # a single opaque token rather than raising here.
        return [element]


def _contains_subsequence(tokens, subsequence):
    n = len(subsequence)
    return any(
        tuple(tokens[i : i + n]) == subsequence for i in range(len(tokens) - n + 1)
    )


def _argv_embeds_denied_subcommand(argv):
    # A single argv element can embed an entire shell command (e.g.
    # ["sh", "-c", "git  push origin main"]) that a positional prefix check
    # on argv[0]/argv[1] would miss entirely. Tokenize every element with
    # shlex (not a literal substring match, which a double space or extra
    # quoting would defeat) and look for a denied prefix as a token
    # subsequence anywhere in the combined token stream.
    tokens = [tok for element in argv for tok in _tokenize_argv_element(element)]
    return any(_contains_subsequence(tokens, prefix) for prefix in DENIED_COMMAND_PREFIXES)


class LocalAgentBoundary:
    def __init__(self, worktree_root):
        # Resolve once at construction: this is the only trusted jail root.
        # Symlinks in the root itself are intentionally resolved here (the
        # worktree path itself is operator-controlled, not model-controlled).
        self._worktree_root = os.path.realpath(worktree_root)

    def check_write(self, path):
        if os.path.isabs(path):
            raise BoundaryViolation(f"absolute path rejected: {path!r}")

        candidate = os.path.join(self._worktree_root, path)
        # Resolve symlinks and ".." components on the full candidate path,
        # not just the input string — a symlink component partway through
        # the path (not just at the leaf) must still be caught, and doing
        # this check against the *real* filesystem defeats a TOCTOU race
        # where a symlink is swapped in between validation and use, because
        # os.path.realpath re-resolves at check time immediately before the
        # caller performs the actual write.
        resolved = os.path.realpath(candidate)

        try:
            common = os.path.commonpath([resolved, self._worktree_root])
        except ValueError:
            # different drives / roots entirely — definitely outside the jail
            raise BoundaryViolation(f"path escapes worktree: {path!r}")

        if common != self._worktree_root:
            raise BoundaryViolation(f"path escapes worktree: {path!r}")

    def check_command(self, argv):
        if not argv:
            raise BoundaryViolation("empty command rejected")

        if _argv_embeds_denied_subcommand(argv):
            raise BoundaryViolation(f"denied command: {argv!r}")

        for prefix in DENIED_COMMAND_PREFIXES:
            if _matches_prefix(argv, prefix):
                raise BoundaryViolation(f"denied command: {argv!r}")

        # No allowlist gate: anything not on the short denylist above
        # passes. Write-side containment is the worktree cwd (T6a), the
        # stripped environment (env_for_subprocess below), and post-run
        # diff-scope validation (T7c-a/b2/b3) — not a command allowlist.
        # This does not sandbox a command's own reads or network access
        # beyond the three denylisted cases; see the module docstring.

    def env_for_subprocess(self):
        return stripped_agent_env()


# D14 finding: a blanket "DUBBRIDGE_" prefix allowlist is unsound — the
# repo's own crates/config/src/lib.rs defines real credential-bearing
# operator env vars under that exact prefix (DUBBRIDGE_AUTH_JWT_SECRET,
# DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET, DUBBRIDGE_DATABASE_URL, etc.). Any
# developer with those set in their shell would have them forwarded straight
# into the untrusted model subprocess. ADR-036 §3 specifies passing through
# "only DUBBRIDGE_ENV=local bindings" — an explicit single variable, not a
# prefix — so the allowlist here is a closed name set, not a prefix match.
ALLOWED_ENV_VAR_NAMES = {"OLLAMA_HOST", "DUBBRIDGE_ENV"}


def stripped_agent_env(source_env=None):
    """Return the minimal environment the local-agent subprocess may see.

    Only the explicitly named ALLOWED_ENV_VAR_NAMES pass through; PATH is
    kept so allowlisted tools (cargo, npm, make) can still be located and
    executed. Everything else — credentials, unrelated tooling env vars, the
    caller's full environment — is dropped by construction (closed name set,
    not a denylist and not a prefix match).
    """
    source_env = source_env if source_env is not None else os.environ
    stripped = {}
    if "PATH" in source_env:
        stripped["PATH"] = source_env["PATH"]
    for key, value in source_env.items():
        if key in ALLOWED_ENV_VAR_NAMES:
            stripped[key] = value
    return stripped
