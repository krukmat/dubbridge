"""Deterministic local dependency and manifest closure (T3c-1).

Given an explicit snapshot root and a set of canonical changed paths (already
resolved by T3b / `packet_schema.py`), compute the bounded, deduplicated set
of local Rust/Python source dependencies and relevant local manifests that
those seeds pull in. This module does not discover or trust anything beyond
the declared snapshot root: no ambient repository scan, package cache/index,
Cargo/pip resolution, subprocess, or network access.

Scope boundary: this module resolves *what is locally reachable*. It does not
resolve governing security boundaries (T3c-2) or assemble the final Antares
packet (T3d). The frozen T3c-0 omission vocabulary and the T3b containment
contract in `packet_schema.py` are reused verbatim; no new omission reason is
introduced here.
"""

from __future__ import annotations

import ast
import configparser
import re
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any

try:
    import tomllib as _toml  # Python >= 3.11
except ModuleNotFoundError:  # pragma: no cover - exercised on Python 3.9/3.10
    import tomli as _toml  # type: ignore[no-redef]

from scripts.antares.packet_schema import (
    CONTEXT_CLOSURE_NO_SEED_REASON,
    OmittedPath,
    build_context_closure_no_seed_omission,
    canonicalize_context_closure_seed_path,
    deterministic_context_closure_seed_order,
)

_UNSUPPORTED_FILE_TYPE_REASON = "context_closure_unsupported_file_type"
_EXPANSION_LIMIT_REASON = "context_closure_expansion_limit_reached"
_OUTSIDE_SNAPSHOT_REASON = "path_outside_snapshot"

_MANIFEST_BASENAMES = frozenset({"Cargo.toml", "Cargo.lock", "pyproject.toml", "setup.py", "setup.cfg"})
_REQUIREMENTS_RE = re.compile(r"^requirements(?:-[A-Za-z0-9_.-]+)?\.txt$")

_SOURCE_SUFFIXES = frozenset({".rs", ".py"})


class ContextClosureResolutionError(Exception):
    """Fail-closed: raised when a local edge cannot be deterministically
    resolved. No partial closure or fabricated omission is ever returned."""

    def __init__(self, reason: str, detail: str) -> None:
        super().__init__(f"{reason}: {detail}")
        self.reason = reason
        self.detail = detail


@dataclass(frozen=True)
class ClosureResult:
    included: tuple[str, ...]
    omitted: tuple[OmittedPath, ...]


def _is_manifest_basename(name: str) -> bool:
    return name in _MANIFEST_BASENAMES or bool(_REQUIREMENTS_RE.match(name))


def _normalize_posix(path: PurePosixPath) -> PurePosixPath:
    """Lexically collapse `.`/`..` segments in a snapshot-relative path
    without touching the filesystem (EC-8: canonical, deterministic
    identity). A `..` that would ascend above the snapshot root collapses
    to itself at the root, matching EC-4's containment check downstream."""
    parts: list[str] = []
    for part in path.parts:
        if part == ".":
            continue
        if part == "..":
            if parts:
                parts.pop()
            continue
        parts.append(part)
    return PurePosixPath(*parts) if parts else PurePosixPath(".")


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError as exc:
        raise ContextClosureResolutionError(
            "invalid_manifest_encoding", f"{path.as_posix()!s} is not valid UTF-8: {exc}"
        ) from exc
    except OSError as exc:
        raise ContextClosureResolutionError(
            "manifest_read_error", f"Unable to read {path.as_posix()!s}: {exc}"
        ) from exc


def _parse_toml(canonical_rel: str, text: str) -> dict[str, Any]:
    try:
        return _toml.loads(text)
    except Exception as exc:  # tomllib/tomli raise their own decode error types
        raise ContextClosureResolutionError(
            "malformed_manifest", f"{canonical_rel!r} is not valid TOML: {exc}"
        ) from exc


