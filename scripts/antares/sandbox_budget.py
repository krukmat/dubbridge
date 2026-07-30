"""Aggregate session accounting for the Antares sandbox runner (T2c-2).

Layers on top of T2c-1's `run_sandboxed` (`scripts/antares/sandbox_runner.py`):
this module owns everything that is invisible to a single process invocation
-- the 15-command wall budget, per-command CPU/RAM/PID/output caps, and
active (not assumed) teardown verification across every T2c-1 exit path.

Scope boundary (see docs/tasks/antares-security-specialist-advisor.md § T2c-2):
T2c-1 owns single-process lifecycle (launch, isolate, per-command timeout,
kill on that timeout). T2c-2 owns session-level policy layered around calls
to that primitive -- it does not re-implement process isolation.
"""

from __future__ import annotations

import importlib.util
import os
import platform
import resource
import selectors
import signal
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

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

_SANDBOX_RUNNER_SCRIPT = Path(__file__).with_name("sandbox_runner.py")
_SANDBOX_RUNNER_SPEC = importlib.util.spec_from_file_location(
    "antares_sandbox_runner", _SANDBOX_RUNNER_SCRIPT
)
if _SANDBOX_RUNNER_SPEC is None or _SANDBOX_RUNNER_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_SANDBOX_RUNNER_SCRIPT}")
_SANDBOX_RUNNER_MOD = importlib.util.module_from_spec(_SANDBOX_RUNNER_SPEC)
sys.modules[_SANDBOX_RUNNER_SPEC.name] = _SANDBOX_RUNNER_MOD
_SANDBOX_RUNNER_SPEC.loader.exec_module(_SANDBOX_RUNNER_MOD)

resolve_network_isolation = _SANDBOX_RUNNER_MOD.resolve_network_isolation
_stripped_environment = _SANDBOX_RUNNER_MOD._stripped_environment
_drop_privileges = _SANDBOX_RUNNER_MOD._drop_privileges
_cleanup_isolation = _SANDBOX_RUNNER_MOD._cleanup_isolation
NetworkIsolation = _SANDBOX_RUNNER_MOD.NetworkIsolation

# Session-wide command budget (HP-2, EC-1).
DEFAULT_COMMAND_BUDGET = 15

# Session-wide wall-clock budget across the whole run, independent of any
# single command's own timeout.
DEFAULT_WALL_BUDGET_SECONDS = 120.0

# Per-command resource caps enforced via POSIX RLIMITs in the child.
DEFAULT_CPU_SECONDS = 10
DEFAULT_ADDRESS_SPACE_BYTES = 512 * 1024 * 1024
DEFAULT_MAX_PROCESSES = 16

# Per-command combined stdout+stderr byte cap, enforced by incremental
# polling -- never by buffering the full output and checking after the fact.
DEFAULT_OUTPUT_CAP_BYTES = 1 * 1024 * 1024

# How long teardown verification waits, after sending the kill signal, before
# concluding the process group is actually gone. Bounded so verification
# itself can never hang or block the caller indefinitely.
_TEARDOWN_GRACE_SECONDS = 1.0
_TEARDOWN_POLL_INTERVAL_SECONDS = 0.05

_READ_CHUNK_BYTES = 4096


def _resource_limits_available() -> bool:
    """Whether the POSIX RLIMIT primitives this module needs actually work.

    Mirrors T2c-1's network-isolation fail-closed precedent: if the platform
    cannot enforce every resource cap this module promises, refuse to run
    rather than execute with a silently unenforced cap.

    Darwin is unconditionally excluded. Two independent RLIMITs are
    unenforceable there for the sandbox's purposes, confirmed empirically
    during T2c-2 implementation:

    - `RLIMIT_AS` (address-space / RAM ceiling): not reliably settable at
      all -- `setrlimit(RLIMIT_AS, ...)` fails even in the parent process on
      current macOS, not just inside a `preexec_fn`.
    - `RLIMIT_NPROC` (process-count ceiling): settable, but scoped to the
      *entire UID* system-wide on BSD-derived kernels, not to the sandboxed
      command's own process tree. A cap tight enough to matter (e.g. 16)
      breaks trivial multi-process pipelines outright, because the real
      user account already runs far more than that system-wide; a cap loose
      enough not to break normal usage (e.g. 2048) does not bound anything
      the sandboxed command could actually do, since the account's ambient
      process count already approaches that range on a live desktop. There
      is no value that is both a real per-command bound and compatible with
      an ordinary shell pipeline on this platform.

    Claiming "resource limits enforced" while a promised cap is either
    unsettable or accidental is exactly the silent-degradation failure mode
    T2c-1 already rejected for network isolation, so the whole session fails
    closed on Darwin instead of enforcing a partial or fake set of caps.
    """
    if platform.system() == "Darwin":
        return False
    return (
        os.name == "posix"
        and hasattr(resource, "RLIMIT_CPU")
        and hasattr(resource, "RLIMIT_AS")
        and hasattr(resource, "RLIMIT_NPROC")
    )


