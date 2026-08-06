"""Unit tests for T4's post-CI touchpoint CLI summary builder."""

from __future__ import annotations

import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from scripts.antares import post_ci_summary


def _stub_dispatch_factory(by_cwe: dict[str, tuple[str, ...]]):
    def _run_pilot_touchpoint(*, touchpoint, cwe_id, snapshot_root, snapshot_id, triage_owner):
        candidates = by_cwe.get(cwe_id, ())
        return SimpleNamespace(
            touchpoint=touchpoint,
            cwe_id=cwe_id,
            snapshot_id=snapshot_id,
            degraded=False,
            detail="",
            candidates=candidates,
            ledger_entries=(),
        )

    return _run_pilot_touchpoint


class HappyPathTest(unittest.TestCase):
    def test_hp3_summary_never_includes_raw_candidate_paths(self) -> None:
        stub = _stub_dispatch_factory({"CWE-89": ("crates/db/src/query.rs",)})
        with mock.patch.object(post_ci_summary, "run_pilot_touchpoint", side_effect=stub):
            summary = post_ci_summary.build_summary(
                snapshot_root=Path("."),
                snapshot_id="sha-1",
                triage_owner="security-team",
                retention_days=30,
            )
        serialized = str(summary)
        self.assertNotIn("crates/db/src/query.rs", serialized)
        self.assertEqual(summary["total_candidates"], 1)
        self.assertEqual(summary["degraded_count"], 0)

    def test_hp3_summary_covers_every_watchlist_cwe(self) -> None:
        stub = _stub_dispatch_factory({})
        with mock.patch.object(post_ci_summary, "run_pilot_touchpoint", side_effect=stub):
            summary = post_ci_summary.build_summary(
                snapshot_root=Path("."),
                snapshot_id="sha-2",
                triage_owner="security-team",
                retention_days=30,
            )
        cwe_ids = {r["cwe_id"] for r in summary["cwe_results"]}
        self.assertEqual(cwe_ids, {"CWE-89", "CWE-306", "CWE-22"})

    def test_hp3_main_writes_summary_and_returns_zero(self) -> None:
        stub = _stub_dispatch_factory({})
        with mock.patch.object(post_ci_summary, "run_pilot_touchpoint", side_effect=stub):
            with __import__("tempfile").TemporaryDirectory() as tmp:
                out_dir = Path(tmp) / "out"
                rc = post_ci_summary.main(
                    [
                        "--snapshot-root",
                        ".",
                        "--snapshot-id",
                        "sha-3",
                        "--out-dir",
                        str(out_dir),
                    ]
                )
                self.assertEqual(rc, 0)
                self.assertTrue((out_dir / "antares-post-ci-summary.json").exists())


class EdgeCaseTest(unittest.TestCase):
    def test_ec1_degraded_cwe_never_raises_and_is_counted(self) -> None:
        def _degraded(*, touchpoint, cwe_id, snapshot_root, snapshot_id, triage_owner):
            return SimpleNamespace(
                touchpoint=touchpoint,
                cwe_id=cwe_id,
                snapshot_id=snapshot_id,
                degraded=True,
                detail="antares-cli not on PATH",
                candidates=(),
                ledger_entries=(),
            )

        with mock.patch.object(post_ci_summary, "run_pilot_touchpoint", side_effect=_degraded):
            summary = post_ci_summary.build_summary(
                snapshot_root=Path("."),
                snapshot_id="sha-4",
                triage_owner="security-team",
                retention_days=30,
            )
        self.assertEqual(summary["degraded_count"], 3)
        self.assertEqual(summary["total_candidates"], 0)

    def test_ec1_empty_triage_owner_degrades_instead_of_crashing_the_job(self) -> None:
        # A caller-input problem (e.g. an operator misconfiguring
        # --triage-owner as empty) must degrade the same as an Antares
        # runtime failure -- it must never propagate as an uncaught
        # PilotError and kill the CI step's exit code.
        with __import__("tempfile").TemporaryDirectory() as tmp:
            out_dir = Path(tmp) / "out"
            rc = post_ci_summary.main(
                [
                    "--snapshot-root",
                    ".",
                    "--snapshot-id",
                    "sha-6",
                    "--triage-owner",
                    "",
                    "--out-dir",
                    str(out_dir),
                ]
            )
            self.assertEqual(rc, 0)
            written = (out_dir / "antares-post-ci-summary.json").read_text()
            self.assertIn('"degraded": true', written)

    def test_ec1_main_returns_zero_even_when_every_cwe_degrades(self) -> None:
        def _degraded(*, touchpoint, cwe_id, snapshot_root, snapshot_id, triage_owner):
            return SimpleNamespace(
                touchpoint=touchpoint,
                cwe_id=cwe_id,
                snapshot_id=snapshot_id,
                degraded=True,
                detail="boom",
                candidates=(),
                ledger_entries=(),
            )

        with mock.patch.object(post_ci_summary, "run_pilot_touchpoint", side_effect=_degraded):
            with __import__("tempfile").TemporaryDirectory() as tmp:
                out_dir = Path(tmp) / "out"
                rc = post_ci_summary.main(
                    [
                        "--snapshot-root",
                        ".",
                        "--snapshot-id",
                        "sha-5",
                        "--out-dir",
                        str(out_dir),
                    ]
                )
                self.assertEqual(rc, 0)


if __name__ == "__main__":
    unittest.main()
