#!/usr/bin/env python3
"""Fail-closed execution boundary for the local agentic runner (ADR-036 §3).

Implements the `check_write(path)` / `check_command(argv)` interface that
`run_local_task.py`'s `NullBoundary` stubbed out. Any rejection raises
`run_local_task.BoundaryViolation` so the runner's existing abort path (added
in T6a) handles it without change.
"""

import os
import shlex
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from run_local_task import BoundaryViolation

# Allowlist-first: an argv[0] not in this set is rejected by default, even if
# it doesn't match anything in the denylist. The denylist exists as explicit,
# auditable defense-in-depth for the highest-risk commands, not as the
# primary gate.
ALLOWED_COMMAND_PREFIXES = (
    ("cargo", "test"),
    ("cargo", "build"),
    ("cargo", "check"),
    ("cargo", "fmt"),
    ("cargo", "clippy"),
    ("npm", "test"),
    ("npm", "run", "lint"),
    ("npm", "run", "typecheck"),
    ("make",),  # qa-* targets checked explicitly below
    # Read-only inspection/verification commands: confirmed necessary from a
    # real pilot run — the model reasonably wanted to grep a file before
    # editing it, and to run a card's own `python3 -m unittest ...`
    # verify_commands via the agentic loop's own run_command tool, not only
    # through the operator-controlled test_runner harness.
    ("grep",),
    ("python3", "-m", "unittest"),
    # Read-only workspace inspection: confirmed necessary from a real pilot
    # run — the model reasonably wanted to inspect the crate/workspace
    # structure before editing Rust code. Emits no side effects (no
    # compilation, no filesystem writes). `cargo metadata` does accept
    # --manifest-path (verified via `cargo metadata --help`), but that's
    # already covered by the existing shared PATH_ACCEPTING_FLAGS check —
    # no new escape surface to add here.
    ("cargo", "metadata"),
)

DENIED_COMMAND_PREFIXES = (
    ("git", "push"),
    ("rm", "-rf"),
    ("docker",),
)


def _is_make_qa_target(argv):
    return len(argv) >= 2 and argv[0] == "make" and argv[1].startswith("qa-")


def _matches_prefix(argv, prefix):
    return len(argv) >= len(prefix) and tuple(argv[: len(prefix)]) == prefix


def _tokenize_argv_element(element):
    try:
        return shlex.split(element)
    except ValueError:
        # unbalanced quotes etc.: fail closed by treating the raw string as
        # a single opaque token rather than raising here — the caller still
        # rejects on "not on either list" if nothing matches.
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


# D14 finding: check_command only inspected argv[0]/argv[1] against fixed
# prefixes — an allowlisted command's own arguments (e.g. `cargo build
# --manifest-path /etc/passwd`, `make qa-fmt -C /`) were never checked, so an
# allowlisted command could still act outside the worktree via its flags.
# These are the flag spellings (both "--flag value" and "--flag=value") that
# accept an arbitrary path for the allowlisted tools in this module. This
# only covers *named*-flag path arguments — grep's bare positional file
# operands are a separate gap, closed by `_grep_argv_escapes_worktree` below
# (a second D14 finding: this list was never a complete accounting of every
# allowlisted tool's path-accepting surface).
PATH_ACCEPTING_FLAGS = (
    "--manifest-path", "-C", "--target-dir", "--prefix",
    # unittest discover's directory flags — see the second D14 finding below.
    "-s", "--start-directory", "-t", "--top-level-directory",
)


def _argv_path_flag_escapes_worktree(argv, worktree_root):
    i = 0
    while i < len(argv):
        token = argv[i]
        flag, _, inline_value = token.partition("=")
        if flag in PATH_ACCEPTING_FLAGS:
            value = inline_value if inline_value else (argv[i + 1] if i + 1 < len(argv) else None)
            if value is not None:
                candidate = value if os.path.isabs(value) else os.path.join(worktree_root, value)
                resolved = os.path.realpath(candidate)
                try:
                    common = os.path.commonpath([resolved, worktree_root])
                except ValueError:
                    return True
                if common != worktree_root:
                    return True
        i += 1
    return False


def _resolves_outside_worktree(value, worktree_root):
    candidate = value if os.path.isabs(value) else os.path.join(worktree_root, value)
    resolved = os.path.realpath(candidate)
    try:
        common = os.path.commonpath([resolved, worktree_root])
    except ValueError:
        return True
    return common != worktree_root


# D14 finding (blocking): grep takes its file operands as bare positional
# arguments, not named flags — PATH_ACCEPTING_FLAGS (built for cargo/make's
# "--flag value" vocabulary) never inspects them at all, so `grep pattern
# /etc/passwd` or `grep -r pattern /` passed check_command completely
# unchecked, letting the model read any file the process can access. Every
# bare (non-flag) token after argv[0] is validated against the worktree jail;
# grep's PATTERN argument is virtually never itself an absolute or
# ".."-escaping path, so this fails closed on the rare cases it might be
# without materially restricting real usage.
def _grep_argv_escapes_worktree(argv, worktree_root):
    for token in argv[1:]:
        if token.startswith("-"):
            continue
        if _resolves_outside_worktree(token, worktree_root):
            return True
    return False


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
            raise BoundaryViolation(f"denied command (shell-embedded): {argv!r}")

        for prefix in DENIED_COMMAND_PREFIXES:
            if _matches_prefix(argv, prefix):
                raise BoundaryViolation(f"denied command: {argv!r}")

        if _argv_path_flag_escapes_worktree(argv, self._worktree_root):
            raise BoundaryViolation(f"command path flag escapes worktree: {argv!r}")

        if argv[0] == "grep" and _grep_argv_escapes_worktree(argv, self._worktree_root):
            raise BoundaryViolation(f"grep argument escapes worktree: {argv!r}")

        if _is_make_qa_target(argv):
            return None

        for prefix in ALLOWED_COMMAND_PREFIXES:
            if prefix == ("make",):
                continue  # only the qa-* form above is allowed for make
            if _matches_prefix(argv, prefix):
                return None

        # Fail closed: anything not explicitly recognized as allowed is
        # rejected, even if it isn't on the denylist either.
        raise BoundaryViolation(f"command not in allowlist: {argv!r}")

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