def _compose_preexec(
    cpu_seconds: int,
    address_space_bytes: int,
    max_processes: int,
) -> Callable[[], None]:
    """Build the single `preexec_fn` passed to `Popen`.

    `subprocess.Popen` accepts exactly one `preexec_fn`; T2c-1's
    `_drop_privileges` and T2c-2's RLIMIT calls must therefore be composed
    into one function rather than passed separately. Ordering matters:
    `setrlimit` runs first, while the child may still hold whatever
    privileges the host process started with, so a limit that requires
    elevated privilege to raise (it never needs privilege to *lower*) is not
    silently rejected by having already dropped to an unprivileged uid.
    Privilege drop runs second and is irreversible for the remaining process
    lifetime, which is the correct order for a security boundary: tighten
    resource limits, then relinquish the privilege that could otherwise
    loosen them again.
    """
    drop_privileges = _drop_privileges()

    def _preexec() -> None:  # pragma: no cover - runs only inside the child
        resource.setrlimit(resource.RLIMIT_CPU, (cpu_seconds, cpu_seconds))
        resource.setrlimit(
            resource.RLIMIT_AS, (address_space_bytes, address_space_bytes)
        )
        resource.setrlimit(resource.RLIMIT_NPROC, (max_processes, max_processes))
        drop_privileges()

    return _preexec


@dataclass
class SessionBudget:
    """Mutable aggregate accounting for one sandbox session.

    Tracks the command counter and wall-clock deadline that a single
    `run_sandboxed` call has no visibility into. One instance is scoped to
    exactly one session (one `terminal` tool-call sequence); it is not
    reused across sessions.
    """

    command_budget: int = DEFAULT_COMMAND_BUDGET
    wall_budget_seconds: float = DEFAULT_WALL_BUDGET_SECONDS
    _commands_started: int = 0
    _session_started: float | None = None

    def _ensure_started(self) -> None:
        if self._session_started is None:
            self._session_started = time.monotonic()

    def elapsed_seconds(self) -> float:
        self._ensure_started()
        assert self._session_started is not None
        return time.monotonic() - self._session_started

    def remaining_wall_seconds(self) -> float:
        return max(0.0, self.wall_budget_seconds - self.elapsed_seconds())

    def check_preflight(self) -> TerminalState | None:
        """Guard evaluated before starting a command.

        Returns a degraded `TerminalState` if the command must not start at
        all, or `None` if the caller may proceed. This is deliberately a
        pre-flight guard, not a runtime result: the 16th command is refused
        before it is ever launched, distinct from a command that started and
        then ran out of its own per-command timeout.
        """
        self._ensure_started()
        if self._commands_started >= self.command_budget:
            return TerminalState(
                kind=TerminalStateKind.SANDBOX_BUDGET_EXHAUSTED,
                detail=(
                    f"Session command budget of {self.command_budget} already "
                    "reached; refusing to start another command."
                ),
            )
        if self.remaining_wall_seconds() <= 0:
            return TerminalState(
                kind=TerminalStateKind.SANDBOX_WALL_BUDGET_EXCEEDED,
                detail=(
                    f"Session wall-clock budget of {self.wall_budget_seconds}s "
                    "already exhausted; refusing to start another command."
                ),
            )
        return None

    def record_command_started(self) -> None:
        self._commands_started += 1


def _kill_process_group(process: "subprocess.Popen[bytes]") -> None:
    try:
        os.killpg(os.getpgid(process.pid), signal.SIGKILL)
    except (ProcessLookupError, PermissionError, OSError):
        # The group-kill can fail because the process already exited between
        # this call and the caller's last check (TOCTOU) -- process.kill()
        # can then raise the same "already gone" family of errors for the
        # same reason. That is success for this function's purpose (the
        # process is not running), not a caller-visible crash, so it is
        # caught here exactly as _verify_teardown treats the identical race.
        try:
            process.kill()
        except (ProcessLookupError, PermissionError, OSError):
            pass


