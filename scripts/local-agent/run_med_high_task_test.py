#!/usr/bin/env python3
"""Tests for run_med_high_task.py (ADR-038 T4): supervisor + evidence bundle."""

import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import med_high_gate
import run_med_high_task as _MOD

CARD_HASH = "a" * 64
_RECEIPT_SHA = "31d923290a7ec004229a8ca7407af072b1de021aeff1ed97fe7bee9eb39befa2"

# Captured after T1, T2, and T4 have all landed (plan D4): covers T4's schema
# change (new "8. Acceptance tests" section, sections 8-11 renumbered to
# 9-12). Frozen once here and never modified by T4b, which adds its own,
# separately-named FINAL_* constants on top of this output.
POST_SCHEMA_BUNDLE_WITH_DIFF = f"""# Escalation packet: `T-MEDHIGH-1`

## 1. Task spec + RRI table

Task ID: `T-MEDHIGH-1`

Spec:

Do the bounded thing.

RRI table:

MISSING

## 2. Plan

MISSING

## 3. Allowed paths

- `src/lib.rs`

## 4. Full diff

```diff
diff --git a/src/lib.rs b/src/lib.rs
+fn foo() {{}}

```

## 5. Commands executed with output

MISSING

## 6. Test results

MISSING

## 7. Per-attempt summaries

- Final status: `budget_exhausted`.

## 8. Acceptance tests

- `cargo test -p demo -- test_foo`


## 9. Refinement artifact (Qwen27)

{{
  "model": {{
    "expected_digest": "sha256:deadbeef",
    "resolved_digest": "sha256:deadbeef",
    "tag": "muse-glimmer:30b-q4_K_M"
  }},
  "packet": {{
    "sha256": "{CARD_HASH}"
  }},
  "profile": "med-high-refinement-v1",
  "response": {{
    "validated": {{
      "route_recommendation": "GO_LOCAL",
      "summary": "refined"
    }}
  }},
  "success": true
}}


## 10. Primary route receipt

{{
  "card_hash": "{CARD_HASH}",
  "decision": "GO_LOCAL",
  "primary_id": "claude-code",
  "rationale": "matches architect recommendation",
  "refinement_artifact_sha256": "{_RECEIPT_SHA}",
  "timestamp": "2026-07-26T00:00:00Z"
}}


## 11. Effective limits

{{
  "band": "Med-high"
}}


## 12. Stop reason and hashes

Stop reason: `budget_exhausted`

Card hash: `{CARD_HASH}`

Refinement artifact SHA-256: `{_RECEIPT_SHA}`

Runner model: `MISSING`

Runner status: `budget_exhausted`
"""


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

    def __init__(self, pid=4242, wait_raises=None, second_wait_raises=None, returncode=0, write_out=None):
        self.pid = pid
        self._wait_raises = wait_raises
        self._second_wait_raises = second_wait_raises
        self.returncode = returncode
        self._write_out = write_out
        self._wait_calls = 0

    def wait(self, timeout=None):
        self._wait_calls += 1
        if self._wait_raises is not None and self._wait_calls == 1:
            raise self._wait_raises
        if self._second_wait_raises is not None and self._wait_calls == 2:
            raise self._second_wait_raises
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


