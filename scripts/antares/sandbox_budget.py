"""Aggregate session accounting for the Antares sandbox runner (T2c-2).

Layers on top of T2c-1's `run_sandboxed` (`scripts/antares/sandbox_runner.py`):
this module owns everything that is invisible to a single process invocation
-- the 15-command wall budget, per-command CPU/RAM/PID/output caps, and
active (not assumed) teardown verification across every T2c-1 exit path.

Scope boundary (see docs/tasks/antares-security-specialist-advisor.md § T2c-2):
T2c-1 owns single-process lifecycle (launch, isolate, per-command timeout,
kill on that timeout). T2c-2 owns session-level policy layered around calls
to that primitive -- it does not re-implement process isolation.

This module is the Facade for T2c-2 (T2e-pre decomposition, pure refactor,
zero intended behavior change): `run_budgeted` composes
`sandbox_resource_limits.py` (RLIMIT/Darwin detection + composed preexec_fn),
`sandbox_session_budget.py` (command/wall-clock accounting), and
`sandbox_process_io.py` (process supervision, teardown verification, capped
incremental read). Every name those modules define that `run_budgeted` or an
existing test depends on is re-exported here as a bare module-level name --
see docs/tasks/antares-security-specialist-advisor.md § T2e-pre EC-1/EC-2 for
why this must be bare-name re-export, not qualified submodule access:
`sandbox_budget_test.py` patches several of these names directly on this
module object (`unittest.mock.patch.object(_MODULE, "_verify_teardown", ...)`
and similar), and Python resolves a bare name inside a function body against
its *defining module's* globals at call time -- `run_budgeted` must therefore
keep calling these as bare names resolved from this module's own namespace,
not as `sandbox_process_io._verify_teardown(...)` qualified access, or the
test's patches would silently stop taking effect.
"""

from __future__ import annotations

import importlib.util
import os  # re-export only: sandbox_budget_test.py patches _MODULE.os.killpg
import subprocess
import sys
import time
from pathlib import Path


def _load_sibling_module(module_name: str, filename: str):
    """Load `filename` as `module_name`, reusing an already-loaded copy from
    `sys.modules` if one exists.

    Required once a single concern is split across sibling files that must
    share one class identity for Enum/dataclass types defined elsewhere
    (e.g. `TerminalStateKind`): `importlib.util.module_from_spec` +
    `exec_module` always re-executes a file from scratch and does not
    consult `sys.modules` on its own, so two independent loads of the same
    file produce two distinct, non-`==`-comparable class objects. See
    docs/tasks/antares-security-specialist-advisor.md § T2e-pre EC-4.
    """
    if module_name in sys.modules:
        return sys.modules[module_name]
    script = Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(module_name, script)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load script spec for {script}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = mod
    spec.loader.exec_module(mod)
    return mod


_TERMINAL_STATE_MOD = _load_sibling_module("antares_terminal_state", "terminal_state.py")
TerminalState = _TERMINAL_STATE_MOD.TerminalState
TerminalStateKind = _TERMINAL_STATE_MOD.TerminalStateKind

# sandbox_session_budget.py must load (and thus resolve its own
# "antares_terminal_state" sys.modules lookup) BEFORE anything below loads
# sandbox_runner.py. sandbox_runner.py is an existing, unmodified file that
# unconditionally re-loads terminal_state.py without checking sys.modules
# first, silently overwriting the shared "antares_terminal_state" entry --
# harmless for this module's own TerminalState/TerminalStateKind names
# (already captured above by the time that happens), but it would otherwise
# race sandbox_session_budget.py's own check-first lookup into inheriting a
# *different* TerminalStateKind class object than this module's, breaking
# every `result.kind == TerminalStateKind.X` comparison in
# sandbox_budget_test.py even though both sides print identically. See
# docs/tasks/antares-security-specialist-advisor.md § T2e-pre EC-4.
_SANDBOX_SESSION_BUDGET_MOD = _load_sibling_module(
    "antares_sandbox_session_budget", "sandbox_session_budget.py"
)
DEFAULT_COMMAND_BUDGET = _SANDBOX_SESSION_BUDGET_MOD.DEFAULT_COMMAND_BUDGET
DEFAULT_WALL_BUDGET_SECONDS = _SANDBOX_SESSION_BUDGET_MOD.DEFAULT_WALL_BUDGET_SECONDS
SessionBudget = _SANDBOX_SESSION_BUDGET_MOD.SessionBudget