def _verify_teardown(process: "subprocess.Popen[bytes]") -> bool:
    """Actively confirm the process group is gone after a kill signal.

    Polls with `os.killpg(pgid, 0)` (signal 0: existence check, sends
    nothing) for up to `_TEARDOWN_GRACE_SECONDS` rather than trusting that
    the kill syscall succeeding means the process has actually exited --
    SIGKILL delivery is not synchronous with process reclamation. Bounded by
    a fixed grace period and a cheap existence probe so verification itself
    cannot hang or reintroduce an unbounded read (the same failure class the
    output cap exists to prevent).
    """
    try:
        pgid = os.getpgid(process.pid)
    except ProcessLookupError:
        return True

    # A stale/reused pgid can make the existence probe raise PermissionError
    # instead of ProcessLookupError on macOS (e.g. the pgid was reassigned to
    # a process owned by another user) -- both mean "we can no longer confirm
    # this is our process group", which is the same as gone for this check.
    deadline = time.monotonic() + _TEARDOWN_GRACE_SECONDS
    while time.monotonic() < deadline:
        try:
            os.killpg(pgid, 0)
        except (ProcessLookupError, PermissionError):
            return True
        time.sleep(_TEARDOWN_POLL_INTERVAL_SECONDS)
    # One last probe after the loop exits: the `while` condition is checked
    # before each poll, so a process that dies in the gap between the final
    # loop iteration's probe and the deadline being crossed would otherwise
    # never get a check performed exactly at expiry. Not reachable in the
    # common case (the loop's own probes already cover it), but cheap enough
    # to keep as the deciding check rather than trust the loop's last result.
    try:
        os.killpg(pgid, 0)
    except (ProcessLookupError, PermissionError):
        return True
    return False


def _close_process_pipes(process: "subprocess.Popen[bytes]") -> None:
    """Close the stdout/stderr pipe fds `Popen` opened for `process`.

    `_read_capped` reads via `os.read` on the raw fds directly rather than
    through `Popen.communicate()`, so nothing else closes these handles.
    Safe to call after any exit path (`process.stdout`/`stderr` are `None`
    only if the caller didn't request pipes, which this module always does).
    """
    if process.stdout is not None:
        process.stdout.close()
    if process.stderr is not None:
        process.stderr.close()


def _read_capped(
    process: "subprocess.Popen[bytes]",
    output_cap_bytes: int,
    timeout_seconds: float,
) -> tuple[bytes, bytes, bool, bool]:
    """Read stdout/stderr incrementally, aborting early on cap or timeout.

    Returns `(stdout, stderr, cap_exceeded, timed_out)`. Never calls
    `process.communicate()`: that call buffers the full stream in memory
    before returning, so a process that writes unbounded output would OOM
    the supervisor before any cap check could fire -- this is the exact
    host-level DoS the phase-1 review flagged. Reads are polled with
    `selectors` so a fixed-size chunk is pulled per iteration and the byte
    total is checked after every chunk, not after the process exits.
    """
    assert process.stdout is not None
    assert process.stderr is not None

    stdout_chunks: list[bytes] = []
    stderr_chunks: list[bytes] = []
    total_bytes = 0
    cap_exceeded = False
    timed_out = False

    sel = selectors.DefaultSelector()
    sel.register(process.stdout, selectors.EVENT_READ, "stdout")
    sel.register(process.stderr, selectors.EVENT_READ, "stderr")
    open_streams = 2
    deadline = time.monotonic() + timeout_seconds

    try:
        while open_streams > 0:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                timed_out = True
                break
            for key, _ in sel.select(timeout=min(remaining, 0.1)):
                chunk = os.read(key.fileobj.fileno(), _READ_CHUNK_BYTES)
                if not chunk:
                    sel.unregister(key.fileobj)
                    open_streams -= 1
                    continue
                total_bytes += len(chunk)
                if key.data == "stdout":
                    stdout_chunks.append(chunk)
                else:
                    stderr_chunks.append(chunk)
                if total_bytes > output_cap_bytes:
                    cap_exceeded = True
                    break
            if cap_exceeded:
                break
            if process.poll() is not None and open_streams > 0:
                # The direct child has exited, but a grandchild it spawned
                # (e.g. `yes` in `yes | head`) can still hold a pipe's write
                # end open, so a plain blocking os.read here can hang
                # forever waiting for data that will never come -- exactly
                # the unbounded-read failure mode this whole function exists
                # to prevent. Drain with the same selector-timeout pattern as
                # the main loop instead of assuming EOF is imminent.
                drain_deadline = time.monotonic() + 0.1
                while open_streams > 0 and time.monotonic() < drain_deadline:
                    for key, _ in sel.select(timeout=0.02):
                        chunk = os.read(key.fileobj.fileno(), _READ_CHUNK_BYTES)
                        if not chunk:
                            sel.unregister(key.fileobj)
                            open_streams -= 1
                            continue
                        total_bytes += len(chunk)
                        if key.data == "stdout":
                            stdout_chunks.append(chunk)
                        else:
                            stderr_chunks.append(chunk)
                        if total_bytes > output_cap_bytes:
                            cap_exceeded = True
                            break
                    if cap_exceeded:
                        break
                break
    finally:
        sel.close()

    return b"".join(stdout_chunks), b"".join(stderr_chunks), cap_exceeded, timed_out


