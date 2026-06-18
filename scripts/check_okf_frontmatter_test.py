#!/usr/bin/env python3
"""Tests for check_okf_frontmatter.py — covers HP-1/2 and EC-1..4."""
import sys
import textwrap
import types
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

# ---------------------------------------------------------------------------
# Bootstrap: import the validator without executing main().
# ---------------------------------------------------------------------------
import importlib.util

_spec = importlib.util.spec_from_file_location(
    "check_okf_frontmatter",
    Path(__file__).parent / "check_okf_frontmatter.py",
)
_mod = importlib.util.module_from_spec(_spec)  # type: ignore[arg-type]
_spec.loader.exec_module(_mod)  # type: ignore[union-attr]

parse_frontmatter = _mod.parse_frontmatter
extract_prose_status = _mod.extract_prose_status
type_matches_location = _mod.type_matches_location
adr_exists = _mod.adr_exists
should_skip = _mod.should_skip
validate = _mod.validate
REPO_ROOT = _mod.REPO_ROOT


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_adr(status_fm: str, status_prose: str) -> str:
    return textwrap.dedent(f"""\
        ---
        type: ADR
        title: "ADR-999: test"
        status: {status_fm}
        ---
        # ADR-999

        - **Status:** {status_prose}

        ## Context
        Some context.
    """)


def _make_plan(governed_by: list[str] | None = None) -> str:
    lines = [
        "---",
        "type: Plan",
        'title: "S-999: test plan"',
        "status: active",
        "slice: S-999",
    ]
    if governed_by is not None:
        items = ", ".join(governed_by)
        lines.append(f"governed_by: [{items}]")
    lines += ["---", "# Plan", ""]
    return "\n".join(lines)


def _fake_path(rel: str, text: str) -> MagicMock:
    p = MagicMock(spec=Path)
    p.read_text.return_value = text
    p.relative_to.return_value = Path(rel)
    return p


def _run_validate(rel: str, text: str) -> list[str]:
    """Run validate() against a single fake file."""
    p = MagicMock(spec=Path)
    p.read_text.return_value = text
    # _rel() calls path.relative_to(REPO_ROOT) -> needs a real Path-like result
    p.relative_to.return_value = Path(rel)
    with patch.object(_mod, "_rel", return_value=rel):
        return validate([p])


# ---------------------------------------------------------------------------
# parse_frontmatter
# ---------------------------------------------------------------------------

class TestParseFrontmatter(unittest.TestCase):
    def test_valid_block(self):
        text = "---\ntype: ADR\nstatus: Accepted\n---\n# body"
        fm = parse_frontmatter(text)
        self.assertEqual(fm, {"type": "ADR", "status": "Accepted"})

    def test_missing_block(self):
        self.assertIsNone(parse_frontmatter("# no frontmatter"))

    def test_malformed_yaml(self):
        self.assertIsNone(parse_frontmatter("---\n: bad: yaml: [\n---\n"))

    def test_non_dict_yaml(self):
        self.assertIsNone(parse_frontmatter("---\n- list item\n---\n"))

    def test_unclosed_block(self):
        self.assertIsNone(parse_frontmatter("---\ntype: ADR\n"))


# ---------------------------------------------------------------------------
# extract_prose_status
# ---------------------------------------------------------------------------

class TestExtractProseStatus(unittest.TestCase):
    def test_accepted(self):
        self.assertEqual(extract_prose_status("- **Status:** Accepted"), "Accepted")

    def test_proposed(self):
        self.assertEqual(extract_prose_status("- **Status:** Proposed"), "Proposed")

    def test_superseded_with_suffix(self):
        self.assertEqual(
            extract_prose_status("- **Status:** Superseded by ADR-031"), "Superseded"
        )

    def test_missing(self):
        self.assertIsNone(extract_prose_status("No status line here"))


# ---------------------------------------------------------------------------
# type_matches_location
# ---------------------------------------------------------------------------

