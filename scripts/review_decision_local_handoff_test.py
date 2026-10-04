#!/usr/bin/env python3
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import review_decision_local_handoff as handoff
import review_decision_queue as queue


class Response:
    def __init__(self, value):
        self.value=value
    def __enter__(self): return self
    def __exit__(self,*_args): return False
    def read(self): return json.dumps(self.value).encode()


def opener_for(version, models):
    def opener(req, timeout):
        if req.full_url.endswith("/api/version"):
            return Response({"version":version})
        if req.full_url.endswith("/api/tags"):
            return Response({"models":[{"name":m} for m in models]})
        raise AssertionError(req.full_url)
    return opener


class LocalHandoffTest(unittest.TestCase):
    def test_normalizes_ollama_host_without_scheme(self):
        self.assertEqual(handoff.normalize_host("127.0.0.1:11434"), "http://127.0.0.1:11434")
        self.assertEqual(handoff.normalize_host("http://127.0.0.1:11434/"), "http://127.0.0.1:11434")

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)/"queue"

    def test_idle_is_not_an_error(self):
        result=handoff.preflight(root=self.root,opener=opener_for("x",["nimble:9b-q4_K_M"]))
        self.assertEqual(result["status"],"idle"); self.assertFalse(result["ready"])

    def test_missing_model_is_blocked_with_non_mutating_remediation(self):
        with patch.object(queue,"pending",return_value=[self.root/"pending"/"a.manifest.json"]):
            result=handoff.preflight(root=self.root,opener=opener_for("x",[]))
        self.assertEqual(result["status"],"blocked")
        self.assertEqual(result["remediation"],"ollama pull nimble:9b-q4_K_M")

    def test_ready_requires_pending_and_model(self):
        with patch.object(queue,"pending",return_value=[self.root/"pending"/"a.manifest.json"]):
            result=handoff.preflight(root=self.root,host="127.0.0.1:11434",opener=opener_for("x",["nimble:9b-q4_K_M"]))
        self.assertTrue(result["ready"]); self.assertEqual(result["pending"],1)
        self.assertEqual(result["host"],"http://127.0.0.1:11434")

    def test_run_uses_systemone_endpoint_only_after_preflight(self):
        with patch.object(handoff,"preflight",return_value={"ready":True,"status":"ready"}), \
             patch.object(queue,"run_local_batch",return_value={"attempted":2,"completed":2,"paired":2,"failures":[]}) as batch:
            result=handoff.run(root=self.root,host="http://localhost:11434",limit=2)
        self.assertEqual(result["batch"]["completed"],2)
        self.assertEqual(batch.call_args.kwargs["endpoint"],"http://localhost:11434/v1/systemone")

    def test_run_does_not_touch_batch_when_preflight_blocks(self):
        with patch.object(handoff,"preflight",return_value={"ready":False,"status":"blocked"}), \
             patch.object(queue,"run_local_batch") as batch:
            result=handoff.run(root=self.root)
        self.assertIsNone(result["batch"]); batch.assert_not_called()


    def test_smoke_validates_live_schema_without_shadow_metrics(self):
        response = {
            "model": "nimble:9b-q4_K_M",
            "answers": {
                "risk": {"type": "choice", "choice": "low", "probabilities": {"low": .99, "moderate": .005, "high": .003, "critical": .002}, "confidence": .9},
                "evidence_complete": {"type": "noul", "noul": .99},
                "scope": {"type": "choice", "choice": "expected", "probabilities": {"expected": .99, "out_of_scope": .005, "uncertain": .005}, "confidence": .9},
                "failure_domain": {"type": "choice", "choice": "none", "probabilities": {"none": .99, "code": .002, "test": .002, "environment": .002, "infrastructure": .002, "security": .001, "unknown": .001}, "confidence": .9},
                "suggested_review": {"type": "choice", "choice": "none", "probabilities": {"none": .99, "local": .004, "advanced": .003, "human": .003}, "confidence": .9},
            },
        }
        def fake_invoke(request, **_kwargs):
            return {
                "request_sha256": queue.request_sha256(request),
                "latency_ms": 42.5,
                "response": response,
            }
        with patch.object(handoff, "preflight", return_value={"model_present": True, "status": "idle"}), \
             patch.object(handoff.systemone_local_adapter, "invoke_systemone", side_effect=fake_invoke):
            result = handoff.smoke(root=self.root)
        self.assertEqual(result["status"], "pass")
        self.assertTrue(result["transport_schema_valid"])
        self.assertEqual(result["answer_count"], 5)
        self.assertTrue(result["metrics_excluded"])
        self.assertTrue(Path(result["receipt"]).exists())

    def test_smoke_does_not_invoke_model_when_missing(self):
        with patch.object(handoff, "preflight", return_value={"model_present": False, "status": "blocked"}), \
             patch.object(handoff.systemone_local_adapter, "invoke_systemone") as invoke:
            result = handoff.smoke(root=self.root)
        self.assertEqual(result["status"], "blocked")
        invoke.assert_not_called()


if __name__ == "__main__": unittest.main()