_SANDBOX_RUNNER_MOD = _load_sibling_module("antares_sandbox_runner", "sandbox_runner.py")
resolve_network_isolation = _SANDBOX_RUNNER_MOD.resolve_network_isolation
_stripped_environment = _SANDBOX_RUNNER_MOD._stripped_environment
_drop_privileges = _SANDBOX_RUNNER_MOD._drop_privileges
_cleanup_isolation = _SANDBOX_RUNNER_MOD._cleanup_isolation
NetworkIsolation = _SANDBOX_RUNNER_MOD.NetworkIsolation

_SANDBOX_RESOURCE_LIMITS_MOD = _load_sibling_module(
    "antares_sandbox_resource_limits", "sandbox_resource_limits.py"
)
_resource_limits_available = _SANDBOX_RESOURCE_LIMITS_MOD._resource_limits_available
_compose_preexec = _SANDBOX_RESOURCE_LIMITS_MOD._compose_preexec

# Self-check, not just documentation (mirrors the existing T2A_KINDS-style
# partition asserts in artifact_schema.py): the comment above explains why
# loading sandbox_session_budget.py before sandbox_runner.py is required,
# but a comment alone does not stop a future edit from silently reordering
# these loads and reintroducing the multi-copy TerminalStateKind bug this
# module and sandbox_session_budget.py must never hit. Fail loudly and
# immediately at import time instead of relying on a human reading the
# comment or noticing a downstream test failure days later. Deliberately
# `if ... raise` rather than `assert`: this check must still run under
# `python -O`/`PYTHONOPTIMIZE`, which strips bare `assert` statements.
if TerminalState is not _SANDBOX_SESSION_BUDGET_MOD.TerminalState:
    raise RuntimeError(
        "sandbox_budget and sandbox_session_budget resolved different "
        "TerminalState class objects -- the load-order invariant documented "
        "above (session_budget must load before sandbox_runner) was violated."
    )
if TerminalStateKind is not _SANDBOX_SESSION_BUDGET_MOD.TerminalStateKind:
    raise RuntimeError(
        "sandbox_budget and sandbox_session_budget resolved different "
        "TerminalStateKind class objects -- the load-order invariant "
        "documented above (session_budget must load before sandbox_runner) "
        "was violated."
    )

_SANDBOX_PROCESS_IO_MOD = _load_sibling_module("antares_sandbox_process_io", "sandbox_process_io.py")
_kill_process_group = _SANDBOX_PROCESS_IO_MOD._kill_process_group
_verify_teardown = _SANDBOX_PROCESS_IO_MOD._verify_teardown
_close_process_pipes = _SANDBOX_PROCESS_IO_MOD._close_process_pipes
_read_capped = _SANDBOX_PROCESS_IO_MOD._read_capped
_TEARDOWN_GRACE_SECONDS = _SANDBOX_PROCESS_IO_MOD._TEARDOWN_GRACE_SECONDS

# Per-command resource caps enforced via POSIX RLIMITs in the child.
DEFAULT_CPU_SECONDS = 10
DEFAULT_ADDRESS_SPACE_BYTES = 512 * 1024 * 1024
DEFAULT_MAX_PROCESSES = 16

# Per-command combined stdout+stderr byte cap, enforced by incremental
# polling -- never by buffering the full output and checking after the fact.
DEFAULT_OUTPUT_CAP_BYTES = 1 * 1024 * 1024


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
