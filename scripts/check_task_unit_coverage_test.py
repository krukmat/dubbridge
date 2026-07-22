#!/usr/bin/env python3
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_SOURCE = REPO_ROOT / "scripts" / "check-task-unit-coverage.sh"

EVIDENCE_TABLE = (
    "| Case ID | Type | Behavior | Unit test evidence | Result |\n"
    "|---|---|---|---|---|\n"
    "| HP-1 | Happy path | something | `apps/api/tests/foo_test.rs::test_case` | passed |\n"
    "| EC-1 | Edge case | something | `apps/api/tests/foo_test.rs::test_case` | passed |\n"
)

OWNER_BLOCK = (
    "### Owner final verification\n\n"
    "- Owner: matias\n"
    "- Date: 2026-07-22\n"
    "- Statement: I verified every happy path and edge case defined for this "
    "task has unit test evidence that replicates the expected behavior.\n"
    "- Commands run: `cargo test`\n"
)

REFLECTION_BLOCK = (
    "### Reflection log\n\n"
    "- Required passes: 1\n"
    "- Pass 1: fixture.\n"
)


def section(title, evidence_line, extra_lines=""):
    return (
        f"## {title}\n\n"
        "- **Status:** [x] Done — 2026-07-22\n"
        "- **Type:** development\n"
        "- **RRI:** 30\n\n"
        "### Happy paths considered\n"
        "- HP-1: something\n\n"
        "### Edge cases considered\n"
        "- EC-1: something\n\n"
        "### Unit coverage certification\n\n"
        f"{EVIDENCE_TABLE}\n"
        f"{OWNER_BLOCK}\n"
        f"{REFLECTION_BLOCK}\n"
        f"{evidence_line}\n"
        f"{extra_lines}"
    )


