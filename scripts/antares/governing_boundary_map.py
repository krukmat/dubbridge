"""Committed governing-boundary map for T3c-2.

Keys are canonical repository boundary roots (aligned with
`cwe_watchlist.py` `CweWatchlistEntry.repository_boundary` values); each
value is the tuple of canonical governing-context target paths (code,
manifest, or governance-doc anchors) that govern that boundary. This module
holds only committed data plus its own fail-closed validation -- it does not
resolve anything against a T3c-1 closure result (see
`governing_boundary_closure.py` for the resolver).
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path, PurePosixPath


class GoverningBoundaryMapValidationError(Exception):
    """Fail-closed: raised when the committed map itself is malformed. No
    partial or best-effort map is ever returned."""

    def __init__(self, reason: str, detail: str) -> None:
        super().__init__(f"{reason}: {detail}")
        self.reason = reason
        self.detail = detail


@dataclass(frozen=True)
class GoverningBoundaryMap:
    entries: dict[str, tuple[str, ...]]


GOVERNING_BOUNDARY_MAP: dict[str, tuple[str, ...]] = {
    "crates/db/": (
        "crates/db/Cargo.toml",
        "docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md",
    ),
    "apps/api/": (
        "apps/api/Cargo.toml",
        "docs/architecture.md",
    ),
    "crates/storage/": (
        "crates/storage/Cargo.toml",
        "docs/architecture.md",
    ),
}


def _is_canonical_boundary_root(root: str) -> bool:
    if not root or not root.endswith("/"):
        return False
    candidate = PurePosixPath(root)
    if candidate.is_absolute() or ".." in candidate.parts:
        return False
    return True


def _is_canonical_target(target: str) -> bool:
    if not target or target.endswith("/"):
        return False
    candidate = PurePosixPath(target)
    if candidate.is_absolute() or ".." in candidate.parts:
        return False
    return True


def validate_governing_boundary_map(
    raw_map: dict[str, tuple[str, ...]],
    *,
    snapshot_root: Path,
) -> GoverningBoundaryMap:
    """Fail-closed validation of the committed map against an explicit
    snapshot root (EC-2, EC-3). Duplicate keys cannot occur in a Python
    dict literal, so "duplicate boundary keys" is enforced structurally by
    construction; this function still re-validates key/target canonicality
    and snapshot containment, which a dict literal cannot guarantee."""
    if not isinstance(raw_map, dict):
        raise GoverningBoundaryMapValidationError(
            "invalid_map", "Governing boundary map must be a dict."
        )

    resolved_root = snapshot_root.resolve()
    validated: dict[str, tuple[str, ...]] = {}
    seen_canonical_roots: dict[str, str] = {}

    for root, targets in raw_map.items():
        if not _is_canonical_boundary_root(root):
            raise GoverningBoundaryMapValidationError(
                "non_canonical_boundary_key",
                f"Boundary root {root!r} is not a canonical snapshot-relative "
                "POSIX directory (must be relative, use '/' separators, end "
                "with '/', and contain no '..' segment).",
            )
        canonical_root = PurePosixPath(root).as_posix() + "/"
        if canonical_root in seen_canonical_roots:
            raise GoverningBoundaryMapValidationError(
                "conflicting_target_mapping",
                f"Boundary key {root!r} collides with "
                f"{seen_canonical_roots[canonical_root]!r} after "
                f"canonicalization (both normalize to {canonical_root!r}).",
            )
        seen_canonical_roots[canonical_root] = root
        if not isinstance(targets, tuple) or not targets:
            raise GoverningBoundaryMapValidationError(
                "invalid_targets",
                f"Boundary root {root!r} must map to a non-empty tuple of "
                "target paths.",
            )

        canonical_targets: list[str] = []
        for target in targets:
            if not _is_canonical_target(target):
                raise GoverningBoundaryMapValidationError(
                    "non_canonical_target",
                    f"Governing-context target {target!r} for boundary "
                    f"{root!r} is not a canonical snapshot-relative POSIX "
                    "file path.",
                )
            abs_target = (resolved_root / target).resolve()
            try:
                canonical_rel = abs_target.relative_to(resolved_root).as_posix()
            except ValueError as exc:
                raise GoverningBoundaryMapValidationError(
                    "target_outside_snapshot",
                    f"Governing-context target {target!r} for boundary "
                    f"{root!r} resolves outside the snapshot root.",
                ) from exc
            if not abs_target.is_file():
                raise GoverningBoundaryMapValidationError(
                    "missing_target",
                    f"Governing-context target {target!r} for boundary "
                    f"{root!r} does not exist in the snapshot.",
                )
            canonical_targets.append(canonical_rel)

        validated[root] = tuple(sorted(dict.fromkeys(canonical_targets)))

    return GoverningBoundaryMap(entries=validated)
