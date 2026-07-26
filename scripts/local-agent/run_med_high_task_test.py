#!/usr/bin/env python3
"""Tests for run_med_high_task.py (ADR-038 T4): supervisor + evidence bundle."""

import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import med_high_gate
import run_med_high_task as _MOD

CARD_HASH = "a" * 64


def _write_json(path, data):
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f)


def _write_text(path, text):
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def _refinement_artifact(route_recommendation="GO_LOCAL", **overrides):
    artifact = {
        "success": True,
        "profile": med_high_gate.MED_HIGH_PROFILE,
        "packet": {"sha256": CARD_HASH},
        "model": {
            "tag": med_high_gate.REQUIRED_MODEL_TAG,
            "resolved_digest": "sha256:deadbeef",
            "expected_digest": "sha256:deadbeef",
        },
        "response": {
            "validated": {
                "route_recommendation": route_recommendation,
                "summary": "refined",
            }
        },
    }
    artifact.update(overrides)
    return artifact


def _primary_receipt(refinement_artifact, decision="GO_LOCAL", **overrides):
    receipt = {
        "primary_id": "claude-code",
        "decision": decision,
        "rationale": "matches architect recommendation",
        "timestamp": "2026-07-26T00:00:00Z",
        "card_hash": CARD_HASH,
        "refinement_artifact_sha256": med_high_gate.sha256_of(refinement_artifact),
    }
    receipt.update(overrides)
    return receipt


def _card(tmp_dir, **overrides):
    data = {
        "task_id": "T-MEDHIGH-1",
        "spec": "Do the bounded thing.",
        "acceptance_tests": ["true"],
        "allowed_paths": ["src/lib.rs"],
    }
    data.update(overrides)
    path = os.path.join(tmp_dir, "card.json")
    _write_json(path, data)
    return path


class _FakeProcess:
    """Stand-in for subprocess.Popen used by run_supervised_runner tests."""

    def __init__(self, pid=4242, wait_raises=None, returncode=0, write_out=None):
        self.pid = pid
        self._wait_raises = wait_raises
        self.returncode = returncode
        self._write_out = write_out
        self._wait_calls = 0

    def wait(self, timeout=None):
        self._wait_calls += 1
        if self._wait_raises is not None and self._wait_calls == 1:
            raise self._wait_raises
        if self._write_out:
            self._write_out()
        return self.returncode


class DecideRouteTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def _paths(self, refinement, receipt):
        r_path = os.path.join(self.tmp.name, "refinement.json")
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(r_path, refinement)
        _write_json(p_path, receipt)
        return r_path, p_path

    def test_hp1_go_local_both_sides_resolves_go_local(self):
        refinement = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(refinement, "GO_LOCAL")
        r_path, p_path = self._paths(refinement, receipt)
        decision = _MOD.decide_route(
            refinement_artifact_path=r_path, primary_receipt_path=p_path,
            card_hash=CARD_HASH, rri=50,
        )
        self.assertEqual(decision.route, med_high_gate.ROUTE_GO_LOCAL)

    def test_hp2_architect_cloud_required_never_upgraded(self):
        refinement = _refinement_artifact("CLOUD_REQUIRED")
        receipt = _primary_receipt(refinement, "GO_LOCAL")
        r_path, p_path = self._paths(refinement, receipt)
        decision = _MOD.decide_route(
            refinement_artifact_path=r_path, primary_receipt_path=p_path,
            card_hash=CARD_HASH, rri=50,
        )
        self.assertEqual(decision.route, med_high_gate.ROUTE_CLOUD_REQUIRED)


class RunSupervisedRunnerTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_hp1_clean_exit_within_wall_clock_reports_runner_exited(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")

        def fake_popen(argv, start_new_session=None):
            self.assertTrue(start_new_session)
            self.assertIn("run_local_task.py", argv[1])
            self.assertIn("--model", argv)
            return _FakeProcess(returncode=0)

        result = _MOD.run_supervised_runner(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            model="qwen3.6:35b-a3b", popen_fn=fake_popen,
        )
        self.assertEqual(result["status"], "runner_exited")
        self.assertEqual(result["returncode"], 0)

    def test_ec1_timeout_kills_process_group_not_just_pid(self):
        killpg_calls = []

        def fake_getpgid(pid):
            return pid

        def fake_killpg(pgid, sig):
            killpg_calls.append((pgid, sig))

        import subprocess as _subprocess

        def fake_popen(argv, start_new_session=None):
            return _FakeProcess(pid=777, wait_raises=_subprocess.TimeoutExpired(cmd=argv, timeout=1))

        orig_getpgid, orig_killpg = os.getpgid, os.killpg
        os.getpgid = fake_getpgid
        os.killpg = fake_killpg
        try:
            card_path = _card(self.tmp.name)
            out_path = os.path.join(self.tmp.name, "out.json")
            result = _MOD.run_supervised_runner(
                card_path=card_path, worktree=self.tmp.name, out_path=out_path,
                model="qwen3.6:35b-a3b", wall_clock_seconds=1, popen_fn=fake_popen,
            )
        finally:
            os.getpgid = orig_getpgid
            os.killpg = orig_killpg

        self.assertEqual(result["status"], "wall_clock_exceeded")
        self.assertEqual(len(killpg_calls), 1)
        self.assertEqual(killpg_calls[0], (777, _MOD.signal.SIGKILL))

    def test_ec1_timeout_preserves_whatever_out_path_already_has(self):
        import subprocess as _subprocess

        def fake_popen(argv, start_new_session=None):
            return _FakeProcess(wait_raises=_subprocess.TimeoutExpired(cmd=argv, timeout=1))

        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        checkpoint = {"status": "in_progress", "turn": 3, "transcript": []}
        _write_json(out_path, checkpoint)

        orig_getpgid, orig_killpg = os.getpgid, os.killpg
        os.getpgid = lambda pid: pid
        os.killpg = lambda pgid, sig: None
        try:
            _MOD.run_supervised_runner(
                card_path=card_path, worktree=self.tmp.name, out_path=out_path,
                model="qwen3.6:35b-a3b", wall_clock_seconds=1, popen_fn=fake_popen,
            )
        finally:
            os.getpgid = orig_getpgid
            os.killpg = orig_killpg

        with open(out_path, encoding="utf-8") as f:
            on_disk = json.load(f)
        self.assertEqual(on_disk, checkpoint)

    def test_ec1c_killpg_permission_error_fails_closed_not_crash(self):
        import subprocess as _subprocess

        def fake_getpgid(pid):
            return pid

        def fake_killpg(pgid, sig):
            raise PermissionError("not permitted")

        def fake_popen(argv, start_new_session=None):
            return _FakeProcess(wait_raises=_subprocess.TimeoutExpired(cmd=argv, timeout=1))

        orig_getpgid, orig_killpg = os.getpgid, os.killpg
        os.getpgid = fake_getpgid
        os.killpg = fake_killpg
        try:
            card_path = _card(self.tmp.name)
            out_path = os.path.join(self.tmp.name, "out.json")
            result = _MOD.run_supervised_runner(
                card_path=card_path, worktree=self.tmp.name, out_path=out_path,
                model="qwen3.6:35b-a3b", wall_clock_seconds=1, popen_fn=fake_popen,
            )
        finally:
            os.getpgid = orig_getpgid
            os.killpg = orig_killpg

        self.assertEqual(result["status"], "wall_clock_exceeded")
        self.assertIn("process group kill failed", result["reason"])

    def test_ec1b_failed_to_spawn_reports_transport_error(self):
        def fake_popen(argv, start_new_session=None):
            raise OSError("no such file")

        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        result = _MOD.run_supervised_runner(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            model="qwen3.6:35b-a3b", popen_fn=fake_popen,
        )
        self.assertEqual(result["status"], "transport_error")


class BuildEvidenceBundleTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_hp_bundle_contains_all_eleven_sections(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, {
            "status": "budget_exhausted",
            "task_id": "T-MEDHIGH-1",
            "transcript": [],
            "model": "qwen3.6:35b-a3b",
        })
        refinement = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(refinement, "GO_LOCAL")
        r_path = os.path.join(self.tmp.name, "refinement.json")
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(r_path, refinement)
        _write_json(p_path, receipt)
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="total_turns_exhausted", refinement_artifact_path=r_path,
            primary_receipt_path=p_path, effective_limits={"band": "Med-high", "max_total_turns": 8},
            card_hash=CARD_HASH,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()

        for heading in [
            "## 1. Task spec + RRI table",
            "## 2. Plan",
            "## 3. Allowed paths",
            "## 4. Full diff",
            "## 5. Commands executed with output",
            "## 6. Test results",
            "## 7. Per-attempt summaries",
            "## 8. Refinement artifact (Qwen27)",
            "## 9. Primary route receipt",
            "## 10. Effective limits",
            "## 11. Stop reason and hashes",
        ]:
            self.assertIn(heading, content)
        self.assertIn("total_turns_exhausted", content)
        self.assertIn(CARD_HASH, content)

    def test_ec2_missing_optional_inputs_render_missing_not_omitted(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, {"status": "aborted", "task_id": "T-MEDHIGH-1", "transcript": []})
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="boundary_violation",
            refinement_artifact_path=None, primary_receipt_path=None,
            effective_limits=None, card_hash=None,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()

        self.assertIn("## 8. Refinement artifact (Qwen27)\n\nMISSING", content)
        self.assertIn("## 9. Primary route receipt\n\nMISSING", content)
        self.assertIn("## 10. Effective limits\n\nMISSING", content)
        self.assertIn("Card hash: `MISSING`", content)


class SuperviseIntegrationTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def _write_gate_inputs(self, route_recommendation="GO_LOCAL", decision="GO_LOCAL"):
        refinement = _refinement_artifact(route_recommendation)
        receipt = _primary_receipt(refinement, decision)
        r_path = os.path.join(self.tmp.name, "refinement.json")
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(r_path, refinement)
        _write_json(p_path, receipt)
        return r_path, p_path

    def test_hp1_go_local_success_records_evidence_without_escalation(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("GO_LOCAL", "GO_LOCAL")

        def fake_popen(argv, start_new_session=None):
            def _write_success():
                _write_json(out_path, {
                    "status": "success", "task_id": "T-MEDHIGH-1", "transcript": [],
                    "model": "qwen3.6:35b-a3b",
                })
            return _FakeProcess(returncode=0, write_out=_write_success)

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=fake_popen,
        )

        self.assertEqual(result.status, "success")
        self.assertEqual(result.route, med_high_gate.ROUTE_GO_LOCAL)
        self.assertIsNone(result.bundle_path)
        self.assertFalse(os.path.isfile(bundle_path))

    def test_hp2_cloud_required_routes_without_launching_runner(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("CLOUD_REQUIRED", "GO_LOCAL")

        def fake_popen(argv, start_new_session=None):
            self.fail("Qwen35 must not be launched when the route is CLOUD_REQUIRED")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=fake_popen,
        )

        self.assertEqual(result.status, "cloud_required")
        self.assertEqual(result.route, med_high_gate.ROUTE_CLOUD_REQUIRED)
        self.assertTrue(os.path.isfile(bundle_path))

    def test_ec1_timeout_emits_bundle_with_wall_clock_exceeded_reason(self):
        import subprocess as _subprocess

        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        _write_json(out_path, {
            "status": "in_progress", "task_id": "T-MEDHIGH-1", "turn": 5, "transcript": [],
        })
        r_path, p_path = self._write_gate_inputs("GO_LOCAL", "GO_LOCAL")

        def fake_popen(argv, start_new_session=None):
            return _FakeProcess(wait_raises=_subprocess.TimeoutExpired(cmd=argv, timeout=1))

        orig_getpgid, orig_killpg = os.getpgid, os.killpg
        os.getpgid = lambda pid: pid
        os.killpg = lambda pgid, sig: None
        try:
            result = _MOD.supervise(
                card_path=card_path, worktree=self.tmp.name, out_path=out_path,
                bundle_out_path=bundle_path, refinement_artifact_path=r_path,
                primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
                wall_clock_seconds=1, popen_fn=fake_popen,
            )
        finally:
            os.getpgid = orig_getpgid
            os.killpg = orig_killpg

        self.assertEqual(result.status, "wall_clock_exceeded")
        self.assertIsNotNone(result.bundle_path)
        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()
        self.assertIn("wall_clock_exceeded", content)

    def test_ec2_failing_tests_route_emits_full_bundle(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("GO_LOCAL", "GO_LOCAL")

        def fake_popen(argv, start_new_session=None):
            def _write_budget_exhausted():
                _write_json(out_path, {
                    "status": "budget_exhausted", "reason": "total_turns_exhausted",
                    "task_id": "T-MEDHIGH-1", "transcript": [],
                    "effective_limits": {"band": "Med-high", "max_total_turns": 8},
                })
            return _FakeProcess(returncode=0, write_out=_write_budget_exhausted)

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=fake_popen,
        )

        self.assertEqual(result.status, "budget_exhausted")
        self.assertTrue(os.path.isfile(bundle_path))
        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()
        self.assertIn("## 10. Effective limits", content)
        self.assertIn("Med-high", content)

    def test_ec2_gate_rejection_before_any_launch_emits_bundle(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        # A tampered / hash-mismatched receipt fails closed inside the gate.
        refinement = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(refinement, "GO_LOCAL", refinement_artifact_sha256="deadbeef")
        r_path = os.path.join(self.tmp.name, "refinement.json")
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(r_path, refinement)
        _write_json(p_path, receipt)

        def fake_popen(argv, start_new_session=None):
            self.fail("a rejected gate must never launch the runner")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=fake_popen,
        )

        self.assertEqual(result.status, "gate_rejected")
        self.assertEqual(result.route, med_high_gate.ROUTE_CLOUD_REQUIRED)
        self.assertTrue(os.path.isfile(bundle_path))


class MainCliTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_hp2_cli_cloud_required_exits_nonzero(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        refinement = _refinement_artifact("CLOUD_REQUIRED")
        receipt = _primary_receipt(refinement, "GO_LOCAL")
        r_path = os.path.join(self.tmp.name, "refinement.json")
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(r_path, refinement)
        _write_json(p_path, receipt)

        exit_code = _MOD.main([
            "--card", card_path,
            "--worktree", self.tmp.name,
            "--out", out_path,
            "--bundle-out", bundle_path,
            "--refinement-artifact", r_path,
            "--primary-receipt", p_path,
            "--card-hash", CARD_HASH,
            "--rri", "50",
        ])

        self.assertEqual(exit_code, 1)
        self.assertTrue(os.path.isfile(bundle_path))


if __name__ == "__main__":
    unittest.main()
