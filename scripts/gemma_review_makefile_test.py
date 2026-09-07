#!/usr/bin/env python3
"""Unit coverage for GEG-2a/2b: the qa-gemma-review Makefile recipe's own
control flow (fail-close on review failure, task-scoped result path, and
reviewer/changed_paths extraction into the committed receipt).

Runs the real, unmodified `Makefile` target against a stub
`scripts/gemma-code-review.py` (no Ollama/network dependency) plus the real
`scripts/parse-review-findings.py`, so these tests exercise the actual
committed recipe logic, not a reimplementation of it.
"""
import os
import shutil
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MAKEFILE_SOURCE = REPO_ROOT / "Makefile"
PARSE_FINDINGS_SOURCE = REPO_ROOT / "scripts" / "parse-review-findings.py"

STUB_GEMMA_CODE_REVIEW = '''#!/usr/bin/env python3
import argparse, json, os, sys

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("packet", nargs="?")
    parser.add_argument("--out")
    args, _ = parser.parse_known_args()
    packet = sys.stdin.read() if args.packet in (None, "-") else open(args.packet).read()

    mode = os.environ.get("STUB_REVIEW_MODE", "pass")
    changed_paths = []
    for line in packet.splitlines():
        if line.startswith("+++ b/"):
            changed_paths.append(line[len("+++ b/"):].strip())

    if mode == "no-usable-passes":
        print("[review] no usable review passes (0/3 parsed)", file=sys.stderr)
        return 3
    if mode == "blocked":
        return 2
    if mode == "sigint":
        # EC-2: real interruption, not a synthetic exit code — the review
        # process dies on SIGINT after reading the packet but before writing
        # --out, which is exactly the shape of an operator-cancelled run.
        import signal
        signal.signal(signal.SIGINT, signal.SIG_DFL)
        os.kill(os.getpid(), signal.SIGINT)

    result = {
        "status": "findings" if mode == "findings" else "pass",
        "summary": "stub",
        "changed_paths": changed_paths,
        "findings": (
            # parse-review-findings.py only fails closed on blocking/major
            # severity (minor/nit are reported but exit 0) — use "major" so
            # this stub actually exercises the FINDINGS-ACKED path.
            [{"path": changed_paths[0] if changed_paths else "x", "line": 1,
              "severity": "major", "detail": "stub", "suggestion": "stub"}]
            if mode == "findings" else []
        ),
        "passes_run": 3,
        "passes_succeeded": 3,
    }
    model = os.environ.get("STUB_REVIEW_MODEL", "__unset__")
    if model != "__omit__":
        result["model"] = model

    if args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            json.dump(result, f)
    else:
        print(json.dumps(result))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
'''


