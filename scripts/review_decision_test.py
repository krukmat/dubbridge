#!/usr/bin/env python3
import json
import unittest

import review_decision as rd
import review_decision_shadow as shadow
import systemone_local_adapter as adapter


class _Response:
    def __init__(self, payload): self.payload = payload
    def __enter__(self): return self
    def __exit__(self, exc_type, exc, tb): return False
    def read(self): return json.dumps(self.payload).encode()


def response(probability=.99):
    return {"model":"nimble:9b-q4_K_M","answers":{
        "risk":{"type":"choice","choice":"low","probabilities":{"low":probability,"moderate":.005,"high":.003,"critical":.002},"confidence":.9},
        "evidence_complete":{"type":"noul","noul":probability},
        "scope":{"type":"choice","choice":"expected","probabilities":{"expected":probability,"out_of_scope":.005,"uncertain":.005},"confidence":.9},
        "failure_domain":{"type":"choice","choice":"none","probabilities":{"none":probability,"code":.002,"test":.002,"environment":.002,"infrastructure":.002,"security":.001,"unknown":.001},"confidence":.9},
        "suggested_review":{"type":"choice","choice":"none","probabilities":{"none":probability,"local":.004,"advanced":.003,"human":.003},"confidence":.9},
    },"usage":{"input_tokens":100,"output_tokens":5}}


def safe_state(rri=12):
    return rd.build_state(content="packet",phase="code",rri=rri,task_id="T",metadata={
        "checks":{"tests":"pass","contracts":"pass"},
        "deterministic":{"security_sensitive":False,"migration_change":False,"architecture_change":False,"dependency_sensitive":False}})


class DecisionContractTest(unittest.TestCase):
    def test_truncation_keeps_head_and_tail(self):
        state=rd.build_state(content="A"*100+"B"*100,phase="code",rri=1,task_id="T",max_chars=80)
        self.assertTrue(state["packet_truncated"]); self.assertIn("A",state["content"]); self.assertIn("B",state["content"])

    def test_request_shape_and_case_id(self):
        state=safe_state(); req=rd.build_systemone_request(state=state)
        self.assertEqual(req["model"],"nimble:9b-q4_K_M"); self.assertTrue(state["case_id"].startswith("T:code:")); self.assertIn("risk",req["questions"])

    def test_safe_shadow_candidate(self):
        state=safe_state(); decision=rd.normalize_systemone_response(response(),request=rd.build_systemone_request(state=state))
        self.assertTrue(rd.fast_path_eligibility(decision=decision,state=state)["eligible"])

    def test_missing_deterministic_evidence_fails_closed(self):
        state=rd.build_state(content="packet",phase="code",rri=12,task_id="T"); decision=rd.normalize_systemone_response(response())
        result=rd.fast_path_eligibility(decision=decision,state=state)
        self.assertFalse(result["eligible"]); self.assertIn("tests_not_confirmed_pass",result["reasons"])

    def test_non_low_never_fast_paths(self):
        state=safe_state(26); decision=rd.normalize_systemone_response(response())
        self.assertIn("rri_not_low",rd.fast_path_eligibility(decision=decision,state=state)["reasons"])

    def test_probability_threshold_and_invalid_choice(self):
        state=safe_state(); decision=rd.normalize_systemone_response(response(.90))
        self.assertFalse(rd.fast_path_eligibility(decision=decision,state=state)["eligible"])
        bad=response(); bad["answers"]["risk"]["choice"]="safe"
        with self.assertRaises(ValueError): rd.normalize_systemone_response(bad)


class AdapterTest(unittest.TestCase):
    def test_mocked_http_invocation(self):
        seen={}
        def opener(req,timeout): seen.update(url=req.full_url,body=json.loads(req.data),timeout=timeout); return _Response({"model":"nimble","answers":{}})
        result=adapter.invoke_systemone({"model":"nimble","state":"x","questions":{}},opener=opener,timeout=7)
        self.assertEqual(seen["url"],"http://localhost:11434/v1/systemone"); self.assertEqual(seen["timeout"],7); self.assertIn("latency_ms",result)

    def test_invalid_json_fails(self):
        class Bad(_Response):
            def read(self): return b"not-json"
        with self.assertRaises(RuntimeError): adapter.invoke_systemone({},opener=lambda req,timeout:Bad({}))


class ShadowEvidenceTest(unittest.TestCase):
    def test_ground_truth_top_level_and_nested(self):
        top=shadow.extract_ground_truth({"verdict":"FINDINGS","findings":[{"severity":"major"}]},"a.json")
        nested=shadow.extract_ground_truth({"message":{"content":json.dumps({"verdict":"FINDINGS","findings":[{"severity":"BLOCKING"}]})}},"b.json")
        self.assertTrue(top["ground_truth"]["critical"]); self.assertEqual(nested["ground_truth"]["max_severity"],"blocking")

    def test_capture_detects_case_mismatch(self):
        state=safe_state(); req=rd.build_systemone_request(state=state); norm={"case_id":"other","decision":{},"fast_path":{}}
        with self.assertRaises(ValueError): shadow.build_capture_row(req,norm,{"verdict":"PASS","findings":[]},"review.json")

    def test_evaluator_detects_critical_false_negative(self):
        row={"case_id":"bad","ground_truth":{"critical":True},"decision":{"answers":{"risk":{"choice":"low"},"suggested_review":{"choice":"none"}}},"fast_path":{"eligible":True},"latency_ms":20}
        result=shadow.evaluate([row]); self.assertEqual(result["critical_false_negatives"],1); self.assertFalse(result["promotion_gate"]["ready_for_policy_review"])

    def test_evaluator_safe_critical_set_can_pass(self):
        rows=[{"case_id":str(i),"ground_truth":{"critical":True},"decision":{"answers":{"risk":{"choice":"high"},"suggested_review":{"choice":"advanced"}}},"fast_path":{"eligible":False},"latency_ms":10+i} for i in range(20)]
        result=shadow.evaluate(rows); self.assertEqual(result["critical_escalation_recall"],1.0); self.assertTrue(result["promotion_gate"]["ready_for_policy_review"])


if __name__ == "__main__": unittest.main()
