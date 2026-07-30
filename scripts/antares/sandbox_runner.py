"""Ephemeral sandboxed subprocess execution for Antares (T2c-1).

Executes one already-validated `COMMAND_PLAN_VALID` argv (from
`command_policy.validate_command`, T2b) inside an isolated subprocess:
enforced network denial, a stripped/credential-free environment, and
privilege drop when the host process runs as root (a no-op otherwise --
see `_drop_privileges`). Owns the single-process lifecycle only -- launch,
isolate, per-command timeout, kill on timeout. Aggregate session accounting
(15-command wall budget, CPU/RAM/PID/output-size caps, cross-run teardown
verification) is T2c-2's responsibility, layered on top of this module.

This module performs no shell evaluation: it always invokes
`subprocess.Popen` with a list argv and `shell=False`.

Network isolation is a real per-platform enforcement, not a stripped
environment that merely hopes an allowlisted read-only tool never dials
out. `NetworkIsolation` is the injectable strategy; `resolve_network_isolation`
picks the platform-appropriate implementation and returns
SANDBOX_RUNTIME_UNAVAILABLE rather than running unisolated when no proven
mechanism exists for the current platform (EC-2) -- there is no "run anyway
without network isolation" success path.
"""

from __future__ import annotations

import importlib.util
import os
import platform
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Callable, Protocol

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

# Per-command wall timeout in seconds. T2c-2 owns the aggregate wall-time
# budget across a whole run; this is only the single-process ceiling.
DEFAULT_COMMAND_TIMEOUT_SECONDS = 10.0

# Minimal PATH so allowlisted executables (grep, find, cat, ls, head, tail,
# wc -- see command_policy.ALLOWED_EXECUTABLES) resolve without inheriting
# the caller's full environment.
_MINIMAL_PATH = "/usr/bin:/bin"

# macOS `sandbox-exec` profile: deny all network sockets outright, allow
# everything else the wrapped command needs by default (file read is further
# constrained by the argv's own path-containment validation from T2b, and by
# running with cwd pinned to the read-only snapshot root).
_MACOS_SANDBOX_PROFILE = "(version 1)\n(allow default)\n(deny network*)\n"


class NetworkIsolation(Protocol):
    """Strategy that wraps a validated argv with real network isolation.

    `wrap(argv)` returns the argv actually passed to `subprocess.Popen` --
    e.g. prefixed with a sandboxing launcher -- or `None` if this platform
    has no available/working isolation mechanism, which the caller must
    treat as EC-2 (SANDBOX_RUNTIME_UNAVAILABLE), never as "proceed without
    isolation".
    """

    def wrap(self, argv: tuple[str, ...]) -> tuple[str, ...] | None: ...

    def cleanup(self) -> None: ...


class MacosSandboxExecIsolation:
    """Network isolation via macOS `sandbox-exec` with a deny-network profile.

    Each `wrap()` call writes a fresh, ephemeral profile file and tracks it
    on the instance so `cleanup()` can remove it after the wrapped process
    has exited -- an isolation strategy that leaks a temp file per
    invocation would defeat T2c-2's later per-run resource accounting.
    """

    def __init__(self, profile_writer: Callable[[str], Path] | None = None) -> None:
        self._profile_writer = profile_writer or _write_macos_profile
        self._last_profile_path: Path | None = None

    def wrap(self, argv: tuple[str, ...]) -> tuple[str, ...] | None:
        sandbox_exec = shutil.which("sandbox-exec")
        if sandbox_exec is None:
            return None
        profile_path = self._profile_writer(_MACOS_SANDBOX_PROFILE)
        self._last_profile_path = profile_path
        return (sandbox_exec, "-f", str(profile_path)) + tuple(argv)

    def cleanup(self) -> None:
        """Remove the profile file written by the most recent `wrap()` call."""
        if self._last_profile_path is not None:
            self._last_profile_path.unlink(missing_ok=True)
            self._last_profile_path = None


class UnavailableNetworkIsolation:
    """No proven network isolation mechanism for this platform.

    Fails closed: `wrap` always returns `None`, forcing every caller onto
    the SANDBOX_RUNTIME_UNAVAILABLE path rather than executing unisolated.
    """

    def wrap(self, argv: tuple[str, ...]) -> tuple[str, ...] | None:
        return None

    def cleanup(self) -> None:
        return None


def _write_macos_profile(profile_text: str) -> Path:
    import tempfile

    fd, path_str = tempfile.mkstemp(prefix="antares-sandbox-", suffix=".sb")
    with os.fdopen(fd, "w") as handle:
        handle.write(profile_text)
    return Path(path_str)


def resolve_network_isolation() -> NetworkIsolation:
    """Pick the platform-appropriate `NetworkIsolation` implementation.

    Only macOS (`sandbox-exec`) has a proven implementation today. Every
    other platform resolves to `UnavailableNetworkIsolation`, which fails
    every run closed to SANDBOX_RUNTIME_UNAVAILABLE until a real
    implementation (e.g. Linux network namespaces) is added -- this is a
    deliberate scope boundary, not an oversight: T2c-1 must never claim
    network isolation it cannot actually enforce.
    """
    if platform.system() == "Darwin":
        return MacosSandboxExecIsolation()
    return UnavailableNetworkIsolation()


