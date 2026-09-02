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
from context_gateway import (
    ContextPolicyError,
    build_artifacts,
    build_expansion_artifacts,
)


class BackendContractTests(unittest.TestCase):
    def test_hp1_valid_payload_normalizes_graph_result(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        self.assertEqual(graph.git_revision, "abc123")
        self.assertEqual(graph.anchors[0], "crate::alpha::run")
        self.assertIn("tests::run_happy_path", graph.tests)
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
            expected_git_revision="abc123",
        )
        receipt_b, capsule_b = build_artifacts(
            task_id="CKG-T2",
            task_text="change alpha",
            target="cloud",
            graph=graph,
            expected_git_revision="abc123",
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
                expected_git_revision="abc123",
            )
            exported = {item["classification"] for item in capsule["source_fragments"]}
            serialized = json.dumps(capsule)
            self.assertNotIn("secret", exported)
            self.assertNotIn("runtime_data", exported)
            self.assertNotIn("TOKEN=secret", serialized)
            self.assertNotIn("runtime data", serialized)

    def test_ec3_cloud_omits_cross_boundary_and_global_architecture(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        _, capsule = build_artifacts(
            task_id="CKG-EC3",
            task_text="inspect alpha",
            target="cloud",
            graph=graph,
            expected_git_revision="abc123",
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
                    "--expected-git-revision",
                    "abc123",
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

    def test_hp41_matching_revision_produces_artifacts(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        receipt, capsule = build_artifacts(
            task_id="M4-T1",
            task_text="fresh graph",
            target="cloud",
            graph=graph,
            expected_git_revision="abc123",
        )
        self.assertEqual(receipt["repository"]["git_revision"], "abc123")
        self.assertEqual(capsule["repository"]["git_revision"], "abc123")

    def test_ec41_stale_graph_is_rejected(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        with self.assertRaises(ContextPolicyError) as caught:
            build_artifacts(
                task_id="M4-T1",
                task_text="stale graph",
                target="cloud",
                graph=graph,
                expected_git_revision="different",
            )
        self.assertIn("expected git revision different", str(caught.exception))
        self.assertIn("received abc123", str(caught.exception))

    def test_hp42_cloud_exports_only_justified_metadata(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        _, capsule = build_artifacts(
            task_id="M4-T2",
            task_text="bounded alpha",
            target="cloud",
            graph=graph,
            expected_git_revision="abc123",
        )
        self.assertEqual(capsule["files"], ["crates/alpha/src/lib.rs"])
        self.assertEqual(capsule["symbols"], ["crate::alpha::run"])
        self.assertEqual(capsule["anchors"], ["crate::alpha::run"])
        self.assertEqual(capsule["tests"], ["tests::run_happy_path"])
        self.assertNotIn("crates/auth/src/lib.rs", capsule["files"])
        self.assertNotIn("auth::entry", capsule["symbols"])

    def test_ec42_unrelated_metadata_is_omitted_from_cloud(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        _, capsule = build_artifacts(
            task_id="M4-T2",
            task_text="bounded alpha",
            target="cloud",
            graph=graph,
            expected_git_revision="abc123",
        )
        serialized = json.dumps(capsule)
        self.assertNotIn("auth::entry", serialized)
        self.assertNotIn("tests::auth_topology", serialized)
        self.assertNotIn("ADR-023", serialized)

    def test_ec43_mislabeled_unsafe_fragment_is_still_denied(self) -> None:
        payload = _valid_payload()
        payload["source_fragments"].append(
            {
                "path": ".env.local",
                "start_line": 1,
                "end_line": 1,
                "content": "GITHUB_TOKEN=should-not-export",
                "classification": "task_local",
            }
        )
        payload["files"].append(".env.local")
        graph = GraphResult.from_mapping(payload)
        _, capsule = build_artifacts(
            task_id="M4-T2",
            task_text="mislabeled secret",
            target="cloud",
            graph=graph,
            expected_git_revision="abc123",
        )
        serialized = json.dumps(capsule)
        self.assertNotIn(".env.local", serialized)
        self.assertNotIn("should-not-export", serialized)

    def test_local_target_remains_richer_than_cloud(self) -> None:
        graph = GraphResult.from_mapping(_valid_payload())
        _, local_capsule = build_artifacts(
            task_id="M4-T2",
            task_text="compare target policy",
            target="local",
            graph=graph,
            expected_git_revision="abc123",
        )
        _, cloud_capsule = build_artifacts(
            task_id="M4-T2",
            task_text="compare target policy",
            target="cloud",
            graph=graph,
            expected_git_revision="abc123",
        )
        self.assertGreater(len(local_capsule["files"]), len(cloud_capsule["files"]))
        self.assertIn("crates/auth/src/lib.rs", local_capsule["files"])
        self.assertNotIn(".env", local_capsule["files"])
        self.assertNotIn("tmp/runtime.json", local_capsule["files"])


class ExpansionTests(unittest.TestCase):
    def test_hp43_bounded_expansion_adds_allowed_context(self) -> None:
        base_graph = GraphResult.from_mapping(_base_payload())
        base_receipt, _ = build_artifacts(
            task_id="M4-T3",
            task_text="base",
            target="cloud",
            graph=base_graph,
            expected_git_revision="abc123",
        )
        expanded_graph = GraphResult.from_mapping(_expanded_payload())
        receipt, capsule = build_expansion_artifacts(
            task_id="M4-T3",
            task_text="base",
            target="cloud",
            graph=expanded_graph,
            expected_git_revision="abc123",
            base_receipt=base_receipt,
            reason="need adjacent helper",
        )
        self.assertIn("crates/alpha/src/helper.rs", capsule["files"])
        self.assertEqual(receipt["expansions"][-1]["decision"], "allow")
        self.assertEqual(
            receipt["expansions"][-1]["base_receipt_sha256"],
            base_receipt["receipt_sha256"],
        )

    def test_ec44_forbidden_expansion_cannot_bypass_cloud_policy(self) -> None:
        base_graph = GraphResult.from_mapping(_base_payload())
        base_receipt, _ = build_artifacts(
            task_id="M4-T3",
            task_text="base",
            target="cloud",
            graph=base_graph,
            expected_git_revision="abc123",
        )
        payload = _base_payload()
        payload["source_fragments"].append(
            {
                "path": "docs/architecture.md",
                "start_line": 1,
                "end_line": 5,
                "content": "global topology",
                "classification": "global_architecture",
            }
        )
        expanded_graph = GraphResult.from_mapping(payload)
        receipt, capsule = build_expansion_artifacts(
            task_id="M4-T3",
            task_text="base",
            target="cloud",
            graph=expanded_graph,
            expected_git_revision="abc123",
            base_receipt=base_receipt,
            reason="give me global topology",
        )
        self.assertNotIn("global topology", json.dumps(capsule))
        self.assertEqual(receipt["expansions"][-1]["decision"], "deny")

    def test_ec45_expansion_with_different_graph_revision_fails_closed(self) -> None:
        base_graph = GraphResult.from_mapping(_base_payload())
        base_receipt, _ = build_artifacts(
            task_id="M4-T3",
            task_text="base",
            target="cloud",
            graph=base_graph,
            expected_git_revision="abc123",
        )
        payload = _expanded_payload()
        payload["graph_revision"] = "graph-other"
        expanded_graph = GraphResult.from_mapping(payload)
        with self.assertRaises(ContextPolicyError):
            build_expansion_artifacts(
                task_id="M4-T3",
                task_text="base",
                target="cloud",
                graph=expanded_graph,
                expected_git_revision="abc123",
                base_receipt=base_receipt,
                reason="need adjacent helper",
            )


def _base_payload() -> dict[str, object]:
    payload = _valid_payload()
    payload["files"] = ["crates/alpha/src/lib.rs"]
    payload["symbols"] = ["crate::alpha::run"]
    payload["anchors"] = ["crate::alpha::run"]
    payload["tests"] = ["tests::run_happy_path"]
    payload["boundaries"] = []
    payload["governance"] = []
    payload["relationships"] = [
        {
            "from": "crate::alpha::run",
            "to": "crate::alpha::run",
            "kind": "self",
            "classification": "task_local",
        }
    ]
    payload["source_fragments"] = [
        {
            "path": "crates/alpha/src/lib.rs",
            "start_line": 10,
            "end_line": 20,
            "content": "pub fn run() {}",
            "classification": "task_local",
        }
    ]
    return payload


def _expanded_payload() -> dict[str, object]:
    payload = _base_payload()
    payload["files"].append("crates/alpha/src/helper.rs")
    payload["symbols"].append("crate::alpha::helper")
    payload["tests"].append("tests::helper_happy_path")
    payload["relationships"].append(
        {
            "from": "crate::alpha::run",
            "to": "crate::alpha::helper",
            "kind": "calls",
            "classification": "task_local",
        }
    )
    payload["source_fragments"].append(
        {
            "path": "crates/alpha/src/helper.rs",
            "start_line": 1,
            "end_line": 5,
            "content": "pub fn helper() {}",
            "classification": "task_local",
        }
    )
    return payload


def _valid_payload() -> dict[str, object]:
    return {
        "git_revision": "abc123",
        "graph_revision": "graph-001",
        "anchors": ["crate::alpha::run", "auth::entry"],
        "files": [
            "crates/alpha/src/lib.rs",
            "crates/auth/src/lib.rs",
            ".env",
            "tmp/runtime.json",
        ],
        "symbols": ["crate::alpha::run", "auth::entry"],
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
        "tests": ["tests::run_happy_path", "tests::auth_topology"],
        "boundaries": ["auth", "storage"],
        "governance": ["ADR-023", "ADR-999"],
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
