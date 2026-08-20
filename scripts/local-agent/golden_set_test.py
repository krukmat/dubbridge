#!/usr/bin/env python3
"""Tests for golden_set.py — HP-1, EC-1 per docs/tasks/local-role-prompt-canonicalization.md LRPC-6.

Deterministic tests only: every live-Ollama call is mocked via
unittest.mock.patch on stream_chat. Live-model fixture runs happen only
through `make qa-golden-set` / direct invocation, never in this suite or in
`make qa-ci`.
"""

import os
import sys
import unittest
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import golden_set
from golden_fixtures import FIXTURES, VERDICT_PASS, VERDICT_VIOLATION, fixtures_for_role, all_roles
from gemma_local import StreamChatResult, StreamUsage


def _stream_result(content: str) -> StreamChatResult:
    return StreamChatResult(content=content, usage=StreamUsage(done_reason="stop"))


class HP1EquivalentFixtureAcrossBothConditionsPasses(unittest.TestCase):
    def test_matching_expected_verdicts_in_both_conditions_marks_equivalent(self):
        fixture = fixtures_for_role("gemma_reviewer")[0]
        content = f'{{"verdict": "{fixture.expected_verdict}", "reason": "matches"}}'
        with mock.patch.object(golden_set, "stream_chat", return_value=_stream_result(content)):
            result = golden_set.run_fixture(
                fixture,
                model="test-model",
                host="http://localhost:11434",
                num_ctx=8192,
                num_predict=512,
                temperature=0.0,
                idle_timeout=5,
                max_wall=10,
            )
        self.assertTrue(result.equivalent)
        self.assertEqual(result.before.verdict, fixture.expected_verdict)
        self.assertEqual(result.after.verdict, fixture.expected_verdict)

    def test_run_all_produces_one_result_per_fixture(self):
        fixtures = fixtures_for_role("local_developer")
        content = '{"verdict": "PASS", "reason": "n/a"}'
        with mock.patch.object(golden_set, "stream_chat", return_value=_stream_result(content)):
            results = golden_set.run_all(
                fixtures,
                model="test-model",
                host="http://localhost:11434",
                num_ctx=8192,
                num_predict=512,
                temperature=0.0,
                idle_timeout=5,
                max_wall=10,
            )
        self.assertEqual(len(results), len(fixtures))


class EC1DivergentConditionVerdictsAreDetectedAsMismatch(unittest.TestCase):
    def test_builder_condition_disagreeing_with_expected_verdict_is_not_equivalent(self):
        """Reproduces the LRPC drift-bug class directly: if the builder's
        compressed clause were lossy (e.g. 'certify coverage' silently
        dropped), the model would answer PASS in the after-condition for a
        transcript that should be a VIOLATION, while the full-prose
        before-condition (which always carries the complete clause) still
        correctly answers VIOLATION. The harness must catch this divergence,
        not silently average or ignore the mismatch. Distinguish the two
        conditions by the harness's own CONDITION_BEFORE/CONDITION_AFTER
        markers passed through progress_label, not by re-deriving prompt
        content -- that would just re-assert what LRPC-1/2 already prove."""
        fixture = next(f for f in FIXTURES if f.fixture_id == "gemma_reviewer-certify-coverage")
        self.assertEqual(fixture.expected_verdict, VERDICT_VIOLATION)

        call_count = {"n": 0}

        def fake_stream_chat(url, payload, idle_timeout, max_wall, progress_label="delegate"):
            # First call is always the before-condition (see run_fixture's
            # call order); simulate a lossy builder by having only the
            # second (after) call disagree with the correct verdict.
            call_count["n"] += 1
            if call_count["n"] == 1:
                return _stream_result('{"verdict": "VIOLATION", "reason": "full prose carries the clause"}')
            return _stream_result('{"verdict": "PASS", "reason": "compressed clause silently dropped this case"}')

        with mock.patch.object(golden_set, "stream_chat", side_effect=fake_stream_chat):
            result = golden_set.run_fixture(
                fixture,
                model="test-model",
                host="http://localhost:11434",
                num_ctx=8192,
                num_predict=512,
                temperature=0.0,
                idle_timeout=5,
                max_wall=10,
            )
        self.assertFalse(result.equivalent)
        self.assertEqual(result.before.verdict, VERDICT_VIOLATION)
        self.assertEqual(result.after.verdict, VERDICT_PASS)

    def test_summarize_lists_mismatched_fixture_ids_as_failures(self):
        fixture = next(f for f in FIXTURES if f.fixture_id == "gemma_reviewer-mark-complete")
        call_count = {"n": 0}

        def fake_stream_chat(url, payload, idle_timeout, max_wall, progress_label="delegate"):
            call_count["n"] += 1
            if call_count["n"] == 1:
                return _stream_result('{"verdict": "VIOLATION", "reason": "matches"}')
            return _stream_result('{"verdict": "PASS", "reason": "no match"}')

        with mock.patch.object(golden_set, "stream_chat", side_effect=fake_stream_chat):
            result = golden_set.run_fixture(
                fixture,
                model="test-model",
                host="http://localhost:11434",
                num_ctx=8192,
                num_predict=512,
                temperature=0.0,
                idle_timeout=5,
                max_wall=10,
            )
        summary = golden_set.summarize([result])
        self.assertEqual(summary["status"], "FAIL")
        self.assertIn(fixture.fixture_id, summary["failed"])


