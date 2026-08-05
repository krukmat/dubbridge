#!/usr/bin/env python3
"""Unit tests for scripts/antares/harness.py (T2e).

Covers HP-1/HP-2 and EC-1..EC-4 from
docs/tasks/antares-security-specialist-advisor.md, plus supplemental
coverage for duplicate-submission composition and the cross-module
TerminalStateKind generation landmine harness.py's `_canonical_kind` exists
to close (see harness.py's module docstring).
"""

from __future__ import annotations

import importlib.util
import os
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path

_MODULE_SCRIPT = Path(__file__).with_name("harness.py")
_MODULE_SPEC = importlib.util.spec_from_file_location("antares_harness", _MODULE_SCRIPT)
if _MODULE_SPEC is None or _MODULE_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_MODULE_SCRIPT}")
_MODULE = importlib.util.module_from_spec(_MODULE_SPEC)
sys.modules[_MODULE_SPEC.name] = _MODULE
_MODULE_SPEC.loader.exec_module(_MODULE)

# replay_fixtures.py is pure data (no TerminalStateKind/Artifact types), so a
# plain reuse of harness.py's own sibling loader is enough -- this also keeps
# the import independent of CWD/invocation mode (`python3 -m pytest
# scripts/antares/ -q` vs `python3 -m unittest scripts.antares.harness_test`
# vs direct script execution), consistent with every other cross-file
# reference in this directory.
fixtures = _MODULE._load_sibling_module("antares_replay_fixtures", "replay_fixtures.py")

# sandbox_budget.py itself is NOT referenced here at module (collection) time
# -- harness.py loads it lazily, on first real use inside a test body, and
# deliberately so (see harness.py's `_sandbox_budget_mod` comment): its own
# internal self-check against sandbox_session_budget.py's captured
# TerminalState generation is only guaranteed consistent if nothing else
# re-executes sandbox_budget.py's module body with an intervening,
# generation-changing reload in between (T2e-pre EC-4's landmine,
# concretely triggered here by sandbox_budget_test.py's own top-level loader,
# which re-executes sandbox_budget.py unconditionally rather than
# cache-checking first). `_sandbox_budget_mod()` below resolves it lazily,
# inside `setUp` (test *execution* time, always after every file's
# collection-time imports have completed), never at this module's own
# top level.
def _sandbox_budget_mod():
    return _MODULE._sandbox_budget_mod()


def _new_session_budget(**kwargs):
    return _MODULE.SessionBudget(**kwargs)


HarnessSession = _MODULE.HarnessSession
Provenance = _MODULE.Provenance
DispositionState = _MODULE.DispositionState
TerminalStateKind = _MODULE.TerminalStateKind
process_tool_call = _MODULE.process_tool_call
dispatch_tool_call = _MODULE.dispatch_tool_call
replay_session = _MODULE.replay_session
validate_artifact = _MODULE.validate_artifact
dispatch_via_cli = _MODULE.dispatch_via_cli
cli_terminal_state_to_artifact = _MODULE.cli_terminal_state_to_artifact


class _AllowAllIsolation:
    """Test double: wraps argv unchanged, as if isolation were proven.
    Mirrors sandbox_runner_test.py's/sandbox_budget_test.py's own double."""

    def wrap(self, argv: tuple[str, ...]) -> tuple[str, ...] | None:
        return argv


def _preexec_without_darwin_unenforceable_rlimits(cpu_seconds, address_space_bytes, max_processes):
    """Verbatim precedent from sandbox_budget_test.py: RLIMIT_AS/RLIMIT_NPROC
    are not usable for this sandbox's purposes on the macOS hosts this suite
    runs on; only RLIMIT_CPU + privilege drop is exercised here."""
    drop_privileges = _sandbox_budget_mod()._drop_privileges()
    import resource as _resource

    def _preexec() -> None:  # pragma: no cover - runs only inside the child
        _resource.setrlimit(_resource.RLIMIT_CPU, (cpu_seconds, cpu_seconds))
        drop_privileges()

    return _preexec