class MakefileHarness(unittest.TestCase):
    """Shared fixture: a temp git repo holding the real Makefile plus stubs.
    Carries no tests of its own."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "scripts").mkdir()
        (self.root / "docs" / "audit" / "gemma-evidence").mkdir(parents=True)
        (self.root / "src").mkdir()

        shutil.copy(MAKEFILE_SOURCE, self.root / "Makefile")
        shutil.copy(PARSE_FINDINGS_SOURCE, self.root / "scripts" / "parse-review-findings.py")

        stub_path = self.root / "scripts" / "gemma-code-review.py"
        stub_path.write_text(STUB_GEMMA_CODE_REVIEW, encoding="utf-8")
        stub_path.chmod(stub_path.stat().st_mode | stat.S_IEXEC)

        self.run_cmd("git", "init")
        self.run_cmd("git", "config", "user.email", "test@example.com")
        self.run_cmd("git", "config", "user.name", "Test User")
        (self.root / "src" / "lib.rs").write_text("fn base() {}\n", encoding="utf-8")
        self.commit_all("base commit")
        # Uncommitted code change so `git diff HEAD -- ` is non-empty and the
        # recipe's no-code-changes early exit doesn't short-circuit the test.
        (self.root / "src" / "lib.rs").write_text("fn base() {}\nfn added() {}\n", encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def run_cmd(self, *args, env=None, check=True):
        full_env = {**os.environ, **(env or {})}
        result = subprocess.run(
            args, cwd=self.root, capture_output=True, text=True, env=full_env
        )
        if check and result.returncode != 0:
            self.fail(f"{args} failed (exit {result.returncode})\nstdout={result.stdout}\nstderr={result.stderr}")
        return result

    def commit_all(self, message):
        self.run_cmd("git", "add", ".")
        self.run_cmd("git", "commit", "-m", message)

    def make_qa_gemma_review(self, extra_vars=None, env=None):
        args = ["make", "qa-gemma-review"]
        for key, value in (extra_vars or {}).items():
            args.append(f"{key}={value}")
        return subprocess.run(
            args, cwd=self.root, capture_output=True, text=True,
            env={**os.environ, **(env or {})}, check=False,
        )

    def read_receipt(self, task_id):
        import json
        path = self.root / "docs" / "audit" / "gemma-evidence" / f"{task_id}.json"
        return json.loads(path.read_text(encoding="utf-8"))

class QaGemmaReviewMakefileTarget(MakefileHarness):
    # -- GEG-2a: fail-close ------------------------------------------------

    def test_hp1_success_no_findings_writes_pass_receipt_exit_zero(self):
        result = self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-HP1"}, env={"STUB_REVIEW_MODE": "pass", "STUB_REVIEW_MODEL": "gemma"}
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        receipt = self.read_receipt("T-HP1")
        self.assertEqual(receipt["verdict"], "PASS")

    def test_hp2_success_with_findings_writes_findings_acked_exit_nonzero(self):
        result = self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-HP2"}, env={"STUB_REVIEW_MODE": "findings", "STUB_REVIEW_MODEL": "gemma"}
        )

        self.assertNotEqual(result.returncode, 0)
        receipt = self.read_receipt("T-HP2")
        self.assertEqual(receipt["verdict"], "FINDINGS-ACKED")

    def test_ec1_no_usable_passes_with_stale_result_aborts_no_receipt(self):
        # Seed a stale, VALID result file at the task-scoped path, exactly as
        # a stale artifact from an earlier invocation would look. Before
        # GEG-2a this is precisely the setup that minted a false PASS receipt
        # for a different task's review (the S-230-T4l incident).
        result_path = self.root / "dubbridge-gemma-review-T-EC1.json"
        result_path.write_text('{"status":"pass","changed_paths":["other-task-file.rs"],"model":"stale-model"}', encoding="utf-8")
        receipt_path = self.root / "docs" / "audit" / "gemma-evidence" / "T-EC1.json"

        result = self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-EC1", "GEMMA_REVIEW_RESULT": str(result_path)},
            env={"STUB_REVIEW_MODE": "no-usable-passes"},
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(receipt_path.exists(), "no receipt should be written when the review command fails")
        self.assertFalse(result_path.exists(), "the stale result file must be removed, not reused")

    def test_unclearable_stale_result_aborts_before_reviewing(self):
        # AC3 says the pre-existing result IS removed, not that removal was
        # attempted. A directory at the result path survives `rm -f`
        # deterministically and portably, standing in for any reason the stale
        # artifact outlives the removal (permissions, immutability flags) --
        # which is why the guard asserts absence rather than trusting rm's
        # exit status. Without it the recipe would review on and then hand the
        # survivor to parse-review-findings.py -- D1's exact shape, one layer
        # down.
        result_path = self.root / "dubbridge-gemma-review-T-NORM.json"
        result_path.mkdir()
        receipt_path = self.root / "docs" / "audit" / "gemma-evidence" / "T-NORM.json"

        result = self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-NORM", "GEMMA_REVIEW_RESULT": str(result_path)},
            env={"STUB_REVIEW_MODE": "pass", "STUB_REVIEW_MODEL": "gemma"},
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(receipt_path.exists(), "no receipt when the stale result could not be cleared")
        self.assertIn("could not clear stale result", result.stdout + result.stderr)

    def test_ec2_interrupted_review_mints_no_receipt(self):
        receipt_path = self.root / "docs" / "audit" / "gemma-evidence" / "T-EC2.json"

        result = self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-EC2"}, env={"STUB_REVIEW_MODE": "sigint"}
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(receipt_path.exists(), "an interrupted review must not mint a receipt")

    def test_blocked_status_aborts_no_receipt(self):
        receipt_path = self.root / "docs" / "audit" / "gemma-evidence" / "T-BLOCKED.json"

        result = self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-BLOCKED"}, env={"STUB_REVIEW_MODE": "blocked"}
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(receipt_path.exists())

    def test_ac2_default_result_path_is_task_scoped(self):
        result = self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-SCOPED"}, env={"STUB_REVIEW_MODE": "pass", "STUB_REVIEW_MODEL": "gemma"}
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("dubbridge-gemma-review-T-SCOPED.json", result.stdout + result.stderr)

    def test_ec3_two_sequential_task_ids_do_not_cross_contaminate(self):
        first = self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-SEQ-A"}, env={"STUB_REVIEW_MODE": "pass", "STUB_REVIEW_MODEL": "gemma"}
        )
        self.assertEqual(first.returncode, 0, first.stdout + first.stderr)

        second = self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-SEQ-B"}, env={"STUB_REVIEW_MODE": "no-usable-passes"}
        )

        self.assertNotEqual(second.returncode, 0)
        receipt_b_path = self.root / "docs" / "audit" / "gemma-evidence" / "T-SEQ-B.json"
        self.assertFalse(receipt_b_path.exists(), "T-SEQ-B must not inherit T-SEQ-A's result")
        receipt_a = self.read_receipt("T-SEQ-A")
        self.assertEqual(receipt_a["task_id"], "T-SEQ-A")

    # -- GEG-2b: reviewer attribution ---------------------------------------

    def test_hp1_receipt_reviewer_matches_resolved_model(self):
        self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-REV1"},
            env={"STUB_REVIEW_MODE": "pass", "STUB_REVIEW_MODEL": "gemma4:26b-a4b-it-qat"},
        )

        receipt = self.read_receipt("T-REV1")
        self.assertEqual(receipt["reviewer"], "gemma4:26b-a4b-it-qat")

    def test_hp2_receipt_reviewer_reflects_fallback_model(self):
        # Same mechanism regardless of which model gemma-code-review.py
        # resolved internally (primary vs. fallback) — the Makefile only
        # ever reads whatever ended up in the aggregate's `model` field.
        self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-REV2"},
            env={"STUB_REVIEW_MODE": "pass", "STUB_REVIEW_MODEL": "gpt-oss:20b"},
        )

        receipt = self.read_receipt("T-REV2")
        self.assertEqual(receipt["reviewer"], "gpt-oss:20b")

    def test_ec1_missing_model_field_yields_unknown_marker_not_gemma(self):
        self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-REV3"},
            env={"STUB_REVIEW_MODE": "pass", "STUB_REVIEW_MODEL": "__omit__"},
        )

        receipt = self.read_receipt("T-REV3")
        self.assertEqual(receipt["reviewer"], "unknown-reviewer")
        self.assertNotEqual(receipt["reviewer"], "gemma")

    # -- GEG-2c: changed_paths carried into the receipt ---------------------

    def test_receipt_carries_changed_paths_from_aggregate(self):
        self.make_qa_gemma_review(
            {"GEMMA_REVIEW_TASK_ID": "T-CP1"},
            env={"STUB_REVIEW_MODE": "pass", "STUB_REVIEW_MODEL": "gemma"},
        )

        receipt = self.read_receipt("T-CP1")
        self.assertEqual(receipt["changed_paths"], ["src/lib.rs"])


STUB_PEER_WORKFLOW_REVIEW = '''#!/usr/bin/env python3
import argparse, json, sys

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--artifact")
    parser.add_argument("--content")
    args, _ = parser.parse_known_args()
    if args.content == "-":
        sys.stdin.read()
    with open(args.artifact, "w", encoding="utf-8") as f:
        json.dump({"verdict": "pass", "reviewer": "codex"}, f)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
'''


class QaPeerWorkflowReviewReceipt(MakefileHarness):
    """The peer target writes into the same receipt directory and is validated
    by the same closure gate, so GEG-2c's changed_paths requirement applies to
    it identically. Without this coverage the gemma target satisfies the gate
    while its sibling silently cannot."""

    def setUp(self):
        super().setUp()
        stub_path = self.root / "scripts" / "peer-workflow-review.py"
        stub_path.write_text(STUB_PEER_WORKFLOW_REVIEW, encoding="utf-8")
        stub_path.chmod(stub_path.stat().st_mode | stat.S_IEXEC)

    def make_qa_peer_review(self, extra_vars=None):
        args = ["make", "qa-peer-workflow-review"]
        for key, value in (extra_vars or {}).items():
            args.append(f"{key}={value}")
        return subprocess.run(
            args, cwd=self.root, capture_output=True, text=True, env=dict(os.environ), check=False
        )

    def test_peer_receipt_carries_non_empty_changed_paths(self):
        result = self.make_qa_peer_review({"PEER_REVIEW_TASK_ID": "T-PEER1"})

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        receipt = self.read_receipt("T-PEER1")
        self.assertEqual(receipt["changed_paths"], ["src/lib.rs"])
        self.assertEqual(receipt["reviewer"], "codex")

    def test_peer_receipt_satisfies_the_closure_gate_changed_paths_check(self):
        # Guards the actual regression: a post-cutover peer receipt must not
        # be failed by the GEG-2c validator for a field its own target never
        # emitted.
        self.make_qa_peer_review({"PEER_REVIEW_TASK_ID": "T-PEER2"})

        receipt = self.read_receipt("T-PEER2")
        self.assertIsInstance(receipt["changed_paths"], list)
        self.assertGreater(len(receipt["changed_paths"]), 0)


if __name__ == "__main__":
    unittest.main()