class TestTypeMatchesLocation(unittest.TestCase):
    def test_adr_match(self):
        self.assertTrue(type_matches_location("ADR", "docs/adr/ADR-006-foo.md"))

    def test_plan_match(self):
        self.assertTrue(type_matches_location("Plan", "docs/plan/s-080-foo.md"))

    def test_roadmap_match(self):
        self.assertTrue(type_matches_location("Roadmap", "docs/plan/roadmap.md"))

    def test_plan_does_not_match_roadmap(self):
        self.assertFalse(type_matches_location("Plan", "docs/plan/roadmap.md"))

    def test_type_wrong_location(self):
        self.assertFalse(type_matches_location("ADR", "docs/plan/s-001.md"))

    def test_unknown_type(self):
        self.assertFalse(type_matches_location("Unknown", "docs/adr/ADR-001.md"))


# ---------------------------------------------------------------------------
# should_skip
# ---------------------------------------------------------------------------

class TestShouldSkip(unittest.TestCase):
    def test_daily(self):
        self.assertTrue(should_skip("docs/daily/2026-06-18.md"))

    def test_template(self):
        self.assertTrue(should_skip("docs/TEMPLATE.md"))

    def test_index_readme(self):
        self.assertTrue(should_skip("docs/adr/README.md"))

    def test_knowledge_readme(self):
        self.assertTrue(should_skip("docs/knowledge/README.md"))

    def test_normal_adr_not_skipped(self):
        self.assertFalse(should_skip("docs/adr/ADR-006-foo.md"))


# ---------------------------------------------------------------------------
# HP-1 — valid ADR with matching status
# ---------------------------------------------------------------------------

class TestHP1ValidADR(unittest.TestCase):
    def test_passes(self):
        text = _make_adr("Accepted", "Accepted")
        errors = _run_validate("docs/adr/ADR-999-test.md", text)
        self.assertEqual(errors, [])


# ---------------------------------------------------------------------------
# HP-2 — TaskList with all governed_by refs resolving
# ---------------------------------------------------------------------------

class TestHP2TaskListGoverned(unittest.TestCase):
    def test_passes_when_adrs_exist(self):
        text = textwrap.dedent("""\
            ---
            type: TaskList
            title: "Tasks: S-999"
            status: active
            governed_by: [ADR-006]
            ---
            # Tasks
        """)
        with patch.object(_mod, "adr_exists", return_value=True):
            errors = _run_validate("docs/tasks/s-999-foo.md", text)
        self.assertEqual(errors, [])


# ---------------------------------------------------------------------------
# EC-1 — ADR status drift
# ---------------------------------------------------------------------------

class TestEC1ADRStatusDrift(unittest.TestCase):
    def test_fails_with_both_tokens(self):
        text = _make_adr("Proposed", "Accepted")
        errors = _run_validate("docs/adr/ADR-999-test.md", text)
        self.assertEqual(len(errors), 1)
        self.assertIn("Proposed", errors[0])
        self.assertIn("Accepted", errors[0])


# ---------------------------------------------------------------------------
# EC-2 — bad type (not in vocab / wrong location)
# ---------------------------------------------------------------------------

class TestEC2BadType(unittest.TestCase):
    def test_unknown_type(self):
        text = "---\ntype: Unknown\n---\n# body"
        errors = _run_validate("docs/adr/ADR-999-test.md", text)
        self.assertEqual(len(errors), 1)
        self.assertIn("closed vocabulary", errors[0])

    def test_type_wrong_location(self):
        text = "---\ntype: Plan\n---\n# body"
        errors = _run_validate("docs/adr/ADR-999-test.md", text)
        self.assertEqual(len(errors), 1)
        self.assertIn("does not match file location", errors[0])


# ---------------------------------------------------------------------------
# EC-3 — dangling governed_by ref
# ---------------------------------------------------------------------------

def _run_validate_with_patches(rel: str, text: str, adr_exists_retval: bool) -> list[str]:
    p = MagicMock(spec=Path)
    p.read_text.return_value = text
    p.relative_to.return_value = Path(rel)
    with patch.object(_mod, "_rel", return_value=rel), \
         patch.object(_mod, "adr_exists", return_value=adr_exists_retval):
        return validate([p])