def _provenance() -> Provenance:
    return Provenance(
        model_version="fdtn-ai/antares-1b@test",
        runtime_version="scripts/antares@test",
        harness_version="antares-harness@test",
        packet_hash="sha256:" + "0" * 64,
        snapshot_hash="sha256:" + "1" * 64,
    )


class HarnessTestBase(unittest.TestCase):
    def setUp(self) -> None:
        self._snapshot_tmp = tempfile.TemporaryDirectory()
        self._trace_tmp = tempfile.TemporaryDirectory()
        self.snapshot_root = Path(self._snapshot_tmp.name)
        self.trace_storage_root = Path(self._trace_tmp.name)
        (self.snapshot_root / "src").mkdir()
        (self.snapshot_root / "src" / "main.rs").write_text("fn main() { unsafe {} }\n")

        self._resource_patch = unittest.mock.patch.object(
            _sandbox_budget_mod(), "_resource_limits_available", return_value=True
        )
        self._resource_patch.start()
        self._preexec_patch = unittest.mock.patch.object(
            _sandbox_budget_mod(),
            "_compose_preexec",
            side_effect=_preexec_without_darwin_unenforceable_rlimits,
        )
        self._preexec_patch.start()

    def tearDown(self) -> None:
        self._preexec_patch.stop()
        self._resource_patch.stop()
        self._trace_tmp.cleanup()
        self._snapshot_tmp.cleanup()

    def _session(self, **overrides) -> HarnessSession:
        kwargs = {"snapshot_root": self.snapshot_root, "network_isolation": _AllowAllIsolation()}
        kwargs.update(overrides)
        return HarnessSession(**kwargs)

    def _process(self, raw_json: str, session: HarnessSession, artifact_id: str = "a1"):
        artifact = process_tool_call(
            raw_json,
            session,
            finding_id="f1",
            artifact_id=artifact_id,
            provenance=_provenance(),
            trace_storage_root=self.trace_storage_root,
        )
        validate_artifact(artifact)  # re-assert: process_tool_call already validates internally
        self.assertEqual(artifact.disposition.state, DispositionState.NEEDS_HUMAN_REVIEW)
        return artifact


