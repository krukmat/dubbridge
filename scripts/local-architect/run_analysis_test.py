#!/usr/bin/env python3
"""Unit tests for scripts/local-architect/run_analysis.py."""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


_SCRIPT = Path(__file__).with_name("run_analysis.py")
_SPEC = importlib.util.spec_from_file_location("run_analysis", _SCRIPT)
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_SCRIPT}")
_MOD = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _MOD
_SPEC.loader.exec_module(_MOD)


def _packet_bytes() -> bytes:
    return json.dumps(
        {
            "work_item_id": "S-140",
            "objective": "Assess the next planning slice.",
            "constraints": ["Keep ADR authority with the primary agent."],
        },
        sort_keys=True,
    ).encode("utf-8")


class FakeFetcher:
    def __init__(self, tag_digest: str, response_text: str, thinking: str | None = None) -> None:
        self.tag_digest = tag_digest
        self.response_text = response_text
        self.thinking = thinking
        self.calls: list[tuple[str, dict[str, object], float, str]] = []

    def __call__(
        self, url: str, payload: dict[str, object], timeout_seconds: float, method: str = "POST"
    ) -> dict[str, object]:
        self.calls.append((url, payload, timeout_seconds, method))
        if url.endswith("/api/tags"):
            return {"models": [{"name": "qwen3.6:27b-q4_K_M", "digest": self.tag_digest}]}
        if url.endswith("/api/generate"):
            generate: dict[str, object] = {
                "response": self.response_text,
                "created_at": "2026-07-20T14:00:00Z",
                "done": True,
                "total_duration": 100,
                "load_duration": 10,
                "prompt_eval_count": 20,
                "prompt_eval_duration": 30,
                "eval_count": 40,
                "eval_duration": 50,
            }
            if self.thinking is not None:
                generate["thinking"] = self.thinking
            return generate
        raise AssertionError(f"Unexpected URL {url}")


class RunAnalysisTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmpdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmpdir.cleanup)
        self.root = Path(self.tmpdir.name)
        self.packet_path = self.root / "packet.json"
        self.packet_path.write_bytes(_packet_bytes())
        self.packet_sha = _MOD.sha256_bytes(self.packet_path.read_bytes())
        self.output_path = self.root / "artifact.json"
        self.base_config = _MOD.Config(
            packet_path=self.packet_path,
            expected_packet_sha256=self.packet_sha,
            output_path=self.output_path,
            model_tag="qwen3.6:27b-q4_K_M",
            expected_model_digest="a50eda8ed977ab48a12431878896b27ffd5cef552c17af3317d9623b939a7f1e",
            ollama_url="http://127.0.0.1:11434",
            timeout_seconds=5.0,
            temperature=0.0,
            num_predict=1024,
            overwrite=False,
        )

    def test_hp1_writes_success_artifact_for_valid_packet_and_model(self) -> None:
        fetcher = FakeFetcher(
            self.base_config.expected_model_digest,
            json.dumps(
                {
                    "objective": "Assess the next planning slice.",
                    "current_state": "T1 is complete and T2 is executing.",
                    "constraints": ["Keep ADR authority with the primary agent."],
                    "risks": ["The packet may omit a controlling ADR."],
                    "recommendations": ["Validate all cited repository facts before adoption."],
                    "open_questions": ["Is S-140 still the best first work item?"],
                    "evidence_gaps": ["No frozen packet revision is included yet."],
                    "claims": [{"statement": "T1 completed successfully.", "label": "SUPPORTED"}],
                }
            ),
        )

        artifact = _MOD.run_analysis(self.base_config, fetcher=fetcher)
        _MOD.write_json_atomic(self.output_path, artifact, overwrite=False)
        written = json.loads(self.output_path.read_text(encoding="utf-8"))

        self.assertTrue(written["success"])
        self.assertEqual(written["status"], "ok")
        self.assertEqual(written["packet"]["sha256"], self.packet_sha)
        self.assertEqual(written["model"]["resolved_digest"], self.base_config.expected_model_digest)
        self.assertEqual(written["prompt"]["version"], _MOD.PROMPT_VERSION)
        self.assertEqual(written["response"]["validated"]["claims"][0]["label"], "SUPPORTED")
        self.assertEqual(len(fetcher.calls), 2)
        self.assertTrue(fetcher.calls[0][0].endswith("/api/tags"))
        self.assertEqual(fetcher.calls[0][3], "GET")
        self.assertTrue(fetcher.calls[1][0].endswith("/api/generate"))
        self.assertEqual(fetcher.calls[1][3], "POST")

    def test_hp1b_sends_think_false_and_captures_thinking_provenance(self) -> None:
        thinking_text = "First I check the packet, then I weigh the constraints."
        fetcher = FakeFetcher(
            self.base_config.expected_model_digest,
            json.dumps(
                {
                    "objective": "Assess the next planning slice.",
                    "current_state": "T1 is complete and T2 is executing.",
                    "constraints": ["Keep ADR authority with the primary agent."],
                    "risks": ["The packet may omit a controlling ADR."],
                    "recommendations": ["Validate all cited repository facts before adoption."],
                    "open_questions": ["Is S-140 still the best first work item?"],
                    "evidence_gaps": ["No frozen packet revision is included yet."],
                    "claims": [{"statement": "T1 completed successfully.", "label": "SUPPORTED"}],
                }
            ),
            thinking=thinking_text,
        )

        artifact = _MOD.run_analysis(self.base_config, fetcher=fetcher)

        generate_call = next(c for c in fetcher.calls if c[0].endswith("/api/generate"))
        self.assertEqual(generate_call[1]["think"], False)
        self.assertTrue(artifact["generation"]["think_disabled"])
        self.assertTrue(artifact["generation"]["thinking_present"])
        self.assertEqual(
            artifact["generation"]["thinking_sha256"],
            _MOD.sha256_bytes(thinking_text.encode("utf-8")),
        )

    def test_hp1c_absent_thinking_field_records_null_provenance(self) -> None:
        fetcher = FakeFetcher(
            self.base_config.expected_model_digest,
            json.dumps(
                {
                    "objective": "Assess the next planning slice.",
                    "current_state": "state",
                    "constraints": ["one"],
                    "risks": ["two"],
                    "recommendations": ["three"],
                    "open_questions": ["four"],
                    "evidence_gaps": ["five"],
                    "claims": [{"statement": "A supported claim.", "label": "SUPPORTED"}],
                }
            ),
        )

        artifact = _MOD.run_analysis(self.base_config, fetcher=fetcher)

        self.assertTrue(artifact["generation"]["think_disabled"])
        self.assertFalse(artifact["generation"]["thinking_present"])
        self.assertIsNone(artifact["generation"]["thinking_sha256"])

    def test_hp2_fetch_json_sends_get_without_body_for_tags_endpoint(self) -> None:
        captured: dict[str, object] = {}
        original_urlopen = _MOD.request.urlopen

        class _FakeResponse:
            def __enter__(self) -> "_FakeResponse":
                return self

            def __exit__(self, *exc_info: object) -> None:
                return None

            def read(self) -> bytes:
                return json.dumps({"models": []}).encode("utf-8")

        def fake_urlopen(req: object, timeout: float) -> _FakeResponse:
            captured["method"] = req.get_method()
            captured["data"] = req.data
            return _FakeResponse()

        _MOD.request.urlopen = fake_urlopen
        try:
            _MOD.fetch_json("http://127.0.0.1:11434/api/tags", {}, 5.0, method="GET")
        finally:
            _MOD.request.urlopen = original_urlopen

        self.assertEqual(captured["method"], "GET")
        self.assertIsNone(captured["data"])

    def test_hp3_fetch_json_converts_socket_timeout_to_analysis_error(self) -> None:
        original_urlopen = _MOD.request.urlopen

        def fake_urlopen(req: object, timeout: float) -> None:
            raise _MOD.socket.timeout("timed out")

        _MOD.request.urlopen = fake_urlopen
        try:
            with self.assertRaises(_MOD.AnalysisError) as ctx:
                _MOD.fetch_json("http://127.0.0.1:11434/api/generate", {}, 5.0)
        finally:
            _MOD.request.urlopen = original_urlopen

        self.assertEqual(ctx.exception.code, "timeout")

    def test_hp4_generate_timeout_preserves_packet_and_model_provenance_in_failure_artifact(self) -> None:
        class TimeoutOnGenerateFetcher:
            def __init__(self, tag_digest: str) -> None:
                self.tag_digest = tag_digest

            def __call__(
                self, url: str, payload: dict[str, object], timeout_seconds: float, method: str = "POST"
            ) -> dict[str, object]:
                if url.endswith("/api/tags"):
                    return {"models": [{"name": "qwen3.6:27b-q4_K_M", "digest": self.tag_digest}]}
                if url.endswith("/api/generate"):
                    raise _MOD.AnalysisError("timeout", f"{url} did not respond within {timeout_seconds}s.")
                raise AssertionError(f"Unexpected URL {url}")

        fetcher = TimeoutOnGenerateFetcher(self.base_config.expected_model_digest)

        with self.assertRaises(_MOD.AnalysisError) as ctx:
            _MOD.run_analysis(self.base_config, fetcher=fetcher)

        artifact = _MOD.build_failure_artifact(self.base_config, ctx.exception)
        _MOD.write_json_atomic(self.output_path, artifact, overwrite=False)
        written = json.loads(self.output_path.read_text(encoding="utf-8"))

        self.assertEqual(ctx.exception.code, "timeout")
        self.assertEqual(written["packet"]["sha256"], self.packet_sha)
        self.assertEqual(written["model"]["resolved_digest"], self.base_config.expected_model_digest)

    def test_ec1_packet_hash_mismatch_stops_before_generation_and_writes_failure_artifact(self) -> None:
        fetcher = FakeFetcher(self.base_config.expected_model_digest, "{}")
        config = _MOD.Config(**{**vars(self.base_config), "expected_packet_sha256": "0" * 64})

        with self.assertRaises(_MOD.AnalysisError) as ctx:
            _MOD.run_analysis(config, fetcher=fetcher)

        artifact = _MOD.build_failure_artifact(config, ctx.exception)
        _MOD.write_json_atomic(self.output_path, artifact, overwrite=False)
        written = json.loads(self.output_path.read_text(encoding="utf-8"))

        self.assertEqual(ctx.exception.code, "packet_hash_mismatch")
        self.assertEqual(written["error"]["code"], "packet_hash_mismatch")
        self.assertFalse(fetcher.calls)

    def test_ec2_model_digest_mismatch_stops_before_generation_and_writes_failure_artifact(self) -> None:
        fetcher = FakeFetcher("different-digest", "{}")

        with self.assertRaises(_MOD.AnalysisError) as ctx:
            _MOD.run_analysis(self.base_config, fetcher=fetcher)

        artifact = _MOD.build_failure_artifact(self.base_config, ctx.exception)
        _MOD.write_json_atomic(self.output_path, artifact, overwrite=False)
        written = json.loads(self.output_path.read_text(encoding="utf-8"))

        self.assertEqual(ctx.exception.code, "model_digest_mismatch")
        self.assertEqual(written["model"]["resolved_digest"], "different-digest")
        self.assertEqual(len(fetcher.calls), 1)

    def test_ec3_invalid_response_schema_writes_failure_artifact(self) -> None:
        fetcher = FakeFetcher(
            self.base_config.expected_model_digest,
            json.dumps(
                {
                    "objective": "Assess the next planning slice.",
                    "current_state": "state",
                    "constraints": ["one"],
                    "risks": ["two"],
                    "recommendations": ["three"],
                    "open_questions": ["four"],
                    "evidence_gaps": ["five"],
                    "claims": [{"statement": "Unlabeled claim", "label": "FACT"}],
                }
            ),
        )

        with self.assertRaises(_MOD.AnalysisError) as ctx:
            _MOD.run_analysis(self.base_config, fetcher=fetcher)

        artifact = _MOD.build_failure_artifact(self.base_config, ctx.exception)
        _MOD.write_json_atomic(self.output_path, artifact, overwrite=False)
        written = json.loads(self.output_path.read_text(encoding="utf-8"))

        self.assertEqual(ctx.exception.code, "invalid_response")
        self.assertEqual(written["error"]["code"], "invalid_response")
        self.assertEqual(len(fetcher.calls), 2)

    def test_ec4_existing_output_without_overwrite_is_rejected_without_rewriting(self) -> None:
        self.output_path.write_text('{"status":"keep"}\n', encoding="utf-8")
        artifact = {"status": "ok"}

        with self.assertRaises(_MOD.AnalysisError) as ctx:
            _MOD.write_json_atomic(self.output_path, artifact, overwrite=False)

        self.assertEqual(ctx.exception.code, "output_exists")
        self.assertEqual(self.output_path.read_text(encoding="utf-8"), '{"status":"keep"}\n')


if __name__ == "__main__":
    unittest.main()
