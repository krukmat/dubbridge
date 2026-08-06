"""Unit tests for T4's observe-only pilot disposition ledger."""

from __future__ import annotations

import unittest

from scripts.antares.disposition_ledger import (
    DispositionLedger,
    DispositionState,
    LedgerEntry,
    LedgerError,
    Touchpoint,
    compute_dedup_key,
    entry_from_dict,
    entry_to_dict,
)
import dataclasses


def _entry(entry_id: str = "e1", **overrides) -> LedgerEntry:
    defaults = dict(
        entry_id=entry_id,
        touchpoint=Touchpoint.REFINEMENT,
        cwe_id="CWE-89",
        candidate_file="crates/db/src/query.rs",
        dedup_key=compute_dedup_key(
            cwe_id="CWE-89", candidate_file="crates/db/src/query.rs", snapshot_id="snap-1"
        ),
        created_at="2026-08-06T00:00:00Z",
        triage_owner="security-team",
        sla_deadline="2026-08-09T00:00:00Z",
    )
    defaults.update(overrides)
    return LedgerEntry(**defaults)


class HappyPathTest(unittest.TestCase):
    def test_hp2_add_then_disposition_records_reviewer_and_state(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry())
        updated = ledger.disposition(
            "e1",
            state=DispositionState.ACCEPTED_NOW,
            reviewer="alice",
            reviewed_at="2026-08-06T01:00:00Z",
        )
        self.assertEqual(updated.state, DispositionState.ACCEPTED_NOW)
        self.assertEqual(updated.reviewer, "alice")
        self.assertEqual(ledger.undisposed(), ())

    def test_hp2_rejected_is_a_human_disposition_not_a_metric(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry())
        updated = ledger.disposition(
            "e1", state=DispositionState.REJECTED, reviewer="bob", reviewed_at="2026-08-06T01:00:00Z"
        )
        self.assertEqual(updated.state, DispositionState.REJECTED)

    def test_hp2_accepted_follow_up_requires_and_stores_ref(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry())
        updated = ledger.disposition(
            "e1",
            state=DispositionState.ACCEPTED_FOLLOW_UP,
            reviewer="carol",
            reviewed_at="2026-08-06T01:00:00Z",
            follow_up_ref="docs/tasks/some-refinement.md#T7",
        )
        self.assertEqual(updated.follow_up_ref, "docs/tasks/some-refinement.md#T7")

    def test_hp2_dedup_key_collapses_same_candidate_across_touchpoints(self) -> None:
        ledger = DispositionLedger()
        e1 = _entry(entry_id="e1", touchpoint=Touchpoint.REFINEMENT)
        e2 = _entry(entry_id="e2", touchpoint=Touchpoint.POST_IMPLEMENTATION)
        ledger.add(e1)
        ledger.add(e2)
        dupes = ledger.duplicates_of(e1.dedup_key)
        self.assertEqual({d.entry_id for d in dupes}, {"e1", "e2"})

    def test_hp2_roundtrip_serialization(self) -> None:
        entry = _entry()
        restored = entry_from_dict(entry_to_dict(entry))
        self.assertEqual(restored, entry)