class DecideRouteGateInputErrorTest(unittest.TestCase):
    """T3 (plan Defect C/D6): decide_route must never let an unreadable or
    malformed gate artifact escape as an uncaught exception."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_ec1_missing_refinement_artifact_raises_gate_input_error(self):
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(p_path, {"decision": "GO_LOCAL"})
        missing_r_path = os.path.join(self.tmp.name, "does-not-exist.json")

        with self.assertRaises(_MOD.GateInputError) as ctx:
            _MOD.decide_route(
                refinement_artifact_path=missing_r_path, primary_receipt_path=p_path,
                card_hash=CARD_HASH, rri=50,
            )
        self.assertEqual(ctx.exception.artifact_label, "refinement artifact")

    def test_ec2_primary_receipt_malformed_json_raises_gate_input_error(self):
        refinement = _refinement_artifact("GO_LOCAL")
        r_path = os.path.join(self.tmp.name, "refinement.json")
        _write_json(r_path, refinement)
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_text(p_path, "{not valid json")

        with self.assertRaises(_MOD.GateInputError) as ctx:
            _MOD.decide_route(
                refinement_artifact_path=r_path, primary_receipt_path=p_path,
                card_hash=CARD_HASH, rri=50,
            )
        self.assertEqual(ctx.exception.artifact_label, "primary receipt")

    def test_ec5_gate_artifact_undecodable_bytes_raises_gate_input_error(self):
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(p_path, {"decision": "GO_LOCAL"})
        r_path = os.path.join(self.tmp.name, "refinement.json")
        with open(r_path, "wb") as f:
            f.write(b"\xff\xfe not utf-8")

        with self.assertRaises(_MOD.GateInputError) as ctx:
            _MOD.decide_route(
                refinement_artifact_path=r_path, primary_receipt_path=p_path,
                card_hash=CARD_HASH, rri=50,
            )
        self.assertEqual(ctx.exception.artifact_label, "refinement artifact")


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

    def test_ec2_post_kill_wait_oserror_still_reaches_structured_result(self):
        import subprocess as _subprocess

        killpg_calls = []

        def fake_popen(argv, start_new_session=None):
            return _FakeProcess(
                pid=999,
                wait_raises=_subprocess.TimeoutExpired(cmd=argv, timeout=1),
                second_wait_raises=OSError("ECHILD"),
            )

        orig_getpgid, orig_killpg = os.getpgid, os.killpg
        os.getpgid = lambda pid: pid
        os.killpg = lambda pgid, sig: killpg_calls.append((pgid, sig))
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

        # Phase-2 review finding (qwen3.6:27b-q4_K_M, LOW): assert the kill
        # actually happened before the post-kill wait failure, not just that
        # a structured result came back.
        self.assertEqual(killpg_calls, [(999, _MOD.signal.SIGKILL)])
        self.assertEqual(result["status"], "wall_clock_exceeded")
        self.assertIn("post-kill wait failed", result["reason"])

    def test_ec3_post_kill_wait_timeout_expired_still_reaches_structured_result(self):
        import subprocess as _subprocess

        def fake_popen(argv, start_new_session=None):
            return _FakeProcess(
                wait_raises=_subprocess.TimeoutExpired(cmd=argv, timeout=1),
                second_wait_raises=_subprocess.TimeoutExpired(cmd=argv, timeout=_MOD.POST_KILL_WAIT_SECONDS),
            )

        orig_getpgid, orig_killpg = os.getpgid, os.killpg
        os.getpgid = lambda pid: pid
        os.killpg = lambda pgid, sig: None
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
        self.assertIn("post-kill wait failed", result["reason"])


# Generated on top of POST_SCHEMA_BUNDLE_WITH_DIFF (plan D4): the ticket's
# actual final-state fixture, once T4b threads elapsed_s through the stop
# reason section. POST_SCHEMA_BUNDLE_WITH_DIFF itself is retained as T4's own
# frozen regression proof and is not modified -- build_evidence_bundle's
# signature change (elapsed_s always rendered) means it is no longer produced
# by current code, which is expected: it recorded T4's landing state, not an
# invariant that survives T4b.
FINAL_BUNDLE_WITH_DIFF = POST_SCHEMA_BUNDLE_WITH_DIFF.replace(
    "Runner status: `budget_exhausted`\n",
    "Runner status: `budget_exhausted`\n\nElapsed: `12.5s`\n",
)


class PostSchemaFixtureConsistencyTest(unittest.TestCase):
    """T4 (plan D4): POST_SCHEMA_BUNDLE_WITH_DIFF is frozen as T4's own
    landing-state proof. This ticket implemented T4 and T4b in the same
    delivery rather than as separate shipped revisions, so the fixture is no
    longer independently reproducible from live code (build_evidence_bundle
    now always renders elapsed time) -- this test instead proves the fixture
    is well-formed and is exactly what FINAL_BUNDLE_WITH_DIFF was derived
    from, so an accidental edit to either constant is caught."""

    def test_post_schema_fixture_has_expected_section_eight_and_twelve_sections(self):
        self.assertIn("## 8. Acceptance tests\n\n- `cargo test -p demo -- test_foo`", POST_SCHEMA_BUNDLE_WITH_DIFF)
        self.assertIn("## 12. Stop reason and hashes", POST_SCHEMA_BUNDLE_WITH_DIFF)
        self.assertNotIn("Elapsed:", POST_SCHEMA_BUNDLE_WITH_DIFF)

    def test_final_fixture_is_post_schema_fixture_plus_elapsed_time_only(self):
        reconstructed = FINAL_BUNDLE_WITH_DIFF.replace(
            "Runner status: `budget_exhausted`\n\nElapsed: `12.5s`\n",
            "Runner status: `budget_exhausted`\n",
        )
        self.assertEqual(reconstructed, POST_SCHEMA_BUNDLE_WITH_DIFF)


class FinalGoldenBundleTest(unittest.TestCase):
    """T4b (plan D4): byte-identical proof of the ticket's final bundle
    shape, generated on top of T4's post_schema_* output plus elapsed time."""

    def test_final_golden_bundle_matches_exactly(self):
        with tempfile.TemporaryDirectory() as tmp:
            card_path = os.path.join(tmp, "card.json")
            _write_json(card_path, {
                "task_id": "T-MEDHIGH-1",
                "spec": "Do the bounded thing.",
                "acceptance_tests": ["cargo test -p demo -- test_foo"],
                "allowed_paths": ["src/lib.rs"],
            })
            out_path = os.path.join(tmp, "out.json")
            _write_json(out_path, {"status": "budget_exhausted", "task_id": "T-MEDHIGH-1", "transcript": []})
            diff_path = os.path.join(tmp, "diff.txt")
            _write_text(diff_path, "diff --git a/src/lib.rs b/src/lib.rs\n+fn foo() {}\n")

            refinement = _refinement_artifact("GO_LOCAL")
            receipt = _primary_receipt(refinement, "GO_LOCAL")
            r_path = os.path.join(tmp, "refinement.json")
            p_path = os.path.join(tmp, "receipt.json")
            _write_json(r_path, refinement)
            _write_json(p_path, receipt)

            bundle_path = os.path.join(tmp, "bundle.md")
            write_result = _MOD.build_evidence_bundle(
                bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
                stop_reason="budget_exhausted", elapsed_s=12.5,
                refinement_artifact_path=r_path,
                primary_receipt_path=p_path, effective_limits={"band": "Med-high"},
                card_hash=CARD_HASH, diff_file=diff_path,
            )

            self.assertTrue(write_result.write_ok)
            self.assertEqual(write_result.path, bundle_path)
            self.assertIsNone(write_result.write_error)

            with open(bundle_path, encoding="utf-8") as f:
                actual = f.read()

            self.assertEqual(actual, FINAL_BUNDLE_WITH_DIFF)


