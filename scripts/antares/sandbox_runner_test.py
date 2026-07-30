#!/usr/bin/env python3
"""Unit tests for scripts/antares/sandbox_runner.py.

Covers the approved T2c-1 happy path HP-1 and edge case EC-2 from
docs/tasks/antares-security-specialist-advisor.md, plus the per-command
timeout kill behavior T2c-1 owns.
"""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

_MODULE_SCRIPT = Path(__file__).with_name("sandbox_runner.py")
_MODULE_SPEC = importlib.util.spec_from_file_location("antares_sandbox_runner", _MODULE_SCRIPT)
if _MODULE_SPEC is None or _MODULE_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_MODULE_SCRIPT}")
_MODULE = importlib.util.module_from_spec(_MODULE_SPEC)
sys.modules[_MODULE_SPEC.name] = _MODULE
_MODULE_SPEC.loader.exec_module(_MODULE)

run_sandboxed = _MODULE.run_sandboxed
resolve_network_isolation = _MODULE.resolve_network_isolation
UnavailableNetworkIsolation = _MODULE.UnavailableNetworkIsolation
TerminalStateKind = _MODULE.TerminalStateKind


class _AllowAllIsolation:
    """Test double: wraps argv unchanged, as if isolation were proven."""

    def wrap(self, argv: tuple[str, ...]) -> tuple[str, ...] | None:
        return argv


class SandboxRunnerTestBase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.snapshot_root = Path(self._tmp.name)
        (self.snapshot_root / "src").mkdir()
        (self.snapshot_root / "src" / "main.rs").write_text("fn main() {}\n")

    def tearDown(self) -> None:
        self._tmp.cleanup()


