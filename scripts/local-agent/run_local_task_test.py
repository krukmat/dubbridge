#!/usr/bin/env python3
"""Unit tests for run_local_task.py (mocked chat endpoint, no live model)."""

import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_local_task as rlt
gemma_local = rlt.gemma_local


def _git(repo, *args):
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    )


def _git_init_worktree(worktree):
    # T7c-b2 wires scope_check.check_scope into the finish handler, and
    # check_scope shells out to `git diff`/`git ls-files` against HEAD — every
    # worktree a test drives through finish must therefore be a real git repo
    # with at least one commit, not a bare temp directory.
    os.makedirs(worktree)
    _git(worktree, "init", "-q")
    _git(worktree, "config", "user.email", "run-local-task-test@example.test")
    _git(worktree, "config", "user.name", "Run Local Task Test")
    _git(worktree, "commit", "-q", "--allow-empty", "-m", "initial")


def _tool_call(name, arguments):
    return {
        "tool_calls": [
            {"function": {"name": name, "arguments": json.dumps(arguments)}}
        ]
    }


def _tool_call_native_object(name, arguments):
    # confirmed against a real qwen3.6:35b-a3b run: models naturally emit
    # "arguments" as a nested JSON object, not a JSON-encoded string.
    return {
        "tool_calls": [
            {"function": {"name": name, "arguments": arguments}}
        ]
    }


def _write_and_finish(path, content):
    return [
        _tool_call("write_file", {"path": path, "content": content}),
        _tool_call("finish", {}),
    ]


