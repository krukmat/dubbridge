#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.dirname(HERE))

import cli  # noqa: E402
import run_local_task as rlt  # noqa: E402
from ckg_adapter import CodebaseMemoryCLIAdapter  # noqa: E402
from ckg_manifest import derive_worktree_identity  # noqa: E402


class Completed:
    def __init__(self, stdout="{}", stderr="", returncode=0):
        self.stdout = stdout
        self.stderr = stderr
        self.returncode = returncode


def _init_git_repo(root):
    subprocess.run(["git", "init", "-q", root], check=True)
    subprocess.run(
        ["git", "-C", root, "config", "user.email", "test@example.com"], check=True
    )
    subprocess.run(
        ["git", "-C", root, "config", "user.name", "Test"], check=True
    )


class RuntimeContextTests(unittest.TestCase):
    def test_cli_builds_prompt_from_active_runtime_context_budget(self):
        with tempfile.TemporaryDirectory() as root:
            _init_git_repo(root)
            card_path = os.path.join(root, "card.json")
            out_path = os.path.join(root, "out.json")
            target = os.path.join(root, "hello.txt")
            with open(target, "w", encoding="utf-8") as handle:
                handle.write("hello\n")
            with open(card_path, "w", encoding="utf-8") as handle:
                json.dump(
                    {
                        "task_id": "S-CKG-RUNTIME",
                        "spec": "Keep hello.txt unchanged.",
                        "allowed_paths": ["hello.txt"],
                        "acceptance_tests": [],
                    },
                    handle,
                )
            subprocess.run(["git", "-C", root, "add", "hello.txt", "card.json"], check=True)
            subprocess.run(
                ["git", "-C", root, "commit", "-qm", "initial"], check=True
            )

            captured = {}
            real_builder = cli.build_tool_calling_system_prompt

            def capture_builder(num_ctx, num_predict):
                captured["num_ctx"] = num_ctx
                captured["num_predict"] = num_predict
                return real_builder(num_ctx, num_predict)

            def chat(messages):
                captured["messages"] = messages
                return {
                    "tool_calls": [
                        {"function": {"name": "finish", "arguments": {}}}
                    ]
                }

            with mock.patch.object(
                cli, "build_tool_calling_system_prompt", side_effect=capture_builder
            ):
                exit_code = rlt.main(
                    [
                        "--card",
                        card_path,
                        "--worktree",
                        root,
                        "--out",
                        out_path,
                        "--num-ctx",
                        "16384",
                        "--num-predict",
                        "4096",
                        "--context-provider",
                        "legacy",
                    ],
                    chat_fn=chat,
                    test_runner=lambda _root: {
                        "passed": True,
                        "output": "",
                        "commands": [],
                    },
                    boundary=rlt.NullBoundary(),
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(captured["num_ctx"], 16384)
            self.assertEqual(captured["num_predict"], 4096)
            self.assertIn(
                "Authorized source context", captured["messages"][0]["content"]
            )

    def test_worktree_hash_tracks_untracked_content_changes(self):
        with tempfile.TemporaryDirectory() as root:
            _init_git_repo(root)
            tracked = os.path.join(root, "tracked.txt")
            with open(tracked, "w", encoding="utf-8") as handle:
                handle.write("tracked\n")
            subprocess.run(["git", "-C", root, "add", "tracked.txt"], check=True)
            subprocess.run(
                ["git", "-C", root, "commit", "-qm", "initial"], check=True
            )
            untracked = os.path.join(root, "new.txt")
            with open(untracked, "w", encoding="utf-8") as handle:
                handle.write("one\n")
            first = derive_worktree_identity(root)
            with open(untracked, "w", encoding="utf-8") as handle:
                handle.write("two\n")
            second = derive_worktree_identity(root)
            self.assertEqual(first.base_revision, second.base_revision)
            self.assertNotEqual(first.state_hash, second.state_hash)

    def test_cbm_one_shot_uses_json_stdin(self):
        calls = []

        def runner(argv, **kwargs):
            calls.append((argv, kwargs))
            return Completed(stdout='{"results": []}')

        adapter = CodebaseMemoryCLIAdapter(binary="cbm", runner=runner)
        result = adapter._call(
            "search_graph",
            {"project": "dubbridge", "query": "playback"},
        )
        self.assertEqual(result, {"results": []})
        argv, kwargs = calls[0]
        self.assertEqual(argv, ["cbm", "cli", "--json", "search_graph"])
        self.assertEqual(
            json.loads(kwargs["input"]),
            {"project": "dubbridge", "query": "playback"},
        )

    def test_cbm_indexes_the_exact_task_worktree(self):
        calls = []

        def runner(argv, **kwargs):
            calls.append((argv, kwargs))
            tool = argv[-1]
            if tool == "index_repository":
                return Completed(stdout='{"project": "task-worktree"}')
            return Completed(stdout='{"results": []}')

        with tempfile.TemporaryDirectory() as root:
            adapter = CodebaseMemoryCLIAdapter(binary="cbm", runner=runner)
            project = adapter.ensure_project(root)

        self.assertEqual(project, "task-worktree")
        argv, kwargs = calls[0]
        self.assertEqual(argv, ["cbm", "cli", "--json", "index_repository"])
        self.assertEqual(
            json.loads(kwargs["input"])["repo_path"], os.path.realpath(root)
        )


if __name__ == "__main__":
    unittest.main()
