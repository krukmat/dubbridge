"""Candidate controller tests using real temporary Git state and no HAA POST."""
from dataclasses import replace
from hashlib import sha256
from pathlib import Path
import json
import subprocess
import tempfile
import unittest
from unittest.mock import Mock

from scripts.haa.authority_lease import LeaseStore
from scripts.haa.live_verify import TrustedTask, bind_trusted_task
from scripts.haa.runner_authority import RunnerAuthority, RunnerAuthorityError, RunnerScope


class CandidateRunnerTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        root = Path(self.tmp.name) / "repo"
        root.mkdir()
        def git(*args):
            subprocess.run(["git", "-C", str(root), *args], check=True, capture_output=True)
        git("init", "-b", "test")
        git("config", "user.name", "Test")
        git("config", "user.email", "test@example.invalid")
        git("remote", "add", "origin", "https://github.com/krukmat/dubbridge.git")
        (root / "README.md").write_text("base")
        git("add", "README.md")
        git("commit", "-m", "base")
        self.root = root
        self.card = root / "card.json"
        self.store = LeaseStore(Path(self.tmp.name) / "leases", signing_key=b"K" * 32)
        self.boundary = Mock()
        self.write_card(25)

    def write_card(self, rri):
        self.card.write_text(json.dumps({
            "schema_version": 2, "card_id": "test", "task_id": "test", "spec": "Test",
            "allowed_paths": ["README.md"], "acceptance_criteria": [],
            "verification_commands": [], "rri": rri, "band": "Low" if rri <= 25 else "Med-high",
            "plan": "docs/plan/test.md",
        }))
        task = TrustedTask(self.root, "krukmat/dubbridge", self.card, "unbound", ("edit", "test"))
        self.task = bind_trusted_task(task, card_sha256=sha256(self.card.read_bytes()).hexdigest(),
                                      authoritative_rri=rri)

    def runner(self):
        return RunnerAuthority(scope=RunnerScope(self.task), leases=self.store,
                               existing_boundary=self.boundary)

    def test_low_bound_card_and_existing_scope(self):
        effect = Mock(return_value="edited")
        with self.runner() as runner:
            self.assertEqual(runner.edit("README.md", effect), "edited")
        self.boundary.check_write.assert_called_once_with("README.md")
        effect.assert_called_once()
        with self.assertRaises(RunnerAuthorityError):
            runner.edit("README.md", effect)

    def test_high_without_lease_never_calls_model_or_effect(self):
        self.write_card(55)
        with self.assertRaisesRegex(RunnerAuthorityError, "WAITING_FOR_HAA"):
            with self.runner():
                self.fail("entered unauthorized session")

    def test_card_mutation_denies_effect(self):
        effect = Mock()
        with self.runner() as runner:
            self.card.write_text(self.card.read_text() + " ")
            with self.assertRaises(RuntimeError):
                runner.edit("README.md", effect)
        effect.assert_not_called()

    def test_existing_boundary_is_not_replaced(self):
        self.boundary.check_write.side_effect = ValueError("scope")
        effect = Mock()
        with self.runner() as runner:
            with self.assertRaises(ValueError):
                runner.edit("other.txt", effect)
        effect.assert_not_called()

    def test_no_unpinned_commands(self):
        with self.runner() as runner:
            with self.assertRaisesRegex(RunnerAuthorityError, "COMMAND_NOT_PINNED"):
                runner.test(["python3", "-c", "pass"])

    def test_unbound_context_denied(self):
        self.task = replace(self.task, session_id="ordinary", card_sha256=None, authoritative_rri=None)
        with self.assertRaisesRegex(RuntimeError, "BOUND_TASK_REQUIRED"):
            self.runner().preflight()


if __name__ == "__main__":
    unittest.main()
