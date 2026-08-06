"""Deterministic governing security-boundary closure (T3c-2).

Given an explicit snapshot root, the canonical T3c-1 closure result, and one
or more caller-declared repository boundary roots, compute the deterministic,
bounded set of additional files that govern the relevant security boundary
for those paths. This module resolves against the committed
`governing_boundary_map` only -- no ambient repository scan, git state, or
network access. It never guesses a nearest parent boundary when coverage is
missing, and never builds packets or decides size-budget policy (T3d).
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path, PurePosixPath

from scripts.antares.governing_boundary_map import (
    GoverningBoundaryMap,
    GoverningBoundaryMapValidationError,
    validate_governing_boundary_map,
)
from scripts.antares.packet_schema import (
    OmittedPath,
    canonicalize_context_closure_seed_path,
)

_MISSING_GOVERNING_BOUNDARY_REASON = "context_closure_missing_governing_boundary"


class GoverningBoundaryValidationError(Exception):
    """Fail-closed: raised when the boundary roots, the committed map, or a
    closure path cannot be deterministically resolved. No partial result is
    ever returned."""

    def __init__(self, reason: str, detail: str) -> None:
        super().__init__(f"{reason}: {detail}")
        self.reason = reason
        self.detail = detail


@dataclass(frozen=True)
class GoverningBoundaryClosureResult:
    included: tuple[str, ...]
    omitted: tuple[OmittedPath, ...]


def _canonicalize_boundary_root(raw_root: str, snapshot_root: Path) -> str:
    if not raw_root or not raw_root.endswith("/"):
        raise GoverningBoundaryValidationError(
            "non_canonical_boundary_root",
            f"Boundary root {raw_root!r} must be a non-empty relative path "
            "ending with '/'.",
        )
    candidate = PurePosixPath(raw_root)
    if candidate.is_absolute() or ".." in candidate.parts:
        raise GoverningBoundaryValidationError(
            "non_canonical_boundary_root",
            f"Boundary root {raw_root!r} must be relative and contain no "
            "'..' segment.",
        )
    resolved_root = snapshot_root.resolve()
    resolved = (resolved_root / raw_root).resolve()
    try:
        canonical_rel = resolved.relative_to(resolved_root).as_posix()
    except ValueError as exc:
        raise GoverningBoundaryValidationError(
            "boundary_root_outside_snapshot",
            f"Boundary root {raw_root!r} resolves outside the snapshot root.",
        ) from exc
    return canonical_rel + "/" if canonical_rel != "." else "./"


def _longest_prefix_match(closure_path: str, boundary_roots: tuple[str, ...]) -> str | None:
    matches = [root for root in boundary_roots if closure_path.startswith(root)]
    if not matches:
        return None
    return max(matches, key=len)


def resolve_governing_boundary_closure(
    snapshot_root: Path,
    t3c1_closure_paths: tuple[str, ...],
    boundary_roots: tuple[str, ...],
    governing_map: dict[str, tuple[str, ...]],
) -> GoverningBoundaryClosureResult:
    """T3c-2 entry point.

    `t3c1_closure_paths` are the canonical, already-sorted paths returned by
    `context_closure.compute_context_closure` (T3c-1). `boundary_roots` are
    caller-declared canonical repository boundary roots (e.g. from T3a
    watchlist entries). `governing_map` is the committed
    `GOVERNING_BOUNDARY_MAP` (or an equivalent dict for tests). Raises
    `GoverningBoundaryValidationError` and returns no partial result on any
    map or root defect.
    """
    try:
        validated_map: GoverningBoundaryMap = validate_governing_boundary_map(
            governing_map, snapshot_root=snapshot_root
        )
    except GoverningBoundaryMapValidationError as exc:
        raise GoverningBoundaryValidationError(exc.reason, exc.detail) from exc

    canonical_roots: list[str] = []
    for raw_root in boundary_roots:
        canonical_roots.append(_canonicalize_boundary_root(raw_root, snapshot_root))

    for root in canonical_roots:
        if root not in validated_map.entries:
            raise GoverningBoundaryValidationError(
                "undeclared_boundary_root",
                f"Boundary root {root!r} has no entry in the committed "
                "governing boundary map.",
            )

    ordered_roots = tuple(sorted(dict.fromkeys(canonical_roots)))

    included: set[str] = set()
    omissions: dict[tuple[str, str], OmittedPath] = {}

    for raw_closure_path in t3c1_closure_paths:
        canonical_closure_path = canonicalize_context_closure_seed_path(
            raw_closure_path, snapshot_root
        )
        if canonical_closure_path is None:
            raise GoverningBoundaryValidationError(
                "closure_path_outside_snapshot",
                f"T3c-1 closure path {raw_closure_path!r} resolves outside "
                "the snapshot root.",
            )

        matched_root = _longest_prefix_match(canonical_closure_path, ordered_roots)
        if matched_root is None:
            key = (canonical_closure_path, _MISSING_GOVERNING_BOUNDARY_REASON)
            omissions[key] = OmittedPath(
                path=canonical_closure_path,
                reason=_MISSING_GOVERNING_BOUNDARY_REASON,
                detail=(
                    f"{canonical_closure_path!r} is not covered by any "
                    "declared repository boundary root."
                ),
            )
            continue

        for target in validated_map.entries[matched_root]:
            included.add(target)

    included_sorted = tuple(sorted(included))
    omitted_sorted = tuple(sorted(omissions.values(), key=lambda o: (o.path, o.reason)))
    return GoverningBoundaryClosureResult(included=included_sorted, omitted=omitted_sorted)