class ParseVerdictResponseIsFailClosed(unittest.TestCase):
    def test_invalid_json_raises(self):
        with self.assertRaises(golden_set.GoldenSetError):
            golden_set.parse_verdict_response("not json")

    def test_unknown_verdict_value_raises(self):
        with self.assertRaises(golden_set.GoldenSetError):
            golden_set.parse_verdict_response('{"verdict": "MAYBE"}')

    def test_valid_verdict_parses(self):
        verdict, reason = golden_set.parse_verdict_response(
            '{"verdict": "PASS", "reason": "ok"}'
        )
        self.assertEqual(verdict, VERDICT_PASS)
        self.assertEqual(reason, "ok")

    def test_markdown_fenced_response_is_stripped(self):
        verdict, _reason = golden_set.parse_verdict_response(
            '```json\n{"verdict": "VIOLATION", "reason": "fenced"}\n```'
        )
        self.assertEqual(verdict, VERDICT_VIOLATION)


class ConditionErrorIsRecordedNotRaised(unittest.TestCase):
    def test_stream_chat_failure_is_captured_as_condition_error(self):
        fixture = fixtures_for_role("local_developer")[0]
        with mock.patch.object(golden_set, "stream_chat", side_effect=RuntimeError("idle timeout")):
            result = golden_set.run_fixture(
                fixture,
                model="test-model",
                host="http://localhost:11434",
                num_ctx=8192,
                num_predict=512,
                temperature=0.0,
                idle_timeout=5,
                max_wall=10,
            )
        self.assertFalse(result.equivalent)
        self.assertIsNotNone(result.before.error)
        self.assertIsNotNone(result.after.error)


class FixtureCoverageSpansAllFourRoles(unittest.TestCase):
    def test_all_four_prompt_anchor_roles_have_at_least_one_fixture(self):
        expected_roles = {
            "gemma_reviewer",
            "local_developer",
            "local_architect_default",
            "local_architect_med_high",
        }
        self.assertEqual(set(all_roles()), expected_roles)

    def test_every_role_has_at_least_one_pass_and_one_violation_fixture(self):
        for role in all_roles():
            role_fixtures = fixtures_for_role(role)
            verdicts = {f.expected_verdict for f in role_fixtures}
            self.assertIn(VERDICT_PASS, verdicts, msg=f"role {role!r} has no PASS fixture")
            self.assertIn(VERDICT_VIOLATION, verdicts, msg=f"role {role!r} has no VIOLATION fixture")


if __name__ == "__main__":
    unittest.main()
