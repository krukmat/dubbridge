#!/usr/bin/env python3
"""Tests for prompt_builder.py — HP-1, EC-1, EC-2 per docs/tasks/local-role-prompt-canonicalization.md LRPC-2."""

import os
import sys
import unittest
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import prompt_anchors
import prompt_builder


class HP1RoleWithSufficientBudgetReturnsAllClausesAndFormatText(unittest.TestCase):
    def test_gemma_reviewer_prompt_contains_every_clause_and_output_format_text(self):
        output_format_text = "Return STATUS/FINDING tagged blocks only."
        prompt = prompt_builder.build_system_prompt(
            "gemma_reviewer",
            num_ctx=65536,
            num_predict=4096,
            output_format_text=output_format_text,
        )
        for clause in prompt_anchors.ROLE_ANCHORS["gemma_reviewer"]:
            self.assertIn(clause.text, prompt)
        self.assertIn(output_format_text, prompt)

    def test_every_role_returns_a_prompt_containing_all_its_clauses(self):
        output_format_text = "Return JSON only."
        for role, clauses in prompt_anchors.ROLE_ANCHORS.items():
            prompt = prompt_builder.build_system_prompt(
                role,
                num_ctx=65536,
                num_predict=4096,
                output_format_text=output_format_text,
            )
            for clause in clauses:
                self.assertIn(clause.text, prompt, msg=f"role {role!r} missing a clause")
            self.assertIn(output_format_text, prompt)


class EC1TooSmallBudgetRaisesBeforeAnyNetworkCall(unittest.TestCase):
    def test_tiny_budget_raises_prompt_budget_exceeded(self):
        with self.assertRaises(prompt_builder.PromptBudgetExceeded):
            prompt_builder.build_system_prompt(
                "gemma_reviewer",
                num_ctx=1,
                num_predict=0,
                output_format_text="x",
            )

    def test_tiny_budget_never_attempts_a_network_call(self):
        with mock.patch("urllib.request.urlopen") as urlopen_mock, mock.patch(
            "urllib.request.Request"
        ) as request_mock:
            with self.assertRaises(prompt_builder.PromptBudgetExceeded):
                prompt_builder.build_system_prompt(
                    "gemma_reviewer",
                    num_ctx=1,
                    num_predict=0,
                    output_format_text="x",
                )
            urlopen_mock.assert_not_called()
            request_mock.assert_not_called()

    def test_exceeded_error_reports_role_and_token_figures(self):
        try:
            prompt_builder.build_system_prompt(
                "gemma_reviewer",
                num_ctx=1,
                num_predict=0,
                output_format_text="x",
            )
            self.fail("expected PromptBudgetExceeded")
        except prompt_builder.PromptBudgetExceeded as exc:
            self.assertEqual(exc.role, "gemma_reviewer")
            self.assertGreater(exc.estimated_tokens, exc.budget_tokens)


class EC2UnknownRoleRaisesTypedErrorNamingTheRole(unittest.TestCase):
    def test_unknown_role_raises_unknown_role_error(self):
        with self.assertRaises(prompt_builder.UnknownRoleError) as ctx:
            prompt_builder.build_system_prompt(
                "nonexistent_role",
                num_ctx=65536,
                num_predict=4096,
                output_format_text="x",
            )
        self.assertIn("nonexistent_role", str(ctx.exception))
        self.assertEqual(ctx.exception.role, "nonexistent_role")

    # Gemma Reviewer phase-2 finding (LRPC-2): the removed
    # test_unknown_role_does_not_return_an_empty_string asserted on `result`
    # inside an `assertRaises` block, which is dead code — the exception
    # fires before the assignment completes, so the assertion never ran.
    # test_unknown_role_raises_unknown_role_error above already proves the
    # call never returns normally for an unknown role (assertRaises fails
    # the test if no exception is raised), so "does not return an empty
    # string" needs no separate test — a call that raises produces no return
    # value at all, empty or otherwise.


class TokenEstimationReusesGemmaLocalHelper(unittest.TestCase):
    def test_derive_prompt_budget_matches_the_documented_subtraction_shape(self):
        num_ctx = 65536
        num_predict = 4096
        budget = prompt_builder.derive_prompt_budget(num_ctx, num_predict)
        self.assertEqual(
            budget,
            num_ctx - num_predict - prompt_builder._overhead_tokens(),
        )

    def test_derive_prompt_budget_clamps_negative_results_to_zero(self):
        # Gemma Reviewer phase-2 finding (LRPC-2, major): an undersized
        # num_ctx or oversized num_predict must never yield a negative
        # budget — that would make PromptBudgetExceeded's own comparison
        # nonsensical (any negative number is trivially "exceeded" for the
        # wrong reason). num_ctx=1, num_predict=0 with a 64-token overhead
        # would be -63 unclamped; it must read 0 instead.
        self.assertEqual(prompt_builder.derive_prompt_budget(1, 0), 0)
        self.assertEqual(prompt_builder.derive_prompt_budget(10, 1000), 0)

    def test_estimate_text_tokens_is_imported_not_reimplemented(self):
        import gemma_local

        self.assertIs(prompt_builder.estimate_text_tokens, gemma_local.estimate_text_tokens)


if __name__ == "__main__":
    unittest.main()