class AtomicWriteTest(unittest.TestCase):
    """T4b (plan D8): the bundle write is atomic and its failure is reported
    structurally via BundleWriteResult, never raised."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_hp2_write_succeeds_returns_write_ok_true(self):
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        result = _MOD._write_bundle_atomically(bundle_path, "content")
        self.assertTrue(result.write_ok)
        self.assertEqual(result.path, bundle_path)
        self.assertIsNone(result.write_error)
        with open(bundle_path, encoding="utf-8") as f:
            self.assertEqual(f.read(), "content")

    def test_ec1_oserror_on_replace_reports_write_failure_not_raise(self):
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        orig_replace = os.replace

        def failing_replace(src, dst):
            raise OSError("cross-device link")

        os.replace = failing_replace
        try:
            result = _MOD._write_bundle_atomically(bundle_path, "content")
        finally:
            os.replace = orig_replace

        self.assertFalse(result.write_ok)
        self.assertEqual(result.path, bundle_path)
        self.assertIn("cross-device link", result.write_error)
        self.assertFalse(os.path.isfile(bundle_path))

    def test_ec4_replace_failure_cleans_up_tmp_file(self):
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        orig_replace = os.replace

        def failing_replace(src, dst):
            raise OSError("permission denied")

        os.replace = failing_replace
        try:
            _MOD._write_bundle_atomically(bundle_path, "content")
        finally:
            os.replace = orig_replace

        self.assertFalse(os.path.isfile(f"{bundle_path}.tmp"))

    def test_ec1_write_failure_does_not_touch_a_pre_existing_bundle(self):
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        _write_text(bundle_path, "previous bundle content")
        orig_replace = os.replace

        def failing_replace(src, dst):
            raise OSError("disk full")

        os.replace = failing_replace
        try:
            result = _MOD._write_bundle_atomically(bundle_path, "new content")
        finally:
            os.replace = orig_replace

        self.assertFalse(result.write_ok)
        with open(bundle_path, encoding="utf-8") as f:
            self.assertEqual(f.read(), "previous bundle content")

    def test_ec5_unicode_encode_error_reports_write_failure_not_raise(self):
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        orig_open = open

        class _FailingFile:
            def __enter__(self):
                return self

            def __exit__(self, *exc_info):
                return False

            def write(self, content):
                raise UnicodeEncodeError("utf-8", "\ud800", 0, 1, "surrogate")

            def flush(self):
                pass

            def fileno(self):
                return 0

        def failing_open(path, *a, **kw):
            if path == f"{bundle_path}.tmp":
                return _FailingFile()
            return orig_open(path, *a, **kw)

        import builtins
        builtins.open = failing_open
        try:
            result = _MOD._write_bundle_atomically(bundle_path, "content with \ud800 surrogate")
        finally:
            builtins.open = orig_open

        self.assertFalse(result.write_ok)
        self.assertIn("surrogate", result.write_error.lower())


class BuildEvidenceBundleTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_hp_bundle_contains_all_twelve_sections(self):
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
            "## 8. Acceptance tests",
            "## 9. Refinement artifact (Qwen27)",
            "## 10. Primary route receipt",
            "## 11. Effective limits",
            "## 12. Stop reason and hashes",
        ]:
            self.assertIn(heading, content)
        self.assertIn("total_turns_exhausted", content)
        self.assertIn(CARD_HASH, content)
        self.assertIn("## 8. Acceptance tests\n\n- `true`", content)

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

        self.assertIn("## 9. Refinement artifact (Qwen27)\n\nMISSING", content)
        self.assertIn("## 10. Primary route receipt\n\nMISSING", content)
        self.assertIn("## 11. Effective limits\n\nMISSING", content)
        self.assertIn("Card hash: `MISSING`", content)

    def test_ec4_card_with_no_acceptance_tests_key_renders_missing_not_keyerror(self):
        # _card() always writes an acceptance_tests key; simulate a card that
        # never had one at all by writing raw JSON without it.
        card_path = os.path.join(self.tmp.name, "card.json")
        _write_json(card_path, {"task_id": "T-MEDHIGH-1", "spec": "Do the bounded thing."})
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, {"status": "aborted", "task_id": "T-MEDHIGH-1", "transcript": []})
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="aborted",
            refinement_artifact_path=None, primary_receipt_path=None,
            effective_limits=None, card_hash=None,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()

        self.assertIn("## 8. Acceptance tests\n\nMISSING", content)

    def test_ec5_card_with_empty_acceptance_tests_list_renders_missing_not_empty_bullets(self):
        card_path = _card(self.tmp.name, acceptance_tests=[])
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, {"status": "aborted", "task_id": "T-MEDHIGH-1", "transcript": []})
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="aborted",
            refinement_artifact_path=None, primary_receipt_path=None,
            effective_limits=None, card_hash=None,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()

        self.assertIn("## 8. Acceptance tests\n\nMISSING", content)

    def test_hp2_multiple_acceptance_tests_render_one_bullet_each(self):
        card_path = _card(self.tmp.name, acceptance_tests=["cargo test -- foo", "cargo test -- bar"])
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, {"status": "aborted", "task_id": "T-MEDHIGH-1", "transcript": []})
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="aborted",
            refinement_artifact_path=None, primary_receipt_path=None,
            effective_limits=None, card_hash=None,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()

        self.assertIn("## 8. Acceptance tests\n\n- `cargo test -- foo`\n- `cargo test -- bar`", content)

    def test_ec1_runner_output_undecodable_bytes_not_crash(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        with open(out_path, "wb") as f:
            f.write(b"\xff\xfe not utf-8")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="transport_error",
            refinement_artifact_path=None, primary_receipt_path=None,
            effective_limits=None, card_hash=None,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()
        self.assertIn("## 7. Per-attempt summaries", content)

    def test_ec2_runner_output_wrong_shape_json_list_not_crash(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, ["not", "a", "dict"])
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="transport_error",
            refinement_artifact_path=None, primary_receipt_path=None,
            effective_limits=None, card_hash=None,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()
        self.assertIn("## 5. Commands executed with output\n\nMISSING", content)
        self.assertIn("## 6. Test results\n\nMISSING", content)
        # Phase-2 review finding (qwen3.6:27b-q4_K_M): the shape-check
        # failure reason must not be silently discarded -- section 7 must
        # carry it, not just render a bare status with no explanation.
        self.assertIn(
            "## 7. Per-attempt summaries\n\n"
            "- Final status: `transcript_shape_invalid` (expected a JSON object, got list).",
            content,
        )

    def test_ec1_refinement_artifact_malformed_json_renders_exact_unreadable_form(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, {"status": "aborted", "task_id": "T-MEDHIGH-1", "transcript": []})
        r_path = os.path.join(self.tmp.name, "refinement.json")
        _write_text(r_path, "{not valid json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="aborted",
            refinement_artifact_path=r_path, primary_receipt_path=None,
            effective_limits=None, card_hash=None,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()

        self.assertIn(f"## 9. Refinement artifact (Qwen27)\n\nMISSING (refinement artifact unreadable: {r_path}:", content)
        self.assertIn("## 10. Primary route receipt\n\nMISSING", content)

    def test_ec2_primary_receipt_unreadable_renders_exact_unreadable_form(self):
        import builtins

        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, {"status": "aborted", "task_id": "T-MEDHIGH-1", "transcript": []})
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(p_path, {"decision": "GO_LOCAL"})
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        real_open = builtins.open

        def failing_open(path, *a, **kw):
            if path == p_path:
                raise PermissionError("denied")
            return real_open(path, *a, **kw)

        builtins.open = failing_open
        try:
            _MOD.build_evidence_bundle(
                bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
                stop_reason="aborted",
                refinement_artifact_path=None, primary_receipt_path=p_path,
                effective_limits=None, card_hash=None,
            )
        finally:
            builtins.open = real_open

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()

        self.assertIn(f"## 10. Primary route receipt\n\nMISSING (primary receipt unreadable: {p_path}: denied)", content)

    def test_ec3_refinement_artifact_unparseable_sha_renders_not_computed(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, {"status": "aborted", "task_id": "T-MEDHIGH-1", "transcript": []})
        r_path = os.path.join(self.tmp.name, "refinement.json")
        _write_text(r_path, "{not valid json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="aborted",
            refinement_artifact_path=r_path, primary_receipt_path=None,
            effective_limits=None, card_hash=None,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()

        self.assertIn("Refinement artifact SHA-256: `MISSING (not computed: source unreadable)`", content)

    def test_ec4_refinement_artifact_undecodable_bytes_render_unreadable_not_crash(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        _write_json(out_path, {"status": "aborted", "task_id": "T-MEDHIGH-1", "transcript": []})
        r_path = os.path.join(self.tmp.name, "refinement.json")
        with open(r_path, "wb") as f:
            f.write(b"\xff\xfe not utf-8")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")

        _MOD.build_evidence_bundle(
            bundle_out_path=bundle_path, card_path=card_path, runner_out_path=out_path,
            stop_reason="aborted",
            refinement_artifact_path=r_path, primary_receipt_path=None,
            effective_limits=None, card_hash=None,
        )

        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()

        self.assertIn(f"## 9. Refinement artifact (Qwen27)\n\nMISSING (refinement artifact unreadable: {r_path}:", content)


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

    def test_hp1_rri_41_45_go_local_launches_devstral_runner(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("GO_LOCAL", "GO_LOCAL")
        _write_json(out_path, {"status": "success", "task_id": "T-MEDHIGH-1"})

        with patch.object(
            _MOD, "run_supervised_runner",
            return_value={"status": "runner_exited", "elapsed_s": 1.0, "returncode": 0},
        ) as runner:
            result = _MOD.supervise(
                card_path=card_path, worktree=self.tmp.name, out_path=out_path,
                bundle_out_path=bundle_path, refinement_artifact_path=r_path,
                primary_receipt_path=p_path, card_hash=CARD_HASH, rri=43,
            )

        self.assertEqual(result.status, "success")
        self.assertEqual(result.route, med_high_gate.ROUTE_GO_LOCAL)
        self.assertIsNone(result.bundle_path)
        self.assertEqual(
            runner.call_args.kwargs["model"],
            "devstral-small-2:24b-instruct-2512-q4_K_M",
        )

    def test_hp1_go_local_is_policy_excluded_and_never_launches_runner(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("GO_LOCAL", "GO_LOCAL")
        launched_argv = []

        def fake_popen(argv, start_new_session=None):
            launched_argv.append(argv)
            self.fail("Med-high GO_LOCAL must not launch a local runner")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=fake_popen,
        )

        self.assertEqual(result.status, "cloud_required")
        self.assertEqual(result.route, med_high_gate.ROUTE_CLOUD_REQUIRED)
        self.assertTrue(os.path.isfile(bundle_path))
        self.assertIn("RRI 46-55 Med-high local execution is cloud-only", result.reason)
        self.assertEqual(result.fallback_selection["status"], "awaiting_fallback_selection")
        self.assertIsNone(result.cloud_instruction)
        self.assertEqual(launched_argv, [])

    def test_hp1_go_local_authorizes_sol_against_policy_bundle(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        selection_path = os.path.join(self.tmp.name, "selection.json")
        r_path, p_path = self._write_gate_inputs("GO_LOCAL", "GO_LOCAL")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=lambda *a, **kw: self.fail("must not launch"),
            fallback_mode="preauthorized", fallback_model="gpt-5.6-sol",
            fallback_reasoning_effort="high", fallback_selected_by="owner",
            fallback_selection_artifact=selection_path,
        )

        self.assertEqual(result.status, "cloud_required")
        self.assertEqual(result.fallback_selection["status"], "fallback_authorized")
        self.assertEqual(result.fallback_selection["recommended_model"], "gpt-5.6-sol")
        self.assertEqual(result.fallback_selection["recommended_reasoning_effort"], "high")
        self.assertEqual(result.cloud_instruction, {
            "model": "gpt-5.6-sol", "reasoning_effort": "high",
        })
        with open(selection_path, encoding="utf-8") as stream:
            checkpoint = json.load(stream)
        self.assertEqual(checkpoint["packet_sha256"], _MOD.fallback_selection.packet_sha256(
            _MOD.build_fallback_packet(card_path=card_path, rri=50, result=result)
        ))

    def test_hp2_cloud_required_routes_without_launching_runner(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("CLOUD_REQUIRED", "GO_LOCAL")

        def fake_popen(argv, start_new_session=None):
            self.fail("local runner must not be launched when the route is CLOUD_REQUIRED")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=fake_popen,
        )

        self.assertEqual(result.status, "cloud_required")
        self.assertEqual(result.route, med_high_gate.ROUTE_CLOUD_REQUIRED)
        self.assertTrue(os.path.isfile(bundle_path))

    def test_hp2_cloud_required_authorizes_sol_without_model_invocation(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("CLOUD_REQUIRED", "GO_LOCAL")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=lambda *a, **kw: self.fail("must not launch"),
            fallback_mode="preauthorized", fallback_model="gpt-5.6-sol",
            fallback_reasoning_effort="high", fallback_selected_by="owner",
        )

        self.assertEqual(result.fallback_selection["trigger_kind"], "capability-risk")
        self.assertEqual(result.fallback_selection["recommended_model"], "gpt-5.6-sol")
        self.assertEqual(result.cloud_instruction, {
            "model": "gpt-5.6-sol", "reasoning_effort": "high",
        })

    def test_ec1_human_selection_pause_preserves_bundle_without_instruction(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("CLOUD_REQUIRED", "GO_LOCAL")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=lambda *a, **kw: self.fail("must not launch"),
        )

        self.assertTrue(os.path.isfile(bundle_path))
        self.assertEqual(result.fallback_selection["status"], "awaiting_fallback_selection")
        self.assertNotIn("authorization_receipt", result.fallback_selection)
        self.assertIsNone(result.cloud_instruction)
        self.assertTrue(os.path.isfile(result.fallback_selection_artifact))

    def test_ec2_bundle_mutation_after_receipt_is_blocked(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("CLOUD_REQUIRED", "GO_LOCAL")
        real_build_checkpoint = _MOD.fallback_selection.build_checkpoint

        def mutate_after_receipt(**kwargs):
            checkpoint = real_build_checkpoint(**kwargs)
            with open(bundle_path, "a", encoding="utf-8") as stream:
                stream.write("\nmutated after authorization\n")
            return checkpoint

        _MOD.fallback_selection.build_checkpoint = mutate_after_receipt
        try:
            result = _MOD.supervise(
                card_path=card_path, worktree=self.tmp.name, out_path=out_path,
                bundle_out_path=bundle_path, refinement_artifact_path=r_path,
                primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
                popen_fn=lambda *a, **kw: self.fail("must not launch"),
                fallback_mode="preauthorized", fallback_model="gpt-5.6-sol",
                fallback_reasoning_effort="high", fallback_selected_by="owner",
            )
        finally:
            _MOD.fallback_selection.build_checkpoint = real_build_checkpoint

        self.assertEqual(result.fallback_selection["status"], "blocked")
        self.assertEqual(result.fallback_selection["summary"], "bundle_receipt_mismatch")
        self.assertNotIn("authorization_receipt", result.fallback_selection)
        self.assertIsNone(result.cloud_instruction)

    def test_ec1_go_local_does_not_enter_timeout_path(self):
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

        self.assertEqual(result.status, "cloud_required")
        self.assertIsNotNone(result.bundle_path)
        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()
        self.assertIn("policy_excluded_local_execution", content)

    def test_ec2_go_local_does_not_enter_runner_failure_path(self):
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

        self.assertEqual(result.status, "cloud_required")
        self.assertTrue(os.path.isfile(bundle_path))
        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()
        self.assertIn("policy_excluded_local_execution", content)

    def test_ec2b_go_local_does_not_consume_runner_output(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("GO_LOCAL", "GO_LOCAL")

        def fake_popen(argv, start_new_session=None):
            def _write_wrong_shape():
                _write_json(out_path, ["not", "a", "dict"])
            return _FakeProcess(returncode=0, write_out=_write_wrong_shape)

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=fake_popen,
        )

        self.assertEqual(result.status, "cloud_required")
        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()
        self.assertIn(
            "policy_excluded_local_execution",
            content,
        )

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

    def test_ec1_supervise_missing_gate_artifact_routes_cloud_required_with_bundle(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(p_path, {"decision": "GO_LOCAL"})
        missing_r_path = os.path.join(self.tmp.name, "does-not-exist.json")

        def fake_popen(argv, start_new_session=None):
            self.fail("an unreadable gate artifact must never launch the runner")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=missing_r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=fake_popen,
        )

        self.assertEqual(result.status, "cloud_required")
        self.assertEqual(result.route, med_high_gate.ROUTE_CLOUD_REQUIRED)
        self.assertTrue(os.path.isfile(bundle_path))

    def test_ec2_supervise_malformed_receipt_routes_cloud_required_with_bundle(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        refinement = _refinement_artifact("GO_LOCAL")
        r_path = os.path.join(self.tmp.name, "refinement.json")
        _write_json(r_path, refinement)
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_text(p_path, "{not valid json")

        def fake_popen(argv, start_new_session=None):
            self.fail("a malformed gate artifact must never launch the runner")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=fake_popen,
        )

        self.assertEqual(result.status, "cloud_required")
        self.assertEqual(result.route, med_high_gate.ROUTE_CLOUD_REQUIRED)
        self.assertTrue(os.path.isfile(bundle_path))

    def test_ec3_supervise_gate_artifact_permission_error_routes_cloud_required(self):
        import builtins

        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        refinement = _refinement_artifact("GO_LOCAL")
        r_path = os.path.join(self.tmp.name, "refinement.json")
        _write_json(r_path, refinement)
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(p_path, {"decision": "GO_LOCAL"})

        real_open = builtins.open

        def failing_open(path, *a, **kw):
            if path == p_path:
                raise PermissionError("denied")
            return real_open(path, *a, **kw)

        def fake_popen(argv, start_new_session=None):
            self.fail("a permission-denied gate artifact must never launch the runner")

        builtins.open = failing_open
        try:
            result = _MOD.supervise(
                card_path=card_path, worktree=self.tmp.name, out_path=out_path,
                bundle_out_path=bundle_path, refinement_artifact_path=r_path,
                primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
                popen_fn=fake_popen,
            )
        finally:
            builtins.open = real_open

        self.assertEqual(result.status, "cloud_required")
        self.assertTrue(os.path.isfile(bundle_path))

    def test_ec4_existing_gate_error_status_and_route_unchanged(self):
        # Proves the new GateInputError handling did not swallow or shadow
        # med_high_gate.GateError, which stays "gate_rejected" as before.
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        refinement = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(refinement, "GO_LOCAL", refinement_artifact_sha256="deadbeef")
        r_path = os.path.join(self.tmp.name, "refinement.json")
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(r_path, refinement)
        _write_json(p_path, receipt)

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=lambda *a, **kw: self.fail("must not launch"),
        )

        self.assertEqual(result.status, "gate_rejected")
        self.assertEqual(result.route, med_high_gate.ROUTE_CLOUD_REQUIRED)

    def test_hp4_gate_input_error_call_site_gets_elapsed_time_and_atomic_write(self):
        # T4b: proves T3's new call site was not missed when elapsed_s and
        # BundleWriteResult were threaded through all four call sites.
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        p_path = os.path.join(self.tmp.name, "receipt.json")
        _write_json(p_path, {"decision": "GO_LOCAL"})
        missing_r_path = os.path.join(self.tmp.name, "does-not-exist.json")

        result = _MOD.supervise(
            card_path=card_path, worktree=self.tmp.name, out_path=out_path,
            bundle_out_path=bundle_path, refinement_artifact_path=missing_r_path,
            primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
            popen_fn=lambda *a, **kw: self.fail("must not launch"),
        )

        self.assertEqual(result.status, "cloud_required")
        self.assertTrue(result.bundle_write_ok)
        with open(bundle_path, encoding="utf-8") as f:
            content = f.read()
        self.assertIn("Elapsed: `0.0s`", content)

    def test_ec1_pre_launch_write_failure_propagates_bundle_write_ok_false(self):
        import builtins

        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("CLOUD_REQUIRED", "GO_LOCAL")

        real_replace = os.replace

        def failing_replace(src, dst):
            raise OSError("disk full")

        os.replace = failing_replace
        try:
            result = _MOD.supervise(
                card_path=card_path, worktree=self.tmp.name, out_path=out_path,
                bundle_out_path=bundle_path, refinement_artifact_path=r_path,
                primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
                popen_fn=lambda *a, **kw: self.fail("must not launch"),
            )
        finally:
            os.replace = real_replace

        self.assertEqual(result.status, "cloud_required")
        self.assertFalse(result.bundle_write_ok)
        self.assertIn("bundle write failed", result.reason)

    def test_ec1_policy_handoff_write_failure_propagates_bundle_write_ok_false(self):
        card_path = _card(self.tmp.name)
        out_path = os.path.join(self.tmp.name, "out.json")
        bundle_path = os.path.join(self.tmp.name, "bundle.md")
        r_path, p_path = self._write_gate_inputs("GO_LOCAL", "GO_LOCAL")

        def fake_popen(argv, start_new_session=None):
            def _write_budget_exhausted():
                _write_json(out_path, {
                    "status": "budget_exhausted", "reason": "total_turns_exhausted",
                    "task_id": "T-MEDHIGH-1", "transcript": [],
                })
            return _FakeProcess(returncode=0, write_out=_write_budget_exhausted)

        real_replace = os.replace

        def failing_replace(src, dst):
            raise OSError("disk full")

        os.replace = failing_replace
        try:
            result = _MOD.supervise(
                card_path=card_path, worktree=self.tmp.name, out_path=out_path,
                bundle_out_path=bundle_path, refinement_artifact_path=r_path,
                primary_receipt_path=p_path, card_hash=CARD_HASH, rri=50,
                popen_fn=fake_popen,
            )
        finally:
            os.replace = real_replace

        self.assertEqual(result.status, "cloud_required")
        self.assertFalse(result.bundle_write_ok)
        self.assertIn("bundle write failed", result.reason)


class MainCliTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_ec1_cli_cloud_required_without_selection_pauses(self):
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

        self.assertEqual(exit_code, _MOD.fallback_selection.HUMAN_SELECTION_EXIT_CODE)
        self.assertTrue(os.path.isfile(bundle_path))

    def test_ec1_cli_partial_preauthorization_blocks(self):
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
            "--fallback-mode", "preauthorized",
            "--fallback-model", "gpt-5.6-sol",
        ])

        self.assertEqual(exit_code, 2)
        selection_path = _MOD.fallback_selection.default_checkpoint_path(bundle_path)
        with open(selection_path, encoding="utf-8") as stream:
            checkpoint = json.load(stream)
        self.assertEqual(checkpoint["status"], "blocked")
        self.assertIn("selected_reasoning_effort", checkpoint["summary"])
        self.assertNotIn("authorization_receipt", checkpoint)


if __name__ == "__main__":
    unittest.main()
