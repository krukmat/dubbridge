"""Unit tests for T4's observe-only pilot runner."""

from __future__ import annotations

import unittest
from pathlib import Path
from types import SimpleNamespace

from scripts.antares.disposition_ledger import DispositionLedger, Touchpoint
from scripts.antares.pilot import (
    PilotError,
    record_pilot_run,
    run_pilot_touchpoint,
    skip_pilot_touchpoint,
)


def _success_dispatch(candidates: tuple[str, ...]):
    def _dispatch(request, *, snapshot_root):
        return SimpleNamespace(
            kind=SimpleNamespace(value="cli_execution_complete"),
            candidates=candidates,
            detail="",
        )

    return _dispatch


def _failure_dispatch(kind_value: str, detail: str):
    def _dispatch(request, *, snapshot_root):
        return SimpleNamespace(kind=SimpleNamespace(value=kind_value), candidates=(), detail=detail)

    return _dispatch


def _raising_dispatch(exc: Exception):
    def _dispatch(request, *, snapshot_root):
        raise exc

    return _dispatch


class HappyPathTest(unittest.TestCase):
    def test_hp2_successful_run_produces_ledger_entries_for_each_candidate(self) -> None:
        result = run_pilot_touchpoint(
            touchpoint=Touchpoint.REFINEMENT,
            cwe_id="CWE-89",
            snapshot_root=Path("."),
            snapshot_id="snap-1",
            triage_owner="security-team",
            dispatch=_success_dispatch(("crates/db/src/query.rs", "crates/db/src/pool.rs")),
        )
        self.assertFalse(result.degraded)
        self.assertEqual(result.candidates, ("crates/db/src/query.rs", "crates/db/src/pool.rs"))
        self.assertEqual(len(result.ledger_entries), 2)
        ledger = DispositionLedger()
        record_pilot_run(ledger, result)
        self.assertEqual(len(ledger.entries), 2)

    def test_hp2_never_blocks_on_result_it_only_records(self) -> None:
        # run_pilot_touchpoint returns a plain data result; nothing about it
        # can raise to short-circuit an approval or CI decision downstream.
        result = run_pilot_touchpoint(
            touchpoint=Touchpoint.POST_CI,
            cwe_id="CWE-22",
            snapshot_root=Path("."),
            snapshot_id="snap-2",
            triage_owner="security-team",
            dispatch=_success_dispatch(()),
        )
        self.assertFalse(result.degraded)
        self.assertEqual(result.candidates, ())
        self.assertEqual(result.ledger_entries, ())


class EdgeCaseTest(unittest.TestCase):
    def test_ec1_dispatch_exception_degrades_without_raising(self) -> None:
        result = run_pilot_touchpoint(
            touchpoint=Touchpoint.POST_IMPLEMENTATION,
            cwe_id="CWE-306",
            snapshot_root=Path("."),
            snapshot_id="snap-3",
            triage_owner="security-team",
            dispatch=_raising_dispatch(RuntimeError("subprocess exploded")),
        )
        self.assertTrue(result.degraded)
        self.assertIn("subprocess exploded", result.detail)
        self.assertEqual(result.ledger_entries, ())

    def test_ec1_cli_binary_unavailable_degrades_without_raising(self) -> None:
        result = run_pilot_touchpoint(
            touchpoint=Touchpoint.REFINEMENT,
            cwe_id="CWE-89",
            snapshot_root=Path("."),
            snapshot_id="snap-4",
            triage_owner="security-team",
            dispatch=_failure_dispatch("cli_binary_unavailable", "antares not on PATH"),
        )
        self.assertTrue(result.degraded)
        self.assertEqual(result.detail, "antares not on PATH")
        self.assertEqual(result.candidates, ())

    def test_ec1_degraded_result_never_yields_ledger_entries(self) -> None:
        result = run_pilot_touchpoint(
            touchpoint=Touchpoint.POST_CI,
            cwe_id="CWE-89",
            snapshot_root=Path("."),
            snapshot_id="snap-5",
            triage_owner="security-team",
            dispatch=_failure_dispatch("cli_output_malformed", "bad json"),
        )
        ledger = DispositionLedger()
        record_pilot_run(ledger, result)  # must not raise
        self.assertEqual(len(ledger.entries), 0)

    def test_ec2_unknown_cwe_raises_before_any_dispatch(self) -> None:
        calls = []

        def _dispatch(request, *, snapshot_root):
            calls.append(request)
            raise AssertionError("dispatch must not be called for an unwatchlisted CWE")

        with self.assertRaises(PilotError) as ctx:
            run_pilot_touchpoint(
                touchpoint=Touchpoint.REFINEMENT,
                cwe_id="CWE-9999-not-real",
                snapshot_root=Path("."),
                snapshot_id="snap-6",
                triage_owner="security-team",
                dispatch=_dispatch,
            )
        self.assertEqual(ctx.exception.code, "cwe_not_on_watchlist")
        self.assertEqual(calls, [])

    def test_ec2_skip_records_typed_reason_and_never_dispatches(self) -> None:
        skip = skip_pilot_touchpoint(Touchpoint.POST_CI, reason="no CWE eligible this cycle")
        self.assertEqual(skip.touchpoint, Touchpoint.POST_CI)
        self.assertIn("no CWE eligible this cycle", skip.detail)

    def test_ec2_skip_with_empty_reason_rejected(self) -> None:
        with self.assertRaises(PilotError) as ctx:
            skip_pilot_touchpoint(Touchpoint.POST_CI, reason="   ")
        self.assertEqual(ctx.exception.code, "empty_skip_reason")

    def test_ec1_dispatch_return_value_missing_kind_attribute_degrades(self) -> None:
        def _dispatch(request, *, snapshot_root):
            return object()  # no .kind at all, unlike a real TerminalState

        result = run_pilot_touchpoint(
            touchpoint=Touchpoint.REFINEMENT,
            cwe_id="CWE-89",
            snapshot_root=Path("."),
            snapshot_id="snap-8",
            triage_owner="security-team",
            dispatch=_dispatch,
        )
        self.assertTrue(result.degraded)
        self.assertEqual(result.ledger_entries, ())

    def test_ec_empty_triage_owner_rejected(self) -> None:
        with self.assertRaises(PilotError) as ctx:
            run_pilot_touchpoint(
                touchpoint=Touchpoint.REFINEMENT,
                cwe_id="CWE-89",
                snapshot_root=Path("."),
                snapshot_id="snap-7",
                triage_owner="   ",
                dispatch=_success_dispatch(()),
            )
        self.assertEqual(ctx.exception.code, "empty_triage_owner")


if __name__ == "__main__":
    unittest.main()