class TestEC3DanglingRef(unittest.TestCase):
    def test_fails_for_nonexistent_adr(self):
        text = _make_plan(governed_by=["ADR-099"])
        errors = _run_validate_with_patches("docs/plan/s-999-foo.md", text, False)
        self.assertEqual(len(errors), 1)
        self.assertIn("ADR-099", errors[0])

    def test_passes_when_adr_exists(self):
        text = _make_plan(governed_by=["ADR-006"])
        errors = _run_validate_with_patches("docs/plan/s-999-foo.md", text, True)
        self.assertEqual(errors, [])


# ---------------------------------------------------------------------------
# EC-4 — missing frontmatter on in-scope file
# ---------------------------------------------------------------------------

class TestEC4MissingFrontmatter(unittest.TestCase):
    def test_in_scope_fails(self):
        text = "# No frontmatter here\n"
        errors = _run_validate("docs/adr/ADR-999-test.md", text)
        self.assertEqual(len(errors), 1)
        self.assertIn("missing or malformed", errors[0])

    def test_out_of_scope_skipped(self):
        # should_skip() is called before validate(); files not in scope
        # never reach validate(). Verify should_skip returns True.
        self.assertTrue(should_skip("docs/daily/2026-06-18.md"))
        self.assertTrue(should_skip("docs/TEMPLATE.md"))


# ---------------------------------------------------------------------------
# Additional coverage: extract_prose_status unknown prefix
# ---------------------------------------------------------------------------

class TestExtractProseStatusUnknown(unittest.TestCase):
    def test_unknown_prefix_returns_none(self):
        self.assertIsNone(extract_prose_status("- **Status:** SomeWeirdStatus"))


# ---------------------------------------------------------------------------
# Additional coverage: governed_by as a plain string (not a list)
# ---------------------------------------------------------------------------

class TestGoverned_byAsString(unittest.TestCase):
    def test_string_value_is_treated_as_single_ref(self):
        text = "\n".join([
            "---",
            "type: TaskList",
            'title: "Tasks: S-999"',
            "status: active",
            "governed_by: ADR-006",
            "---",
            "# Tasks",
        ])
        errors = _run_validate_with_patches("docs/tasks/s-999-foo.md", text, True)
        self.assertEqual(errors, [])

    def test_string_value_dangling(self):
        text = "\n".join([
            "---",
            "type: TaskList",
            'title: "Tasks: S-999"',
            "status: active",
            "governed_by: ADR-099",
            "---",
            "# Tasks",
        ])
        errors = _run_validate_with_patches("docs/tasks/s-999-foo.md", text, False)
        self.assertEqual(len(errors), 1)
        self.assertIn("ADR-099", errors[0])


# ---------------------------------------------------------------------------
# Additional coverage: adr_exists (real filesystem, no mock)
# ---------------------------------------------------------------------------

class TestAdrExistsReal(unittest.TestCase):
    def test_existing_adr(self):
        self.assertTrue(adr_exists("ADR-006"))

    def test_nonexistent_adr(self):
        self.assertFalse(adr_exists("ADR-999"))


# ---------------------------------------------------------------------------
# Additional coverage: collect_in_scope_files returns paths
# ---------------------------------------------------------------------------

class TestCollectInScopeFiles(unittest.TestCase):
    def test_returns_list_of_paths(self):
        files = _mod.collect_in_scope_files()
        self.assertIsInstance(files, list)
        rels = [_mod._rel(f) for f in files]
        # ADRs must be included
        self.assertTrue(any(r.startswith("docs/adr/ADR-") for r in rels))
        # Daily notes must not be included
        self.assertFalse(any("docs/daily/" in r for r in rels))
        # Index READMEs must not be included
        self.assertNotIn("docs/adr/README.md", rels)


# ---------------------------------------------------------------------------
# Additional coverage: main() happy and fail paths
# ---------------------------------------------------------------------------

class TestMain(unittest.TestCase):
    def test_main_pass(self):
        with patch.object(_mod, "collect_in_scope_files", return_value=[]), \
             patch.object(_mod, "validate", return_value=[]):
            rc = _mod.main()
        self.assertEqual(rc, 0)

    def test_main_fail(self):
        with patch.object(_mod, "collect_in_scope_files", return_value=[]), \
             patch.object(_mod, "validate", return_value=["some error"]):
            rc = _mod.main()
        self.assertEqual(rc, 1)


if __name__ == "__main__":
    unittest.main()
