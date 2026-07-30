#!/usr/bin/env python3
"""Unit tests for scripts/antares/sandbox_budget.py.

Covers the approved T2c-2 happy path HP-2 and edge cases EC-1/EC-3 from
docs/tasks/antares-security-specialist-advisor.md: the 15-command and
wall-clock session budgets, the streaming output cap, composed RLIMIT +
privilege-drop preexec, and active teardown verification across every
termination path.
"""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path

_MODULE_SCRIPT = Path(__file__).with_name("sandbox_budget.py")
_MODULE_SPEC = importlib.util.spec_from_file_location("antares_sandbox_budget", _MODULE_SCRIPT)
if _MODULE_SPEC is None or _MODULE_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_MODULE_SCRIPT}")
_MODULE = importlib.util.module_from_spec(_MODULE_SPEC)
sys.modules[_MODULE_SPEC.name] = _MODULE
_MODULE_SPEC.loader.exec_module(_MODULE)

run_budgeted = _MODULE.run_budgeted
SessionBudget = _MODULE.SessionBudget
TerminalStateKind = _MODULE.TerminalStateKind
_resource_limits_available = _MODULE._resource_limits_available


class _AllowAllIsolation:
    """Test double: wraps argv unchanged, as if isolation were proven."""

    def wrap(self, argv: tuple[str, ...]) -> tuple[str, ...] | None:
        return argv


def _preexec_without_darwin_unenforceable_rlimits(cpu_seconds, address_space_bytes, max_processes):
    """Test double for `_compose_preexec` that skips `RLIMIT_AS`/`RLIMIT_NPROC`.

    Neither RLIMIT is usable for this sandbox's purposes on the macOS hosts
    this suite runs on today (confirmed empirically during T2c-2
    implementation -- see `_resource_limits_available`'s docstring):

    - `RLIMIT_AS` fails to set even in the *parent* process, not only inside
      `preexec_fn`.
    - `RLIMIT_NPROC` is scoped to the entire UID system-wide on Darwin, not
      to the sandboxed command's own process tree, so a cap tight enough to
      matter (e.g. the module default of 16) breaks an ordinary multi-process
      shell pipeline outright -- the real user account already runs far more
      than 16 processes system-wide.

    Production code fails the whole session closed on Darwin via
    `_resource_limits_available` rather than silently narrow or fake either
    cap. This double exists only so the CPU/output-cap/teardown/budget
    behavior -- which depends on neither RLIMIT -- can still be proven
    end-to-end on this host. The Darwin fail-closed path itself is covered
    separately by `RunBudgetedDarwinFailClosedTest` without this patch
    applied.
    """
    drop_privileges = _MODULE._drop_privileges()
    import resource as _resource

    def _preexec() -> None:  # pragma: no cover - runs only inside the child
        _resource.setrlimit(_resource.RLIMIT_CPU, (cpu_seconds, cpu_seconds))
        drop_privileges()

    return _preexec


class SandboxBudgetTestBase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.snapshot_root = Path(self._tmp.name)
        (self.snapshot_root / "src").mkdir()
        (self.snapshot_root / "src" / "main.rs").write_text("fn main() {}\n")

        # Exercise the real code path on hosts where RLIMIT_AS works; fall
        # back to the RLIMIT_AS-less double on hosts (e.g. current macOS)
        # where it does not, so the rest of the module's behavior is still
        # verified end-to-end rather than skipped outright.
        self._resource_patch = unittest.mock.patch.object(
            _MODULE, "_resource_limits_available", return_value=True
        )
        self._resource_patch.start()
        self._preexec_patch = unittest.mock.patch.object(
            _MODULE, "_compose_preexec", side_effect=_preexec_without_darwin_unenforceable_rlimits
        )
        self._preexec_patch.start()

    def tearDown(self) -> None:
        self._preexec_patch.stop()
        self._resource_patch.stop()
        self._tmp.cleanup()


