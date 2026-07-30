"""Session-level command/wall-clock accounting for the Antares sandbox (T2c-2).

Split out of sandbox_budget.py's run_budgeted orchestration (T2e-pre, pure
refactor, zero intended behavior change): pure accounting, no process I/O.
"""

from __future__ import annotations

import importlib.util
import sys
import time
from dataclasses import dataclass
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

# Session-wide command budget (HP-2, EC-1).
DEFAULT_COMMAND_BUDGET = 15

# Session-wide wall-clock budget across the whole run, independent of any
# single command's own timeout.
DEFAULT_WALL_BUDGET_SECONDS = 120.0


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
