#!/usr/bin/env python3
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import review_decision_queue as queue
import review_decision_shadow


def safe_metadata():
    return {"checks":{"tests":"pass","contracts":"pass"},"deterministic":{"security_sensitive":False,"migration_change":False,"architecture_change":False,"dependency_sensitive":False}}


def safe_response(p=0.99):
    return {"model":"nimble:9b-q4_K_M","answers":{"risk":{"type":"choice","choice":"low","probabilities":{"low":p,"moderate":.004,"high":.003,"critical":.003},"confidence":.9},"evidence_complete":{"type":"noul","noul":p},"scope":{"type":"choice","choice":"expected","probabilities":{"expected":p,"out_of_scope":.005,"uncertain":.005},"confidence":.9},"failure_domain":{"type":"choice","choice":"none","probabilities":{"none":p,"code":.002,"test":.002,"environment":.002,"infrastructure":.002,"security":.001,"unknown":.001},"confidence":.9},"suggested_review":{"type":"choice","choice":"none","probabilities":{"none":p,"local":.004,"advanced":.003,"human":.003},"confidence":.9}},"usage":{"input_tokens":100,"output_tokens":5}}


class ReviewDecisionQueueTest(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)/"queue"
        self.review=Path(self.tmp.name)/"review.json"
        self.review.write_text(json.dumps({"verdict":"FINDINGS","phase":"code","reviewer":"gpt-oss:20b","findings":[{"severity":"blocking","detail":"real defect"}]}),encoding="utf-8")
        self.metadata=Path(self.tmp.name)/"metadata.json"; self.metadata.write_text(json.dumps(safe_metadata()),encoding="utf-8")

    def enqueue(self):
        return queue.enqueue(content="diff --git a/a b/a\n+change\n",phase="code",rri=12,task_id="T-1",review_artifact=str(self.review),metadata_path=str(self.metadata),root=self.root)

    def test_enqueue_is_idempotent_and_model_free(self):
        a=self.enqueue(); b=self.enqueue(); self.assertEqual(a["case_id"],b["case_id"]); self.assertEqual(a["request_sha256"],b["request_sha256"])
        request=queue._json(self.root/a["request_file"]); self.assertEqual(queue.request_sha256(request),a["request_sha256"])

    def test_ingest_rejects_response_bound_to_another_request(self):
        m=self.enqueue(); response=Path(self.tmp.name)/"response.json"; response.write_text(json.dumps({"request_sha256":"0"*64,"response":safe_response()}))
        with self.assertRaisesRegex(ValueError,"request_sha256 mismatch"): queue.ingest(manifest_path=m["manifest_file"],response_path=response)

    def test_pair_exposes_critical_false_negative(self):
        m=self.enqueue(); response=Path(self.tmp.name)/"response.json"; response.write_text(json.dumps({"request_sha256":m["request_sha256"],"latency_ms":21.5,"response":safe_response()}))
        normalized=queue.ingest(manifest_path=m["manifest_file"],response_path=response); self.assertTrue(normalized["fast_path"]["eligible"])
        paired=queue.pair(manifest_path=m["manifest_file"]); rows=[json.loads(x) for x in Path(paired["dataset_file"]).read_text().splitlines() if x.strip()]
        metrics=review_decision_shadow.evaluate(rows); self.assertEqual(metrics["critical_false_negatives"],1); self.assertFalse(metrics["promotion_gate"]["ready_for_policy_review"])

    def test_pair_is_idempotent(self):
        m=self.enqueue(); response=Path(self.tmp.name)/"response.json"; response.write_text(json.dumps({"request_sha256":m["request_sha256"],"response":safe_response()}))
        queue.ingest(manifest_path=m["manifest_file"],response_path=response); a=queue.pair(manifest_path=m["manifest_file"]); b=queue.pair(manifest_path=m["manifest_file"])
        self.assertTrue(a["appended"]); self.assertFalse(b["appended"])

    def test_adapter_binds_response_to_request_hash(self):
        request={"model":"nimble","state":{"case_id":"x"},"questions":{}}
        class Response:
            def __enter__(self): return self
            def __exit__(self,*_args): return False
            def read(self): return json.dumps({"model":"nimble","answers":{}}).encode()
        result=queue.local_adapter.invoke_systemone(request,opener=lambda req,timeout:Response(),timeout=3)
        self.assertEqual(result["request_sha256"],queue.request_sha256(request))

    def test_run_local_batch_is_explicit_local_step(self):
        m=self.enqueue()
        def fake(request,**_kwargs): return {"request_sha256":queue.request_sha256(request),"latency_ms":7,"response":safe_response()}
        with patch.object(queue.local_adapter,"invoke_systemone",side_effect=fake): result=queue.run_local_batch(root=self.root)
        self.assertEqual(result["completed"],1); self.assertEqual(result["paired"],1); self.assertEqual(result["failures"],[])
        self.assertEqual(queue._json(Path(m["manifest_file"]))["status"],"paired")


    def test_run_local_batch_leaves_intermediate_review_unpaired(self):
        self.review.write_text(json.dumps({
            "verdict": "awaiting_fallback_selection",
            "phase": "code",
            "reviewer": "d14",
            "findings": [],
        }), encoding="utf-8")
        manifest = self.enqueue()
        def fake(request, **_kwargs):
            return {
                "request_sha256": queue.request_sha256(request),
                "latency_ms": 8,
                "response": safe_response(),
            }
        with patch.object(queue.local_adapter, "invoke_systemone", side_effect=fake):
            result = queue.run_local_batch(root=self.root)
        self.assertEqual(result["completed"], 1)
        self.assertEqual(result["paired"], 0)
        self.assertEqual(result["failures"], [])
        self.assertEqual(queue._json(Path(manifest["manifest_file"]))["status"], "decision_ready")


if __name__ == "__main__": unittest.main()