class TaskUnitCoverageEvidenceGate(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "scripts").mkdir()
        shutil.copy(SCRIPT_SOURCE, self.root / "scripts" / "check-task-unit-coverage.sh")
        os.chmod(self.root / "scripts" / "check-task-unit-coverage.sh", 0o755)
        (self.root / "docs" / "tasks").mkdir(parents=True)
        (self.root / "docs" / "audit" / "gemma-evidence").mkdir(parents=True)
        (self.root / "apps" / "api" / "tests").mkdir(parents=True)
        (self.root / "apps" / "api" / "tests" / "foo_test.rs").write_text(
            "#[test]\nfn test_case() {}\n", encoding="utf-8"
        )
        self.run_cmd("git", "init")
        self.run_cmd("git", "config", "user.email", "test@example.com")
        self.run_cmd("git", "config", "user.name", "Test User")

    def tearDown(self):
        self.tmp.cleanup()

    def run_cmd(self, *args, check=True):
        result = subprocess.run(args, cwd=self.root, capture_output=True, text=True)
        if check and result.returncode != 0:
            self.fail(f"{args} failed\nstdout={result.stdout}\nstderr={result.stderr}")
        return result

    def write(self, path, contents):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(contents, encoding="utf-8")

    def commit_all(self):
        self.run_cmd("git", "add", ".")
        self.run_cmd("git", "commit", "-m", "fixture")

    def head_sha(self):
        return self.run_cmd("git", "rev-parse", "HEAD").stdout.strip()

    def write_ledger(self, rows=""):
        header = (
            "| Task ID | Override type | Reason | Waiver-by / Failed-attempt / Scope-note | Date |\n"
            "|---|---|---|---|---|\n"
        )
        self.write("docs/audit/gemma-review-overrides.md", header + rows)

    def check_gate(self):
        return self.run_cmd(
            "bash", "scripts/check-task-unit-coverage.sh", "docs/tasks/corpus.md", check=False
        )

    def write_corpus(self, body):
        self.write(
            "docs/tasks/corpus.md",
            "Behavioral coverage contract: unit-v1\n\n" + body,
        )

    # -- artifact branch --------------------------------------------------

    def test_valid_review_artifact_passes(self):
        self.write_corpus(section("T-PASS", "- Review artifact: docs/audit/gemma-evidence/T-PASS.json"))
        self.write_ledger()
        self.commit_all()
        sha = self.head_sha()
        self.write(
            "docs/audit/gemma-evidence/T-PASS.json",
            f'{{"task_id":"T-PASS","commit_sha":"{sha}","reviewer":"gemma","verdict":"PASS","timestamp":"2026-07-22T00:00:00Z"}}',
        )

        result = self.check_gate()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_mismatched_task_id_fails(self):
        self.write_corpus(section("T-MISMATCH", "- Review artifact: docs/audit/gemma-evidence/T-MISMATCH.json"))
        self.write_ledger()
        self.commit_all()
        sha = self.head_sha()
        self.write(
            "docs/audit/gemma-evidence/T-MISMATCH.json",
            f'{{"task_id":"T-WRONG","commit_sha":"{sha}","reviewer":"gemma","verdict":"PASS","timestamp":"2026-07-22T00:00:00Z"}}',
        )

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not match section", result.stdout)

    def test_invalid_commit_sha_fails(self):
        self.write_corpus(
            section("T-INVALIDSHA", "- Review artifact: docs/audit/gemma-evidence/T-INVALIDSHA.json")
        )
        self.write_ledger()
        self.write(
            "docs/audit/gemma-evidence/T-INVALIDSHA.json",
            '{"task_id":"T-INVALIDSHA","commit_sha":"0000000000000000000000000000000000dead",'
            '"reviewer":"gemma","verdict":"PASS","timestamp":"2026-07-22T00:00:00Z"}',
        )
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("is not a valid commit object", result.stdout)

    def test_unreachable_commit_sha_fails(self):
        # A real commit object that exists in the repo's object database but
        # is not an ancestor of HEAD (lives on an orphan branch instead).
        self.write("orphan-seed.txt", "seed\n")
        self.commit_all()
        original_branch = self.run_cmd(
            "git", "rev-parse", "--abbrev-ref", "HEAD"
        ).stdout.strip()
        self.run_cmd("git", "checkout", "--orphan", "unreachable-branch")
        self.run_cmd("git", "rm", "-rf", ".")
        self.write("orphan-only.txt", "orphan\n")
        self.run_cmd("git", "add", "orphan-only.txt")
        self.run_cmd("git", "commit", "-m", "orphan commit")
        orphan_sha = self.head_sha()
        self.run_cmd("git", "checkout", original_branch)

        self.write_corpus(
            section("T-UNREACHABLE", "- Review artifact: docs/audit/gemma-evidence/T-UNREACHABLE.json")
        )
        self.write_ledger()
        self.write(
            "docs/audit/gemma-evidence/T-UNREACHABLE.json",
            f'{{"task_id":"T-UNREACHABLE","commit_sha":"{orphan_sha}",'
            '"reviewer":"gemma","verdict":"PASS","timestamp":"2026-07-22T00:00:00Z"}',
        )
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("is not reachable from reviewed history", result.stdout)

    def test_no_evidence_at_all_fails(self):
        self.write_corpus(section("T-NOEVID", ""))
        self.write_ledger()
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing Review artifact or REVIEW-OVERRIDE evidence", result.stdout)

    # -- override branches --------------------------------------------------

    def test_urgency_override_complete_passes(self):
        self.write_corpus(
            section(
                "T-URGENCY",
                "- REVIEW-OVERRIDE: urgency — production incident\n- Waiver-by: matias",
            )
        )
        self.write_ledger("| T-URGENCY | urgency | fixture | Waiver-by: matias | 2026-07-22 |\n")
        self.commit_all()

        result = self.check_gate()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_urgency_override_missing_waiver_by_fails(self):
        self.write_corpus(section("T-URGENCY-BAD", "- REVIEW-OVERRIDE: urgency — production incident"))
        self.write_ledger("| T-URGENCY-BAD | urgency | fixture | Waiver-by: matias | 2026-07-22 |\n")
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing companion Waiver-by", result.stdout)

    def test_pipeline_failure_override_complete_passes(self):
        self.write_corpus(
            section(
                "T-PIPEFAIL",
                "- REVIEW-OVERRIDE: pipeline-failure — reviewer aborted\n"
                "- Failed-attempt: malformed_tool_call_repeated",
            )
        )
        self.write_ledger("| T-PIPEFAIL | pipeline-failure | fixture | Failed-attempt: aborted | 2026-07-22 |\n")
        self.commit_all()

        result = self.check_gate()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_pipeline_failure_override_missing_failed_attempt_fails(self):
        self.write_corpus(section("T-PIPEFAIL-BAD", "- REVIEW-OVERRIDE: pipeline-failure — reviewer aborted"))
        self.write_ledger("| T-PIPEFAIL-BAD | pipeline-failure | fixture | Failed-attempt: aborted | 2026-07-22 |\n")
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing companion Failed-attempt", result.stdout)

    def test_not_applicable_override_complete_passes(self):
        self.write_corpus(
            section(
                "T-NOTAPP",
                "- REVIEW-OVERRIDE: not-applicable — config-only change\n"
                "- Scope-note: no code review needed",
            )
        )
        self.write_ledger("| T-NOTAPP | not-applicable | fixture | Scope-note: config-only | 2026-07-22 |\n")
        self.commit_all()

        result = self.check_gate()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_not_applicable_override_missing_scope_note_fails(self):
        self.write_corpus(section("T-NOTAPP-BAD", "- REVIEW-OVERRIDE: not-applicable — config-only change"))
        self.write_ledger("| T-NOTAPP-BAD | not-applicable | fixture | Scope-note: config-only | 2026-07-22 |\n")
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing companion Scope-note", result.stdout)

    def test_invalid_override_type_fails(self):
        self.write_corpus(section("T-BADTYPE", "- REVIEW-OVERRIDE: whatever — not a real type"))
        self.write_ledger()
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("is not one of urgency, not-applicable, pipeline-failure", result.stdout)

    def test_override_absent_from_ledger_fails(self):
        self.write_corpus(
            section(
                "T-NOLEDGER",
                "- REVIEW-OVERRIDE: urgency — production incident\n- Waiver-by: matias",
            )
        )
        self.write_ledger()  # empty ledger, no matching row
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no matching row in", result.stdout)

    # -- grandfather cutover --------------------------------------------------

    def test_pre_cutover_section_uses_legacy_gemma_check_not_new_gate(self):
        # Pre-cutover sections keep the pre-GEG-1 RRI<=40 "Gemma Reviewer
        # evidence" contract and are exempt from the new artifact-or-override
        # gate entirely — no Review artifact / REVIEW-OVERRIDE line required.
        legacy_evidence = (
            "### Gemma Reviewer evidence\n\n"
            "- Command: `make qa-gemma-review`\n"
            "- Quorum: met\n"
            "- Primary-agent disposition: accepted\n"
        )
        old_section = section("T-OLD", "", extra_lines=legacy_evidence).replace(
            "- Date: 2026-07-22", "- Date: 2026-01-01"
        )
        self.write_corpus(old_section)
        self.write_ledger()
        self.commit_all()

        result = self.check_gate()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_pre_cutover_section_without_new_evidence_still_requires_legacy_block(self):
        old_section = section("T-OLD-BARE", "").replace(
            "- Date: 2026-07-22", "- Date: 2026-01-01"
        )
        self.write_corpus(old_section)
        self.write_ledger()
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing Gemma Reviewer evidence section", result.stdout)
        self.assertNotIn("missing Review artifact or REVIEW-OVERRIDE evidence", result.stdout)


if __name__ == "__main__":
    unittest.main()