class RunSandboxedHappyPathTest(SandboxRunnerTestBase):
    def test_hp1_validated_command_completes_with_captured_output_and_timing(self) -> None:
        result = run_sandboxed(
            ("cat", "src/main.rs"),
            self.snapshot_root,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertTrue(result.is_success)
        self.assertEqual(result.stdout, "fn main() {}\n")
        self.assertEqual(result.exit_code, 0)
        self.assertGreaterEqual(result.elapsed_seconds, 0.0)

    def test_hp1_captures_stderr_and_nonzero_exit_code(self) -> None:
        result = run_sandboxed(
            ("cat", "src/does_not_exist.rs"),
            self.snapshot_root,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertNotEqual(result.exit_code, 0)
        self.assertNotEqual(result.stderr, "")


class RunSandboxedRuntimeUnavailableTest(SandboxRunnerTestBase):
    def test_ec2_missing_snapshot_root_is_runtime_unavailable(self) -> None:
        missing_root = self.snapshot_root / "does-not-exist"
        result = run_sandboxed(
            ("cat", "src/main.rs"), missing_root, network_isolation=_AllowAllIsolation()
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE)
        self.assertFalse(result.is_success)

    def test_ec2_unavailable_network_isolation_is_runtime_unavailable_not_unisolated_run(
        self,
    ) -> None:
        # This is the core fail-closed guarantee: when no proven network
        # isolation exists, T2c-1 must never fall through to an unisolated
        # execution success path.
        result = run_sandboxed(
            ("cat", "src/main.rs"),
            self.snapshot_root,
            network_isolation=UnavailableNetworkIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE)
        self.assertFalse(result.is_success)

    def test_ec2_bootstrap_failure_from_nonexistent_executable_is_runtime_unavailable(
        self,
    ) -> None:
        result = run_sandboxed(
            ("/nonexistent/binary-xyz",), self.snapshot_root, network_isolation=_AllowAllIsolation()
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE)


class RunSandboxedTimeoutTest(SandboxRunnerTestBase):
    def test_timeout_kills_subprocess_and_returns_timed_out_state(self) -> None:
        result = run_sandboxed(
            ("sleep", "5"),
            self.snapshot_root,
            timeout_seconds=0.2,
            network_isolation=_AllowAllIsolation(),
        )
        self.assertEqual(result.kind, TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT)
        self.assertFalse(result.is_success)
        self.assertGreaterEqual(result.elapsed_seconds, 0.2)


class ResolveNetworkIsolationTest(unittest.TestCase):
    def test_resolve_returns_a_usable_strategy_object(self) -> None:
        isolation = resolve_network_isolation()
        self.assertTrue(hasattr(isolation, "wrap"))


class MacosSandboxExecIsolationTest(unittest.TestCase):
    def _skip_unless_macos_sandbox_available(self) -> None:
        import platform
        import shutil

        if platform.system() != "Darwin" or shutil.which("sandbox-exec") is None:
            self.skipTest("sandbox-exec not available on this platform")

    def test_macos_isolation_actually_blocks_network_when_available(self) -> None:
        import subprocess

        self._skip_unless_macos_sandbox_available()
        MacosSandboxExecIsolation = _MODULE.MacosSandboxExecIsolation

        isolation = MacosSandboxExecIsolation()
        wrapped = isolation.wrap(("curl", "-s", "--max-time", "3", "https://example.com"))
        self.assertIsNotNone(wrapped)
        completed = subprocess.run(wrapped, capture_output=True, timeout=10)
        # curl exit code 6/7 = could not resolve/connect -- proof the
        # network was actually unreachable under the sandbox profile.
        self.assertIn(completed.returncode, (6, 7))

    def test_cleanup_removes_the_profile_file_written_by_wrap(self) -> None:
        self._skip_unless_macos_sandbox_available()
        MacosSandboxExecIsolation = _MODULE.MacosSandboxExecIsolation

        isolation = MacosSandboxExecIsolation()
        wrapped = isolation.wrap(("cat", "src/main.rs"))
        assert wrapped is not None
        profile_path = Path(wrapped[2])
        self.assertTrue(profile_path.exists())
        isolation.cleanup()
        self.assertFalse(profile_path.exists())

    def test_run_sandboxed_cleans_up_profile_file_after_completion(self) -> None:
        self._skip_unless_macos_sandbox_available()
        MacosSandboxExecIsolation = _MODULE.MacosSandboxExecIsolation

        with tempfile.TemporaryDirectory() as tmp:
            snapshot_root = Path(tmp)
            (snapshot_root / "main.rs").write_text("fn main() {}\n")
            isolation = MacosSandboxExecIsolation()
            run_sandboxed(("cat", "main.rs"), snapshot_root, network_isolation=isolation)
            self.assertIsNone(isolation._last_profile_path)

    def test_timeout_kills_grandchild_process_spawned_under_the_wrapper(self) -> None:
        # Reproduces the real macOS path (sandbox-exec as parent, the actual
        # command as its child) to prove a timeout kill reaches the whole
        # process group, not just the sandbox-exec wrapper PID.
        import subprocess
        import time as time_module

        self._skip_unless_macos_sandbox_available()
        MacosSandboxExecIsolation = _MODULE.MacosSandboxExecIsolation

        with tempfile.TemporaryDirectory() as tmp:
            snapshot_root = Path(tmp)
            marker = snapshot_root / "still-running"
            script = (
                f"trap 'rm -f {marker}' EXIT; "
                f"touch {marker}; sleep 5"
            )
            result = run_sandboxed(
                ("/bin/sh", "-c", script),
                snapshot_root,
                timeout_seconds=0.3,
                network_isolation=MacosSandboxExecIsolation(),
            )
            self.assertEqual(result.kind, TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT)
            # Give the killed process group a brief moment to finish exiting.
            time_module.sleep(0.3)
            still_running = subprocess.run(
                ("pgrep", "-f", "sleep 5"), capture_output=True, text=True
            )
            self.assertEqual(
                still_running.stdout.strip(),
                "",
                "a grandchild process survived the timeout kill",
            )


if __name__ == "__main__":
    unittest.main()
