#!/usr/bin/env python3
"""Unit tests for the local code-intelligence boundary."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from backend import BackendPayloadError, GraphResult
from context_gateway import build_artifacts


class BackendContractTests(unittest.TestCase):
    def test_hp1_valid_payload_normalizes_graph_result(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        self.assertEqual(graph.git_revision, "abc123")
        self.assertEqual(graph.anchors, ("crate::alpha::run",))
        self.assertEqual(graph.tests, ("tests::run_happy_path",))
        self.assertEqual(graph.source_fragments[0].classification, "task_local")

    def test_ec1_missing_revision_fails_closed(self) -> None:
        payload = _valid_payload()
        del payload["graph_revision"]
        with self.assertRaises(BackendPayloadError):
            GraphResult.from_mapping(payload)


class ContextGatewayTests(unittest.TestCase):
    def test_hp2_cloud_artifacts_are_deterministic_and_bounded(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        receipt_a, capsule_a = build_artifacts(
            task_id="CKG-T2",
            task_text="change alpha",
            target="cloud",
            graph=graph,
        )
        receipt_b, capsule_b = build_artifacts(
            task_id="CKG-T2",
            task_text="change alpha",
            target="cloud",
            graph=graph,
        )
        self.assertEqual(receipt_a, receipt_b)
        self.assertEqual(capsule_a, capsule_b)
        self.assertEqual(len(receipt_a["capsule_sha256"]), 64)
        self.assertEqual(len(receipt_a["receipt_sha256"]), 64)
        self.assertEqual(
            [item["classification"] for item in capsule_a["source_fragments"]],
            ["task_local"],
        )

    def test_ec2_secret_and_runtime_data_are_never_exported(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        for target in ("local", "cloud"):
            _, capsule = build_artifacts(
                task_id="CKG-EC2",
                task_text="inspect alpha",
                target=target,
                graph=graph,
            )
            exported = {item["classification"] for item in capsule["source_fragments"]}
            self.assertNotIn("secret", exported)
            self.assertNotIn("runtime_data", exported)

    def test_ec3_cloud_omits_cross_boundary_and_global_architecture(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        _, capsule = build_artifacts(
            task_id="CKG-EC3",
            task_text="inspect alpha",
            target="cloud",
            graph=graph,
        )
        fragment_classes = {
            item["classification"] for item in capsule["source_fragments"]
        }
        relationship_classes = {
            item.get("classification", "task_local")
            for item in capsule["relationships"]
        }
        self.assertNotIn("cross_boundary", fragment_classes)
        self.assertNotIn("global_architecture", fragment_classes)
        self.assertNotIn("cross_boundary", relationship_classes)
        self.assertNotIn("global_architecture", relationship_classes)

    def test_ec4_cli_invalid_payload_leaves_no_success_artifacts(self) -> None:
        script = Path(__file__).with_name("context_gateway.py")
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            invalid = root / "invalid.json"
            invalid.write_text('{"git_revision":"abc123"}', encoding="utf-8")
            output = root / "out"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(script),
                    "--task-id",
                    "CKG-EC4",
                    "--task",
                    "invalid payload",
                    "--backend-json",
                    str(invalid),
                    "--target",
                    "cloud",
                    "--output-dir",
                    str(output),
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertNotEqual(completed.returncode, 0)
            self.assertFalse((output / "context-receipt.json").exists())
            self.assertFalse((output / "context-capsule.json").exists())


def _valid_payload() -> dict[str, object]:
    return {
        "git_revision": "abc123",
        "graph_revision": "graph-001",
        "anchors": ["crate::alpha::run", "crate::alpha::run"],
        "files": ["crates/alpha/src/lib.rs"],
        "symbols": ["crate::alpha::run"],
        "relationships": [
            {
                "from": "crate::alpha::run",
                "to": "crate::beta::call",
                "kind": "calls",
                "classification": "task_local",
            },
            {
                "from": "auth::entry",
                "to": "db::session",
                "kind": "calls",
                "classification": "cross_boundary",
            },
            {
                "from": "root",
                "to": "all-crates",
                "kind": "contains",
                "classification": "global_architecture",
            },
        ],
        "tests": ["tests::run_happy_path"],
        "boundaries": ["auth", "storage"],
        "governance": ["ADR-023"],
        "source_fragments": [
            {
                "path": "crates/alpha/src/lib.rs",
                "start_line": 10,
                "end_line": 20,
                "content": "pub fn run() {}",
                "classification": "task_local",
            },
            {
                "path": "crates/auth/src/lib.rs",
                "start_line": 1,
                "end_line": 5,
                "content": "auth topology",
                "classification": "cross_boundary",
            },
            {
                "path": "docs/architecture.md",
                "start_line": 1,
                "end_line": 30,
                "content": "global topology",
                "classification": "global_architecture",
            },
            {
                "path": ".env",
                "start_line": 1,
                "end_line": 1,
                "content": "TOKEN=secret",
                "classification": "secret",
            },
            {
                "path": "tmp/runtime.json",
                "start_line": 1,
                "end_line": 1,
                "content": "runtime data",
                "classification": "runtime_data",
            },
        ],
    }


if __name__ == "__main__":
    unittest.main()
