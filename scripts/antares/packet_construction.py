"""Touchpoint packet construction (T3d).

Integrates the already-implemented T3a watchlist, T3b packet schema, and
T3c-1/T3c-2 context/boundary closure into the two touchpoint-facing
entrypoints that actually construct an Antares packet: a refinement/review
packet and a post-CI watchlist-entry packet. This module composes only the
existing public contracts of its predecessors -- it introduces no new
omission-reason vocabulary and never hands a raw, un-closed caller path
directly to `build_packet`.
"""

from __future__ import annotations

from pathlib import Path

from scripts.antares.context_closure import compute_context_closure
from scripts.antares.governing_boundary_closure import resolve_governing_boundary_closure
from scripts.antares.packet_schema import (
    SCHEMA_VERSION,
    CweHypothesis,
    Packet,
    SizeBudgetPolicy,
    build_context_closure_no_seed_omission,
    build_packet,
    hypothesis_from_watchlist,
    validate_packet,
)
from scripts.antares.cwe_watchlist import load_watchlist, WatchlistValidationError


def _merged_raw_paths(t3c1_included: tuple[str, ...], t3c2_included: tuple[str, ...]) -> tuple[str, ...]:
    return tuple(sorted(set(t3c1_included) | set(t3c2_included)))


def _merge_omissions(packet: Packet, *extra_omitted_groups: tuple) -> Packet:
    """Fold T3c-1/T3c-2 omissions into `packet.omitted`, deduplicated on
    `(path, reason)` so an omission surfaced by more than one predecessor (or
    already present in `packet.omitted`) is never emitted twice."""
    seen: dict[tuple[str, str], object] = {}
    for existing in packet.omitted:
        seen[(existing.path, existing.reason)] = existing
    for group in extra_omitted_groups:
        for omission in group:
            key = (omission.path, omission.reason)
            if key not in seen:
                seen[key] = omission
    merged = tuple(sorted(seen.values(), key=lambda o: (o.path, o.reason)))
    return Packet(
        schema_version=packet.schema_version,
        cwe_id=packet.cwe_id,
        cwe_description=packet.cwe_description,
        cwe_source=packet.cwe_source,
        baseline_snapshot_id=packet.baseline_snapshot_id,
        candidate_snapshot_id=packet.candidate_snapshot_id,
        size_budget_bytes=packet.size_budget_bytes,
        budget_policy=packet.budget_policy,
        included=packet.included,
        omitted=merged,
    )


def _compose_packet(
    hypothesis: CweHypothesis,
    *,
    repository_boundary: str,
    snapshot_root: Path,
    baseline_snapshot_id: str,
    candidate_snapshot_id: str,
    changed_paths: tuple[str, ...],
    governing_map: dict[str, tuple[str, ...]],
    size_budget_bytes: int,
    expansion_limit: int | None,
    budget_policy: SizeBudgetPolicy,
) -> Packet:
    t3c1_result = compute_context_closure(
        snapshot_root, changed_paths, expansion_limit=expansion_limit
    )
    t3c2_result = resolve_governing_boundary_closure(
        snapshot_root, t3c1_result.included, (repository_boundary,), governing_map
    )
    raw_paths = _merged_raw_paths(t3c1_result.included, t3c2_result.included)

    if not raw_paths:
        # Both closures resolved to zero included paths -- either the caller
        # supplied no seed at all, or every candidate was omitted (e.g. an
        # expansion-limit cutoff or a boundary miss). Either way there is
        # nothing left to hand to build_packet (it requires >=1 raw path), so
        # the packet is constructed directly from the merged omissions alone.
        # An empty-seed call still needs the frozen no-seed sentinel omission
        # T3c-1 itself would have produced; every other empty-raw_paths case
        # already carries a real omission from T3c-1/T3c-2.
        omissions = t3c1_result.omitted + t3c2_result.omitted
        if not omissions:
            omissions = (build_context_closure_no_seed_omission(),)
        return _packet_with_no_includes(
            hypothesis,
            baseline_snapshot_id=baseline_snapshot_id,
            candidate_snapshot_id=candidate_snapshot_id,
            size_budget_bytes=size_budget_bytes,
            budget_policy=budget_policy,
            omitted=omissions,
        )

    packet = build_packet(
        hypothesis,
        baseline_snapshot_id=baseline_snapshot_id,
        candidate_snapshot_id=candidate_snapshot_id,
        snapshot_root=snapshot_root,
        raw_paths=raw_paths,
        size_budget_bytes=size_budget_bytes,
        budget_policy=budget_policy,
    )
    return _merge_omissions(packet, t3c1_result.omitted, t3c2_result.omitted)