def _make_card(tmp_dir):
    card = {
        "task_id": "toy-1",
        "spec": "Write hello.txt containing 'hi'.",
        "acceptance_tests": ["HP-1"],
        "allowed_paths": ["hello.txt"],
    }
    path = os.path.join(tmp_dir, "card.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(card, f)
    return path


class ChatSequencer:
    def __init__(self, responses):
        self._responses = list(responses)
        self.calls = 0

    def __call__(self, messages):
        response = self._responses[self.calls]
        self.calls += 1
        return response


class HP1ToyCardCompletes(unittest.TestCase):
    def test_success_writes_diff_and_transcript(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer(_write_and_finish("hello.txt", "hi"))
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(os.path.join(worktree, "hello.txt"), encoding="utf-8") as f:
                self.assertEqual(f.read(), "hi")
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")
            self.assertEqual(transcript["task_id"], "toy-1")
            self.assertGreater(len(transcript["transcript"]), 0)


class SystemPromptIncludesToolContract(unittest.TestCase):
    def test_first_message_prepends_tool_calling_contract_to_card_spec(self):
        # regression: chat_fn used to receive only card.spec as the system
        # message, with no explanation of the {"tool_calls": [...]} format —
        # a real model given only the task description replies with plain
        # prose and every turn is bounced as malformed. The system prompt
        # must actually reach the model.
        captured = {}

        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer(_write_and_finish("hello.txt", "hi"))

            def chat_fn(messages):
                captured["messages"] = messages
                return chat(messages)

            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat_fn,
                test_runner=passing_tests,
            )

        system_message = captured["messages"][0]
        self.assertEqual(system_message["role"], "system")
        self.assertIn("tool_calls", system_message["content"])
        self.assertIn("write_file", system_message["content"])
        self.assertIn("run_command", system_message["content"])
        self.assertIn("finish", system_message["content"])
        # the card's own task description must still be present, not replaced
        self.assertIn("Write hello.txt containing", system_message["content"])


class HP2RepairThenSuccess(unittest.TestCase):
    def test_exactly_two_repair_turns_then_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = (
                _write_and_finish("hello.txt", "wrong-1")
                + _write_and_finish("hello.txt", "wrong-2")
                + _write_and_finish("hello.txt", "hi")
            )
            chat = ChatSequencer(responses)

            test_call_count = {"n": 0}

            def flaky_tests(wt):
                test_call_count["n"] += 1
                passed = test_call_count["n"] >= 3
                return {"passed": passed, "output": f"attempt {test_call_count['n']}"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=flaky_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")
            test_events = [
                e for e in transcript["transcript"] if e.get("event") == "test_result"
            ]
            self.assertEqual(len(test_events), 3)
            self.assertFalse(test_events[0]["result"]["passed"])
            self.assertFalse(test_events[1]["result"]["passed"])
            self.assertTrue(test_events[2]["result"]["passed"])


class TotalTurnBudgetExhausted(unittest.TestCase):
    # Discovered live: a model that keeps issuing valid, successful tool
    # calls (e.g. read_file repeatedly) without ever calling finish is bounded
    # by neither MAX_REPAIR_ATTEMPTS (only counts failed finish->test cycles)
    # nor MAX_MALFORMED_BOUNCES (only counts consecutive malformed calls) —
    # a real qwen3.6:35b-a3b session ran past 300 turns this way before being
    # killed manually. MAX_TOTAL_TURNS is the independent hard backstop.
    def test_unbounded_successful_read_file_calls_stop_at_turn_budget(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            with open(os.path.join(worktree, "existing.txt"), "w", encoding="utf-8") as f:
                f.write("content")
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            # Always a well-formed, always-successful read_file call — never
            # malformed, never a finish — so neither of the other two budgets
            # would ever fire on their own.
            def infinite_read_file(messages):
                return _tool_call_native_object("read_file", {"path": "existing.txt"})

            unused_tests = lambda wt: self.fail("finish is never called in this scenario")

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=infinite_read_file,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "budget_exhausted")
            self.assertEqual(transcript["reason"], "total_turns_exhausted")
            turn_events = [
                e for e in transcript["transcript"] if e.get("event") == "turn_budget_exhausted"
            ]
            self.assertEqual(len(turn_events), 1)
            self.assertEqual(turn_events[0]["total_turns"], rlt.MAX_TOTAL_TURNS)

    def test_turn_budget_is_generous_enough_for_a_normal_successful_session(self):
        # regression guard: the new budget must not be so tight that it
        # clips a normal HP-1-style session (a few real turns) before finish.
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer(_write_and_finish("hello.txt", "hi"))
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")


class EC1RepairBudgetExhausted(unittest.TestCase):
    def test_stops_after_two_failed_repairs_no_third_attempt(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = (
                _write_and_finish("hello.txt", "wrong-1")
                + _write_and_finish("hello.txt", "wrong-2")
                + _write_and_finish("hello.txt", "wrong-3")
            )
            chat = ChatSequencer(responses)
            always_fail = lambda wt: {"passed": False, "output": "still broken"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=always_fail,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "budget_exhausted")
            self.assertEqual(transcript["attempts"], rlt.MAX_REPAIR_ATTEMPTS)
            # exactly 3 test_result events: initial + 2 repairs, no 4th/3rd-repair attempt
            test_events = [
                e for e in transcript["transcript"] if e.get("event") == "test_result"
            ]
            self.assertEqual(len(test_events), 3)
            # 3 attempts (initial + 2 repairs), 2 chat turns each (write_file, finish)
            self.assertEqual(chat.calls, 6)


class EC2MalformedToolCall(unittest.TestCase):
    def test_bounced_within_budget_then_aborted_on_repeat(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            # MAX_MALFORMED_BOUNCES = 3: malformed_bounces increments to
            # 1, 2, 3 across the first 3 calls (all bounced, since the abort
            # check is `> MAX_MALFORMED_BOUNCES`), so the 4th consecutive
            # malformed call (count reaches 4, 4 > 3) is what triggers abort.
            responses = [{"tool_calls": []}] * 4
            chat = ChatSequencer(responses)
            unused_tests = lambda wt: self.fail("tests must not run on abort path")

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "aborted")
            self.assertEqual(transcript["reason"], "malformed_tool_call_repeated")
            self.assertEqual(chat.calls, 4)

    def test_recovers_after_bounces_within_budget(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            # 3 consecutive malformed calls (== budget) followed by a
            # recovery must still succeed, confirming the raised budget is
            # actually honored end-to-end, not just off-by-one at the edge.
            responses = [{"tool_calls": []}] * 3 + _write_and_finish("hello.txt", "hi")
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")

    def test_recovers_after_single_malformed_bounce(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [{"tool_calls": []}] + _write_and_finish("hello.txt", "hi")
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")

    def test_chat_fn_raising_malformed_tool_call_directly_is_bounced_not_crashed(self):
        # regression: build_live_chat_fn's chat_fn raises MalformedToolCall
        # itself (non-JSON model prose instead of a tool call) — this must
        # be bounced with a retry message like any other malformed call, not
        # escape the first try/except in run_loop uncaught (which crashed
        # main() with a traceback and no transcript, discovered when a real
        # model actually replied with prose on its first turn).
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = iter(
                [None] + _write_and_finish("hello.txt", "hi")
            )

            def chat_fn_first_call_raises(messages):
                response = next(responses)
                if response is None:
                    raise rlt.MalformedToolCall("non-JSON model response: prose reply")
                return response

            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat_fn_first_call_raises,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")
            malformed_events = [
                e for e in transcript["transcript"] if e.get("event") == "malformed_tool_call"
            ]
            self.assertEqual(len(malformed_events), 1)

    def test_malformed_bounce_budget_resets_after_a_valid_call(self):
        # regression: an isolated malformed call earlier in the session must
        # not count toward a *later*, unrelated malformed call's bounce
        # budget — only consecutive malformed calls should trigger abort.
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = (
                [{"tool_calls": []}]  # malformed (1st, isolated)
                + _write_and_finish("hello.txt", "wrong")  # valid, resets budget
                + [{"tool_calls": []}]  # malformed again (should still be allowed once)
                + _write_and_finish("hello.txt", "hi")  # valid, then finish
            )
            chat = ChatSequencer(responses)

            call_count = {"n": 0}

            def tests_pass_on_second_finish(wt):
                call_count["n"] += 1
                return {"passed": call_count["n"] >= 2, "output": f"try {call_count['n']}"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=tests_pass_on_second_finish,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")


class DefaultBoundaryIsReal(unittest.TestCase):
    """main() with no boundary= override must use the real LocalAgentBoundary,
    not NullBoundary — a path-escape attempt that NullBoundary would allow
    must still be rejected when main() is invoked exactly as the CLI does."""

    def test_main_without_explicit_boundary_rejects_path_escape(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer(_write_and_finish("../escape.txt", "bad"))
            unused_tests = lambda wt: self.fail("tests must not run after violation")

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "boundary_violation")
            self.assertFalse(
                os.path.exists(os.path.join(tmp, "escape.txt")),
                "escape write must not have landed outside the worktree",
            )


class BoundaryViolationPath(unittest.TestCase):
    def test_boundary_violation_stops_loop_immediately(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer(_write_and_finish("../escape.txt", "bad"))

            class DenyAllBoundary:
                def check_write(self, path):
                    raise rlt.BoundaryViolation(f"path escape: {path}")

                def check_command(self, argv):
                    return None

            unused_tests = lambda wt: self.fail("tests must not run after violation")

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
                boundary=DenyAllBoundary(),
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "boundary_violation")


class TransportErrorPath(unittest.TestCase):
    # D14 finding #1 (blocking): transport-level exceptions from gemma_local
    # (idle/wall timeout, truncated-response RuntimeError) must not escape
    # run_loop uncaught — every one of them must still produce a transcript.
    def test_idle_timeout_from_chat_fn_still_writes_transcript(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            def timing_out_chat(messages):
                raise gemma_local.GemmaIdleTimeout(180)

            unused_tests = lambda wt: self.fail("tests must not run on transport error")

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=timing_out_chat,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "transport_error")

    def test_truncated_response_runtime_error_still_writes_transcript(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            def truncating_chat(messages):
                raise RuntimeError("response cut by token limit; output may be truncated")

            unused_tests = lambda wt: self.fail("tests must not run on transport error")

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=truncating_chat,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "transport_error")


class NativeObjectArguments(unittest.TestCase):
    # Discovered against a real qwen3.6:35b-a3b pilot run: the model
    # consistently emitted "arguments" as a nested JSON object rather than a
    # JSON-encoded string (despite the original prompt asking for a string),
    # and every call was bounced as malformed. parse_tool_call must accept
    # both shapes.
    def test_write_file_with_native_object_arguments_succeeds(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call_native_object("write_file", {"path": "hello.txt", "content": "hi"}),
                _tool_call_native_object("finish", {}),
            ]
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(os.path.join(worktree, "hello.txt"), encoding="utf-8") as f:
                self.assertEqual(f.read(), "hi")

    def test_arguments_of_wrong_type_is_malformed_not_a_crash(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            # 4 consecutive malformed calls needed to exceed the
            # MAX_MALFORMED_BOUNCES = 3 budget (see EC2MalformedToolCall for
            # the exact off-by-one accounting).
            responses = [
                {"tool_calls": [{"function": {"name": "write_file", "arguments": 42}}]},
            ] * 4
            chat = ChatSequencer(responses)
            unused_tests = lambda wt: self.fail("tests must not run on abort path")

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "aborted")


class ConversationStateGrowsAfterToolCalls(unittest.TestCase):
    # Structural bug found live against both qwen3.6:35b-a3b and
    # gemma4:26b-a4b-it-qat: after a successful non-finish tool call, the
    # code appended to `transcript` (an internal log) but never to
    # `messages` (what's actually resent to the model) — so the model's next
    # turn saw an unchanged conversation and, having no memory of its own
    # prior action or its result, repeated the exact same tool call
    # indefinitely (both real models got stuck calling read_file on the same
    # path dozens of times). This test inspects the actual `messages` list
    # passed into chat_fn on later turns to prove it actually grows.
    def test_messages_include_assistant_turn_and_tool_result_after_read_file(self):
        captured_messages_per_call = []

        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            with open(os.path.join(worktree, "existing.txt"), "w", encoding="utf-8") as f:
                f.write("the real file content")
            # committed as a pre-existing fixture, not part of the model's own
            # diff, so T7c-b2's scope_check does not flag it as out-of-scope.
            _git(worktree, "add", "existing.txt")
            _git(worktree, "commit", "-q", "-m", "add existing fixture")
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = iter(
                [
                    _tool_call_native_object("read_file", {"path": "existing.txt"}),
                ]
                + _write_and_finish("hello.txt", "hi")
            )

            def chat_fn(messages):
                captured_messages_per_call.append(list(messages))
                return next(responses)

            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat_fn,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)

        # 3 calls: read_file, write_file, finish. Each call's `messages` must
        # be strictly longer than the previous — proving real conversation
        # growth, not a frozen/repeated list.
        self.assertEqual(len(captured_messages_per_call), 3)
        lengths = [len(m) for m in captured_messages_per_call]
        self.assertEqual(lengths, sorted(lengths))
        self.assertLess(lengths[0], lengths[1])
        self.assertLess(lengths[1], lengths[2])

        # The 2nd call (deciding what to do after read_file) must actually
        # see the real file content somewhere in its messages — not just a
        # longer list, but the *right* new information.
        second_call_content = json.dumps(captured_messages_per_call[1])
        self.assertIn("the real file content", second_call_content)


class ReadFileTool(unittest.TestCase):
    def test_read_file_returns_existing_content_from_worktree(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            with open(os.path.join(worktree, "existing.txt"), "w", encoding="utf-8") as f:
                f.write("original content")
            # committed as a pre-existing fixture, not part of the model's own
            # diff, so T7c-b2's scope_check does not flag it as out-of-scope.
            _git(worktree, "add", "existing.txt")
            _git(worktree, "commit", "-q", "-m", "add existing fixture")
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call_native_object("read_file", {"path": "existing.txt"}),
                _tool_call_native_object("finish", {}),
            ]
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            read_events = [
                e["result"] for e in transcript["transcript"]
                if e.get("event") == "tool_result" and e["result"].get("tool") == "read_file"
            ]
            self.assertEqual(len(read_events), 1)
            self.assertEqual(read_events[0]["content"], "original content")

    def test_read_file_missing_file_is_malformed_not_a_crash(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call_native_object("read_file", {"path": "does-not-exist.txt"}),
            ] + _write_and_finish("hello.txt", "hi")
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            malformed_events = [
                e for e in transcript["transcript"] if e.get("event") == "malformed_tool_call"
            ]
            self.assertEqual(len(malformed_events), 1)

    def test_read_file_path_escape_is_boundary_violation(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call_native_object("read_file", {"path": "../escape.txt"}),
            ]
            chat = ChatSequencer(responses)
            unused_tests = lambda wt: self.fail("tests must not run after violation")

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "boundary_violation")


class MissingToolArgument(unittest.TestCase):
    # D14 finding #2 (blocking): a syntactically valid tool call with a
    # missing required argument must count against the malformed-call bounce
    # budget (EC-2), not crash past it with an uncaught KeyError.
    def test_write_file_missing_path_counts_as_malformed_not_crash(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            # 4 consecutive malformed calls needed to exceed the
            # MAX_MALFORMED_BOUNCES = 3 budget (see EC2MalformedToolCall for
            # the exact off-by-one accounting).
            responses = [
                _tool_call("write_file", {"content": "no path key"})  # missing "path"
            ] * 4
            chat = ChatSequencer(responses)
            unused_tests = lambda wt: self.fail("tests must not run on abort path")

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "aborted")
            self.assertEqual(transcript["reason"], "malformed_tool_call_repeated")

    def test_write_file_missing_path_recovers_after_valid_retry(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call("write_file", {"content": "no path key"}),
            ] + _write_and_finish("hello.txt", "hi")
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")


class _AllowAnyCommandBoundary:
    """Permissive stand-in used only where the test's subject is run_command's
    subprocess mechanics (real exit code/stdout capture), not boundary policy
    — boundary.py's own allowlist is exercised separately in boundary_test.py
    and integration_test.py. Using this instead of the real default keeps
    these fixtures (echo, sh -c) independent of allowlist changes."""

    def check_write(self, path):
        return None

    def check_command(self, argv):
        return None

    def env_for_subprocess(self):
        return None


class RunCommandTimeout(unittest.TestCase):
    # Found live: a real `cargo test` (first build of a crate) ran past
    # COMMAND_TIMEOUT_SECONDS during an actual pilot run — subprocess.run's
    # TimeoutExpired escaped apply_tool_call uncaught and crashed the whole
    # benchmark process with a traceback instead of a structured, recoverable
    # tool result the model (and the runner) could act on.
    def test_command_timeout_reports_structured_failure_not_a_crash(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call_native_object("run_command", {"argv": ["sleep", "999"]}),
            ] + _write_and_finish("hello.txt", "hi")
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            with patch.object(rlt, "COMMAND_TIMEOUT_SECONDS", 0.1):
                exit_code = rlt.main(
                    ["--card", card_path, "--worktree", worktree, "--out", out_path],
                    chat_fn=chat,
                    test_runner=passing_tests,
                    boundary=_AllowAnyCommandBoundary(),
                )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")
            tool_events = [
                e["result"] for e in transcript["transcript"]
                if e.get("event") == "tool_result" and e["result"].get("tool") == "run_command"
            ]
            self.assertEqual(len(tool_events), 1)
            self.assertFalse(tool_events[0]["ok"])
            self.assertIn("timed out", tool_events[0]["stderr"])

    def test_grandchild_process_is_killed_not_orphaned(self):
        # D14 finding: subprocess.run's default timeout handling only
        # signals the direct child — a multi-process command (like the real
        # `cargo test` that triggered this bug) can leave its own children
        # running in the background after the timeout is caught. This test
        # spawns a real grandchild (sh -c "sleep 999 &" backgrounds it, so
        # the direct child exits quickly but leaves sleep running under a
        # new pid) and confirms killpg on the process group reaps it too.
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            marker = os.path.join(tmp, "grandchild.pid")
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            shell_cmd = f"sleep 999 & echo $! > {marker}; wait"
            responses = [
                _tool_call_native_object("run_command", {"argv": ["sh", "-c", shell_cmd]}),
            ] + _write_and_finish("hello.txt", "hi")
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            with patch.object(rlt, "COMMAND_TIMEOUT_SECONDS", 0.3):
                rlt.main(
                    ["--card", card_path, "--worktree", worktree, "--out", out_path],
                    chat_fn=chat,
                    test_runner=passing_tests,
                    boundary=_AllowAnyCommandBoundary(),
                )

            time.sleep(0.5)  # give the OS a moment past the kill
            with open(marker, encoding="utf-8") as f:
                grandchild_pid = int(f.read().strip())
            with self.assertRaises(ProcessLookupError):
                os.kill(grandchild_pid, 0)  # signal 0: raises iff pid is gone


class RunCommandExecutesReally(unittest.TestCase):
    # D14 finding #3 (major): run_command must actually invoke a subprocess
    # and report its real exit status/output, not fabricate {"ok": True}.
    def test_run_command_executes_and_captures_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call("run_command", {"argv": ["echo", "hello-from-command"]}),
                _tool_call("finish", {}),
            ]
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
                boundary=_AllowAnyCommandBoundary(),
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            tool_events = [
                e["result"] for e in transcript["transcript"]
                if e.get("event") == "tool_result" and e["result"].get("tool") == "run_command"
            ]
            self.assertEqual(len(tool_events), 1)
            self.assertEqual(tool_events[0]["returncode"], 0)
            self.assertIn("hello-from-command", tool_events[0]["stdout"])

    def test_run_command_reports_nonzero_exit_as_not_ok(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call("run_command", {"argv": ["sh", "-c", "exit 7"]}),
                _tool_call("finish", {}),
            ]
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
                boundary=_AllowAnyCommandBoundary(),
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            tool_events = [
                e["result"] for e in transcript["transcript"]
                if e.get("event") == "tool_result" and e["result"].get("tool") == "run_command"
            ]
            self.assertEqual(tool_events[0]["returncode"], 7)
            self.assertFalse(tool_events[0]["ok"])


class BuildLiveChatFn(unittest.TestCase):
    """Covers the gemma_local integration seam the other tests bypass by
    injecting chat_fn directly — no live network in these tests either."""

    def test_wraps_stream_chat_and_parses_json_tool_call(self):
        fake_stream_result = gemma_local.StreamChatResult(
            content=json.dumps(_tool_call("finish", {})),
            usage=gemma_local.StreamUsage(),
        )
        with patch.object(gemma_local, "ensure_model_available", return_value="qwen3.6:35b-a3b"):
            with patch.object(gemma_local, "stream_chat", return_value=fake_stream_result):
                chat_fn = rlt.build_live_chat_fn(
                    "http://localhost:11434", "qwen3.6:35b-a3b", idle_timeout=5, max_wall=30
                )
                response = chat_fn([{"role": "system", "content": "spec"}])

        self.assertEqual(response, _tool_call("finish", {}))

    def test_non_json_model_response_raises_malformed_tool_call(self):
        fake_stream_result = gemma_local.StreamChatResult(
            content="not json at all",
            usage=gemma_local.StreamUsage(),
        )
        with patch.object(gemma_local, "ensure_model_available", return_value="qwen3.6:35b-a3b"):
            with patch.object(gemma_local, "stream_chat", return_value=fake_stream_result):
                chat_fn = rlt.build_live_chat_fn(
                    "http://localhost:11434", "qwen3.6:35b-a3b", idle_timeout=5, max_wall=30
                )
                with self.assertRaises(rlt.MalformedToolCall):
                    chat_fn([{"role": "system", "content": "spec"}])


class AuditLogEmission(unittest.TestCase):
    # T6c: append_audit_log must be called for every run_loop exit path, not
    # only the success path — audit visibility must not depend on how the
    # session ended.
    def _run_and_capture_audit_record(self, tmp, chat, test_runner, boundary=None):
        worktree = os.path.join(tmp, "worktree")
        _git_init_worktree(worktree)
        card_path = _make_card(tmp)
        out_path = os.path.join(tmp, "transcript.json")

        captured = {}

        def fake_append_audit_log(record):
            captured["record"] = record

        with patch.object(gemma_local, "append_audit_log", side_effect=fake_append_audit_log):
            rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=test_runner,
                boundary=boundary,
            )
        return captured.get("record")

    def test_hp1_success_emits_audit_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            chat = ChatSequencer(_write_and_finish("hello.txt", "hi"))
            passing_tests = lambda wt: {"passed": True, "output": "ok"}
            record = self._run_and_capture_audit_record(tmp, chat, passing_tests)

        self.assertIsNotNone(record)
        self.assertEqual(record["role"], "local-implementer")
        self.assertEqual(record["outcome"], "SUCCESS")
        self.assertEqual(record["task_id"], "toy-1")
        self.assertFalse(record["escalated"])
        self.assertEqual(record["attempts"], 1)
        self.assertEqual(record["test_results"], [True])

    def test_ec1_budget_exhausted_still_emits_audit_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            responses = (
                _write_and_finish("hello.txt", "wrong-1")
                + _write_and_finish("hello.txt", "wrong-2")
                + _write_and_finish("hello.txt", "wrong-3")
            )
            chat = ChatSequencer(responses)
            always_fail = lambda wt: {"passed": False, "output": "still broken"}
            record = self._run_and_capture_audit_record(tmp, chat, always_fail)

        self.assertIsNotNone(record)
        self.assertEqual(record["outcome"], "BUDGET_EXHAUSTED")
        self.assertTrue(record["escalated"])
        self.assertEqual(record["attempts"], 3)

    def test_boundary_violation_still_emits_audit_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            chat = ChatSequencer(_write_and_finish("../escape.txt", "bad"))

            class DenyAllBoundary:
                def check_write(self, path):
                    raise rlt.BoundaryViolation(f"path escape: {path}")

                def check_command(self, argv):
                    return None

                def env_for_subprocess(self):
                    return None

            unused_tests = lambda wt: self.fail("tests must not run after violation")
            record = self._run_and_capture_audit_record(
                tmp, chat, unused_tests, boundary=DenyAllBoundary()
            )

        self.assertIsNotNone(record)
        self.assertEqual(record["outcome"], "BOUNDARY_VIOLATION")
        self.assertTrue(record["escalated"])
        self.assertEqual(record["boundary_violations"], 1)
        # finish (and its scope_check) is never reached on this exit path.
        self.assertIsNone(record["scope_check"])

    def test_transport_error_still_emits_audit_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            def timing_out_chat(messages):
                raise gemma_local.GemmaIdleTimeout(180)

            unused_tests = lambda wt: self.fail("tests must not run on transport error")
            record = self._run_and_capture_audit_record(tmp, timing_out_chat, unused_tests)

        self.assertIsNotNone(record)
        self.assertEqual(record["outcome"], "TRANSPORT_ERROR")
        self.assertTrue(record["escalated"])
        # finish (and its scope_check) is never reached on this exit path.
        self.assertIsNone(record["scope_check"])

    def test_commands_executed_are_captured_in_audit_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            responses = [
                _tool_call("run_command", {"argv": ["cargo", "test"]}),
                _tool_call("finish", {}),
            ]
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}
            record = self._run_and_capture_audit_record(tmp, chat, passing_tests)

        self.assertEqual(record["commands"], [["cargo", "test"]])


class T7B1RealBoundaryEnvStrippingEndToEnd(unittest.TestCase):
    # T7b-1 (ADR-036 corrective loop): boundary_test.py's EC3EnvironmentStripping
    # tests already prove stripped_agent_env() is correct in isolation, and one
    # bare `subprocess.run(["env"], env=stripped)` probe. Neither exercises the
    # actual run_command tool-call path through rlt.main() with the *real*
    # LocalAgentBoundary (build_default_boundary) — every existing run_command
    # test in this file uses `_AllowAnyCommandBoundary`, whose
    # env_for_subprocess() returns None (inherit caller env unchanged), the
    # opposite of what a real pilot session gets. This test closes that gap:
    # a real secret in the real parent os.environ, the real boundary, and an
    # allowlisted command (python3 -m unittest) whose own body is the adversary
    # checking for the leak, so nothing about the assertion depends on trusting
    # this test's own process to inspect the child correctly.
    def test_run_command_through_real_boundary_does_not_leak_parent_secret(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            probe_path = os.path.join(worktree, "env_leak_probe_test.py")
            with open(probe_path, "w", encoding="utf-8") as f:
                f.write(
                    "import os, unittest\n"
                    "class Probe(unittest.TestCase):\n"
                    "    def test_secret_absent(self):\n"
                    "        assert 'DUBBRIDGE_T7B1_SENTINEL' not in os.environ\n"
                )
            # committed as a pre-existing fixture, not part of the model's own
            # diff — otherwise T7c-b2's scope_check would flag it (outside
            # allowed_paths=["hello.txt"]) and fail this test's finish() with
            # out_of_scope before run_command's env-stripping is ever probed.
            _git(worktree, "add", "env_leak_probe_test.py")
            _git(worktree, "commit", "-q", "-m", "add probe fixture")
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = [
                _tool_call_native_object(
                    "run_command",
                    {"argv": ["python3", "-m", "unittest", "env_leak_probe_test.py"]},
                ),
            ] + _write_and_finish("hello.txt", "hi")
            chat = ChatSequencer(responses)
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            with patch.dict(os.environ, {"DUBBRIDGE_T7B1_SENTINEL": "leak-if-unstripped"}):
                exit_code = rlt.main(
                    ["--card", card_path, "--worktree", worktree, "--out", out_path],
                    chat_fn=chat,
                    test_runner=passing_tests,
                    # no boundary= override: exercises build_default_boundary,
                    # i.e. the real LocalAgentBoundary against this worktree —
                    # the exact path a real pilot session takes.
                )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            tool_events = [
                e["result"] for e in transcript["transcript"]
                if e.get("event") == "tool_result" and e["result"].get("tool") == "run_command"
            ]
            self.assertEqual(len(tool_events), 1)
            self.assertTrue(
                tool_events[0]["ok"],
                f"probe subprocess failed, meaning the sentinel secret leaked "
                f"into its environment: {tool_events[0]}",
            )


class SystemPromptCopyTest(unittest.TestCase):
    """T7c-b1: the prompt must not imply a fixed command allowlist."""

    def test_prompt_states_no_fixed_allowlist_and_ordinary_commands_permitted(self):
        # HP-1: prompt tells the model the worktree is disposable and
        # ordinary development commands are permitted, with the scoped diff
        # and acceptance tests determining success.
        prompt = rlt.TOOL_CALLING_SYSTEM_PROMPT
        self.assertIn("disposable", prompt)
        self.assertIn("no fixed command allowlist", prompt)
        self.assertIn("scoped diff", prompt)
        self.assertIn("operator-controlled acceptance", prompt)

    def test_prompt_contains_no_allowlist_implying_language(self):
        # EC-1: none of the historical allowlist-implying phrasings survive
        # in the prompt text (case-insensitive substring check).
        prompt_lower = rlt.TOOL_CALLING_SYSTEM_PROMPT.lower()
        for phrase in ("allowed commands", "allowlisted", "whitelist", "permitted commands"):
            self.assertNotIn(phrase, prompt_lower)


class T7cB2ScopeCheckGate(unittest.TestCase):
    """T7c-b2: finish must call scope_check.check_scope before acceptance tests."""

    def test_hp1_in_scope_diff_reaches_acceptance_tests_as_before(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)  # allowed_paths: ["hello.txt"]
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer(_write_and_finish("hello.txt", "hi"))
            passing_tests = lambda wt: {"passed": True, "output": "ok"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=passing_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")
            scope_events = [
                e for e in transcript["transcript"] if e.get("event") == "scope_check"
            ]
            self.assertEqual(len(scope_events), 1)
            self.assertTrue(scope_events[0]["in_scope"])
            self.assertEqual(scope_events[0]["offending_paths"], [])

    def test_ec1_out_of_scope_diff_fails_before_acceptance_tests(self):
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)  # allowed_paths: ["hello.txt"]
            out_path = os.path.join(tmp, "transcript.json")

            # writes a path outside allowed_paths, then finishes.
            chat = ChatSequencer(_write_and_finish("outside.txt", "escape"))
            unused_tests = lambda wt: self.fail(
                "acceptance tests must not run on an out-of-scope diff"
            )

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "out_of_scope")
            self.assertEqual(transcript["offending_paths"], ["outside.txt"])
            test_events = [
                e for e in transcript["transcript"] if e.get("event") == "test_result"
            ]
            self.assertEqual(test_events, [])

    def test_ec2_clean_finish_still_reaches_acceptance_tests(self):
        # a no-diff finish is not automatically treated as in-scope-and-skip:
        # it falls through to the existing acceptance-test policy unchanged
        # (T7c-a's has_diff=False case), same as before this task was wired.
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer([_tool_call("finish", {})])
            test_call_count = {"n": 0}

            def tests_ran(wt):
                test_call_count["n"] += 1
                return {"passed": True, "output": "no change required"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=tests_ran,
            )

            self.assertEqual(exit_code, 0)
            self.assertEqual(test_call_count["n"], 1)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")
            scope_events = [
                e for e in transcript["transcript"] if e.get("event") == "scope_check"
            ]
            self.assertFalse(scope_events[0]["has_diff"])

    def test_ec3_repair_budget_unaffected_by_scope_check_on_in_scope_diffs(self):
        # EC-3: repair-attempt counting for in-scope diffs must be identical
        # to pre-T7c-b2 behavior — the scope-check branch must not consume or
        # interfere with repair_attempt when the diff stays in scope.
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            responses = (
                _write_and_finish("hello.txt", "wrong-1")
                + _write_and_finish("hello.txt", "wrong-2")
                + _write_and_finish("hello.txt", "hi")
            )
            chat = ChatSequencer(responses)
            test_call_count = {"n": 0}

            def flaky_tests(wt):
                test_call_count["n"] += 1
                passed = test_call_count["n"] >= 3
                return {"passed": passed, "output": f"attempt {test_call_count['n']}"}

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=flaky_tests,
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "success")
            scope_events = [
                e for e in transcript["transcript"] if e.get("event") == "scope_check"
            ]
            # one scope_check per finish call (3 finishes: 2 failed repairs + success)
            self.assertEqual(len(scope_events), 3)
            self.assertTrue(all(e["in_scope"] for e in scope_events))
            test_events = [
                e for e in transcript["transcript"] if e.get("event") == "test_result"
            ]
            self.assertEqual(len(test_events), 3)

    def test_ec3_out_of_scope_does_not_consume_a_repair_attempt(self):
        # an out-of-scope finish must not count against MAX_REPAIR_ATTEMPTS —
        # it is a different, non-retryable failure class, so the loop must
        # stop immediately rather than treating it as attempt 1 of a repair
        # cycle bounded by rlt.MAX_REPAIR_ATTEMPTS.
        with tempfile.TemporaryDirectory() as tmp:
            worktree = os.path.join(tmp, "worktree")
            _git_init_worktree(worktree)
            card_path = _make_card(tmp)
            out_path = os.path.join(tmp, "transcript.json")

            chat = ChatSequencer(_write_and_finish("outside.txt", "escape"))
            unused_tests = lambda wt: self.fail(
                "acceptance tests must not run on an out-of-scope diff"
            )

            exit_code = rlt.main(
                ["--card", card_path, "--worktree", worktree, "--out", out_path],
                chat_fn=chat,
                test_runner=unused_tests,
            )

            self.assertNotEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                transcript = json.load(f)
            self.assertEqual(transcript["status"], "out_of_scope")
            self.assertNotIn("attempts", transcript)


if __name__ == "__main__":
    unittest.main(verbosity=2)
