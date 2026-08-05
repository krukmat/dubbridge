#!/usr/bin/env python3
"""Unit tests for scripts/antares/artifact_schema.py."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

_SCRIPT = Path(__file__).with_name("artifact_schema.py")
_SPEC = importlib.util.spec_from_file_location("antares_artifact_schema", _SCRIPT)
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_SCRIPT}")
_MOD = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _MOD
_SPEC.loader.exec_module(_MOD)

TerminalStateKind = _MOD.TerminalStateKind

Artifact = _MOD.Artifact
ArtifactSchemaError = _MOD.ValidationError
Budget = _MOD.Budget
Disposition = _MOD.Disposition
DispositionState = _MOD.DispositionState
Provenance = _MOD.Provenance
TraceRef = _MOD.TraceRef
SCHEMA_VERSION = _MOD.SCHEMA_VERSION
ALLOWED_TRACE_STORAGE_PREFIX = _MOD.ALLOWED_TRACE_STORAGE_PREFIX
artifact_from_dict = _MOD.artifact_from_dict
artifact_to_dict = _MOD.artifact_to_dict
compute_content_hash = _MOD.compute_content_hash
generate_example_artifacts = _MOD.generate_example_artifacts
validate_artifact = _MOD.validate_artifact
validate_supersede_chain = _MOD.validate_supersede_chain
verify_trace_ref_roundtrip = _MOD.verify_trace_ref_roundtrip
write_raw_trace = _MOD.write_raw_trace


def _provenance() -> Provenance:
    return Provenance(
        model_version="test-model",
        runtime_version="test-runtime",
        harness_version="test-harness",
        packet_hash="sha256:" + "a" * 64,
        snapshot_hash="sha256:" + "b" * 64,
    )


def _trace_ref(name: str = "example") -> TraceRef:
    return TraceRef(
        content_hash="sha256:" + "c" * 64,
        storage_uri=f"file://{ALLOWED_TRACE_STORAGE_PREFIX}{name}.trace",
        byte_length=10,
    )


class HappyPathTest(unittest.TestCase):
    def test_hp1_vulnerable_files_carries_trace_ref_and_no_raw_content(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_VULNERABLE_FILES,
            finding_id="f1",
            artifact_id="f1-r1",
            provenance=_provenance(),
            candidates=("src/vuln.py",),
        )
        validate_artifact(artifact)
        self.assertEqual(artifact.disposition.state, DispositionState.NEEDS_HUMAN_REVIEW)
        serialized = artifact_to_dict(artifact)
        self.assertNotIn("raw_stdout", serialized)
        self.assertNotIn("raw_stderr", serialized)

    def test_hp2_no_vulnerability_found_matches_positive_result_shape(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="f2",
            artifact_id="f2-r1",
            provenance=_provenance(),
        )
        validate_artifact(artifact)
        self.assertEqual(artifact.disposition.state, DispositionState.NEEDS_HUMAN_REVIEW)


class EdgeCaseTest(unittest.TestCase):
    def test_ec1_degraded_states_are_distinct_and_start_needs_human_review(self) -> None:
        degraded_kinds = (
            TerminalStateKind.SANDBOX_BUDGET_EXHAUSTED,
            TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT,
            TerminalStateKind.SANDBOX_OUTPUT_CAP_EXCEEDED,
        )
        seen_kinds = set()
        for kind in degraded_kinds:
            examples = generate_example_artifacts()
            artifact = examples[kind]
            self.assertEqual(artifact.kind, kind)
            self.assertEqual(artifact.disposition.state, DispositionState.NEEDS_HUMAN_REVIEW)
            seen_kinds.add(artifact.kind)
        self.assertEqual(len(seen_kinds), len(degraded_kinds))

    def test_ec2_rejects_raw_trace_alongside_populated_trace_ref(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
            finding_id="f3",
            artifact_id="f3-r1",
            provenance=_provenance(),
            argv=("ls",),
            exit_code=0,
            elapsed_seconds=1.0,
            trace_ref=_trace_ref(),
            raw_stdout="leaked raw content",
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "unredacted_trace_leak")

    def test_ec3_needs_human_review_is_schema_valid_not_a_violation(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_VULNERABLE_FILES,
            finding_id="f4",
            artifact_id="f4-r1",
            provenance=_provenance(),
            candidates=("a.py",),
            disposition=Disposition(state=DispositionState.NEEDS_HUMAN_REVIEW),
        )
        validate_artifact(artifact)  # must not raise

    def test_ec3_disposition_missing_reviewer_on_non_open_state_is_rejected(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_VULNERABLE_FILES,
            finding_id="f5",
            artifact_id="f5-r1",
            provenance=_provenance(),
            candidates=("a.py",),
            disposition=Disposition(state=DispositionState.ACCEPTED_NOW),
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "incomplete_disposition")

    def test_ec4_empty_candidates_with_submitted_vulnerable_files_is_rejected(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_VULNERABLE_FILES,
            finding_id="f6",
            artifact_id="f6-r1",
            provenance=_provenance(),
            candidates=(),
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "empty_candidates")

    def test_ec5_unrecognized_schema_version_is_rejected(self) -> None:
        artifact = Artifact(
            schema_version=999,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="f7",
            artifact_id="f7-r1",
            provenance=_provenance(),
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "unrecognized_schema_version")

    def test_ec6_chain_shares_finding_id_and_resolves_to_latest_revision(self) -> None:
        base = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="F",
            artifact_id="A1",
            provenance=_provenance(),
            supersedes=None,
        )
        revised = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="F",
            artifact_id="A2",
            provenance=_provenance(),
            supersedes="A1",
            disposition=Disposition(
                state=DispositionState.ACCEPTED_NOW, reviewer="alice", reviewed_at="2026-07-30T00:00:00Z"
            ),
        )
        head = validate_supersede_chain([base, revised])
        self.assertEqual(head.artifact_id, "A2")

    def test_ec6_rejects_finding_id_change_mid_chain(self) -> None:
        base = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="F",
            artifact_id="A1",
            provenance=_provenance(),
        )
        drifted = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="OTHER",
            artifact_id="A2",
            provenance=_provenance(),
            supersedes="A1",
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_supersede_chain([base, drifted])
        self.assertEqual(ctx.exception.code, "chain_finding_id_mismatch")

    def test_ec6_chain_rejects_an_individually_malformed_artifact(self) -> None:
        malformed = Artifact(
            schema_version=999,  # EC-5 violation
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="F",
            artifact_id="A1",
            provenance=_provenance(),
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_supersede_chain([malformed])
        self.assertEqual(ctx.exception.code, "unrecognized_schema_version")

    def test_ec6_single_artifact_chain_resolves_to_itself(self) -> None:
        only = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="F",
            artifact_id="A1",
            provenance=_provenance(),
        )
        head = validate_supersede_chain([only])
        self.assertEqual(head.artifact_id, "A1")

    def test_ec6_rejects_forked_chain_with_two_heads(self) -> None:
        base = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="F",
            artifact_id="A1",
            provenance=_provenance(),
        )
        fork_a = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="F",
            artifact_id="A2",
            provenance=_provenance(),
            supersedes="A1",
        )
        fork_b = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="F",
            artifact_id="A3",
            provenance=_provenance(),
            supersedes="A1",
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_supersede_chain([base, fork_a, fork_b])
        self.assertEqual(ctx.exception.code, "ambiguous_chain_head")

    def test_ec6_rejects_dangling_supersedes(self) -> None:
        orphan = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
            finding_id="F",
            artifact_id="A2",
            provenance=_provenance(),
            supersedes="DOES_NOT_EXIST",
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_supersede_chain([orphan])
        self.assertEqual(ctx.exception.code, "dangling_supersedes")

    def test_writer_hash_roundtrip(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            storage_root = Path(tmp)
            raw_bytes = b"raw sandbox stdout/stderr content"
            trace_ref = write_raw_trace(raw_bytes, storage_root, artifact_id="f8-r1")
            self.assertEqual(trace_ref.content_hash, compute_content_hash(raw_bytes))
            self.assertTrue(verify_trace_ref_roundtrip(trace_ref, storage_root))

    def test_writer_roundtrip_fails_if_bytes_are_tampered(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            storage_root = Path(tmp)
            trace_ref = write_raw_trace(b"original", storage_root, artifact_id="f9-r1")
            (storage_root / "f9-r1.trace").write_bytes(b"tampered")
            self.assertFalse(verify_trace_ref_roundtrip(trace_ref, storage_root))


class CategoryFieldTest(unittest.TestCase):
    def test_all_24_kinds_are_covered_and_partitioned(self) -> None:
        self.assertEqual(len(list(TerminalStateKind)), 24)
        self.assertEqual(
            _MOD.T2A_KINDS
            | _MOD.T2B_KINDS
            | _MOD.T2C1_KINDS
            | _MOD.T2C2_KINDS
            | _MOD.T2CLI_KINDS,
            frozenset(TerminalStateKind),
        )

    def test_t2c1_runtime_unavailable_forbids_trace_ref(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE,
            finding_id="f10",
            artifact_id="f10-r1",
            provenance=_provenance(),
            argv=("cmd",),
            elapsed_seconds=0.1,
            detail="runtime missing",
            trace_ref=_trace_ref("nope"),
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "unexpected_trace_ref")

    def test_t2c1_execution_complete_requires_trace_ref(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
            finding_id="f11",
            artifact_id="f11-r1",
            provenance=_provenance(),
            argv=("cmd",),
            exit_code=0,
            elapsed_seconds=1.0,
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "missing_trace_ref")

    def test_t2c2_teardown_unconfirmed_requires_grace_seconds(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED,
            finding_id="f12",
            artifact_id="f12-r1",
            provenance=_provenance(),
            elapsed_seconds=5.0,
            budget=Budget(limit=1.0, consumed=1.0, unit="commands"),
            trace_ref=_trace_ref("f12"),
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "missing_teardown_grace_seconds")

    def test_malformed_content_hash_is_rejected(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
            finding_id="f14",
            artifact_id="f14-r1",
            provenance=_provenance(),
            argv=("cmd",),
            exit_code=0,
            elapsed_seconds=1.0,
            trace_ref=TraceRef(
                content_hash="sha256:not-actually-hex",
                storage_uri=f"file://{ALLOWED_TRACE_STORAGE_PREFIX}f14.trace",
                byte_length=1,
            ),
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "invalid_content_hash")

    def test_storage_uri_outside_allowed_root_is_rejected(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
            finding_id="f13",
            artifact_id="f13-r1",
            provenance=_provenance(),
            argv=("cmd",),
            exit_code=0,
            elapsed_seconds=1.0,
            trace_ref=TraceRef(content_hash="sha256:" + "d" * 64, storage_uri="file://docs/leaked.trace", byte_length=1),
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "storage_uri_outside_allowed_root")

    def test_storage_uri_tilde_path_is_rejected(self) -> None:
        artifact = Artifact(
            schema_version=SCHEMA_VERSION,
            kind=TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
            finding_id="f15",
            artifact_id="f15-r1",
            provenance=_provenance(),
            argv=("cmd",),
            exit_code=0,
            elapsed_seconds=1.0,
            trace_ref=TraceRef(content_hash="sha256:" + "e" * 64, storage_uri="file://~/evil.trace", byte_length=1),
        )
        with self.assertRaises(ArtifactSchemaError) as ctx:
            validate_artifact(artifact)
        self.assertEqual(ctx.exception.code, "storage_uri_outside_allowed_root")


class ExampleArtifactsTest(unittest.TestCase):
    def test_all_24_examples_are_generated_and_schema_valid(self) -> None:
        examples = generate_example_artifacts()
        self.assertEqual(len(examples), len(list(TerminalStateKind)))
        for kind, artifact in examples.items():
            self.assertEqual(artifact.kind, kind)
            validate_artifact(artifact)  # must not raise

    def test_examples_serialize_and_round_trip_through_json_shape(self) -> None:
        examples = generate_example_artifacts()
        for artifact in examples.values():
            serialized = artifact_to_dict(artifact)
            reloaded = artifact_from_dict(serialized)
            validate_artifact(reloaded)
            self.assertEqual(reloaded.kind, artifact.kind)

    def test_examples_never_carry_raw_trace_content(self) -> None:
        examples = generate_example_artifacts()
        for artifact in examples.values():
            self.assertEqual(artifact.raw_stdout, "")
            self.assertEqual(artifact.raw_stderr, "")
            serialized = artifact_to_dict(artifact)
            self.assertNotIn("raw_stdout", serialized)
            self.assertNotIn("raw_stderr", serialized)


class CommittedExampleFixtureTest(unittest.TestCase):
    """EC-2: asserts against the actual committed example files on disk --
    not just in-memory objects -- that no raw trace body is ever present."""

    EXAMPLES_DIR = Path(__file__).with_name("examples")

    def test_committed_examples_directory_has_one_file_per_kind(self) -> None:
        files = sorted(self.EXAMPLES_DIR.glob("*.json"))
        kinds_from_files = {f.stem for f in files}
        kinds_from_enum = {kind.value for kind in TerminalStateKind}
        self.assertEqual(len(files), len(kinds_from_enum))
        self.assertEqual(kinds_from_files, kinds_from_enum)

    def test_committed_examples_contain_no_raw_trace_fields_and_validate(self) -> None:
        import json

        for path in sorted(self.EXAMPLES_DIR.glob("*.json")):
            with self.subTest(file=path.name):
                data = json.loads(path.read_text())
                self.assertNotIn("raw_stdout", data)
                self.assertNotIn("raw_stderr", data)
                self.assertNotIn("stdout", data)
                self.assertNotIn("stderr", data)
                artifact = artifact_from_dict(data)
                validate_artifact(artifact)  # must not raise
                self.assertEqual(artifact.disposition.state, DispositionState.NEEDS_HUMAN_REVIEW)


if __name__ == "__main__":
    unittest.main()
