#!/usr/bin/env python3
"""Tests for prompt_anchors.py — HP-1, EC-1, EC-2 per docs/tasks/local-role-prompt-canonicalization.md LRPC-1."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import prompt_anchors

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

REQUIRED_ROLES = (
    "gemma_reviewer",
    "local_developer",
    "local_architect_default",
    "local_architect_med_high",
)


class StructuralRoleAnchorsAreComplete(unittest.TestCase):
    def test_role_anchors_has_exactly_the_four_required_roles(self):
        self.assertEqual(set(prompt_anchors.ROLE_ANCHORS.keys()), set(REQUIRED_ROLES))

    def test_every_role_has_at_least_one_clause(self):
        for role in REQUIRED_ROLES:
            self.assertTrue(
                len(prompt_anchors.ROLE_ANCHORS[role]) > 0,
                msg=f"role {role!r} has no clauses",
            )


class HP1GemmaReviewerAnchorMatchesCanonicalSentence(unittest.TestCase):
    def test_gemma_reviewer_clause_is_substring_of_canonical_sentence(self):
        canonical = (
            "may not write files, apply patches, approve tasks,\n"
            "  certify coverage, or mark tasks complete"
        )
        clauses = prompt_anchors.ROLE_ANCHORS["gemma_reviewer"]
        matches = [
            c
            for c in clauses
            if c.text in canonical
            and c.source_file == "docs/playbooks/AGENT_WORKFLOW_GUIDE.md"
        ]
        self.assertTrue(
            matches,
            msg="no gemma_reviewer clause is a substring of the canonical "
            "authority-boundary sentence with the expected source_file",
        )


class EC1DownstreamConsumptionClauseIsExcluded(unittest.TestCase):
    def test_no_role_carries_the_downstream_consumption_sentence(self):
        excluded = "never fails the review gate by itself"
        for role, clauses in prompt_anchors.ROLE_ANCHORS.items():
            for clause in clauses:
                self.assertNotIn(
                    excluded,
                    clause.text,
                    msg=f"role {role!r} carries an excluded downstream-consumption clause",
                )


class EC2EveryClauseIsVerbatimInItsCitedSource(unittest.TestCase):
    def test_every_clause_text_is_a_literal_substring_of_its_source_file(self):
        source_cache = {}
        for role, clauses in prompt_anchors.ROLE_ANCHORS.items():
            for clause in clauses:
                if clause.source_file not in source_cache:
                    path = os.path.join(REPO_ROOT, clause.source_file)
                    with open(path, encoding="utf-8") as f:
                        source_cache[clause.source_file] = f.read()
                content = source_cache[clause.source_file]
                self.assertIn(
                    clause.text,
                    content,
                    msg=(
                        f"role {role!r} clause {clause.text[:40]!r}... is not a "
                        f"verbatim substring of {clause.source_file}"
                    ),
                )


if __name__ == "__main__":
    unittest.main()
