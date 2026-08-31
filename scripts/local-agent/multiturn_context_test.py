#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import sys
import unittest
from types import SimpleNamespace
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.dirname(HERE))

import session_loop  # noqa: E402
from diagnostics import summarize_failure  # noqa: E402
from working_history import compact_assistant_action, compact_tool_result  # noqa: E402


class Card:
    task_id = "S-M2-T"
    spec = "Edit src/a.rs"
    allowed_paths = ["src/a.rs"]
    acceptance_tests = ["cargo test -p a"]


class Limits:
    max_total_turns = 6
    max_repair_attempts = 2


class FakeBoundary:
    pass


class FakeFileTools:
    def __init__(self):
        self.current = "fn a() {}\n"
        self.edited_paths = ("src/a.rs",)

    def handle(self, call):
        if call.name == "write_file":
            self.current = call.arguments["content"]
            return {"tool": "write_file", "ok": True, "path": "src/a.rs", "created": False}
        if call.name == "apply_patch":
            return {"tool": "apply_patch", "ok": True, "path": "src/a.rs"}
        return None

    def read_checked(self, path):
        return self.current

    def preload_context(self, allowed_paths):
        return [{"path": "src/a.rs", "missing": False, "content": self.current}]


class FakeProvider:
    def __init__(self):
        self.refresh_calls = []

    def render_initial(self):
        return "SOURCE_INITIAL"

    def render_refresh(self, reason, hints=None):
        self.refresh_calls.append((reason, hints or {}))
        return "SOURCE_REFRESHED"

    def manifest(self):
        return None


def tool_response(name, arguments=None):
    return {
        "tool_calls": [
            {"function": {"name": name, "arguments": arguments or {}}}
        ]
    }


class MultiTurnContextTests(unittest.TestCase):
    def _run(self, responses, test_runner, *, provider=None, file_tools=None):
        snapshots = []
        iterator = iter(responses)

        def chat_fn(messages):
            snapshots.append(json.loads(json.dumps(messages)))
            return next(iterator)

        scope = SimpleNamespace(in_scope=True, offending_paths=[], has_diff=True)
        with mock.patch.object(session_loop.scope_check, "check_scope", return_value=scope):
            result = session_loop.run_loop(
                Card(),
                chat_fn,
                test_runner,
                ".",
                FakeBoundary(),
                file_tools or FakeFileTools(),
                limits=Limits(),
                context_provider=provider or FakeProvider(),
                resolve_effective_limits=lambda _card: Limits(),
                max_malformed_bounces=3,
                tool_calling_system_prompt="SYSTEM {MAX_TOTAL_TURNS}",
            )
        return result, snapshots

    def test_large_write_body_is_not_replayed_as_working_history(self):
        marker = "UNIQUE_GENERATED_BODY_" * 500
        response = tool_response(
            "write_file", {"path": "src/a.rs", "content": marker}
        )
        result, snapshots = self._run(
            [response, tool_response("finish")],
            lambda _root: {"passed": True, "output": "", "commands": []},
        )
        self.assertEqual(result["status"], "success")
        second_turn = json.dumps(snapshots[1])
        self.assertNotIn(marker, second_turn)
        self.assertIn("ACTION: write_file", second_turn)
        self.assertIn("CURRENT_SOURCE_SHA256", second_turn)
        # The audit transcript remains lossless and therefore still contains
        # the original raw model tool payload.
        self.assertIn(marker, json.dumps(result["transcript"]))

    def test_repair_replaces_source_snapshot_and_uses_compact_diagnostic(self):
        provider = FakeProvider()
        calls = {"count": 0}
        raw_failure = (
            "noise\n" * 300
            + "error[E0308]: mismatched types\n"
            + " --> src/a.rs:12:5\n"
            + "test tests::repairs_context ... FAILED\n"
            + "noise-tail\n" * 300
        )

        def test_runner(_root):
            calls["count"] += 1
            if calls["count"] == 1:
                return {
                    "passed": False,
                    "output": raw_failure,
                    "commands": [
                        {
                            "ok": False,
                            "argv": ["cargo", "test", "-p", "a"],
                            "returncode": 101,
                        }
                    ],
                }
            return {"passed": True, "output": "ok", "commands": []}

        result, snapshots = self._run(
            [tool_response("finish"), tool_response("finish")],
            test_runner,
            provider=provider,
        )
        self.assertEqual(result["status"], "success")
        second = snapshots[1]
        system = second[0]["content"]
        self.assertIn("SOURCE_REFRESHED", system)
        self.assertNotIn("SOURCE_INITIAL", system)
        self.assertEqual(system.count("Authorized source context:"), 1)
        repair_message = second[-1]["content"]
        self.assertIn("error[E0308]", repair_message)
        self.assertIn("src/a.rs:12:5", repair_message)
        self.assertLess(len(repair_message), 7000)
        self.assertEqual(provider.refresh_calls[0][0], "acceptance_failure")
        hints = provider.refresh_calls[0][1]
        self.assertIn("src/a.rs", hints["edited_paths"])
        self.assertIn("error[E0308]", hints["diagnostic_summary"])
        # Raw failure remains in the audit transcript even though the repair
        # message contains only the deterministic summary.
        self.assertIn(raw_failure, json.dumps(result["transcript"]))

    def test_diagnostics_preserve_signal_and_bound_noise(self):
        result = {
            "passed": False,
            "output": (
                "noise\n" * 1000
                + "thread 'x' panicked at src/a.rs:9:2\n"
                + "assertion failed: expected 1 actual 2\n"
                + "noise\n" * 1000
            ),
            "commands": [
                {"ok": False, "argv": ["cargo", "test"], "returncode": 101}
            ],
        }
        summary = summarize_failure(result, "acceptance")
        self.assertIn("COMMAND: cargo test", summary)
        self.assertIn("RETURN_CODE: 101", summary)
        self.assertIn("panicked at src/a.rs:9:2", summary)
        self.assertIn("expected 1 actual 2", summary)
        self.assertLess(len(summary), 6500)

    def test_compact_helpers_never_embed_generated_payloads(self):
        body = "secret-body" * 1000
        call = session_loop.ToolCall(
            "write_file", {"path": "src/a.rs", "content": body}
        )
        assistant = compact_assistant_action(call)
        self.assertNotIn(body, assistant)
        tools = FakeFileTools()
        tools.current = body
        user = compact_tool_result(
            {"tool": "write_file", "ok": True, "path": "src/a.rs"}, tools
        )
        self.assertNotIn(body, user)
        self.assertIn("CURRENT_SOURCE_SHA256", user)


if __name__ == "__main__":
    unittest.main()
