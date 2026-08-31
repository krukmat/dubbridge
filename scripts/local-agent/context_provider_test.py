#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.dirname(HERE))

from ckg_adapter import GraphCandidate, extract_task_anchors, rank_candidates  # noqa: E402
from ckg_manifest import derive_worktree_identity  # noqa: E402
from context_budget import derive_invocation_budget  # noqa: E402
from context_provider import (  # noqa: E402
    CKGContextProvider,
    FallbackContextProvider,
    LegacyContextProvider,
)
import ollama_lifecycle  # noqa: E402


class Card:
    task_id = "S-CKG-T"
    spec = "Change `target_fn` in src/target.rs and preserve tests."
    allowed_paths = ["src/target.rs"]
    acceptance_tests = ["cargo test -p target"]
    capsule_hash = "capsule-1"


class FakeBoundaryError(RuntimeError):
    pass


class FakeBoundary:
    def check_path(self, path):
        if path != "src/target.rs":
            raise FakeBoundaryError(f"path outside allowed_paths: {path}")


class FakeFileTools:
    def __init__(self, contents):
        self.contents = contents

    def preload_context(self, allowed_paths):
        return [
            {
                "path": path,
                "missing": path not in self.contents,
                "content": self.contents.get(path),
            }
            for path in allowed_paths
        ]

    def read_checked(self, path):
        if path not in self.contents:
            raise FileNotFoundError(path)
        return self.contents[path]


class FakeAdapter:
    backend_name = "fake-ckg"

    def discover(self, task_text, worktree_dir):
        return {
            "project": "dubbridge",
            "anchors": extract_task_anchors(task_text),
            "candidates": rank_candidates(
                [
                    GraphCandidate(
                        path="src/target.rs", symbol="target_fn", priority=5
                    ),
                    GraphCandidate(
                        path="src/external.rs",
                        symbol="external_fn",
                        relation="CALLS",
                    ),
                ],
                extract_task_anchors(task_text),
            ),
        }

    def coverage(self, project, paths):
        return {"status": "verified", "raw": {}}


class PartialAdapter(FakeAdapter):
    def coverage(self, project, paths):
        return {"status": "partial", "raw": {"gaps": paths}}


class BrokenAdapter:
    backend_name = "broken"

    def discover(self, task_text, worktree_dir):
        raise RuntimeError("backend unavailable")


class ContextProviderTests(unittest.TestCase):
    def test_runtime_budget_tracks_num_ctx(self):
        common = dict(
            num_predict=8192,
            system_prompt="system",
            task_spec="task",
            allowed_paths=["src"],
            acceptance_tests=["test"],
        )
        large = derive_invocation_budget(num_ctx=32768, **common)
        small = derive_invocation_budget(num_ctx=16384, **common)
        self.assertGreater(large.retrieval_budget_tokens, small.retrieval_budget_tokens)

    def test_explicit_anchor_ranks_before_dependency(self):
        anchors = extract_task_anchors("edit `target_fn` in src/target.rs")
        ranked = rank_candidates(
            [
                GraphCandidate("src/dep.rs", symbol="dep", relation="CALLS"),
                GraphCandidate("src/target.rs", symbol="target_fn"),
            ],
            anchors,
        )
        self.assertEqual(ranked[0].path, "src/target.rs")
        self.assertEqual(ranked[0].priority, 0)

    def test_worktree_state_hash_changes_without_head_change(self):
        with tempfile.TemporaryDirectory() as root:
            subprocess.run(["git", "init", "-q", root], check=True)
            subprocess.run(
                ["git", "-C", root, "config", "user.email", "test@example.com"],
                check=True,
            )
            subprocess.run(
                ["git", "-C", root, "config", "user.name", "Test"], check=True
            )
            path = os.path.join(root, "file.txt")
            with open(path, "w", encoding="utf-8") as handle:
                handle.write("one\n")
            subprocess.run(["git", "-C", root, "add", "file.txt"], check=True)
            subprocess.run(
                ["git", "-C", root, "commit", "-qm", "initial"], check=True
            )
            clean = derive_worktree_identity(root)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write("two\n")
            dirty = derive_worktree_identity(root)
            self.assertEqual(clean.base_revision, dirty.base_revision)
            self.assertFalse(clean.dirty)
            self.assertTrue(dirty.dirty)
            self.assertNotEqual(clean.state_hash, dirty.state_hash)

    def _provider(self, root, adapter):
        current = "fn target_fn() { /* current worktree */ }\n"
        os.makedirs(os.path.join(root, "src"), exist_ok=True)
        with open(
            os.path.join(root, "src", "target.rs"), "w", encoding="utf-8"
        ) as handle:
            handle.write(current)
        tools = FakeFileTools({"src/target.rs": current})
        provider = CKGContextProvider(
            card=Card(),
            worktree_dir=root,
            boundary=FakeBoundary(),
            file_tools=tools,
            retrieval_budget_tokens=4096,
            budget_details={"retrieval_budget_tokens": 4096},
            adapter=adapter,
            boundary_error=FakeBoundaryError,
        )
        return provider, tools

    def test_ckg_provider_filters_scope_and_uses_current_source(self):
        with tempfile.TemporaryDirectory() as root:
            provider, _tools = self._provider(root, FakeAdapter())
            rendered = provider.render_initial()
            manifest = provider.manifest()
            self.assertIn("current worktree", rendered)
            self.assertNotIn("external_fn", rendered)
            self.assertEqual(manifest["selection"][0]["context_source"], "worktree")
            self.assertEqual(manifest["scope_gaps"][0]["path"], "src/external.rs")

    def test_partial_coverage_falls_back_to_legacy(self):
        with tempfile.TemporaryDirectory() as root:
            primary, tools = self._provider(root, PartialAdapter())
            provider = FallbackContextProvider(
                primary, LegacyContextProvider(Card(), tools)
            )
            rendered = provider.render_initial()
            self.assertIn("current worktree", rendered)
            self.assertIn("coverage is partial", provider.last_fallback_reason)
            self.assertEqual(provider.manifest()["graph"]["coverage"], "partial")

    def test_backend_failure_falls_back_to_legacy(self):
        tools = FakeFileTools({"src/target.rs": "fn target_fn() {}\n"})
        primary = CKGContextProvider(
            card=Card(),
            worktree_dir=".",
            boundary=FakeBoundary(),
            file_tools=tools,
            retrieval_budget_tokens=4096,
            budget_details={},
            adapter=BrokenAdapter(),
            boundary_error=FakeBoundaryError,
        )
        provider = FallbackContextProvider(
            primary, LegacyContextProvider(Card(), tools)
        )
        rendered = provider.render_initial()
        self.assertIn("target_fn", rendered)
        self.assertEqual(provider.last_fallback_reason, "backend unavailable")

    def test_unload_model_sends_keep_alive_zero(self):
        response = mock.MagicMock()
        response.__enter__.return_value = response
        response.__exit__.return_value = False
        response.read.return_value = b"{}"
        with mock.patch("urllib.request.urlopen", return_value=response) as urlopen:
            self.assertTrue(
                ollama_lifecycle.unload_model("http://localhost:11434", "model")
            )
        request = urlopen.call_args.args[0]
        self.assertIn(b'"keep_alive": 0', request.data)


if __name__ == "__main__":
    unittest.main()