class EdgeCaseTest(unittest.TestCase):
    def test_ec3_backlog_reports_undisposed_past_sla_not_silently_closed(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry(sla_deadline="2026-08-01T00:00:00Z"))
        backlog = ledger.backlog(now="2026-08-06T00:00:00Z")
        self.assertEqual(len(backlog), 1)
        self.assertEqual(backlog[0].entry_id, "e1")
        # Still present in the ledger, not removed/closed.
        self.assertIn("e1", ledger.entries)
        self.assertEqual(ledger.entries["e1"].state, DispositionState.NEEDS_HUMAN_REVIEW)

    def test_ec3_within_sla_is_not_backlog(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry(sla_deadline="2026-08-09T00:00:00Z"))
        backlog = ledger.backlog(now="2026-08-06T00:00:00Z")
        self.assertEqual(backlog, ())

    def test_ec_new_entry_must_start_needs_human_review(self) -> None:
        ledger = DispositionLedger()
        with self.assertRaises(LedgerError) as ctx:
            ledger.add(_entry(state=DispositionState.ACCEPTED_NOW))
        self.assertEqual(ctx.exception.code, "new_entry_must_start_needs_review")

    def test_ec_duplicate_entry_id_rejected(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry())
        with self.assertRaises(LedgerError) as ctx:
            ledger.add(_entry())
        self.assertEqual(ctx.exception.code, "duplicate_entry_id")

    def test_ec_disposition_twice_rejected_not_reopened(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry())
        ledger.disposition(
            "e1", state=DispositionState.REJECTED, reviewer="bob", reviewed_at="2026-08-06T01:00:00Z"
        )
        with self.assertRaises(LedgerError) as ctx:
            ledger.disposition(
                "e1",
                state=DispositionState.ACCEPTED_NOW,
                reviewer="carol",
                reviewed_at="2026-08-06T02:00:00Z",
            )
        self.assertEqual(ctx.exception.code, "already_dispositioned")

    def test_ec_accepted_follow_up_without_ref_rejected(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry())
        with self.assertRaises(LedgerError) as ctx:
            ledger.disposition(
                "e1",
                state=DispositionState.ACCEPTED_FOLLOW_UP,
                reviewer="carol",
                reviewed_at="2026-08-06T01:00:00Z",
            )
        self.assertEqual(ctx.exception.code, "missing_follow_up_ref")

    def test_ec_empty_triage_owner_rejected_at_creation(self) -> None:
        ledger = DispositionLedger()
        with self.assertRaises(LedgerError) as ctx:
            ledger.add(_entry(triage_owner=""))
        self.assertEqual(ctx.exception.code, "empty_triage_owner")

    def test_ec_unknown_entry_id_disposition_rejected(self) -> None:
        ledger = DispositionLedger()
        with self.assertRaises(LedgerError) as ctx:
            ledger.disposition(
                "missing",
                state=DispositionState.ACCEPTED_NOW,
                reviewer="alice",
                reviewed_at="2026-08-06T01:00:00Z",
            )
        self.assertEqual(ctx.exception.code, "unknown_entry_id")

    def test_ec_empty_entry_id_rejected_at_creation(self) -> None:
        ledger = DispositionLedger()
        with self.assertRaises(LedgerError) as ctx:
            ledger.add(_entry(entry_id="   "))
        self.assertEqual(ctx.exception.code, "empty_entry_id")

    def test_ec_empty_sla_deadline_rejected_at_creation(self) -> None:
        ledger = DispositionLedger()
        with self.assertRaises(LedgerError) as ctx:
            ledger.add(_entry(sla_deadline=""))
        self.assertEqual(ctx.exception.code, "empty_sla_deadline")

    def test_ec_disposition_cannot_target_needs_human_review(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry())
        with self.assertRaises(LedgerError) as ctx:
            ledger.disposition(
                "e1",
                state=DispositionState.NEEDS_HUMAN_REVIEW,
                reviewer="alice",
                reviewed_at="2026-08-06T01:00:00Z",
            )
        self.assertEqual(ctx.exception.code, "invalid_target_state")

    def test_ec_empty_reviewer_rejected(self) -> None:
        ledger = DispositionLedger()
        ledger.add(_entry())
        with self.assertRaises(LedgerError) as ctx:
            ledger.disposition(
                "e1", state=DispositionState.ACCEPTED_NOW, reviewer="  ", reviewed_at="2026-08-06T01:00:00Z"
            )
        self.assertEqual(ctx.exception.code, "empty_reviewer")

    def test_ec_entry_from_dict_missing_field_raises_ledger_error(self) -> None:
        data = entry_to_dict(_entry())
        del data["triage_owner"]
        with self.assertRaises(LedgerError) as ctx:
            entry_from_dict(data)
        self.assertEqual(ctx.exception.code, "missing_field")

    def test_ec_entry_from_dict_invalid_enum_value_raises_ledger_error(self) -> None:
        data = entry_to_dict(_entry())
        data["state"] = "not-a-real-state"
        with self.assertRaises(LedgerError) as ctx:
            entry_from_dict(data)
        self.assertEqual(ctx.exception.code, "invalid_enum_value")


if __name__ == "__main__":
    unittest.main()
