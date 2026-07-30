"""Raw trace hashing/writing for the versioned artifact schema (T2d).

Split out of artifact_schema.py (T2e-pre, pure refactor, zero intended
behavior change). Formalizes a boundary the original module's own docstrings
already named 3 times: computing/writing/verifying raw trace bytes is a
writer-module concern, distinct from `validate_artifact`'s in-memory-only,
no-IO shape/consistency checks.
"""

from __future__ import annotations

import hashlib
import importlib.util
import sys
from pathlib import Path


def _load_sibling_module(module_name: str, filename: str):
    """Load `filename` as `module_name`, reusing an already-loaded copy from
    `sys.modules` if one exists.

    Required once a single concern is split across sibling files that must
    share one class identity for Enum/dataclass types defined elsewhere:
    `importlib.util.module_from_spec` + `exec_module` always re-executes a
    file from scratch and does not consult `sys.modules` on its own, so two
    independent loads of the same file produce two distinct,
    non-`==`-comparable class objects. See
    docs/tasks/antares-security-specialist-advisor.md § T2e-pre EC-4.
    """
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


_ARTIFACT_SCHEMA_MOD = _load_sibling_module("antares_artifact_schema", "artifact_schema.py")
TraceRef = _ARTIFACT_SCHEMA_MOD.TraceRef
ALLOWED_TRACE_STORAGE_PREFIX = _ARTIFACT_SCHEMA_MOD.ALLOWED_TRACE_STORAGE_PREFIX
CURRENT_REDACTION_VERSION = _ARTIFACT_SCHEMA_MOD.CURRENT_REDACTION_VERSION


def compute_content_hash(raw_bytes: bytes) -> str:
    return f"sha256:{hashlib.sha256(raw_bytes).hexdigest()}"


def write_raw_trace(raw_bytes: bytes, storage_root: Path, artifact_id: str) -> TraceRef:
    """Writer: computes the hash from raw bytes *before* anything is
    written, then persists the raw bytes outside any tracked path.

    This is the property the validator cannot itself confirm (it has no
    disk access) -- the writer is what actually guarantees `content_hash`
    matches the bytes at `storage_uri`, verified by
    `verify_trace_ref_roundtrip` in a writer-module test.
    """

    content_hash = compute_content_hash(raw_bytes)
    storage_root.mkdir(parents=True, exist_ok=True)
    target = storage_root / f"{artifact_id}.trace"
    target.write_bytes(raw_bytes)
    relative = f"{ALLOWED_TRACE_STORAGE_PREFIX}{artifact_id}.trace"
    return TraceRef(
        content_hash=content_hash,
        storage_uri=f"file://{relative}",
        byte_length=len(raw_bytes),
        redaction_version=CURRENT_REDACTION_VERSION,
    )


def verify_trace_ref_roundtrip(trace_ref: TraceRef, storage_root: Path) -> bool:
    """Writer-module test helper: reads the bytes back from disk (relative
    to `storage_root`, stripping the shared `ALLOWED_TRACE_STORAGE_PREFIX`)
    and confirms they hash and size-match `trace_ref`. This is deliberately
    outside `validate_artifact`, which never touches disk."""

    relative = trace_ref.storage_uri[len("file://") :]
    filename = relative[len(ALLOWED_TRACE_STORAGE_PREFIX) :]
    data = (storage_root / filename).read_bytes()
    return compute_content_hash(data) == trace_ref.content_hash and len(data) == trace_ref.byte_length