def _packet_with_no_includes(
    hypothesis: CweHypothesis,
    *,
    baseline_snapshot_id: str,
    candidate_snapshot_id: str,
    size_budget_bytes: int,
    budget_policy: SizeBudgetPolicy,
    omitted: tuple,
) -> Packet:
    deduped: dict[tuple[str, str], object] = {}
    for omission in omitted:
        deduped[(omission.path, omission.reason)] = omission
    packet = Packet(
        schema_version=SCHEMA_VERSION,
        cwe_id=hypothesis.cwe_id,
        cwe_description=hypothesis.description,
        cwe_source=hypothesis.source.value,
        baseline_snapshot_id=baseline_snapshot_id.strip(),
        candidate_snapshot_id=candidate_snapshot_id.strip(),
        size_budget_bytes=size_budget_bytes,
        budget_policy=budget_policy.value,
        included=(),
        omitted=tuple(sorted(deduped.values(), key=lambda o: (o.path, o.reason))),
    )
    validate_packet(packet)
    return packet


def build_refinement_packet(
    hypothesis: CweHypothesis,
    *,
    repository_boundary: str,
    snapshot_root: Path,
    baseline_snapshot_id: str,
    candidate_snapshot_id: str,
    changed_paths: tuple[str, ...],
    governing_map: dict[str, tuple[str, ...]],
    size_budget_bytes: int,
    expansion_limit: int | None = None,
    budget_policy: SizeBudgetPolicy = SizeBudgetPolicy.FAIL_CLOSED,
) -> Packet:
    """Refinement/review touchpoint entrypoint.

    `hypothesis` is caller-resolved (via `hypothesis_from_watchlist` or
    `explicit_hypothesis`); `repository_boundary` is the caller-declared
    canonical boundary root to resolve T3c-2 against. `changed_paths` are raw
    seed paths -- they are never passed to `build_packet` directly; they
    always go through `compute_context_closure` (T3c-1) and
    `resolve_governing_boundary_closure` (T3c-2) first.
    """
    return _compose_packet(
        hypothesis,
        repository_boundary=repository_boundary,
        snapshot_root=snapshot_root,
        baseline_snapshot_id=baseline_snapshot_id,
        candidate_snapshot_id=candidate_snapshot_id,
        changed_paths=changed_paths,
        governing_map=governing_map,
        size_budget_bytes=size_budget_bytes,
        expansion_limit=expansion_limit,
        budget_policy=budget_policy,
    )


def build_watchlist_entry_packet(
    cwe_id: str,
    *,
    snapshot_root: Path,
    baseline_snapshot_id: str,
    candidate_snapshot_id: str,
    changed_paths: tuple[str, ...],
    governing_map: dict[str, tuple[str, ...]],
    size_budget_bytes: int,
    expansion_limit: int | None = None,
    budget_policy: SizeBudgetPolicy = SizeBudgetPolicy.FAIL_CLOSED,
) -> Packet:
    """Post-CI watchlist-entry touchpoint entrypoint.

    Resolves the `CweHypothesis` and `repository_boundary` from the T3a
    watchlist by `cwe_id`, then delegates to the same composition path as
    `build_refinement_packet`.
    """
    watchlist = load_watchlist()
    entry = watchlist.get(cwe_id)
    if entry is None:
        raise WatchlistValidationError(f"{cwe_id!r} is not present in the T3a watchlist.")
    hypothesis = hypothesis_from_watchlist(cwe_id)
    return _compose_packet(
        hypothesis,
        repository_boundary=entry.repository_boundary,
        snapshot_root=snapshot_root,
        baseline_snapshot_id=baseline_snapshot_id,
        candidate_snapshot_id=candidate_snapshot_id,
        changed_paths=changed_paths,
        governing_map=governing_map,
        size_budget_bytes=size_budget_bytes,
        expansion_limit=expansion_limit,
        budget_policy=budget_policy,
    )