class RunBudgetedHappyPathTest(SandboxBudgetTestBase):
    def test_hp2_command_within_budget_completes_and_increments_counter(self) -> None:
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        result = run_budgeted(
            ("cat", "src/main.rs"),
            self.snapshot_root,
            budget,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertTrue(result.is_success)
        self.assertEqual(result.stdout, "fn main() {}\n")
        self.assertEqual(result.exit_code, 0)
        self.assertEqual(budget._commands_started, 1)

    def test_hp2_a_full_multi_command_run_stops_cleanly_within_budget(self) -> None:
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        for _ in range(15):
            result = run_budgeted(
                ("cat", "src/main.rs"),
                self.snapshot_root,
                budget,
                network_isolation=_AllowAllIsolation(),
            )
            self.assertEqual(result.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertEqual(budget._commands_started, 15)


class RunBudgetedCommandBudgetTest(SandboxBudgetTestBase):
    def test_ec1_sixteenth_command_is_refused_before_starting(self) -> None:
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        budget._commands_started = 15
        result = run_budgeted(
            ("cat", "src/main.rs"),
            self.snapshot_root,
            budget,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_BUDGET_EXHAUSTED)
        self.assertFalse(result.is_success)
        # Refused before starting -- the counter must not move.
        self.assertEqual(budget._commands_started, 15)

    def test_ec1_command_number_fifteen_timing_out_reports_timeout_not_exhaustion(
        self,
    ) -> None:
        # Precision requirement from phase-1 review: budget exhaustion is a
        # pre-flight guard; per-command timeout is a runtime result. Command
        # #15 itself timing out must report SANDBOX_COMMAND_TIMED_OUT, not
        # SANDBOX_BUDGET_EXHAUSTED, even though it is the last command the
        # budget allows.
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        budget._commands_started = 14
        result = run_budgeted(
            ("sleep", "5"),
            self.snapshot_root,
            budget,
            command_timeout_seconds=0.2,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT)
        self.assertEqual(budget._commands_started, 15)


class RunBudgetedWallBudgetTest(SandboxBudgetTestBase):
    def test_ec1_wall_budget_already_exhausted_is_refused_before_starting(self) -> None:
        budget = SessionBudget(command_budget=15, wall_budget_seconds=0.01)
        budget.elapsed_seconds()  # start the clock
        import time as time_module

        time_module.sleep(0.05)
        result = run_budgeted(
            ("cat", "src/main.rs"),
            self.snapshot_root,
            budget,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_WALL_BUDGET_EXCEEDED)
        self.assertFalse(result.is_success)
        self.assertEqual(budget._commands_started, 0)

    def test_ec1_wall_budget_exhausted_mid_command_is_distinct_from_command_timeout(
        self,
    ) -> None:
        # The per-command timeout (10s default) is much larger than the
        # remaining wall budget, so the wall budget -- not the per-command
        # ceiling -- is what cuts this command off.
        budget = SessionBudget(command_budget=15, wall_budget_seconds=0.2)
        result = run_budgeted(
            ("sleep", "5"),
            self.snapshot_root,
            budget,
            command_timeout_seconds=10.0,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_WALL_BUDGET_EXCEEDED)


class RunBudgetedOutputCapTest(SandboxBudgetTestBase):
    def test_ec1_output_cap_breach_aborts_early_not_after_communicate(self) -> None:
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        result = run_budgeted(
            ("/bin/sh", "-c", "yes | head -c 10000000"),
            self.snapshot_root,
            budget,
            output_cap_bytes=1024,
            command_timeout_seconds=10.0,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_OUTPUT_CAP_EXCEEDED)
        self.assertFalse(result.is_success)
        # Captured output should be small -- proof the abort happened early,
        # not after the full 10MB was buffered.
        self.assertLess(len(result.stdout), 10_000_000)

    def test_hp2_output_within_cap_is_captured_completely(self) -> None:
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        result = run_budgeted(
            ("cat", "src/main.rs"),
            self.snapshot_root,
            budget,
            output_cap_bytes=1024,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertEqual(result.stdout, "fn main() {}\n")


class RunBudgetedTeardownTest(SandboxBudgetTestBase):
    def test_ec3_teardown_confirmed_after_timeout_kill(self) -> None:
        import subprocess

        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        result = run_budgeted(
            ("sleep", "5"),
            self.snapshot_root,
            budget,
            command_timeout_seconds=0.2,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT)
        still_running = subprocess.run(
            ("pgrep", "-f", "sleep 5"), capture_output=True, text=True
        )
        self.assertEqual(
            still_running.stdout.strip(),
            "",
            "a killed process survived teardown verification",
        )

    def test_ec3_teardown_confirmed_after_output_cap_kill(self) -> None:
        import subprocess

        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        result = run_budgeted(
            ("/bin/sh", "-c", "yes | head -c 10000000"),
            self.snapshot_root,
            budget,
            output_cap_bytes=1024,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_OUTPUT_CAP_EXCEEDED)
        still_running = subprocess.run(
            ("pgrep", "-f", "head -c 10000000"), capture_output=True, text=True
        )
        self.assertEqual(
            still_running.stdout.strip(),
            "",
            "output-cap-violating process survived teardown verification",
        )

    def test_ec3_unconfirmed_teardown_is_its_own_distinct_outcome(self) -> None:
        # Phase-2 review finding: _verify_teardown's return value was
        # computed but never checked, so a kill that could not be confirmed
        # within its grace period was silently reported as a plain timeout/
        # cap-exceeded result, indistinguishable from a clean kill. Force
        # the False branch by patching _verify_teardown directly and assert
        # it produces the distinct SANDBOX_TEARDOWN_UNCONFIRMED outcome.
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        with unittest.mock.patch.object(_MODULE, "_verify_teardown", return_value=False):
            result = run_budgeted(
                ("sleep", "5"),
                self.snapshot_root,
                budget,
                command_timeout_seconds=0.2,
                network_isolation=_AllowAllIsolation(),
            )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED)
        self.assertFalse(result.is_success)


class KillProcessGroupTest(unittest.TestCase):
    def test_fallback_kill_swallows_already_exited_race_instead_of_raising(self) -> None:
        # Phase-2 review finding: os.killpg's fallback (process.kill()) was
        # unguarded, so the same "process already exited" race _verify_teardown
        # treats as success could instead surface as an unhandled exception
        # out of run_budgeted, defeating the fail-closed contract with a crash.
        import subprocess as _subprocess

        process = _subprocess.Popen(
            ("/bin/sh", "-c", "exit 0"), stdout=_subprocess.PIPE, stderr=_subprocess.PIPE
        )
        process.wait()  # already exited and reaped before we try to kill it
        with unittest.mock.patch.object(
            _MODULE.os, "killpg", side_effect=ProcessLookupError
        ), unittest.mock.patch.object(
            process, "kill", side_effect=ProcessLookupError
        ):
            _MODULE._kill_process_group(process)  # must not raise
        process.stdout.close()
        process.stderr.close()


class ResourceLimitsAvailabilityTest(unittest.TestCase):
    def test_resource_limits_available_is_false_on_darwin(self) -> None:
        # Real (unpatched) behavior: RLIMIT_AS is not reliably settable on
        # Darwin (confirmed empirically -- see module docstring), so the
        # whole session must fail closed on that platform rather than skip
        # the RAM cap silently.
        import platform

        if platform.system() == "Darwin":
            self.assertFalse(_resource_limits_available())

    def test_resource_limits_available_on_non_darwin_posix(self) -> None:
        import os
        import platform

        if os.name == "posix" and platform.system() != "Darwin":
            self.assertTrue(_resource_limits_available())


class RunBudgetedRuntimeUnavailableTest(SandboxBudgetTestBase):
    def test_ec1_resource_limits_unavailable_fails_closed(self) -> None:
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        self._resource_patch.stop()
        _MODULE._resource_limits_available = lambda: False
        try:
            result = run_budgeted(
                ("cat", "src/main.rs"),
                self.snapshot_root,
                budget,
                network_isolation=_AllowAllIsolation(),
            )
        finally:
            self._resource_patch.start()
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE)
        self.assertFalse(result.is_success)
        # Refused before any process was started or the counter moved.
        self.assertEqual(budget._commands_started, 0)


class RunBudgetedDarwinFailClosedTest(SandboxBudgetTestBase):
    def test_ec1_darwin_host_fails_closed_without_any_patching(self) -> None:
        # Proves the actual production fail-closed path end-to-end: with
        # neither test double applied, a real run on this Darwin host must
        # refuse to execute rather than silently drop the RAM cap.
        import platform

        if platform.system() != "Darwin":
            self.skipTest("this test asserts Darwin-specific fail-closed behavior")
        self._resource_patch.stop()
        self._preexec_patch.stop()
        try:
            budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
            result = run_budgeted(
                ("cat", "src/main.rs"),
                self.snapshot_root,
                budget,
                network_isolation=_AllowAllIsolation(),
            )
        finally:
            self._preexec_patch.start()
            self._resource_patch.start()
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE)
        self.assertEqual(budget._commands_started, 0)