def _parse_ini(canonical_rel: str, text: str) -> configparser.ConfigParser:
    parser = configparser.ConfigParser()
    try:
        parser.read_string(text)
    except configparser.Error as exc:
        raise ContextClosureResolutionError(
            "malformed_manifest", f"{canonical_rel!r} is not valid INI: {exc}"
        ) from exc
    return parser


@dataclass(frozen=True)
class _Snapshot:
    root: Path

    def canonical(self, raw_path: str) -> str | None:
        return canonicalize_context_closure_seed_path(raw_path, self.root)

    def abs_path(self, canonical_rel: str) -> Path:
        return self.root / canonical_rel

    def exists(self, canonical_rel: str) -> bool:
        return self.abs_path(canonical_rel).is_file()


def _rust_mod_targets(canonical_rel: str, mod_name: str, snapshot: _Snapshot) -> str:
    """HP-1: resolve `mod name;` to `<parent>/<name>.rs` then
    `<parent>/<name>/mod.rs`, matching Rust's own module resolution order."""
    parent = PurePosixPath(canonical_rel).parent
    file_candidate = str(parent / f"{mod_name}.rs")
    dir_candidate = str(parent / mod_name / "mod.rs")
    if snapshot.exists(file_candidate):
        return file_candidate
    if snapshot.exists(dir_candidate):
        return dir_candidate
    raise ContextClosureResolutionError(
        "unresolved_rust_mod",
        f"mod {mod_name!r} declared in {canonical_rel!r} resolves to neither "
        f"{file_candidate!r} nor {dir_candidate!r}.",
    )


