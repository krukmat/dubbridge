"""Observe-only pilot disposition ledger (T4).

Tracks the durable human disposition of every Antares candidate produced
during the pilot, independent of the calibration metrics in
`calibration.py`. `rejected` here always means a human decided the candidate
was not useful -- it is never a substitute for an adjudicated
false-positive label, and only `calibration.py`'s ground-truth-backed
metrics may support false-positive/precision claims (see
docs/tasks/antares-security-specialist-advisor.md T4 acceptance criteria).

This module is pure in-memory bookkeeping; the caller owns persistence
(JSON serialization is provided via `entry_to_dict`/`entry_from_dict` but
writing to disk is the caller's responsibility, consistent with
`artifact_schema.py`'s trace-ref split between validation and I/O).
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Any


class LedgerError(ValueError):
    """A fail-closed rejection with a stable machine-readable code."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


class Touchpoint(Enum):
    """The three non-blocking invocation points named in the task ledger."""

    REFINEMENT = "refinement"
    POST_IMPLEMENTATION = "post-implementation"
    POST_CI = "post-ci"


class DispositionState(Enum):
    """Durable human dispositions for a pilot candidate. Mirrors
    `artifact_schema.DispositionState`'s vocabulary but is declared
    independently -- the ledger's disposition state is about pilot triage
    outcome, not about the underlying artifact's own lifecycle state, and
    the two must be allowed to diverge without a shared enum coupling them."""

    NEEDS_HUMAN_REVIEW = "needs-human-review"
    ACCEPTED_NOW = "accepted-now"
    ACCEPTED_FOLLOW_UP = "accepted-follow-up"
    REJECTED = "rejected"


@dataclass(frozen=True)
class LedgerEntry:
    """One pilot candidate's durable triage record.

    `dedup_key` is caller-computed (see `compute_dedup_key`) so ledger
    callers can detect the same underlying candidate surfacing at more than
    one touchpoint without the ledger itself guessing at similarity.
    `follow_up_ref` is mandatory once `state` is `ACCEPTED_FOLLOW_UP` (EC-3's
    "link from accepted-follow-up to a task/refinement record").
    """

    entry_id: str
    touchpoint: Touchpoint
    cwe_id: str
    candidate_file: str
    dedup_key: str
    created_at: str
    triage_owner: str
    sla_deadline: str
    state: DispositionState = DispositionState.NEEDS_HUMAN_REVIEW
    reviewer: str | None = None
    reviewed_at: str | None = None
    note: str | None = None
    follow_up_ref: str | None = None


def compute_dedup_key(*, cwe_id: str, candidate_file: str, snapshot_id: str) -> str:
    """Deterministic dedup key: same CWE + same candidate file + same
    snapshot always collapses to the same key, independent of touchpoint or
    timestamp, so the same underlying finding surfacing at refinement and
    again post-implementation is detectable as a duplicate."""
    raw = f"{cwe_id.strip()}|{candidate_file.strip()}|{snapshot_id.strip()}"
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()


def _validate_new_entry(entry: LedgerEntry) -> None:
    if not entry.entry_id.strip():
        raise LedgerError("empty_entry_id", "LedgerEntry.entry_id must be non-empty.")
    if not entry.triage_owner.strip():
        raise LedgerError(
            "empty_triage_owner",
            f"LedgerEntry {entry.entry_id!r} must name a triage_owner -- undisposed "
            "candidates must always have a named owner for SLA backlog reporting.",
        )
    if not entry.sla_deadline.strip():
        raise LedgerError(
            "empty_sla_deadline", f"LedgerEntry {entry.entry_id!r} must declare an sla_deadline."
        )
    if entry.state is not DispositionState.NEEDS_HUMAN_REVIEW:
        raise LedgerError(
            "new_entry_must_start_needs_review",
            f"LedgerEntry {entry.entry_id!r} must be created with state=NEEDS_HUMAN_REVIEW; "
            "nothing may be pre-closed at creation time.",
        )


def _validate_transition(entry: LedgerEntry, new_state: DispositionState, follow_up_ref: str | None) -> None:
    if entry.state != DispositionState.NEEDS_HUMAN_REVIEW:
        raise LedgerError(
            "already_dispositioned",
            f"LedgerEntry {entry.entry_id!r} already carries a durable disposition "
            f"({entry.state.value!r}); dispositions are not re-opened by this ledger.",
        )
    if new_state is DispositionState.ACCEPTED_FOLLOW_UP and not (follow_up_ref or "").strip():
        raise LedgerError(
            "missing_follow_up_ref",
            f"LedgerEntry {entry.entry_id!r} disposition ACCEPTED_FOLLOW_UP requires a "
            "non-empty follow_up_ref linking to a task/refinement record.",
        )