def _stripped_environment() -> dict[str, str]:
    """Credentials-stripped environment for the sandboxed subprocess.

    No inherited variables: no API keys, no tokens, no user-specific config
    paths. Only a minimal PATH so the allowlisted read-only executables can
    be located.
    """
    return {"PATH": _MINIMAL_PATH}


def _drop_privileges() -> Callable[[], None]:
    """Build a `preexec_fn` that drops privileges in the child process.

    If the current process is not running as root, there are no privileges
    to drop and this is a no-op -- the sandbox's isolation guarantee in that
    case rests on the read-only working directory, stripped environment, and
    the enforced `NetworkIsolation` strategy, not on a uid/gid change. This
    function prevents privilege retention when already running as root; it
    does not and cannot grant isolation the host process itself lacks.
    """

    def _preexec() -> None:  # pragma: no cover - exercised only as root
        if hasattr(os, "getuid") and os.getuid() == 0:
            nogroup = os.environ.get("ANTARES_SANDBOX_GID")
            nouser = os.environ.get("ANTARES_SANDBOX_UID")
            if nogroup is not None:
                os.setgid(int(nogroup))
            if nouser is not None:
                os.setuid(int(nouser))

    return _preexec


def _runtime_unavailable(detail: str) -> TerminalState:
    return TerminalState(kind=TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE, detail=detail)


def run_sandboxed(
    argv: tuple[str, ...],
    snapshot_root: Path,
    timeout_seconds: float = DEFAULT_COMMAND_TIMEOUT_SECONDS,
    network_isolation: NetworkIsolation | None = None,
) -> TerminalState:
    """Run a validated argv in an isolated subprocess and capture its output.

    Caller contract: `argv` must already be `COMMAND_PLAN_VALID` from
    `command_policy.validate_command` -- this function does not re-validate
    the executable/option allowlist or path containment; it only isolates
    and executes.

    HP-1: the process completes within `timeout_seconds` -> returns
    SANDBOX_EXECUTION_COMPLETE with captured stdout/stderr, exit code, and
    measured elapsed time.
    EC-2: no working `NetworkIsolation` is available for this platform, or
    the subprocess cannot even be started (missing interpreter, permission
    failure, snapshot root missing) -> SANDBOX_RUNTIME_UNAVAILABLE. There is
    no fallback to unsandboxed execution on this path -- a bootstrap failure
    is reported, never silently retried outside the sandbox.
    Timeout: the process exceeds `timeout_seconds` -> T2c-1 kills the
    subprocess itself (`Popen.kill()` after the timeout fires, not left for
    a later layer to notice) and returns SANDBOX_COMMAND_TIMED_OUT with
    whatever output was captured before the kill.
    """
    if not snapshot_root.is_dir():
        return _runtime_unavailable(
            f"Snapshot root {snapshot_root!s} does not exist or is not a directory."
        )

    isolation = network_isolation or resolve_network_isolation()
    wrapped_argv = isolation.wrap(argv)
    if wrapped_argv is None:
        return _runtime_unavailable(
            "No proven network-isolation mechanism is available for this platform; "
            "refusing to execute unisolated."
        )

    started = time.monotonic()
    try:
        process = subprocess.Popen(
            list(wrapped_argv),
            cwd=str(snapshot_root),
            env=_stripped_environment(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            shell=False,
            # New session (own process group) so a timeout kill can reach
            # every descendant the wrapped command spawns (e.g. the actual
            # command process forked underneath `sandbox-exec`), not just
            # the wrapper's own PID -- an orphaned grandchild surviving
            # teardown is exactly the failure class this harness treats as
            # a security-relevant defect, not a cosmetic one.
            start_new_session=True,
            preexec_fn=_drop_privileges() if os.name == "posix" else None,
        )
    except OSError as exc:
        _cleanup_isolation(isolation)
        return _runtime_unavailable(f"Sandbox bootstrap failed: {exc}")

    try:
        stdout, stderr = process.communicate(timeout=timeout_seconds)
    except subprocess.TimeoutExpired:
        _kill_process_group(process)
        stdout, stderr = process.communicate()
        elapsed = time.monotonic() - started
        _cleanup_isolation(isolation)
        return TerminalState(
            kind=TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT,
            argv=tuple(argv),
            stdout=stdout,
            stderr=stderr,
            elapsed_seconds=elapsed,
            detail=f"Command exceeded {timeout_seconds}s per-command timeout and was killed.",
        )

    elapsed = time.monotonic() - started
    _cleanup_isolation(isolation)
    return TerminalState(
        kind=TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
        argv=tuple(argv),
        stdout=stdout,
        stderr=stderr,
        elapsed_seconds=elapsed,
        exit_code=process.returncode,
    )


def _kill_process_group(process: subprocess.Popen[str]) -> None:
    """Kill the whole process group started for `process`, not just its PID.

    Falls back to `process.kill()` if the group kill fails (e.g. the
    process already exited, or the platform does not support process
    groups) so a timeout can never leave the call site without having
    attempted termination at all.
    """
    try:
        os.killpg(os.getpgid(process.pid), signal.SIGKILL)
    except (ProcessLookupError, PermissionError, OSError):
        process.kill()


def _cleanup_isolation(isolation: NetworkIsolation) -> None:
    cleanup = getattr(isolation, "cleanup", None)
    if callable(cleanup):
        cleanup()