def run_budgeted(
    argv: tuple[str, ...],
    snapshot_root: Path,
    budget: SessionBudget,
    command_timeout_seconds: float = 10.0,
    output_cap_bytes: int = DEFAULT_OUTPUT_CAP_BYTES,
    cpu_seconds: int = DEFAULT_CPU_SECONDS,
    address_space_bytes: int = DEFAULT_ADDRESS_SPACE_BYTES,
    max_processes: int = DEFAULT_MAX_PROCESSES,
    network_isolation: "NetworkIsolation | None" = None,
) -> TerminalState:
    """Run one command under T2c-2's session and resource accounting.

    Caller contract: `argv` must already be `COMMAND_PLAN_VALID`, exactly as
    for T2c-1's `run_sandboxed` -- this function adds session/resource policy
    around the same isolated-execution primitive, it does not re-validate
    the command itself.

    HP-2: `budget` still has room -> the command counter is incremented, the
    process runs under composed RLIMIT + privilege-drop, and a normal
    completion returns SANDBOX_EXECUTION_COMPLETE.
    EC-1: budget already exhausted (pre-flight) -> SANDBOX_BUDGET_EXHAUSTED or
    SANDBOX_WALL_BUDGET_EXCEEDED without starting a process; output cap
    breached mid-run -> SANDBOX_OUTPUT_CAP_EXCEEDED; per-command timeout ->
    SANDBOX_COMMAND_TIMED_OUT. These are mutually exclusive, sequentially
    checked outcomes, never collapsed into one generic failure.
    EC-3: every path below guarantees `_verify_teardown` runs before
    returning whenever a process was actually started.
    """
    if not _resource_limits_available():
        return TerminalState(
            kind=TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE,
            detail=(
                "POSIX resource-limit primitives (RLIMIT_CPU/RLIMIT_AS/"
                "RLIMIT_NPROC) are unavailable on this platform; refusing to "
                "run without enforced resource caps."
            ),
        )

    preflight_state = budget.check_preflight()
    if preflight_state is not None:
        return preflight_state

    if not snapshot_root.is_dir():
        return TerminalState(
            kind=TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE,
            detail=f"Snapshot root {snapshot_root!s} does not exist or is not a directory.",
        )

    isolation = network_isolation or resolve_network_isolation()
    wrapped_argv = isolation.wrap(argv)
    if wrapped_argv is None:
        return TerminalState(
            kind=TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE,
            detail=(
                "No proven network-isolation mechanism is available for this "
                "platform; refusing to execute unisolated."
            ),
        )

    budget.record_command_started()
    started = time.monotonic()
    try:
        process = subprocess.Popen(
            list(wrapped_argv),
            cwd=str(snapshot_root),
            env=_stripped_environment(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            shell=False,
            start_new_session=True,
            preexec_fn=_compose_preexec(cpu_seconds, address_space_bytes, max_processes),
        )
    except OSError as exc:
        _cleanup_isolation(isolation)
        return TerminalState(
            kind=TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE,
            detail=f"Sandbox bootstrap failed: {exc}",
        )

    remaining_wall_seconds = budget.remaining_wall_seconds()
    # Decide the discriminator once, here, from the two inputs directly --
    # not later by comparing effective_timeout back against
    # command_timeout_seconds. The wall budget is what actually cut the
    # command off whenever it is the tighter of the two ceilings, including
    # the exact-equality case (wall budget and per-command timeout expiring
    # at the same instant): treating that as a per-command timeout would
    # misattribute a session-level condition to this one command.
    wall_budget_is_binding = remaining_wall_seconds <= command_timeout_seconds
    effective_timeout = min(command_timeout_seconds, remaining_wall_seconds)
    stdout_bytes, stderr_bytes, cap_exceeded, timed_out = _read_capped(
        process, output_cap_bytes, effective_timeout
    )

    if cap_exceeded or timed_out:
        _kill_process_group(process)
        # Verify teardown BEFORE reaping (process.wait()) -- not after. The
        # kernel will not recycle process.pid until the zombie is reaped, so
        # os.getpgid(process.pid) inside _verify_teardown is guaranteed to
        # still refer to this process (or nothing), never a newly-spawned,
        # unrelated process that reused the same pid. Reaping first would
        # reopen exactly the TOCTOU race active verification exists to close.
        teardown_confirmed = _verify_teardown(process)
        # Reap the zombie and release the pipe file descriptors. Teardown
        # verification confirms the process *group* is gone; it does not
        # reap this specific child or close the stdout/stderr pipes Popen
        # opened for it -- leaving both would leak an FD and a zombie entry
        # per killed command across a session, silently working against the
        # very resource accounting this module exists to enforce.
        process.wait()
        _close_process_pipes(process)
        elapsed = time.monotonic() - started
        _cleanup_isolation(isolation)
        if not teardown_confirmed:
            # A kill that cannot be confirmed within its bounded grace period
            # is a more urgent signal than the triggering condition (output
            # cap or timeout) -- surface it as its own outcome rather than
            # silently reporting "cap exceeded"/"timed out" as if the kill
            # were clean. This is the one case active verification exists
            # to catch, so it must be observable, not just computed and
            # discarded.
            return TerminalState(
                kind=TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED,
                argv=tuple(argv),
                stdout=stdout_bytes.decode("utf-8", errors="replace"),
                stderr=stderr_bytes.decode("utf-8", errors="replace"),
                elapsed_seconds=elapsed,
                detail=(
                    "Process group did not confirm termination within "
                    f"{_TEARDOWN_GRACE_SECONDS}s of being killed."
                ),
            )
        if cap_exceeded:
            return TerminalState(
                kind=TerminalStateKind.SANDBOX_OUTPUT_CAP_EXCEEDED,
                argv=tuple(argv),
                stdout=stdout_bytes.decode("utf-8", errors="replace"),
                stderr=stderr_bytes.decode("utf-8", errors="replace"),
                elapsed_seconds=elapsed,
                detail=f"Combined stdout+stderr exceeded {output_cap_bytes} byte cap.",
            )
        if wall_budget_is_binding:
            return TerminalState(
                kind=TerminalStateKind.SANDBOX_WALL_BUDGET_EXCEEDED,
                argv=tuple(argv),
                stdout=stdout_bytes.decode("utf-8", errors="replace"),
                stderr=stderr_bytes.decode("utf-8", errors="replace"),
                elapsed_seconds=elapsed,
                detail="Session wall-clock budget exhausted while this command was running.",
            )
        return TerminalState(
            kind=TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT,
            argv=tuple(argv),
            stdout=stdout_bytes.decode("utf-8", errors="replace"),
            stderr=stderr_bytes.decode("utf-8", errors="replace"),
            elapsed_seconds=elapsed,
            detail=f"Command exceeded {command_timeout_seconds}s per-command timeout and was killed.",
        )

    process.wait()
    elapsed = time.monotonic() - started
    _verify_teardown(process)
    _close_process_pipes(process)
    _cleanup_isolation(isolation)
    return TerminalState(
        kind=TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
        argv=tuple(argv),
        stdout=stdout_bytes.decode("utf-8", errors="replace"),
        stderr=stderr_bytes.decode("utf-8", errors="replace"),
        elapsed_seconds=elapsed,
        exit_code=process.returncode,
    )
