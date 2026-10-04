#!/usr/bin/env python3
"""Integration tests for non-authoritative review-decision capture."""
import importlib.util
import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch

SCRIPTS=os.path.dirname(os.path.abspath(__file__))
if SCRIPTS not in sys.path: sys.path.insert(0,SCRIPTS)
PEER=os.path.join(SCRIPTS,"peer-workflow-review.py")
spec=importlib.util.spec_from_file_location("peer_workflow_review_shadow",PEER); peer=importlib.util.module_from_spec(spec); spec.loader.exec_module(peer)


class PeerReviewShadowCaptureTest(unittest.TestCase):
    def run_low(self, extra=None, enqueue_error=None):
        td=tempfile.TemporaryDirectory(); self.addCleanup(td.cleanup); artifact=os.path.join(td.name,"review.json")
        argv=["peer-workflow-review.py","--phase","code","--rri","12","--task-id","C6-T1","--artifact",artifact]+(extra or [])
        result={"reviewer":"gpt-oss:20b","phase":"code","verdict":"pass","summary":"authoritative pass","findings":[]}
        with patch("sys.argv",argv), patch.object(peer.gemma_local,"read_packet",return_value="diff content"), patch.object(peer,"_run_gpt_oss_review",return_value=(result,None)):
            if enqueue_error:
                with patch.object(peer.review_decision_queue,"enqueue",side_effect=enqueue_error): code=peer.main()
            else: code=peer.main()
        with open(artifact, encoding="utf-8") as stream:
            review = json.load(stream)
        return code, review

    def test_shadow_failure_never_changes_authoritative_review(self):
        code,review=self.run_low(enqueue_error=RuntimeError("shadow unavailable")); self.assertEqual(code,0); self.assertEqual(review["verdict"],"pass")

    def test_normal_review_queues_without_running_nimble(self):
        td=tempfile.TemporaryDirectory(); self.addCleanup(td.cleanup); root=os.path.join(td.name,"shadow")
        code,review=self.run_low(["--shadow-root",root]); self.assertEqual(code,0); self.assertEqual(review["verdict"],"pass")
        names=os.listdir(os.path.join(root,"pending")); self.assertEqual(len([x for x in names if x.endswith(".manifest.json")]),1); self.assertEqual(len([x for x in names if x.endswith(".request.json")]),1)

    def test_disable_switch_preserves_review(self):
        with patch.object(peer.review_decision_queue,"enqueue") as enqueue:
            code,review=self.run_low(["--no-shadow-capture"])
        self.assertEqual(code,0); self.assertEqual(review["verdict"],"pass"); enqueue.assert_not_called()


if __name__ == "__main__": unittest.main()