class HappyPathTest(HarnessTestBase):
    def test_hp1_terminal_then_submit_vulnerable_files_replays_deterministically(self) -> None:
        session = self._session()
        command_artifact = self._process(fixtures.HP1_TERMINAL_COMMAND, session, "a1")
        self.assertEqual(command_artifact.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertEqual(command_artifact.exit_code, 0)

        submit_artifact = self._process(fixtures.HP1_SUBMIT_VULNERABLE_FILES, session, "a2")
        self.assertEqual(submit_artifact.kind, TerminalStateKind.SUBMITTED_VULNERABLE_FILES)
        self.assertEqual(submit_artifact.candidates, ("src/main.rs",))
        self.assertIsNone(submit_artifact.trace_ref)
        self.assertEqual(submit_artifact.raw_stdout, "")
        self.assertEqual(submit_artifact.raw_stderr, "")

        # Deterministic replay: structural fields are byte-identical on a
        # second pass through a fresh session against the same fixtures.
        session2 = self._session()
        self._process(fixtures.HP1_TERMINAL_COMMAND, session2, "b1")
        submit_artifact2 = self._process(fixtures.HP1_SUBMIT_VULNERABLE_FILES, session2, "b2")
        self.assertEqual(submit_artifact.kind, submit_artifact2.kind)
        self.assertEqual(submit_artifact.candidates, submit_artifact2.candidates)

    def test_hp2_terminal_then_submit_no_vulnerability_found_is_unambiguous(self) -> None:
        session = self._session()
        self._process(fixtures.HP2_TERMINAL_COMMAND, session, "a1")
        result = self._process(fixtures.HP2_SUBMIT_NO_VULNERABILITY_FOUND, session, "a2")
        self.assertEqual(result.kind, TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND)
        self.assertEqual(result.candidates, ())


class EdgeCase1BudgetExhaustionTest(HarnessTestBase):
    def test_ec1_command_over_configured_budget_is_refused_before_starting(self) -> None:
        # A small configured command_budget exercises the exact
        # SessionBudget.check_preflight() path DEFAULT_COMMAND_BUDGET=15
        # would eventually hit, without needing 16 real subprocess spawns.
        session = self._session(budget=_new_session_budget(command_budget=2, wall_budget_seconds=60.0))
        first = self._process(fixtures.EC1_TERMINAL_COMMAND, session, "a1")
        second = self._process(fixtures.EC1_TERMINAL_COMMAND, session, "a2")
        third = self._process(fixtures.EC1_TERMINAL_COMMAND, session, "a3")
        self.assertEqual(first.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertEqual(second.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertEqual(third.kind, TerminalStateKind.SANDBOX_BUDGET_EXHAUSTED)
        self.assertEqual(third.disposition.state, DispositionState.NEEDS_HUMAN_REVIEW)
        self.assertEqual(third.budget.limit, 2.0)
        self.assertEqual(third.budget.consumed, 2.0)
        self.assertIsNotNone(third.trace_ref)


class EdgeCase2DistinctLayerFailuresTest(HarnessTestBase):
    def test_ec2_malformed_json_is_t2a_kind(self) -> None:
        artifact = self._process(fixtures.EC2_MALFORMED_JSON, self._session())
        self.assertEqual(artifact.kind, TerminalStateKind.MALFORMED_TOOL_CALL)

    def test_ec2_unsupported_tool_is_t2a_kind(self) -> None:
        artifact = self._process(fixtures.EC2_UNSUPPORTED_TOOL, self._session())
        self.assertEqual(artifact.kind, TerminalStateKind.UNSUPPORTED_TOOL_NAME)

    def test_ec2_disallowed_executable_is_t2b_kind(self) -> None:
        artifact = self._process(fixtures.EC2_DISALLOWED_EXECUTABLE, self._session())
        self.assertEqual(artifact.kind, TerminalStateKind.COMMAND_REJECTED_EXECUTABLE_NOT_ALLOWED)

    def test_ec2_sandbox_success_is_t2c1_kind_distinct_from_policy_kinds(self) -> None:
        artifact = self._process(fixtures.HP1_TERMINAL_COMMAND, self._session())
        self.assertEqual(artifact.kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertNotEqual(artifact.kind, TerminalStateKind.COMMAND_PLAN_VALID)


class EdgeCase3SandboxEscapeFixturesTest(HarnessTestBase):
    def test_ec3_shell_metacharacter_fails_closed(self) -> None:
        artifact = self._process(fixtures.EC3_SHELL_METACHARACTER, self._session())
        self.assertEqual(artifact.kind, TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX)

    def test_ec3_disallowed_executable_fails_closed(self) -> None:
        artifact = self._process(fixtures.EC3_DISALLOWED_EXECUTABLE, self._session())
        self.assertEqual(artifact.kind, TerminalStateKind.COMMAND_REJECTED_EXECUTABLE_NOT_ALLOWED)

    def test_ec3_disallowed_option_fails_closed(self) -> None:
        artifact = self._process(fixtures.EC3_DISALLOWED_OPTION, self._session())
        self.assertEqual(artifact.kind, TerminalStateKind.COMMAND_REJECTED_OPTION_NOT_ALLOWED)

    def test_ec3_path_traversal_fails_closed(self) -> None:
        artifact = self._process(fixtures.EC3_PATH_TRAVERSAL, self._session())
        self.assertEqual(artifact.kind, TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE)


class EdgeCase4PoisonedPayloadBoundedTest(HarnessTestBase):
    def _make_fifo(self) -> str:
        fifo_path = self.snapshot_root / "poisoned.fifo"
        os.mkfifo(fifo_path)
        return "poisoned.fifo"

    def test_ec4_hanging_command_resolves_to_bounded_timeout_not_a_hang(self) -> None:
        relative_fifo = self._make_fifo()
        session = self._session(command_timeout_seconds=0.2)
        artifact = self._process(fixtures.ec4_cat_command(relative_fifo), session)
        self.assertEqual(artifact.kind, TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT)
        self.assertGreaterEqual(artifact.elapsed_seconds, 0.2)
        self.assertEqual(artifact.exit_code, _MODULE._EXIT_CODE_UNAVAILABLE_SENTINEL)

    def test_ec4_wall_budget_cuts_off_a_hanging_command_distinctly(self) -> None:
        relative_fifo = self._make_fifo()
        session = self._session(
            command_timeout_seconds=10.0,
            budget=_new_session_budget(command_budget=15, wall_budget_seconds=0.2),
        )
        artifact = self._process(fixtures.ec4_cat_command(relative_fifo), session)
        self.assertEqual(artifact.kind, TerminalStateKind.SANDBOX_WALL_BUDGET_EXCEEDED)

    def test_ec4_output_flood_resolves_to_bounded_cap_not_unbounded_capture(self) -> None:
        large_file = self.snapshot_root / "large.txt"
        large_file.write_bytes(b"x" * (200 * 1024))
        session = self._session(output_cap_bytes=1024)
        artifact = self._process(fixtures.ec4_cat_command("large.txt"), session)
        self.assertEqual(artifact.kind, TerminalStateKind.SANDBOX_OUTPUT_CAP_EXCEEDED)
        self.assertLess(artifact.budget.consumed, 200 * 1024)


class SupplementalTeardownUnconfirmedTest(HarnessTestBase):
    """terminal_state_to_artifact has a dedicated SANDBOX_TEARDOWN_UNCONFIRMED
    branch (sets teardown_grace_seconds, lazily resolves
    sandbox_budget._TEARDOWN_GRACE_SECONDS) that none of the named EC-4
    fixtures reach naturally -- sandbox_budget_test.py's own precedent
    (test_ec3_unconfirmed_teardown_is_its_own_distinct_outcome) forces it via
    a patched `_verify_teardown`, since a real, confirmed kill never produces
    it. Reflection pass 1 found this branch untested through the composed
    harness; this closes that gap the same way."""

    def _make_fifo(self) -> str:
        fifo_path = self.snapshot_root / "poisoned.fifo"
        os.mkfifo(fifo_path)
        return "poisoned.fifo"

    def test_unconfirmed_teardown_carries_grace_seconds_and_validates(self) -> None:
        relative_fifo = self._make_fifo()
        session = self._session(command_timeout_seconds=0.2)
        with unittest.mock.patch.object(_sandbox_budget_mod(), "_verify_teardown", return_value=False):
            artifact = self._process(fixtures.ec4_cat_command(relative_fifo), session)
        self.assertEqual(artifact.kind, TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED)
        self.assertEqual(
            artifact.teardown_grace_seconds, _sandbox_budget_mod()._TEARDOWN_GRACE_SECONDS
        )
        self.assertIsNotNone(artifact.trace_ref)
        self.assertEqual(artifact.disposition.state, DispositionState.NEEDS_HUMAN_REVIEW)


class SupplementalDuplicateSubmissionTest(HarnessTestBase):
    def test_second_submission_in_one_session_is_refused_as_duplicate(self) -> None:
        session = self._session()
        first = self._process(fixtures.HP1_SUBMIT_VULNERABLE_FILES, session, "a1")
        second = self._process(fixtures.SUPPLEMENTAL_DUPLICATE_SUBMISSION_SECOND, session, "a2")
        self.assertEqual(first.kind, TerminalStateKind.SUBMITTED_VULNERABLE_FILES)
        self.assertEqual(second.kind, TerminalStateKind.DUPLICATE_TERMINAL_SUBMISSION)


class SupplementalCanonicalKindLandmineTest(HarnessTestBase):
    """Proves _canonical_kind actually closes the cross-module
    TerminalStateKind generation-identity gap documented in harness.py's
    module docstring, for one state from each originating layer."""

    def test_every_layer_produced_kind_validates_through_the_canonical_generation(self) -> None:
        cases = (
            (fixtures.EC2_MALFORMED_JSON, "malformed_tool_call"),
            (fixtures.EC2_DISALLOWED_EXECUTABLE, "command_rejected_executable_not_allowed"),
            (fixtures.EC2_PATH_TRAVERSAL, "path_rejected_containment_escape"),
            (fixtures.HP1_TERMINAL_COMMAND, "sandbox_execution_complete"),
        )
        for raw_json, expected_value in cases:
            with self.subTest(expected_value=expected_value):
                artifact = self._process(raw_json, self._session())
                self.assertIsInstance(artifact.kind, TerminalStateKind)
                self.assertEqual(artifact.kind.value, expected_value)
                # validate_artifact (artifact_schema's own _category_of) must
                # accept it without raising -- the real proof no generation
                # mismatch occurred (self._process already calls this, but
                # a second explicit call here documents the intent).
                validate_artifact(artifact)


class SupplementalSubmitCandidatePathTraversalTest(HarnessTestBase):
    """A submit_vulnerable_files candidate escaping the snapshot is validated
    by the harness's own check_path_containment call, a distinct code path
    from EC-3's terminal-command-operand path traversal (which goes through
    command_policy.validate_command's internal resolve_within_snapshot
    instead). Reflection pass 3 found this branch uncovered."""

    def test_escaping_candidate_is_rejected_not_silently_narrowed(self) -> None:
        artifact = self._process(fixtures.SUPPLEMENTAL_SUBMIT_CANDIDATE_PATH_TRAVERSAL, self._session())
        self.assertEqual(artifact.kind, TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE)
        self.assertEqual(artifact.candidates, ())


class SupplementalConverterDirectContractTest(HarnessTestBase):
    """Direct-call contracts on terminal_state_to_artifact/__getattr__ that
    process_tool_call's own pipeline never exercises (it always supplies a
    real session_budget, and never accesses an undefined module attribute).
    Reflection pass 3 found both branches uncovered."""

    def test_t2c2_state_without_session_budget_raises(self) -> None:
        session = self._session(budget=_new_session_budget(command_budget=0, wall_budget_seconds=60.0))
        state = dispatch_tool_call(fixtures.EC1_TERMINAL_COMMAND, session)
        self.assertEqual(state.kind.value, "sandbox_budget_exhausted")
        with self.assertRaises(ValueError):
            _MODULE.terminal_state_to_artifact(
                state,
                finding_id="f1",
                artifact_id="a1",
                provenance=_provenance(),
                trace_storage_root=self.trace_storage_root,
                session_budget=None,
            )

    def test_unrecognized_module_attribute_raises_attribute_error(self) -> None:
        with self.assertRaises(AttributeError):
            _MODULE.this_name_does_not_exist


class _UnavailableIsolation:
    """Test double: no proven isolation mechanism, matching
    sandbox_runner.UnavailableNetworkIsolation's own contract (`wrap` always
    returns None, forcing the SANDBOX_RUNTIME_UNAVAILABLE fail-closed path)
    without depending on sandbox_runner.py's own (separately-generationed)
    class."""

    def wrap(self, argv: tuple[str, ...]):
        return None


class SupplementalSandboxRuntimeUnavailableTest(HarnessTestBase):
    """SANDBOX_RUNTIME_UNAVAILABLE is the one T2C1 kind terminal_state_to_artifact
    handles with an early return (no exit_code/trace_ref) and the one kind
    dispatch_tool_call's argv-backfill exists specifically for. Reflection
    pass 3 found the composed harness never actually reaches it."""

    def test_no_network_isolation_is_runtime_unavailable_with_backfilled_argv(self) -> None:
        session = self._session(network_isolation=_UnavailableIsolation())
        artifact = self._process(fixtures.HP1_TERMINAL_COMMAND, session)
        self.assertEqual(artifact.kind, TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE)
        self.assertEqual(artifact.argv, ("cat", "src/main.rs"))
        self.assertIsNone(artifact.trace_ref)
        self.assertIsNone(artifact.exit_code)


class ReplaySessionTest(HarnessTestBase):
    def test_replay_session_produces_one_artifact_per_message_in_order(self) -> None:
        session = self._session()
        artifacts = replay_session(
            (fixtures.HP1_TERMINAL_COMMAND, fixtures.HP1_SUBMIT_VULNERABLE_FILES),
            session,
            finding_id="f1",
            provenance=_provenance(),
            trace_storage_root=self.trace_storage_root,
            artifact_id_for_index=lambda i: f"f1-r{i}",
        )
        self.assertEqual(len(artifacts), 2)
        self.assertEqual(artifacts[0].kind, TerminalStateKind.SANDBOX_EXECUTION_COMPLETE)
        self.assertEqual(artifacts[1].kind, TerminalStateKind.SUBMITTED_VULNERABLE_FILES)
        for artifact in artifacts:
            validate_artifact(artifact)


class _FakeCliProcess:
    """Stub for subprocess.Popen -- Element 3 Subtask B's task card requires
    testing against a stub, never the real antares-cli binary."""

    def __init__(self, stdout: str, stderr: str = "", returncode: int = 0, raises_timeout: bool = False):
        self._stdout = stdout
        self._stderr = stderr
        self.returncode = returncode
        self._raises_timeout = raises_timeout
        self._timed_out_once = False
        self.killed = False

    def communicate(self, input: str | None = None, timeout: float | None = None):
        if self._raises_timeout and not self._timed_out_once:
            self._timed_out_once = True
            import subprocess as _subprocess

            raise _subprocess.TimeoutExpired(cmd="antares", timeout=timeout or 0)
        return self._stdout, self._stderr

    def kill(self) -> None:
        self.killed = True


class DispatchViaCliTest(HarnessTestBase):
    """HP-1/EC-1/EC-2/EC-3 for Element 3 Subtask B's antares-cli subprocess
    dispatch path. All subprocess interaction is stubbed via
    unittest.mock.patch on harness.subprocess.Popen -- per the approved task
    card, these tests must never invoke the real antares-cli binary."""

    def _patched_which(self, resolved: str | None):
        return unittest.mock.patch.object(_MODULE.shutil, "which", return_value=resolved)

    def _patched_popen(self, fake_process: "_FakeCliProcess"):
        return unittest.mock.patch.object(_MODULE.subprocess, "Popen", return_value=fake_process)

    def test_hp1_valid_query_maps_findings_to_candidates(self) -> None:
        stdout = (
            '{"summary": {"total_findings": 1}, '
            '"findings": [{"title": "Improper Authentication", '
            '"file_path": "src/issuer.rs", "cwe_ids": ["CWE-287"], '
            '"likelihood_of_exploit": "High"}], "metadata": {}}'
        )
        with self._patched_which("/usr/local/bin/antares"), self._patched_popen(
            _FakeCliProcess(stdout=stdout, returncode=0)
        ):
            state = dispatch_via_cli({"target": ".", "cwe_ids": ["CWE-287"]}, snapshot_root=self.snapshot_root)
        self.assertEqual(state.kind, TerminalStateKind.CLI_EXECUTION_COMPLETE)
        self.assertEqual(state.candidates, ("src/issuer.rs",))
        self.assertEqual(state.argv, ("/usr/local/bin/antares", "tool", "query", "--stdin"))
        artifact = cli_terminal_state_to_artifact(
            state,
            finding_id="f1",
            artifact_id="f1-r1",
            provenance=_provenance(),
            trace_storage_root=self.trace_storage_root,
        )
        validate_artifact(artifact)
        self.assertEqual(artifact.disposition.state, DispositionState.NEEDS_HUMAN_REVIEW)

    def test_hp1_no_vulnerability_found_maps_to_empty_candidates(self) -> None:
        stdout = '{"summary": {"total_findings": 0}, "findings": [], "metadata": {}}'
        with self._patched_which("/usr/local/bin/antares"), self._patched_popen(
            _FakeCliProcess(stdout=stdout, returncode=0)
        ):
            state = dispatch_via_cli({"target": ".", "cwe_ids": ["CWE-20"]}, snapshot_root=self.snapshot_root)
        self.assertEqual(state.kind, TerminalStateKind.CLI_EXECUTION_COMPLETE)
        self.assertEqual(state.candidates, ())

    def test_hp1_operational_failure_exit_2_still_completes(self) -> None:
        # tool.py raises typer.Exit(code=2) on has_operational_failures while
        # still printing valid WorkflowResult JSON first.
        stdout = '{"summary": {"total_findings": 0}, "findings": [], "metadata": {}, "warnings": ["x"]}'
        with self._patched_which("/usr/local/bin/antares"), self._patched_popen(
            _FakeCliProcess(stdout=stdout, returncode=2)
        ):
            state = dispatch_via_cli({"target": ".", "cwe_ids": ["CWE-20"]}, snapshot_root=self.snapshot_root)
        self.assertEqual(state.kind, TerminalStateKind.CLI_EXECUTION_COMPLETE)
        self.assertEqual(state.exit_code, 2)

    def test_ec1_nonzero_exit_without_valid_json_is_execution_failed(self) -> None:
        with self._patched_which("/usr/local/bin/antares"), self._patched_popen(
            _FakeCliProcess(stdout="", stderr="fatal: model unavailable", returncode=1)
        ):
            state = dispatch_via_cli({"target": ".", "cwe_ids": ["CWE-20"]}, snapshot_root=self.snapshot_root)
        self.assertEqual(state.kind, TerminalStateKind.CLI_EXECUTION_FAILED)
        self.assertEqual(state.exit_code, 1)

    def test_ec1_timeout_is_execution_failed_and_kills_process(self) -> None:
        fake = _FakeCliProcess(stdout="", stderr="", raises_timeout=True)
        with self._patched_which("/usr/local/bin/antares"), self._patched_popen(fake):
            state = dispatch_via_cli(
                {"target": ".", "cwe_ids": ["CWE-20"]},
                snapshot_root=self.snapshot_root,
                timeout_seconds=0.01,
            )
        self.assertEqual(state.kind, TerminalStateKind.CLI_EXECUTION_FAILED)
        self.assertTrue(fake.killed)

    def test_ec1_malformed_json_stdout_is_output_malformed(self) -> None:
        with self._patched_which("/usr/local/bin/antares"), self._patched_popen(
            _FakeCliProcess(stdout="not json{{{", returncode=0)
        ):
            state = dispatch_via_cli({"target": ".", "cwe_ids": ["CWE-20"]}, snapshot_root=self.snapshot_root)
        self.assertEqual(state.kind, TerminalStateKind.CLI_OUTPUT_MALFORMED)

    def test_ec1_valid_json_missing_findings_key_is_output_malformed(self) -> None:
        with self._patched_which("/usr/local/bin/antares"), self._patched_popen(
            _FakeCliProcess(stdout='{"summary": {}}', returncode=0)
        ):
            state = dispatch_via_cli({"target": ".", "cwe_ids": ["CWE-20"]}, snapshot_root=self.snapshot_root)
        self.assertEqual(state.kind, TerminalStateKind.CLI_OUTPUT_MALFORMED)

    def test_ec2_missing_binary_fails_closed_before_any_subprocess(self) -> None:
        with self._patched_which(None), unittest.mock.patch.object(_MODULE.subprocess, "Popen") as popen:
            state = dispatch_via_cli({"target": ".", "cwe_ids": ["CWE-20"]}, snapshot_root=self.snapshot_root)
        self.assertEqual(state.kind, TerminalStateKind.CLI_BINARY_UNAVAILABLE)
        popen.assert_not_called()
        artifact = cli_terminal_state_to_artifact(
            state,
            finding_id="f1",
            artifact_id="f1-r1",
            provenance=_provenance(),
            trace_storage_root=self.trace_storage_root,
        )
        validate_artifact(artifact)
        self.assertIsNone(artifact.trace_ref)

    def test_ec3_argv_is_binary_and_subcommand_only_no_shell(self) -> None:
        stdout = '{"summary": {"total_findings": 0}, "findings": [], "metadata": {}}'
        with self._patched_which("/usr/local/bin/antares"), self._patched_popen(
            _FakeCliProcess(stdout=stdout, returncode=0)
        ) as popen:
            dispatch_via_cli(
                {"target": "/some/../path", "cwe_ids": ["CWE-20; rm -rf /"]},
                snapshot_root=self.snapshot_root,
            )
        call_kwargs = popen.call_args.kwargs
        call_args = popen.call_args.args[0]
        self.assertEqual(call_args, ["/usr/local/bin/antares", "tool", "query", "--stdin"])
        self.assertFalse(call_kwargs.get("shell", False))
        # The caller-supplied target/cwe_ids never appear in argv -- they are
        # only ever passed as stdin bytes via `communicate(input=...)`.
        for token in call_args:
            self.assertNotIn("rm -rf", token)


if __name__ == "__main__":
    unittest.main()