_RUST_MOD_RE = re.compile(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")


def _rust_mod_edges(canonical_rel: str, text: str, snapshot: _Snapshot) -> list[str]:
    edges: list[str] = []
    for match in _RUST_MOD_RE.finditer(text):
        edges.append(_rust_mod_targets(canonical_rel, match.group(1), snapshot))
    return edges


def _python_module_candidates(module_parts: tuple[str, ...]) -> tuple[str, str]:
    base = "/".join(module_parts)
    return f"{base}.py", f"{base}/__init__.py"


def _resolve_python_module(
    module_parts: tuple[str, ...], snapshot: _Snapshot, *, source_desc: str
) -> str | None:
    """Resolve a dotted module path against the fixed local mapping,
    unconditionally (caller decides whether this path should be treated as
    local at all). Returns the resolved canonical path, or None if neither
    candidate file exists."""
    if not module_parts:
        return None
    file_candidate, init_candidate = _python_module_candidates(module_parts)
    file_exists = snapshot.exists(file_candidate)
    init_exists = snapshot.exists(init_candidate)
    if file_exists and init_exists:
        raise ContextClosureResolutionError(
            "ambiguous_python_module",
            f"{source_desc}: module {'.'.join(module_parts)!r} matches both "
            f"{file_candidate!r} and {init_candidate!r}.",
        )
    if file_exists:
        return file_candidate
    if init_exists:
        return init_candidate
    return None


def _resolve_absolute_python_import(
    module_parts: tuple[str, ...], snapshot: _Snapshot, *, source_desc: str
) -> str | None:
    """HP-2/EC-9: `a.b.c` is local only if the top-level segment `a` exists
    as a package (`a/__init__.py`) -- a plain directory or a bare top-level
    `.py` module without that top-level `__init__.py` is external (ignored),
    matching 'absolute imports are local only for an existing top-level
    module/package with __init__.py'. Once local, resolve the full dotted
    path; an unresolved full path is a local edge that failed to resolve
    (EC-5), not an external import."""
    if not module_parts:
        return None
    top = module_parts[0]
    if not snapshot.exists(f"{top}/__init__.py"):
        return None  # plain directory, stdlib, or third-party: external.

    resolved = _resolve_python_module(module_parts, snapshot, source_desc=source_desc)
    if resolved is None:
        raise ContextClosureResolutionError(
            "unresolved_absolute_import",
            f"{source_desc}: absolute import {'.'.join(module_parts)!r} is under "
            f"local top-level package {top!r} but does not resolve to a file.",
        )
    return resolved


def _package_ancestor_dirs(canonical_rel: str, snapshot: _Snapshot) -> list[PurePosixPath]:
    """Walk upward from the importing file's directory while each directory
    has an `__init__.py`, collecting the package chain (nearest first)."""
    current = PurePosixPath(canonical_rel).parent
    chain: list[PurePosixPath] = []
    while snapshot.exists(str(current / "__init__.py")):
        chain.append(current)
        if current == PurePosixPath("."):
            break
        parent = current.parent
        if parent == current:
            break
        current = parent
    return chain


def _python_import_edges(canonical_rel: str, text: str, snapshot: _Snapshot) -> list[str]:
    try:
        tree = ast.parse(text, filename=canonical_rel)
    except SyntaxError as exc:
        raise ContextClosureResolutionError(
            "invalid_python_source", f"{canonical_rel!r} could not be parsed: {exc}"
        ) from exc

    edges: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                parts = tuple(alias.name.split("."))
                resolved = _resolve_absolute_python_import(
                    parts, snapshot, source_desc=canonical_rel
                )
                if resolved is not None:
                    edges.append(resolved)
        elif isinstance(node, ast.ImportFrom):
            edges.extend(_resolve_import_from(canonical_rel, node, snapshot))
    return edges


def _resolve_import_from(canonical_rel: str, node: ast.ImportFrom, snapshot: _Snapshot) -> list[str]:
    level = node.level or 0
    module = node.module

    if level == 0:
        # ast guarantees `module` is non-None when level == 0 (a bare
        # `from import x` is a syntax error and never parses).
        assert module is not None
        parts = tuple(module.split("."))
        resolved = _resolve_absolute_python_import(parts, snapshot, source_desc=canonical_rel)
        return [resolved] if resolved is not None else []

    # Relative import: always local (EC-9).
    package_chain = _package_ancestor_dirs(canonical_rel, snapshot)
    if not package_chain:
        raise ContextClosureResolutionError(
            "unresolved_relative_import",
            f"{canonical_rel!r} uses a relative import but has no ancestor "
            "package directory with __init__.py.",
        )
    # level=1 -> current package (package_chain[0]); level=2 -> its parent; etc.
    ascend_index = level - 1
    if ascend_index >= len(package_chain):
        raise ContextClosureResolutionError(
            "unresolved_relative_import",
            f"{canonical_rel!r} relative import ascends above its topmost "
            "package ancestor.",
        )
    base_dir = package_chain[ascend_index]

    if module is None:
        # `from . import name [, name2 ...]`
        edges: list[str] = []
        for alias in node.names:
            target_parts = tuple(str(base_dir).split("/")) + (alias.name,) if str(base_dir) != "." else (alias.name,)
            resolved = _resolve_python_module(target_parts, snapshot, source_desc=canonical_rel)
            if resolved is None:
                raise ContextClosureResolutionError(
                    "unresolved_relative_import",
                    f"{canonical_rel!r}: relative import target {alias.name!r} "
                    "could not be resolved locally.",
                )
            edges.append(resolved)
        return edges

    # `from .sub import x` -> resolves only `.sub`.
    sub_parts = tuple(module.split("."))
    target_parts = tuple(str(base_dir).split("/")) + sub_parts if str(base_dir) != "." else sub_parts
    resolved = _resolve_python_module(target_parts, snapshot, source_desc=canonical_rel)
    if resolved is None:
        raise ContextClosureResolutionError(
            "unresolved_relative_import",
            f"{canonical_rel!r}: relative import target {module!r} could not "
            "be resolved locally.",
        )
    return [resolved]


def _manifest_ancestors(canonical_rel: str, snapshot: _Snapshot) -> list[str]:
    """EC-7: walk from the containing directory to the snapshot root,
    inclusive, collecting every allowlisted manifest basename found."""
    current = PurePosixPath(canonical_rel).parent
    found: list[str] = []
    while True:
        if snapshot.abs_path(str(current)).is_dir():
            entries = sorted(p.name for p in snapshot.abs_path(str(current)).iterdir())
            for name in entries:
                if _is_manifest_basename(name):
                    candidate = str(current / name) if str(current) != "." else name
                    found.append(candidate)
        if current == PurePosixPath("."):
            break
        parent = current.parent
        if parent == current:
            break
        current = parent
    return found


def _nearest_package_cargo_toml(canonical_rel: str, snapshot: _Snapshot) -> str | None:
    """A package Cargo.toml directly containing the target -- no upward
    search. Used for path-dependency target resolution (EC-7)."""
    normalized = _normalize_posix(PurePosixPath(canonical_rel))
    candidate = str(normalized / "Cargo.toml") if str(normalized) != "." else "Cargo.toml"
    if snapshot.exists(candidate):
        return candidate
    return None


def _cargo_entrypoints(manifest_dir: PurePosixPath, table: dict[str, Any], snapshot: _Snapshot) -> list[str]:
    """EC-7: selected Rust entrypoints for a package manifest."""
    entrypoints: list[str] = []

    package = table.get("package")
    autobins = True
    if isinstance(package, dict) and package.get("autobins") is False:
        autobins = False

    lib_table = table.get("lib")
    if isinstance(lib_table, dict) and isinstance(lib_table.get("path"), str):
        entrypoints.append(str(_normalize_posix(manifest_dir / lib_table["path"])))
    else:
        default_lib = str(manifest_dir / "src" / "lib.rs")
        if snapshot.exists(default_lib):
            entrypoints.append(default_lib)

    explicit_bins = table.get("bin")
    if isinstance(explicit_bins, list):
        for entry in explicit_bins:
            if not isinstance(entry, dict):
                continue
            name = entry.get("name")
            path = entry.get("path")
            if isinstance(path, str):
                entrypoints.append(str(_normalize_posix(manifest_dir / path)))
            elif isinstance(name, str):
                file_candidate = str(manifest_dir / "src" / "bin" / f"{name}.rs")
                dir_candidate = str(manifest_dir / "src" / "bin" / name / "main.rs")
                file_exists = snapshot.exists(file_candidate)
                dir_exists = snapshot.exists(dir_candidate)
                if file_exists and dir_exists:
                    raise ContextClosureResolutionError(
                        "ambiguous_cargo_bin",
                        f"[[bin]] {name!r} without an explicit path matches both "
                        f"{file_candidate!r} and {dir_candidate!r}.",
                    )
                if file_exists:
                    entrypoints.append(file_candidate)
                elif dir_exists:
                    entrypoints.append(dir_candidate)
            else:
                raise ContextClosureResolutionError(
                    "invalid_cargo_bin_entry",
                    f"[[bin]] entry in {manifest_dir!s} has neither name nor path.",
                )

    if not explicit_bins and autobins:
        default_main = str(manifest_dir / "src" / "main.rs")
        if snapshot.exists(default_main):
            entrypoints.append(default_main)
        bin_dir = manifest_dir / "src" / "bin"
        bin_dir_abs = snapshot.abs_path(str(bin_dir))
        if bin_dir_abs.is_dir():
            for entry in sorted(bin_dir_abs.iterdir()):
                if entry.is_file() and entry.suffix == ".rs":
                    entrypoints.append(str(bin_dir / entry.name))
                elif entry.is_dir():
                    main_candidate = bin_dir / entry.name / "main.rs"
                    if snapshot.exists(str(main_candidate)):
                        entrypoints.append(str(main_candidate))

    return entrypoints


def _cargo_path_dependencies(manifest_dir: PurePosixPath, table: dict[str, Any]) -> list[str]:
    """EC-7: direct local `[dependencies]` path entries only (not dev/build,
    not workspace-level dependency tables)."""
    targets: list[str] = []
    deps = table.get("dependencies")
    if isinstance(deps, dict):
        for spec in deps.values():
            if isinstance(spec, dict) and isinstance(spec.get("path"), str):
                targets.append(str(_normalize_posix(manifest_dir / spec["path"])))
    return targets


@dataclass
class _ClosureState:
    snapshot: _Snapshot
    visited_sources: set[str]
    visited_manifests: set[str]
    context_manifests: set[str]
    omissions: dict[tuple[str, str], OmittedPath]
    expansion_limit: int | None
    consumed: int = 0


def _record_omission(state: _ClosureState, path: str, reason: str, detail: str) -> None:
    state.omissions[(path, reason)] = OmittedPath(path=path, reason=reason, detail=detail)


def _classify_seed(canonical_rel: str) -> str:
    name = PurePosixPath(canonical_rel).name
    if _is_manifest_basename(name):
        return "manifest"
    if PurePosixPath(canonical_rel).suffix in _SOURCE_SUFFIXES:
        return "source"
    return "unsupported"


def _process_manifest(state: _ClosureState, canonical_rel: str, pending: list[str]) -> None:
    if canonical_rel in state.visited_manifests:
        return
    state.visited_manifests.add(canonical_rel)
    state.context_manifests.add(canonical_rel)

    name = PurePosixPath(canonical_rel).name
    text = _read_text(state.snapshot.abs_path(canonical_rel))

    if name in ("Cargo.toml",):
        table = _parse_toml(canonical_rel, text)
        if not table:
            return  # empty manifest: context-only no-op (EC-7).
        manifest_dir = PurePosixPath(canonical_rel).parent
        is_package = isinstance(table.get("package"), dict)
        if is_package:
            for entrypoint in _cargo_entrypoints(manifest_dir, table, state.snapshot):
                if entrypoint not in state.visited_sources:
                    pending.append(entrypoint)
            for dep_manifest_dir in _cargo_path_dependencies(manifest_dir, table):
                target_manifest = _nearest_package_cargo_toml(str(dep_manifest_dir), state.snapshot)
                if target_manifest is None:
                    raise ContextClosureResolutionError(
                        "unresolved_cargo_path_dependency",
                        f"Path dependency at {dep_manifest_dir!s} does not directly "
                        "contain a package Cargo.toml.",
                    )
                if target_manifest not in state.visited_manifests:
                    _process_manifest(state, target_manifest, pending)
        # workspace-only manifests: context-only no-op (EC-7).
        return

    if name == "Cargo.lock":
        return  # context-only no-op.
    if name == "pyproject.toml":
        _parse_toml(canonical_rel, text)  # strict parse only; context-only.
        return
    if name == "setup.cfg":
        _parse_ini(canonical_rel, text)  # strict parse only; context-only.
        return
    if name == "setup.py":
        return  # UTF-8 decoded only (already done above), never executed.
    if _REQUIREMENTS_RE.match(name):
        return  # opaque lines; context-only.


def _process_source(state: _ClosureState, canonical_rel: str, pending: list[str]) -> None:
    if canonical_rel in state.visited_sources:
        return
    state.visited_sources.add(canonical_rel)

    if state.expansion_limit is not None:
        assert state.consumed < state.expansion_limit, (
            "compute_context_closure must stop a source at the expansion limit "
            "before calling _process_source; this is an internal invariant."
        )
        state.consumed += 1

    for ancestor in _manifest_ancestors(canonical_rel, state.snapshot):
        _process_manifest(state, ancestor, pending)

    suffix = PurePosixPath(canonical_rel).suffix
    text = _read_text(state.snapshot.abs_path(canonical_rel))
    if suffix == ".rs":
        edges = _rust_mod_edges(canonical_rel, text, state.snapshot)
    elif suffix == ".py":
        edges = _python_import_edges(canonical_rel, text, state.snapshot)
    else:
        raise AssertionError(
            f"compute_context_closure must classify {canonical_rel!r} as "
            "unsupported before calling _process_source; this is an internal "
            "invariant."
        )

    for edge in edges:
        if edge not in state.visited_sources:
            pending.append(edge)

    # Package-manifest / path-dependency-target Cargo.toml seeds enqueue
    # their entrypoints only when *that manifest* is the seed itself; ordinary
    # source files never enqueue sibling entrypoints. Nothing further here.


def compute_context_closure(
    snapshot_root: Path,
    changed_paths: tuple[str, ...],
    *,
    expansion_limit: int | None = None,
) -> ClosureResult:
    """T3c-1 entry point.

    `changed_paths` are raw (not-yet-canonicalized) seed paths, exactly as
    T3b would hand them off. `snapshot_root` is caller-supplied and never
    inferred. Returns a deterministic, deduplicated closure or raises
    `ContextClosureResolutionError` -- never a partial result.
    """
    snapshot = _Snapshot(root=snapshot_root)

    if not changed_paths:
        return ClosureResult(included=(), omitted=(build_context_closure_no_seed_omission(),))

    canonical_seeds: list[str] = []
    outside_omissions: dict[tuple[str, str], OmittedPath] = {}
    for raw_path in changed_paths:
        canonical = snapshot.canonical(raw_path)
        if canonical is None:
            resolved_root = snapshot_root.resolve()
            escape_path = (resolved_root / Path(raw_path)).resolve(strict=False).as_posix() if not Path(raw_path).is_absolute() else str(Path(raw_path))
            key = (escape_path, _OUTSIDE_SNAPSHOT_REASON)
            outside_omissions[key] = OmittedPath(
                path=escape_path,
                reason=_OUTSIDE_SNAPSHOT_REASON,
                detail=f"Seed {raw_path!r} resolves outside snapshot root {snapshot_root.as_posix()!s}.",
            )
            continue
        canonical_seeds.append(canonical)

    ordered_seeds = deterministic_context_closure_seed_order(tuple(canonical_seeds))

    state = _ClosureState(
        snapshot=snapshot,
        visited_sources=set(),
        visited_manifests=set(),
        context_manifests=set(),
        omissions=dict(outside_omissions),
        expansion_limit=expansion_limit,
    )

    pending: list[str] = list(ordered_seeds)
    seed_set = set(ordered_seeds)

    while pending:
        pending.sort()
        canonical_rel = pending.pop(0)

        if canonical_rel in state.visited_sources or canonical_rel in state.visited_manifests:
            continue

        if not snapshot.exists(canonical_rel):
            if canonical_rel in seed_set:
                raise ContextClosureResolutionError(
                    "missing_seed", f"Seed {canonical_rel!r} does not exist in the snapshot."
                )
            raise ContextClosureResolutionError(
                "unresolved_local_edge", f"Local edge target {canonical_rel!r} does not exist in the snapshot."
            )

        kind = _classify_seed(canonical_rel)
        if kind == "manifest":
            if canonical_rel in seed_set:
                _seed_manifest(state, canonical_rel, pending)
            else:
                _process_manifest(state, canonical_rel, pending)
        elif kind == "source":
            if state.expansion_limit is not None and state.consumed >= state.expansion_limit:
                _record_omission(
                    state,
                    canonical_rel,
                    _EXPANSION_LIMIT_REASON,
                    f"Expansion limit {state.expansion_limit} reached before processing {canonical_rel!r}.",
                )
                continue
            _process_source(state, canonical_rel, pending)
        else:
            _record_omission(
                state,
                canonical_rel,
                _UNSUPPORTED_FILE_TYPE_REASON,
                f"{canonical_rel!r} has an unsupported file type for context closure.",
            )

    included = tuple(sorted(state.visited_sources | state.context_manifests))
    omitted = tuple(sorted(state.omissions.values(), key=lambda o: (o.path, o.reason)))
    return ClosureResult(included=included, omitted=omitted)


def _seed_manifest(state: _ClosureState, canonical_rel: str, pending: list[str]) -> None:
    """A manifest given directly as a seed also follows path-dependency /
    entrypoint discovery, same as an ancestor-discovered package manifest."""
    _process_manifest(state, canonical_rel, pending)