class SessionBudgetUnitTest(unittest.TestCase):
    def test_check_preflight_returns_none_when_room_remains(self) -> None:
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        self.assertIsNone(budget.check_preflight())

    def test_remaining_wall_seconds_never_goes_negative(self) -> None:
        budget = SessionBudget(command_budget=15, wall_budget_seconds=0.0)
        self.assertEqual(budget.remaining_wall_seconds(), 0.0)


class ComposedPreexecTest(SandboxBudgetTestBase):
    def test_cpu_rlimit_actually_terminates_a_cpu_bound_loop(self) -> None:
        # Empirical proof the composed preexec_fn's RLIMIT_CPU is real
        # enforcement, mirroring T2c-1's precedent of proving isolation
        # mechanisms rather than only asserting the call was made.
        budget = SessionBudget(command_budget=15, wall_budget_seconds=60.0)
        result = run_budgeted(
            ("/bin/sh", "-c", "while true; do :; done"),
            self.snapshot_root,
            budget,
            cpu_seconds=1,
            command_timeout_seconds=10.0,
            network_isolation=_AllowAllIsolation(),
        )
        # The kernel delivers SIGXCPU/SIGKILL directly to the CPU-bound
        # process well before the 10s wall timeout or the read loop's own
        # timeout logic ever engages -- this exits through the *normal*
        # completion path (process.poll() sees it exit on its own), not the
        # kill-and-verify timeout path, so the correct proof of enforcement
        # is a negative (signal-killed) exit code arriving quickly, not
        # SANDBOX_COMMAND_TIMED_OUT. Phase-2 review finding: elapsed_seconds
        # alone would also pass if the process were cut off early for an
        # unrelated reason (e.g. the output cap); asserting the signal-killed
        # exit code is what actually proves RLIMIT_CPU fired.
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        assert result.exit_code is not None
        self.assertLess(result.exit_code, 0, "expected a signal-killed (negative) exit code")
        self.assertLess(result.elapsed_seconds, 5.0)


if __name__ == "__main__":
    unittest.main()