@dataclass
class DispositionLedger:
    """In-memory ledger of pilot candidates keyed by `entry_id`."""

    entries: dict[str, LedgerEntry] = field(default_factory=dict)

    def add(self, entry: LedgerEntry) -> None:
        _validate_new_entry(entry)
        if entry.entry_id in self.entries:
            raise LedgerError(
                "duplicate_entry_id", f"entry_id {entry.entry_id!r} already exists in the ledger."
            )
        self.entries[entry.entry_id] = entry

    def disposition(
        self,
        entry_id: str,
        *,
        state: DispositionState,
        reviewer: str,
        reviewed_at: str,
        note: str | None = None,
        follow_up_ref: str | None = None,
    ) -> LedgerEntry:
        """Apply a durable human disposition to an existing entry. `state`
        must not be `NEEDS_HUMAN_REVIEW` (that is the pre-disposition
        default, not a valid target) and the entry must not already carry a
        disposition -- see `_validate_transition`."""
        if entry_id not in self.entries:
            raise LedgerError("unknown_entry_id", f"entry_id {entry_id!r} is not in the ledger.")
        if state is DispositionState.NEEDS_HUMAN_REVIEW:
            raise LedgerError(
                "invalid_target_state",
                "disposition() cannot set state back to NEEDS_HUMAN_REVIEW; that is "
                "only the creation default.",
            )
        if not reviewer.strip():
            raise LedgerError("empty_reviewer", "disposition() requires a non-empty reviewer.")

        entry = self.entries[entry_id]
        _validate_transition(entry, state, follow_up_ref)
        updated = LedgerEntry(
            entry_id=entry.entry_id,
            touchpoint=entry.touchpoint,
            cwe_id=entry.cwe_id,
            candidate_file=entry.candidate_file,
            dedup_key=entry.dedup_key,
            created_at=entry.created_at,
            triage_owner=entry.triage_owner,
            sla_deadline=entry.sla_deadline,
            state=state,
            reviewer=reviewer,
            reviewed_at=reviewed_at,
            note=note,
            follow_up_ref=follow_up_ref,
        )
        self.entries[entry_id] = updated
        return updated

    def undisposed(self) -> tuple[LedgerEntry, ...]:
        return tuple(
            e for e in self.entries.values() if e.state is DispositionState.NEEDS_HUMAN_REVIEW
        )

    def backlog(self, *, now: str) -> tuple[LedgerEntry, ...]:
        """EC-3: undisposed candidates past their SLA deadline, reported as
        backlog rather than silently closed. Comparison is lexicographic on
        ISO-8601 UTC strings, which sort correctly by construction."""
        return tuple(e for e in self.undisposed() if e.sla_deadline < now)

    def duplicates_of(self, dedup_key: str) -> tuple[LedgerEntry, ...]:
        return tuple(e for e in self.entries.values() if e.dedup_key == dedup_key)


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def entry_to_dict(entry: LedgerEntry) -> dict[str, Any]:
    return {
        "entry_id": entry.entry_id,
        "touchpoint": entry.touchpoint.value,
        "cwe_id": entry.cwe_id,
        "candidate_file": entry.candidate_file,
        "dedup_key": entry.dedup_key,
        "created_at": entry.created_at,
        "triage_owner": entry.triage_owner,
        "sla_deadline": entry.sla_deadline,
        "state": entry.state.value,
        "reviewer": entry.reviewer,
        "reviewed_at": entry.reviewed_at,
        "note": entry.note,
        "follow_up_ref": entry.follow_up_ref,
    }


def entry_from_dict(data: dict[str, Any]) -> LedgerEntry:
    try:
        return LedgerEntry(
            entry_id=data["entry_id"],
            touchpoint=Touchpoint(data["touchpoint"]),
            cwe_id=data["cwe_id"],
            candidate_file=data["candidate_file"],
            dedup_key=data["dedup_key"],
            created_at=data["created_at"],
            triage_owner=data["triage_owner"],
            sla_deadline=data["sla_deadline"],
            state=DispositionState(data["state"]),
            reviewer=data.get("reviewer"),
            reviewed_at=data.get("reviewed_at"),
            note=data.get("note"),
            follow_up_ref=data.get("follow_up_ref"),
        )
    except KeyError as exc:
        raise LedgerError("missing_field", f"LedgerEntry dict is missing required field {exc}.") from exc
    except ValueError as exc:
        raise LedgerError("invalid_enum_value", str(exc)) from exc
