#!/usr/bin/env python3
"""Integration-level Makefile coverage for M3 local-only reviewer enrichment."""

import json
import os
import shutil
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MAKEFILE_SOURCE = REPO_ROOT / "Makefile"

REVIEW_CONTEXT_STUB = r'''#!/usr/bin/env python3
import argparse, json, os, sys
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument("diff")
p.add_argument("--worktree")
p.add_argument("--task-id", default="")
p.add_argument("--metadata-out")
p.add_argument("--acceptance-file")
p.add_argument("--allowed-path", action="append", default=[])
args, _ = p.parse_known_args()
diff = Path(args.diff).read_text(encoding="utf-8")
Path(os.environ["REVIEW_CONTEXT_CALLED"]).write_text("called", encoding="utf-8")
if args.metadata_out:
    Path(args.metadata_out).write_text(json.dumps({"schema":"review-context-v1","status":"enriched"}), encoding="utf-8")
print("# Local reviewer packet")
print("LOCAL_CKG_CONTEXT_STUB")
print(f"task_id={args.task_id}")
print("allowed=" + ",".join(args.allowed_path))
if args.acceptance_file:
    print(Path(args.acceptance_file).read_text(encoding="utf-8"))
print(diff)
'''

GEMMA_STUB = r'''#!/usr/bin/env python3
import argparse, json, os, sys
p = argparse.ArgumentParser(); p.add_argument("packet", nargs="?"); p.add_argument("--out")
args, _ = p.parse_known_args()
packet = sys.stdin.read()
Path = __import__('pathlib').Path
Path(os.environ["GEMMA_PACKET_CAPTURE"]).write_text(packet, encoding="utf-8")
changed=[]
for line in packet.splitlines():
    if line.startswith("+++ b/"):
        changed.append(line[len("+++ b/"):].strip())
result={"status":"pass","summary":"stub","changed_paths":changed,"findings":[],"passes_run":1,"passes_succeeded":1,"model":"stub-reviewer"}
Path(args.out).write_text(json.dumps(result), encoding="utf-8")
'''

PARSE_STUB = "#!/usr/bin/env python3\nraise SystemExit(0)\n"

PEER_STUB = r'''#!/usr/bin/env python3
import argparse, json, sys
from pathlib import Path
p=argparse.ArgumentParser(); p.add_argument("--artifact"); p.add_argument("--content"); args,_=p.parse_known_args()
if args.content == "-": sys.stdin.read()
Path(args.artifact).write_text(json.dumps({"verdict":"pass","reviewer":"codex"}), encoding="utf-8")
'''


class MakefileM3Harness(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "scripts").mkdir()
        (self.root / "src").mkdir()
        (self.root / "docs" / "audit" / "gemma-evidence").mkdir(parents=True)
        shutil.copy(MAKEFILE_SOURCE, self.root / "Makefile")
        self._write_exec("scripts/review_context.py", REVIEW_CONTEXT_STUB)
        self._write_exec("scripts/gemma-code-review.py", GEMMA_STUB)
        self._write_exec("scripts/parse-review-findings.py", PARSE_STUB)
        self._write_exec("scripts/peer-workflow-review.py", PEER_STUB)
        self._run("git", "init")
        self._run("git", "config", "user.email", "m3@example.com")
        self._run("git", "config", "user.name", "M3 Test")
        (self.root / "src" / "lib.rs").write_text("fn base() {}\n", encoding="utf-8")
        self._run("git", "add", ".")
        self._run("git", "commit", "-m", "base")
        (self.root / "src" / "lib.rs").write_text(
            "fn base() {}\nfn changed() {}\n", encoding="utf-8"
        )
        (self.root / "task.md").write_text("HP-1: preserve changed behavior\n", encoding="utf-8")
        self.called = self.root / "review-context-called"
        self.capture = self.root / "gemma-packet.txt"

    def tearDown(self):
        self.tmp.cleanup()

    def _write_exec(self, path, content):
        target = self.root / path
        target.write_text(content, encoding="utf-8")
        target.chmod(target.stat().st_mode | stat.S_IEXEC)

    def _run(self, *args):
        r = subprocess.run(args, cwd=self.root, capture_output=True, text=True)
        if r.returncode != 0:
            self.fail(f"{args} failed\nstdout={r.stdout}\nstderr={r.stderr}")
        return r

    def test_local_gemma_target_enriches_packet_before_existing_reviewer(self):
        metadata = self.root / "context.json"
        env = {
            **os.environ,
            "REVIEW_CONTEXT_CALLED": str(self.called),
            "GEMMA_PACKET_CAPTURE": str(self.capture),
        }
        result = subprocess.run(
            [
                "make",
                "qa-gemma-review",
                "GEMMA_REVIEW_TASK_ID=M3-T3",
                "GEMMA_REVIEW_BASE=HEAD",
                "REVIEW_TASK_FILE=task.md",
                "REVIEW_CONTEXT_ALLOWED_PATHS=src",
                f"GEMMA_REVIEW_CONTEXT_METADATA={metadata}",
            ],
            cwd=self.root,
            capture_output=True,
            text=True,
            env=env,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertTrue(self.called.exists())
        packet = self.capture.read_text(encoding="utf-8")
        self.assertIn("LOCAL_CKG_CONTEXT_STUB", packet)
        self.assertIn("task_id=M3-T3", packet)
        self.assertIn("allowed=src", packet)
        self.assertIn("HP-1: preserve changed behavior", packet)
        self.assertIn("+++ b/src/lib.rs", packet)
        self.assertEqual(json.loads(metadata.read_text())["status"], "enriched")

    def test_cross_vendor_peer_target_does_not_invoke_review_context(self):
        peer_artifact = self.root / "peer.json"
        env = {
            **os.environ,
            "REVIEW_CONTEXT_CALLED": str(self.called),
            "GEMMA_PACKET_CAPTURE": str(self.capture),
        }
        result = subprocess.run(
            [
                "make",
                "qa-peer-workflow-review",
                "PEER_REVIEW_RRI=56",
                "PEER_REVIEW_PHASE=code",
                "PEER_REVIEW_CALLER=claude-code",
                "PEER_REVIEW_BASE=HEAD",
                f"PEER_REVIEW_ARTIFACT={peer_artifact}",
            ],
            cwd=self.root,
            capture_output=True,
            text=True,
            env=env,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertTrue(peer_artifact.exists())
        self.assertFalse(
            self.called.exists(),
            "cross-vendor peer path must not receive M3 CKG source enrichment",
        )


if __name__ == "__main__":
    unittest.main()
