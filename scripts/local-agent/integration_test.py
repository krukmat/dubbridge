#!/usr/bin/env python3
"""End-to-end integration test: run_local_task.py wired to the real
LocalAgentBoundary (boundary.py), not a test double — closes the gap where
T6a's tests only exercised NullBoundary/hand-written doubles and T6b's tests
only exercised boundary.py in isolation, so `env=boundary.env_for_subprocess()`
being wired correctly was never actually verified end-to-end."""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_local_task as rlt
import boundary as b


_audit_log_patch = None


def setUpModule():
    global _audit_log_patch
    # T7g: keep every rlt.main() call in this module off the real
    # logs/gemma-audit/*.jsonl sink unless a test deliberately overrides the
    # seam to inspect the emitted record.
    _audit_log_patch = patch.object(
        rlt.gemma_local,
        "append_audit_log",
        side_effect=lambda record, **kwargs: None,
    )
    _audit_log_patch.start()


def tearDownModule():
    global _audit_log_patch
    if _audit_log_patch is not None:
        _audit_log_patch.stop()
        _audit_log_patch = None


def _git(repo, *args):
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    )


def _git_init_worktree(worktree):
    # T7c-b2 wires scope_check.check_scope into the finish handler; any
    # worktree driven through rlt.main() to a finish call must be a real git
    # repo, since check_scope shells out to `git diff`/`git ls-files`.
    os.makedirs(worktree)
    _git(worktree, "init", "-q")
    _git(worktree, "config", "user.email", "integration-test@example.test")
    _git(worktree, "config", "user.name", "Integration Test")
    _git(worktree, "commit", "-q", "--allow-empty", "-m", "initial")


def _tool_call(name, arguments):
    return {
        "tool_calls": [
            {"function": {"name": name, "arguments": json.dumps(arguments)}}
        ]
    }


class ChatSequencer:
    def __init__(self, responses):
        self._responses = list(responses)
        self.calls = 0

    def __call__(self, messages):
        response = self._responses[self.calls]
        self.calls += 1
        return response


