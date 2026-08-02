"""Packet schema and hard security-exclusion guarantees (T3b).

Builds deterministic Antares packets over an explicit, already-given path
list. This module does not discover repository context on its own -- that is
T3c's job. Its responsibilities are narrower:

1. bind a justified CWE hypothesis to the packet;
2. canonicalize/deduplicate the caller-supplied path list;
3. exclude sensitive/generated/out-of-snapshot paths before size budgeting;
4. enforce a deterministic size-budget policy (`fail-closed` or
   `deterministic-partition`);
5. serialize the resulting packet deterministically.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from dataclasses import dataclass
from enum import Enum
from pathlib import Path, PurePosixPath
from typing import Any

SCHEMA_VERSION = 1

_SENSITIVE_SEGMENTS = frozenset({"credentials", "secrets"})
_SENSITIVE_SUFFIXES = frozenset({".crt", ".key", ".pem", ".p12", ".pfx"})
_SENSITIVE_FILENAMES = frozenset({"id_ed25519", "id_rsa"})
_GENERATED_TOP_LEVEL_DIRS = frozenset(
    {".git", ".pytest_cache", "__pycache__", "build", "coverage", "dist", "logs", "target"}
)
_GENERATED_SUFFIXES = frozenset({".pyc", ".pyo"})
_GENERATED_FILENAMES = frozenset({".coverage"})
CONTEXT_CLOSURE_NO_SEED_PATH = "__seed__"
CONTEXT_CLOSURE_NO_SEED_REASON = "context_closure_no_seed"
CONTEXT_CLOSURE_OMISSION_REASONS = frozenset(
    {
        CONTEXT_CLOSURE_NO_SEED_REASON,
        "context_closure_unsupported_file_type",
        "context_closure_missing_governing_boundary",
        "context_closure_expansion_limit_reached",
    }
)
_OMISSION_REASONS = frozenset(
    {
        "path_outside_snapshot",
        "security_excluded_credentials",
        "security_excluded_env_file",
        "security_excluded_generated_output",
        "security_excluded_production_config",
        "size_budget_fragment_omitted_remainder",
        "size_budget_omitted",
        *CONTEXT_CLOSURE_OMISSION_REASONS,
    }
)


def _load_sibling_module(module_name: str, filename: str):
    if module_name in sys.modules:
        return sys.modules[module_name]
    script = Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(module_name, script)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load script spec for {script}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = mod
    spec.loader.exec_module(mod)
    return mod


_CWE_WATCHLIST_MOD = _load_sibling_module("antares_cwe_watchlist", "cwe_watchlist.py")
load_watchlist = _CWE_WATCHLIST_MOD.load_watchlist
_PATH_CONTAINMENT_MOD = _load_sibling_module("antares_path_containment", "path_containment.py")
resolve_within_snapshot = _PATH_CONTAINMENT_MOD.resolve_within_snapshot


class PacketValidationError(ValueError):
    """Raised when a packet or packet input violates the T3b contract."""


class PacketSizeBudgetExceeded(PacketValidationError):
    """Raised when `fail-closed` size-budget enforcement rejects a packet."""


class CweSource(Enum):
    WATCHLIST = "watchlist"
    EXPLICIT = "explicit"


class SizeBudgetPolicy(Enum):
    FAIL_CLOSED = "fail-closed"
    DETERMINISTIC_PARTITION = "deterministic-partition"


@dataclass(frozen=True)
class CweHypothesis:
    cwe_id: str
    description: str
    source: CweSource

@dataclass(frozen=True)
class FragmentMetadata:
    start_byte: int
    end_byte_exclusive: int
    total_file_bytes: int

@dataclass(frozen=True)
class IncludedPath:
    path: str
    content: str
    byte_length: int
    sha256: str
    fragment: FragmentMetadata | None = None


@dataclass(frozen=True)
class OmittedPath:
    path: str
    reason: str
    detail: str

@dataclass(frozen=True)
class Packet:
    schema_version: int
    cwe_id: str
    cwe_description: str
    cwe_source: str
    baseline_snapshot_id: str
    candidate_snapshot_id: str
    size_budget_bytes: int
    budget_policy: str
    included: tuple[IncludedPath, ...]
    omitted: tuple[OmittedPath, ...]


def hypothesis_from_watchlist(cwe_id: str) -> CweHypothesis:
    watchlist = load_watchlist()
    entry = watchlist.get(cwe_id)
    if entry is None:
        raise PacketValidationError(f"{cwe_id!r} is not present in the T3a watchlist.")
    return CweHypothesis(
        cwe_id=entry.cwe_id,
        description=entry.description,
        source=CweSource.WATCHLIST,
    )


def explicit_hypothesis(cwe_id: str, description: str) -> CweHypothesis:
    if not cwe_id.strip():
        raise PacketValidationError("explicit hypothesis requires a non-empty cwe_id.")
    if not description.strip():
        raise PacketValidationError("explicit hypothesis requires a non-empty description.")
    return CweHypothesis(
        cwe_id=cwe_id.strip(),
        description=description.strip(),
        source=CweSource.EXPLICIT,
    )


def build_packet(
    hypothesis: CweHypothesis,
    *,
    baseline_snapshot_id: str,
    candidate_snapshot_id: str,
    snapshot_root: Path,
    raw_paths: tuple[str, ...],
    size_budget_bytes: int,
    budget_policy: SizeBudgetPolicy = SizeBudgetPolicy.FAIL_CLOSED,
) -> Packet:
    if not baseline_snapshot_id.strip():
        raise PacketValidationError("baseline_snapshot_id must be non-empty.")
    if not candidate_snapshot_id.strip():
        raise PacketValidationError("candidate_snapshot_id must be non-empty.")
    if size_budget_bytes <= 0:
        raise PacketValidationError("size_budget_bytes must be > 0.")
    if not raw_paths:
        raise PacketValidationError("raw_paths must contain at least one path.")

    root = snapshot_root.resolve()
    staged_includes: list[tuple[str, Path]] = []
    omitted: list[OmittedPath] = []
    seen_paths: set[str] = set()

    for raw_path in raw_paths:
        recorded_path = _canonical_record_path(raw_path, root)
        resolved = resolve_within_snapshot(raw_path, root)
        if resolved is None:
            if recorded_path not in seen_paths:
                omitted.append(
                    OmittedPath(
                        path=recorded_path,
                        reason="path_outside_snapshot",
                        detail=(
                            f"Path {raw_path!r} resolves outside snapshot root "
                            f"{root.as_posix()} after canonical resolution."
                        ),
                    )
                )
                seen_paths.add(recorded_path)
            continue

        canonical_rel = validate_context_closure_seed_path(resolved.relative_to(root).as_posix())
        if canonical_rel in seen_paths:
            continue
        seen_paths.add(canonical_rel)

        exclusion = _security_exclusion_for(canonical_rel)
        if exclusion is not None:
            reason, detail = exclusion
            omitted.append(OmittedPath(path=canonical_rel, reason=reason, detail=detail))
            continue

        staged_includes.append((canonical_rel, resolved))

    staged_includes.sort(key=lambda item: item[0])
    omitted.sort(key=lambda item: (item.path, item.reason))

    included = _apply_size_budget(
        staged_includes=tuple(staged_includes),
        size_budget_bytes=size_budget_bytes,
        budget_policy=budget_policy,
        omitted=omitted,
    )
    included.sort(key=lambda entry: entry.path)
    omitted.sort(key=lambda entry: (entry.path, entry.reason))

    packet = Packet(
        schema_version=SCHEMA_VERSION,
        cwe_id=hypothesis.cwe_id,
        cwe_description=hypothesis.description,
        cwe_source=hypothesis.source.value,
        baseline_snapshot_id=baseline_snapshot_id.strip(),
        candidate_snapshot_id=candidate_snapshot_id.strip(),
        size_budget_bytes=size_budget_bytes,
        budget_policy=budget_policy.value,
        included=tuple(included),
        omitted=tuple(omitted),
    )
    validate_packet(packet)
    return packet


def validate_packet(packet: Packet) -> None:
    if packet.schema_version != SCHEMA_VERSION:
        raise PacketValidationError(
            f"unsupported schema_version {packet.schema_version}; expected {SCHEMA_VERSION}."
        )
    if packet.cwe_source not in {source.value for source in CweSource}:
        raise PacketValidationError(f"unsupported cwe_source {packet.cwe_source!r}.")
    if not packet.cwe_id.strip():
        raise PacketValidationError("packet cwe_id must be non-empty.")
    if not packet.cwe_description.strip():
        raise PacketValidationError("packet cwe_description must be non-empty.")
    if not packet.baseline_snapshot_id.strip() or not packet.candidate_snapshot_id.strip():
        raise PacketValidationError("packet snapshot identities must be non-empty.")
    if packet.size_budget_bytes <= 0:
        raise PacketValidationError("packet size_budget_bytes must be > 0.")
    included_bytes = 0
    seen_included: set[str] = set()
    for included in packet.included:
        _validate_relative_path(included.path)
        if included.path == CONTEXT_CLOSURE_NO_SEED_PATH:
            raise PacketValidationError(
                "__seed__ is reserved for the empty-seed sentinel and cannot be included."
            )
        if included.path in seen_included:
            raise PacketValidationError(f"duplicate included path {included.path!r}.")
        seen_included.add(included.path)
        if included.byte_length <= 0:
            raise PacketValidationError(f"included path {included.path!r} must have bytes.")
        if included.byte_length != len(included.content.encode("utf-8")):
            raise PacketValidationError(
                f"included path {included.path!r} byte_length does not match serialized content."
            )
        if included.sha256 != _sha256_hex(included.content.encode("utf-8")):
            raise PacketValidationError(
                f"included path {included.path!r} sha256 does not match serialized content."
            )
        if included.fragment is None:
            pass
        else:
            fragment = included.fragment
            if fragment.start_byte != 0:
                raise PacketValidationError("fragment start_byte must be 0 for deterministic prefix partitioning.")
            if fragment.end_byte_exclusive != included.byte_length:
                raise PacketValidationError(
                    "fragment end_byte_exclusive must equal the included byte_length."
                )
            if fragment.total_file_bytes <= fragment.end_byte_exclusive:
                raise PacketValidationError(
                    "fragment total_file_bytes must exceed the included byte_length."
                )
        included_bytes += included.byte_length

    if included_bytes > packet.size_budget_bytes:
        raise PacketValidationError("included bytes exceed packet size budget.")

    seen_omitted: set[tuple[str, str]] = set()
    for omitted in packet.omitted:
        if omitted.reason not in _OMISSION_REASONS:
            raise PacketValidationError(f"unsupported omission reason {omitted.reason!r}.")
        if not omitted.detail.strip():
            raise PacketValidationError("omitted-path detail must be non-empty.")
        if omitted.reason == "path_outside_snapshot":
            if not Path(omitted.path).is_absolute():
                raise PacketValidationError(
                    "path_outside_snapshot omissions must record the canonical absolute escape path."
                )
        else:
            _validate_relative_path(omitted.path)
        if omitted.reason == CONTEXT_CLOSURE_NO_SEED_REASON:
            if omitted.path != CONTEXT_CLOSURE_NO_SEED_PATH:
                raise PacketValidationError(
                    "context_closure_no_seed omissions must use the reserved __seed__ path."
                )
        elif omitted.path == CONTEXT_CLOSURE_NO_SEED_PATH:
            raise PacketValidationError(
                "__seed__ is reserved for the context_closure_no_seed sentinel omission."
            )
        identity = (omitted.path, omitted.reason)
        if identity in seen_omitted:
            raise PacketValidationError(f"duplicate omission record for {identity!r}.")
        seen_omitted.add(identity)


def packet_to_dict(packet: Packet) -> dict[str, Any]:
    validate_packet(packet)
    return {
        "schema_version": packet.schema_version,
        "cwe": {
            "id": packet.cwe_id,
            "description": packet.cwe_description,
            "source": packet.cwe_source,
        },
        "snapshot_identity": {
            "baseline": packet.baseline_snapshot_id,
            "candidate": packet.candidate_snapshot_id,
        },
        "size_budget_bytes": packet.size_budget_bytes,
        "budget_policy": packet.budget_policy,
        "included": [
            {
                "path": entry.path,
                "byte_length": entry.byte_length,
                "sha256": entry.sha256,
                "content_kind": "fragment" if entry.fragment is not None else "whole_file",
                "fragment": None
                if entry.fragment is None
                else {
                    "start_byte": entry.fragment.start_byte,
                    "end_byte_exclusive": entry.fragment.end_byte_exclusive,
                    "total_file_bytes": entry.fragment.total_file_bytes,
                },
                "content": entry.content,
            }
            for entry in packet.included
        ],
        "omitted": [
            {"path": entry.path, "reason": entry.reason, "detail": entry.detail}
            for entry in packet.omitted
        ],
    }


def serialize_packet(packet: Packet) -> str:
    return json.dumps(packet_to_dict(packet), sort_keys=True, separators=(",", ":"))


def _apply_size_budget(
    *,
    staged_includes: tuple[tuple[str, Path], ...],
    size_budget_bytes: int,
    budget_policy: SizeBudgetPolicy,
    omitted: list[OmittedPath],
) -> list[IncludedPath]:
    remaining = size_budget_bytes
    included: list[IncludedPath] = []

    for canonical_rel, resolved in staged_includes:
        raw_bytes = resolved.read_bytes()
        whole_entry = _whole_file_entry(canonical_rel, raw_bytes)
        if whole_entry.byte_length <= remaining:
            included.append(whole_entry)
            remaining -= whole_entry.byte_length
            continue

        if budget_policy == SizeBudgetPolicy.FAIL_CLOSED:
            raise PacketSizeBudgetExceeded(
                f"Packet exceeded size budget {size_budget_bytes} on path {canonical_rel!r}."
            )

        if remaining > 0:
            fragment_entry = _fragment_entry_with_budget(
                canonical_rel, raw_bytes, remaining
            )
            included.append(fragment_entry)
            omitted.append(
                OmittedPath(
                    path=canonical_rel,
                    reason="size_budget_fragment_omitted_remainder",
                    detail=(
                        f"Included deterministic prefix bytes [0:{fragment_entry.byte_length}) "
                        f"of {len(raw_bytes)} "
                        f"for {canonical_rel!r}; omitted the remainder to stay within budget."
                    ),
                )
            )
            remaining = 0
            continue

        omitted.append(
            OmittedPath(
                path=canonical_rel,
                reason="size_budget_omitted",
                detail=(
                    f"Size budget already exhausted before {canonical_rel!r}; "
                    "file omitted deterministically."
                ),
            )
        )

    return included


def _whole_file_entry(path: str, raw_bytes: bytes) -> IncludedPath:
    content = raw_bytes.decode("utf-8", errors="replace")
    encoded = content.encode("utf-8")
    return IncludedPath(
        path=path,
        content=content,
        byte_length=len(encoded),
        sha256=_sha256_hex(encoded),
    )


def _fragment_entry(path: str, raw_bytes: bytes, total_file_bytes: int) -> IncludedPath:
    content = raw_bytes.decode("utf-8", errors="replace")
    encoded = content.encode("utf-8")
    return IncludedPath(
        path=path,
        content=content,
        byte_length=len(encoded),
        sha256=_sha256_hex(encoded),
        fragment=FragmentMetadata(
            start_byte=0,
            end_byte_exclusive=len(encoded),
            total_file_bytes=total_file_bytes,
        ),
    )


def _fragment_entry_with_budget(path: str, raw_bytes: bytes, byte_budget: int) -> IncludedPath:
    low = 1
    high = min(len(raw_bytes), byte_budget)
    best: IncludedPath | None = None
    while low <= high:
        mid = (low + high) // 2
        candidate = _fragment_entry(path, raw_bytes[:mid], len(raw_bytes))
        if candidate.byte_length <= byte_budget:
            best = candidate
            low = mid + 1
        else:
            high = mid - 1
    if best is None or best.byte_length <= 0:
        raise PacketSizeBudgetExceeded(
            f"Packet budget {byte_budget} for {path!r} could not hold any serialized bytes."
        )
    return best


def _validate_relative_path(path_text: str) -> None:
    path = PurePosixPath(path_text)
    if not path_text or path.is_absolute() or ".." in path.parts:
        raise PacketValidationError(f"path {path_text!r} is not a canonical relative packet path.")


def build_context_closure_no_seed_omission(detail: str | None = None) -> OmittedPath:
    normalized_detail = (
        detail.strip()
        if detail is not None
        else "No changed paths were provided; context closure must not widen into repository scan."
    )
    if not normalized_detail:
        raise PacketValidationError("context_closure_no_seed detail must be non-empty.")
    return OmittedPath(
        path=CONTEXT_CLOSURE_NO_SEED_PATH,
        reason=CONTEXT_CLOSURE_NO_SEED_REASON,
        detail=normalized_detail,
    )


def validate_context_closure_seed_path(path_text: str) -> str:
    _validate_relative_path(path_text)
    if path_text == CONTEXT_CLOSURE_NO_SEED_PATH:
        raise PacketValidationError(
            "__seed__ is reserved for the empty-seed sentinel; use "
            "build_context_closure_no_seed_omission() instead of passing it as a real path."
        )
    return path_text


def canonicalize_context_closure_seed_path(raw_path: str, snapshot_root: Path) -> str | None:
    candidate = PurePosixPath(raw_path)
    if not raw_path.strip() or candidate.is_absolute() or ".." in candidate.parts:
        return None
    resolved_root = snapshot_root.resolve()
    resolved = (resolved_root / Path(raw_path)).resolve()
    try:
        canonical_rel = resolved.relative_to(resolved_root).as_posix()
    except ValueError:
        return None
    return validate_context_closure_seed_path(canonical_rel)


def deterministic_context_closure_seed_order(
    paths: tuple[str | None, ...] | list[str | None],
) -> tuple[str, ...]:
    canonical_paths: list[str] = []
    for path_text in paths:
        if not isinstance(path_text, str):
            raise PacketValidationError(
                "context closure seed order received an invalid or non-canonical path."
            )
        canonical_paths.append(validate_context_closure_seed_path(path_text))
    return tuple(sorted(canonical_paths))


def _canonical_record_path(raw_path: str, snapshot_root: Path) -> str:
    if not raw_path.strip():
        return raw_path
    path = Path(raw_path)
    if path.is_absolute():
        return path.resolve(strict=False).as_posix()
    return (snapshot_root / path).resolve(strict=False).as_posix()


def _security_exclusion_for(canonical_rel: str) -> tuple[str, str] | None:
    path = PurePosixPath(canonical_rel)
    name = path.name
    parts = set(path.parts)

    if len(path.parts) >= 2 and tuple(path.parts[-2:]) == ("config", "production.toml"):
        return (
            "security_excluded_production_config",
            "config/production.toml is always excluded from Antares packets.",
        )
    if name == ".env" or name.startswith(".env."):
        return (
            "security_excluded_env_file",
            ".env files are always excluded from Antares packets.",
        )
    if parts & _SENSITIVE_SEGMENTS or name in _SENSITIVE_FILENAMES or path.suffix in _SENSITIVE_SUFFIXES:
        return (
            "security_excluded_credentials",
            "Credential-bearing paths are always excluded from Antares packets.",
        )
    if (
        path.parts
        and path.parts[0] in _GENERATED_TOP_LEVEL_DIRS
        or "__pycache__" in parts
        or name in _GENERATED_FILENAMES
        or path.suffix in _GENERATED_SUFFIXES
    ):
        return (
            "security_excluded_generated_output",
            "Generated output is always excluded from Antares packets.",
        )
    return None


def _sha256_hex(raw_bytes: bytes) -> str:
    return "sha256:" + hashlib.sha256(raw_bytes).hexdigest()
