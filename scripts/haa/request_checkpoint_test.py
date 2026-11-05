"""W3.T3.3 checkpoint persistence and failure semantics (local only)."""
from __future__ import annotations

from dataclasses import replace
import json
from pathlib import Path
import tempfile
import unittest

from scripts.haa.contracts import digest_json
from scripts.haa.request_checkpoint import (
    Checkpoint, CheckpointError, CheckpointStore,
    create_attempt, reconcile_status,
)


class Requester:
    def __init__(self):
        self.config = type("Config", (), {"executor_audience": "executor-1"})()
        self.calls = 0
        self.fail = False
        self.status = "PENDING"
        self.digest = ""

    def request_approval(self, action, *, ttl_ms):
        self.calls += 1
        self.digest = digest_json(action)
        if self.fail:
            raise TimeoutError("response lost after POST")
        return {"id": "remote-1", "actionDigest": self.digest,
                "intent": {"executorAudience": "executor-1"}, "state": "PENDING"}

    def get_request(self, request_id):
        return {"id": request_id, "actionDigest": self.digest,
                "intent": {"executorAudience": "executor-1"}, "state": self.status}


class CheckpointTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.store = CheckpointStore(Path(self.temp.name) / "haa")
        self.requester = Requester()
        self.action = {"schema": "haa.action.v1", "type": "dubbridge.task.start.v1",
                       "payload": {"taskId": "t"}}

    def issue(self):
        return create_attempt(self.requester, self.store, self.action,
                              plan_id="p", task_id="t", session_id="s", rri=55)

    def test_hp_persist_waiting_and_update_only_from_bound_status(self):
        item = self.issue()
        self.assertEqual(item.status, "WAITING_FOR_HUMAN")
        self.assertEqual(self.store.load(item.attempt_id), item)
        self.assertEqual(self.store.directory.stat().st_mode & 0o077, 0)
        self.assertEqual(self.store._file(item.attempt_id).stat().st_mode & 0o077, 0)
        self.assertNotIn("executor-secret", self.store._file(item.attempt_id).read_text())
        self.requester.status = "APPROVED"
        updated = reconcile_status(self.requester, self.store, item.attempt_id)
        self.assertEqual(updated.status, "APPROVED_NOT_CONSUMED")
        self.assertEqual(updated.execution_id, item.execution_id)
        with self.assertRaises(CheckpointError):
            self.store.transition(item.attempt_id, "WAITING_FOR_HUMAN", "VERIFIED_NO_SIDE_EFFECT")

    def test_ec_request_transport_uncertain_is_terminal_not_automatically_retried(self):
        self.requester.fail = True
        with self.assertRaises(TimeoutError):
            self.issue()
        files = list(self.store.directory.glob("*.json"))
        self.assertEqual(len(files), 1)
        item = self.store.load(files[0].stem)
        self.assertEqual(item.status, "REQUEST_UNCERTAIN")
        self.assertEqual(self.requester.calls, 1)
        self.assertEqual(reconcile_status(self.requester, self.store, item.attempt_id).status,
                         "REQUEST_UNCERTAIN")
        with self.assertRaises(CheckpointError):
            self.store.transition(item.attempt_id, "REQUEST_UNCERTAIN", "WAITING_FOR_HUMAN")

    def test_ec_reject_expire_and_mismatched_remote_binding_fail_closed(self):
        item = self.issue()
        self.requester.status = "REJECTED"
        self.assertEqual(reconcile_status(self.requester, self.store, item.attempt_id).status,
                         "REJECTED")
        self.assertEqual(reconcile_status(self.requester, self.store, item.attempt_id).status,
                         "REJECTED")

        other = self.issue()
        self.requester.digest = "sha256:wrong"
        with self.assertRaisesRegex(CheckpointError, "STATUS_DIGEST_MISMATCH"):
            reconcile_status(self.requester, self.store, other.attempt_id)
        self.assertEqual(self.store.load(other.attempt_id).status, "WAITING_FOR_HUMAN")

    def test_ec_invalid_checkpoint_and_stale_transition_denied(self):
        item = self.issue()
        with self.assertRaisesRegex(CheckpointError, "INVALID_OR_STALE_TRANSITION"):
            self.store.transition(item.attempt_id, "PREPARED", "APPROVED_NOT_CONSUMED")
        with self.assertRaisesRegex(CheckpointError, "INVALID_CHECKPOINT_CONTENT"):
            Checkpoint.parse({**item.record(), "rri": True})
        with self.assertRaisesRegex(CheckpointError, "INVALID_ATTEMPT_ID"):
            self.store.load("../other")


if __name__ == "__main__":
    unittest.main()
