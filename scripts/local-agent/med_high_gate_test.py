#!/usr/bin/env python3
"""Unit tests for scripts/local-agent/med_high_gate.py (ADR-038 T2)."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

_SCRIPT = Path(__file__).with_name("med_high_gate.py")
_SPEC = importlib.util.spec_from_file_location("med_high_gate", _SCRIPT)
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_SCRIPT}")
_MOD = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _MOD
_SPEC.loader.exec_module(_MOD)

CARD_HASH = "a" * 64


def _refinement_artifact(route_recommendation="GO_LOCAL", **overrides):
    artifact = {
        "success": True,
        "profile": _MOD.MED_HIGH_PROFILE,
        "packet": {"sha256": CARD_HASH},
        "model": {
            "tag": _MOD.REQUIRED_MODEL_TAG,
            "expected_digest": "digest-1",
            "resolved_digest": "digest-1",
        },
        "response": {
            "validated": {
                "route_recommendation": route_recommendation,
                "summary": "Refined scope for the approved card.",
            }
        },
    }
    artifact.update(overrides)
    return artifact


def _primary_receipt(refinement_artifact, decision="GO_LOCAL", **overrides):
    receipt = {
        "primary_id": "claude-code",
        "decision": decision,
        "rationale": "Scope is bounded and within local eligibility gates.",
        "timestamp": "2026-07-26T12:00:00Z",
        "card_hash": CARD_HASH,
        "refinement_artifact_sha256": _MOD.sha256_of(refinement_artifact),
    }
    receipt.update(overrides)
    return receipt


class ValidateRefinementArtifactTest(unittest.TestCase):
    def test_hp1_valid_go_local_artifact_returns_validated_payload(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        validated = _MOD.validate_refinement_artifact(artifact, expected_card_hash=CARD_HASH)
        self.assertEqual(validated["route_recommendation"], "GO_LOCAL")

    def test_hp1b_valid_cloud_required_artifact_returns_validated_payload(self) -> None:
        artifact = _refinement_artifact("CLOUD_REQUIRED")
        validated = _MOD.validate_refinement_artifact(artifact, expected_card_hash=CARD_HASH)
        self.assertEqual(validated["route_recommendation"], "CLOUD_REQUIRED")

    def test_ec1_card_hash_mismatch_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_refinement_artifact(artifact, expected_card_hash="b" * 64)
        self.assertEqual(ctx.exception.code, "card_hash_mismatch")

    def test_ec1b_wrong_profile_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL", profile="adr037")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_refinement_artifact(artifact, expected_card_hash=CARD_HASH)
        self.assertEqual(ctx.exception.code, "wrong_profile")

    def test_ec1c_unsuccessful_artifact_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL", success=False)
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_refinement_artifact(artifact, expected_card_hash=CARD_HASH)
        self.assertEqual(ctx.exception.code, "refinement_failed")

    def test_ec1d_model_tag_mismatch_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        artifact["model"]["tag"] = "some-other-model"
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_refinement_artifact(artifact, expected_card_hash=CARD_HASH)
        self.assertEqual(ctx.exception.code, "model_tag_mismatch")

    def test_ec1e_model_digest_mismatch_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        artifact["model"]["resolved_digest"] = "different-digest"
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_refinement_artifact(artifact, expected_card_hash=CARD_HASH)
        self.assertEqual(ctx.exception.code, "model_digest_mismatch")

    def test_ec1f_invalid_route_recommendation_fails_closed(self) -> None:
        artifact = _refinement_artifact("MAYBE")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_refinement_artifact(artifact, expected_card_hash=CARD_HASH)
        self.assertEqual(ctx.exception.code, "invalid_route")

    def test_ec1g_missing_field_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        del artifact["model"]
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_refinement_artifact(artifact, expected_card_hash=CARD_HASH)
        self.assertEqual(ctx.exception.code, "missing_field")

    def test_ec1h_non_dict_artifact_fails_closed(self) -> None:
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_refinement_artifact("not-a-dict", expected_card_hash=CARD_HASH)
        self.assertEqual(ctx.exception.code, "invalid_artifact")


class ValidatePrimaryReceiptTest(unittest.TestCase):
    def test_hp1_valid_matching_receipt_passes(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, decision="GO_LOCAL")
        _MOD.validate_primary_receipt(
            receipt,
            expected_card_hash=CARD_HASH,
            expected_refinement_sha256=_MOD.sha256_of(artifact),
        )  # no raise

    def test_ec1_stale_refinement_hash_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, decision="GO_LOCAL")
        tampered_artifact = _refinement_artifact("GO_LOCAL", summary_marker="tampered")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_primary_receipt(
                receipt,
                expected_card_hash=CARD_HASH,
                expected_refinement_sha256=_MOD.sha256_of(tampered_artifact),
            )
        self.assertEqual(ctx.exception.code, "refinement_hash_mismatch")

    def test_ec1b_card_hash_mismatch_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, card_hash="c" * 64)
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_primary_receipt(
                receipt,
                expected_card_hash=CARD_HASH,
                expected_refinement_sha256=_MOD.sha256_of(artifact),
            )
        self.assertEqual(ctx.exception.code, "card_hash_mismatch")

    def test_ec1c_invalid_decision_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, decision="MAYBE")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_primary_receipt(
                receipt,
                expected_card_hash=CARD_HASH,
                expected_refinement_sha256=_MOD.sha256_of(artifact),
            )
        self.assertEqual(ctx.exception.code, "invalid_receipt_decision")

    def test_ec1d_missing_primary_id_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, primary_id="")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_primary_receipt(
                receipt,
                expected_card_hash=CARD_HASH,
                expected_refinement_sha256=_MOD.sha256_of(artifact),
            )
        self.assertEqual(ctx.exception.code, "missing_field")

    def test_ec1f_falsy_primary_id_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, primary_id=0)
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_primary_receipt(
                receipt,
                expected_card_hash=CARD_HASH,
                expected_refinement_sha256=_MOD.sha256_of(artifact),
            )
        self.assertEqual(ctx.exception.code, "missing_field")

    def test_ec1e_empty_rationale_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, rationale="   ")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.validate_primary_receipt(
                receipt,
                expected_card_hash=CARD_HASH,
                expected_refinement_sha256=_MOD.sha256_of(artifact),
            )
        self.assertEqual(ctx.exception.code, "missing_field")


class EvaluateRouteTest(unittest.TestCase):
    def test_hp1_go_local_plus_matching_go_local_receipt_is_eligible(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, decision="GO_LOCAL")
        decision = _MOD.evaluate_route(
            refinement_artifact=artifact, primary_receipt=receipt, card_hash=CARD_HASH, rri=47
        )
        self.assertEqual(decision.route, _MOD.ROUTE_GO_LOCAL)

    def test_hp2_primary_may_downgrade_go_local_to_cloud(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, decision="CLOUD_REQUIRED")
        decision = _MOD.evaluate_route(
            refinement_artifact=artifact, primary_receipt=receipt, card_hash=CARD_HASH, rri=47
        )
        self.assertEqual(decision.route, _MOD.ROUTE_CLOUD_REQUIRED)

    def test_ec1_cloud_required_architect_route_is_never_upgraded_to_local(self) -> None:
        # Even if a (misbehaving) primary receipt claims GO_LOCAL, the
        # architect's CLOUD_REQUIRED must win -- ADR-038 s.3 forbids upgrade.
        artifact = _refinement_artifact("CLOUD_REQUIRED")
        receipt = _primary_receipt(artifact, decision="GO_LOCAL")
        decision = _MOD.evaluate_route(
            refinement_artifact=artifact, primary_receipt=receipt, card_hash=CARD_HASH, rri=47
        )
        self.assertEqual(decision.route, _MOD.ROUTE_CLOUD_REQUIRED)

    def test_ec1b_rri_below_med_high_band_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, decision="GO_LOCAL")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_route(
                refinement_artifact=artifact, primary_receipt=receipt, card_hash=CARD_HASH, rri=40
            )
        self.assertEqual(ctx.exception.code, "rri_out_of_band")

    def test_ec1c_rri_above_med_high_band_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, decision="GO_LOCAL")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_route(
                refinement_artifact=artifact, primary_receipt=receipt, card_hash=CARD_HASH, rri=56
            )
        self.assertEqual(ctx.exception.code, "rri_out_of_band")

    def test_ec1d_missing_evidence_never_starts_local(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        del artifact["model"]
        receipt = _primary_receipt(_refinement_artifact("GO_LOCAL"), decision="GO_LOCAL")
        with self.assertRaises(_MOD.GateError):
            _MOD.evaluate_route(
                refinement_artifact=artifact, primary_receipt=receipt, card_hash=CARD_HASH, rri=47
            )

    def test_ec1e_tampered_refinement_after_receipt_issued_fails_closed(self) -> None:
        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, decision="GO_LOCAL")
        tampered = _refinement_artifact("GO_LOCAL", summary_marker="tampered-after-receipt")
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_route(
                refinement_artifact=tampered, primary_receipt=receipt, card_hash=CARD_HASH, rri=47
            )
        self.assertEqual(ctx.exception.code, "refinement_hash_mismatch")


class MainCliTest(unittest.TestCase):
    """Fail-closed CLI entry-point error paths (phase-2 review LOW finding)."""

    def setUp(self) -> None:
        import tempfile

        self.tmpdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmpdir.cleanup)
        self.root = Path(self.tmpdir.name)

    def _write(self, name, content):
        path = self.root / name
        path.write_text(content, encoding="utf-8")
        return path

    def test_ec1_missing_refinement_file_exits_nonzero_with_cloud_required(self) -> None:
        import contextlib
        import io

        receipt_path = self._write("receipt.json", "{}")
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = _MOD.main(
                [
                    "--refinement-artifact",
                    str(self.root / "does-not-exist.json"),
                    "--primary-receipt",
                    str(receipt_path),
                    "--card-hash",
                    CARD_HASH,
                    "--rri",
                    "47",
                ]
            )
        self.assertEqual(exit_code, 1)
        output = _json_loads(buf.getvalue())
        self.assertEqual(output["route"], _MOD.ROUTE_CLOUD_REQUIRED)
        self.assertEqual(output["error"]["code"], "io_error")

    def test_ec1b_invalid_json_refinement_file_exits_nonzero_with_cloud_required(self) -> None:
        import contextlib
        import io

        refinement_path = self._write("refinement.json", "{not valid json")
        receipt_path = self._write("receipt.json", "{}")
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = _MOD.main(
                [
                    "--refinement-artifact",
                    str(refinement_path),
                    "--primary-receipt",
                    str(receipt_path),
                    "--card-hash",
                    CARD_HASH,
                    "--rri",
                    "47",
                ]
            )
        self.assertEqual(exit_code, 1)
        output = _json_loads(buf.getvalue())
        self.assertEqual(output["route"], _MOD.ROUTE_CLOUD_REQUIRED)
        self.assertEqual(output["error"]["code"], "io_error")

    def test_ec1c_gate_rejection_exits_nonzero_with_cloud_required(self) -> None:
        import contextlib
        import io
        import json as json_mod

        artifact = _refinement_artifact("CLOUD_REQUIRED")
        receipt = _primary_receipt(artifact, decision="GO_LOCAL")
        refinement_path = self._write("refinement.json", json_mod.dumps(artifact))
        receipt_path = self._write("receipt.json", json_mod.dumps(receipt))
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = _MOD.main(
                [
                    "--refinement-artifact",
                    str(refinement_path),
                    "--primary-receipt",
                    str(receipt_path),
                    "--card-hash",
                    CARD_HASH,
                    "--rri",
                    "47",
                ]
            )
        self.assertEqual(exit_code, 0)
        output = _json_loads(buf.getvalue())
        self.assertEqual(output["route"], _MOD.ROUTE_CLOUD_REQUIRED)

    def test_hp1_valid_go_local_cli_run_exits_zero(self) -> None:
        import contextlib
        import io
        import json as json_mod

        artifact = _refinement_artifact("GO_LOCAL")
        receipt = _primary_receipt(artifact, decision="GO_LOCAL")
        refinement_path = self._write("refinement.json", json_mod.dumps(artifact))
        receipt_path = self._write("receipt.json", json_mod.dumps(receipt))
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = _MOD.main(
                [
                    "--refinement-artifact",
                    str(refinement_path),
                    "--primary-receipt",
                    str(receipt_path),
                    "--card-hash",
                    CARD_HASH,
                    "--rri",
                    "47",
                ]
            )
        self.assertEqual(exit_code, 0)
        output = _json_loads(buf.getvalue())
        self.assertEqual(output["route"], _MOD.ROUTE_GO_LOCAL)


def _json_loads(text):
    import json as json_mod

    return json_mod.loads(text)


if __name__ == "__main__":
    unittest.main()