def _make_card(tmp_dir, allowed_paths=None):
    card = {
        "task_id": "integration-1",
        "spec": "spec",
        "acceptance_tests": [],
        "allowed_paths": allowed_paths or [],
    }
    path = os.path.join(tmp_dir, "card.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(card, f)
    return path


class RealBoundaryWiredIntoRunner(unittest.TestCase):
    def test_run_command_wires_stripped_env_from_real_boundary_into_subprocess(self):
        # Verifies the actual integration gap this test class exists to
        # close: apply_tool_call (via _run_command_with_timeout) must launch
        # the subprocess with env=boundary.env_for_subprocess(), not the
        # inherited full environment. Asserted by inspecting the real
        # subprocess.Popen call arguments (not by hoping a shell command
        # prints env back out, which depends on allowlisted commands
        # supporting that at all).
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call("run_command", {"argv": ["cargo", "check"]}),
                _tool_call("finish", {}),
            ]
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            real_boundary = b.LocalAgentBoundary(worktree)
            captured_env = {}

            class FakeProcess:
                returncode = 0

                def communicate(self, timeout=None):
                    return "", ""

            real_popen = subprocess.Popen

            def fake_popen(*args, **kwargs):
                # T7c-b2: finish now calls scope_check.check_scope, which
                # shells out to real `git` subprocesses via subprocess.run ->
                # Popen. Only run_local_task's own run_command Popen call
                # (identifiable by its argv/cwd/env kwarg shape) is faked
                # here; anything else (git) must go through the real Popen or
                # scope_check breaks.
                if kwargs.get("env") is not None or "start_new_session" in kwargs:
                    captured_env["value"] = kwargs.get("env")
                    return FakeProcess()
                return real_popen(*args, **kwargs)

            with patch.object(rlt.subprocess, "Popen", side_effect=fake_popen):
                exit_code = rlt.main(
                    ["--card", card_path, "--worktree", worktree, "--out", out_path],
                    chat_fn=chat,
                    test_runner=passing_tests,
                    boundary=real_boundary,
                )

            self.assertEqual(exit_code, 0)
            self.assertIsNotNone(captured_env["value"])
            self.assertNotIn("HOME", captured_env["value"])
            self.assertIn("PATH", captured_env["value"])

    def test_path_escape_via_real_boundary_aborts_before_write(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            os.makedirs(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer([_tool_call("write_file", {"path": "../escape.txt", "content": "x"})])
            unused_tests = lambda wt: self.fail("must not run after boundary violation")
            real_boundary = b.LocalAgentBoundary(worktree)

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
                boundary=real_boundary,
            )

            self.assertNotEqual(exit_code, 0)
            self.assertFalse(os.path.exists(os.path.join(tmp, "escape.txt")))
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "boundary_violation")

    def test_previously_allowlist_rejected_command_via_real_boundary_now_reaches_execution(self):
        # T7b-3: check_command no longer allowlists by executable/argument
        # vocabulary — a command like `curl` (previously rejected because it
        # wasn't on ALLOWED_COMMAND_PREFIXES, not because it was denylisted)
        # must now reach real execution via run_command, not abort with
        # boundary_violation. `git push` is a different case — see
        # test_denylisted_command_via_real_boundary_still_aborts below — the
        # short DENIED_COMMAND_PREFIXES denylist retained as defense-in-depth
        # still rejects it. Popen is faked the same way
        # test_run_command_wires_stripped_env_... fakes it above, so no real
        # network call is made.
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            curl_argv = ["curl", "https://example.com"]
            responses = [
                _tool_call("run_command", {"argv": curl_argv}),
                _tool_call("finish", {}),
            ]
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}
            real_boundary = b.LocalAgentBoundary(worktree)

            class FakeProcess:
                returncode = 0

                def communicate(self, timeout=None):
                    return "", ""

            real_popen = subprocess.Popen
            captured_calls = []

            def fake_popen(*args, **kwargs):
                if kwargs.get("env") is not None or "start_new_session" in kwargs:
                    captured_calls.append((args, kwargs))
                    return FakeProcess()
                return real_popen(*args, **kwargs)

            with patch.object(rlt.subprocess, "Popen", side_effect=fake_popen):
                exit_code = rlt.main(
                    ["--card", card_path, "--worktree", worktree, "--out", out_path],
                    chat_fn=chat,
                    test_runner=passing_tests,
                    boundary=real_boundary,
                )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                result = json.load(f)
            self.assertNotEqual(result["status"], "boundary_violation")
            run_command_events = [
                e for e in result["transcript"]
                if e.get("event") == "tool_result"
                and e["result"].get("tool") == "run_command"
            ]
            self.assertEqual(len(run_command_events), 1)
            self.assertEqual(run_command_events[0]["result"]["returncode"], 0)

            # Codex peer review finding: prove the exact previously-rejected
            # argv actually reached subprocess.Popen with the worktree cwd
            # and the stripped (credential-free) environment, not just that
            # check_command allowed it and the runner reported success.
            self.assertEqual(len(captured_calls), 1)
            call_args, call_kwargs = captured_calls[0]
            self.assertEqual(call_args[0], curl_argv)
            self.assertEqual(call_kwargs["cwd"], worktree)
            self.assertTrue(call_kwargs.get("start_new_session"))
            self.assertNotIn("HOME", call_kwargs["env"])
            self.assertIn("PATH", call_kwargs["env"])

    def test_denylisted_command_via_real_boundary_still_aborts(self):
        # Reintroduced denylist (post pre-push Gemma Reviewer finding): the
        # three highest-severity commands (git push, docker, rm -rf) remain
        # denylisted even though the broader allowlist is gone. This must
        # still abort with boundary_violation before run_command executes.
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            os.makedirs(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer([_tool_call("run_command", {"argv": ["git", "push"]})])
            unused_tests = lambda wt: self.fail("must not run after boundary violation")
            real_boundary = b.LocalAgentBoundary(worktree)

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
                boundary=real_boundary,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "boundary_violation")


