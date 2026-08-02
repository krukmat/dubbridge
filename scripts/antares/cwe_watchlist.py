"""Versioned CWE watchlist (T3a).

The sole source of justified CWE hypotheses for downstream Antares packet
construction (T3b/T3c/T3d). Antares never invents its own CWE hypothesis --
every candidate it may act on must trace back to an entry in this static,
human-curated, versioned list.

CWE-732 (Incorrect Permission Assignment for Critical Resource) is
deliberately excluded from the initial watchlist: it is a weak/overbroad
class for this repository's current detection precision (see
docs/tasks/antares-security-specialist-advisor.md), not an oversight.
"""

from __future__ import annotations

from dataclasses import dataclass

WATCHLIST_VERSION = "2026-08-02.1"


class WatchlistValidationError(ValueError):
    """Raised when a watchlist entry or the watchlist itself is malformed."""


@dataclass(frozen=True)
class CweWatchlistEntry:
    cwe_id: str
    description: str
    repository_boundary: str
    owner: str
    justification: str


_REQUIRED_FIELDS = ("cwe_id", "description", "repository_boundary", "owner", "justification")


def validate_entry(entry: CweWatchlistEntry) -> None:
    """Raise WatchlistValidationError naming the first missing/empty field."""
    for field_name in _REQUIRED_FIELDS:
        value = getattr(entry, field_name)
        if not isinstance(value, str) or not value.strip():
            raise WatchlistValidationError(f"watchlist entry missing required field: {field_name}")


class CweWatchlist:
    """An ordered, deterministic, version-stamped collection of validated entries."""

    def __init__(self, version: str, entries: tuple[CweWatchlistEntry, ...]):
        self.version = version
        self._entries = entries
        self._by_id = {}
        for entry in entries:
            validate_entry(entry)
            if entry.cwe_id in self._by_id:
                raise WatchlistValidationError(
                    f"duplicate cwe_id in watchlist: {entry.cwe_id}"
                )
            self._by_id[entry.cwe_id] = entry

    @property
    def entries(self) -> tuple[CweWatchlistEntry, ...]:
        return self._entries

    def get(self, cwe_id: str) -> CweWatchlistEntry | None:
        """Return the entry for cwe_id, or None if it is not on the watchlist."""
        return self._by_id.get(cwe_id)

    def cwe_ids(self) -> tuple[str, ...]:
        return tuple(entry.cwe_id for entry in self._entries)


# Static, versioned, human-curated seed entries. Order here is the
# authoritative deterministic load order (HP-2).
_SEED_ENTRIES: tuple[CweWatchlistEntry, ...] = (
    CweWatchlistEntry(
        cwe_id="CWE-89",
        description="Improper Neutralization of Special Elements used in an SQL Command "
        "('SQL Injection')",
        repository_boundary="crates/db/",
        owner="security-team",
        justification=(
            "crates/db is the sole SQLx repository boundary and the system of "
            "record (CLAUDE.md); any raw-query or string-interpolated SQL path "
            "introduced there is a direct injection risk against the primary "
            "data store."
        ),
    ),
    CweWatchlistEntry(
        cwe_id="CWE-306",
        description="Missing Authentication for Critical Function",
        repository_boundary="apps/api/",
        owner="security-team",
        justification=(
            "apps/api exposes the HTTP surface for asset ingestion, rights, "
            "finalize, and playback endpoints; a handler wired without the "
            "auth crate's scope enforcement would silently expose a "
            "governance-critical function (ADR-008 fail-closed rights gate)."
        ),
    ),
    CweWatchlistEntry(
        cwe_id="CWE-22",
        description="Improper Limitation of a Pathname to a Restricted Directory "
        "('Path Traversal')",
        repository_boundary="crates/storage/",
        owner="security-team",
        justification=(
            "crates/storage owns the canonical key layout for local-fs and "
            "S3-compatible backends; a key or path built from unsanitized "
            "input could escape the intended storage root."
        ),
    ),
)


def load_watchlist() -> CweWatchlist:
    """Return the current versioned CweWatchlist.

    Deterministic and side-effect free: builds the same CweWatchlist instance
    (same version, same entry order) on every call.
    """
    return CweWatchlist(WATCHLIST_VERSION, _SEED_ENTRIES)