class AuditRecordScopeCheckAggregation(unittest.TestCase):
    # T7c-b3: build_audit_record must surface the scope_check outcome that
    # T7c-b2 already wires into the finish handler. These drive rlt.main()
    # end-to-end (real boundary, real git-backed scope_check) rather than
    # constructing a transcript by hand, so the wiring between run_loop's
    # emitted event and build_audit_record's aggregation is actually proven,
    # not just each side's own unit tests in isolation.
    def test_in_scope_finish_records_passing_scope_check_in_audit_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp, allowed_paths=["src"])
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call("write_file", {"path": "src/main.rs", "content": "fn main() {}"}),
                _tool_call("finish", {}),
            ]
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}
            real_boundary = b.LocalAgentBoundary(worktree)

            captured = {}

            def fake_append_audit_log(record, **kwargs):
                captured["record"] = record

            with patch.object(rlt.gemma_local, "append_audit_log", side_effect=fake_append_audit_log):
                exit_code = rlt.main(
                    ["--card", card_path, "--worktree", worktree, "--out", out_path],
                    chat_fn=chat,
                    test_runner=passing_tests,
                    boundary=real_boundary,
                )

            self.assertEqual(exit_code, 0)
            record = captured["record"]
            self.assertEqual(record["outcome"], "SUCCESS")
            self.assertEqual(
                record["scope_check"],
                {"in_scope": True, "offending_paths": []},
            )
            # Superset check: every field build_audit_record produced before
            # this task still exists, unchanged in kind.
            for field in (
                "ts", "role", "outcome", "model", "task_id", "rri", "band",
                "attempts", "commands", "test_results", "boundary_violations",
                "escalated", "elapsed_s",
            ):
                self.assertIn(field, record)

    def test_out_of_scope_finish_records_offending_paths_in_audit_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp, allowed_paths=["src"])
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call("write_file", {"path": "outside.txt", "content": "not allowed"}),
                _tool_call("finish", {}),
            ]
            chat = ChatSequencer(responses)
            unused_tests = lambda wt: self.fail("must not run after an out-of-scope finish")
            real_boundary = b.LocalAgentBoundary(worktree)

            captured = {}

            def fake_append_audit_log(record, **kwargs):
                captured["record"] = record

            with patch.object(rlt.gemma_local, "append_audit_log", side_effect=fake_append_audit_log):
                exit_code = rlt.main(
                    ["--card", card_path, "--worktree", worktree, "--out", out_path],
                    chat_fn=chat,
                    test_runner=unused_tests,
                    boundary=real_boundary,
                )

            self.assertNotEqual(exit_code, 0)
            record = captured["record"]
            self.assertEqual(record["outcome"], "OUT_OF_SCOPE")
            self.assertEqual(
                record["scope_check"],
                {"in_scope": False, "offending_paths": ["outside.txt"]},
            )


class TOCTOUWriteRaceAgainstRealOpen(unittest.TestCase):
    # D14 finding (high severity, CONFIRMED by reproduction): check_write()
    # resolves and validates the path, but the actual write in
    # run_local_task.apply_tool_call previously used a plain open(), which
    # follows a symlink unconditionally — a swap between check_write's
    # realpath() resolution and open() could still escape the jail. This
    # exercises apply_tool_call directly (not check_write in isolation) with
    # a symlink swapped in immediately after boundary construction, proving
    # the fix (O_NOFOLLOW at the actual open) rejects it at write time.
    def test_symlink_pointing_outside_worktree_is_rejected_at_open_time(self):
        with tempfile.TemporaryDirectory() as outside:
            with tempfile.TemporaryDirectory() as tmp:
                worktree = os.path.join(tmp, "worktree")
                os.makedirs(worktree)

                outside_target = os.path.join(outside, "secret.txt")
                with open(outside_target, "w", encoding="utf-8") as f:
                    f.write("SECRET DATA")

                link_path = os.path.join(worktree, "link")
                os.symlink(outside_target, link_path)

                real_boundary = b.LocalAgentBoundary(worktree)
                call = rlt.ToolCall("write_file", {"path": "link", "content": "PWNED"})

                with self.assertRaises(rlt.BoundaryViolation):
                    rlt.apply_tool_call(call, worktree, real_boundary)

                with open(outside_target, encoding="utf-8") as f:
                    self.assertEqual(f.read(), "SECRET DATA")

    def test_symlink_swapped_between_check_and_open_is_rejected(self):
        # Simulates the actual race: check_write validates a symlink that,
        # at that instant, points inside the worktree; before open() runs,
        # the link is swapped to point outside. O_NOFOLLOW at open() must
        # still reject it rather than following the swapped target.
        with tempfile.TemporaryDirectory() as outside:
            with tempfile.TemporaryDirectory() as tmp:
                worktree = os.path.join(tmp, "worktree")
                os.makedirs(worktree)

                inside_target = os.path.join(worktree, "inside.txt")
                with open(inside_target, "w", encoding="utf-8") as f:
                    f.write("inside")

                link_path = os.path.join(worktree, "link")
                os.symlink(inside_target, link_path)

                real_boundary = b.LocalAgentBoundary(worktree)
                outside_target = os.path.join(outside, "secret.txt")
                with open(outside_target, "w", encoding="utf-8") as f:
                    f.write("SECRET DATA")

                original_check_write = real_boundary.check_write

                def check_write_then_swap(path):
                    original_check_write(path)
                    os.remove(link_path)
                    os.symlink(outside_target, link_path)

                real_boundary.check_write = check_write_then_swap
                call = rlt.ToolCall("write_file", {"path": "link", "content": "PWNED"})

                with self.assertRaises(rlt.BoundaryViolation):
                    rlt.apply_tool_call(call, worktree, real_boundary)

                with open(outside_target, encoding="utf-8") as f:
                    self.assertEqual(f.read(), "SECRET DATA")


if __name__ == "__main__":
    unittest.main(verbosity=2)
