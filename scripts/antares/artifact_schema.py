"""Versioned artifact schema and redacted trace-reference contract (T2d).

Normalizes every `TerminalState` produced by T2a/T2b/T2c-1/T2c-2 into a
durable, versioned `Artifact`. Two properties are structural, not just
validated: raw trace content (stdout/stderr) is never part of the
serialized/committed form -- only a `TraceRef` (hash + external URI) is --
and every artifact carries a mandatory `Disposition` so nothing produced by
this advisory-only tool can appear closed without a durable human decision
(see docs/policies/HITL_AUTONOMY_POLICY.md).
"""

from __future__ import annotations

import hashlib
import importlib.util
import sys
from dataclasses import asdict, dataclass, field
from enum import Enum
from pathlib import Path
from typing import Any

_TERMINAL_STATE_SCRIPT = Path(__file__).with_name("terminal_state.py")
_TERMINAL_STATE_SPEC = importlib.util.spec_from_file_location(
    "antares_terminal_state", _TERMINAL_STATE_SCRIPT
)
if _TERMINAL_STATE_SPEC is None or _TERMINAL_STATE_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_TERMINAL_STATE_SCRIPT}")
_TERMINAL_STATE_MOD = importlib.util.module_from_spec(_TERMINAL_STATE_SPEC)
sys.modules[_TERMINAL_STATE_SPEC.name] = _TERMINAL_STATE_MOD
_TERMINAL_STATE_SPEC.loader.exec_module(_TERMINAL_STATE_MOD)

TerminalStateKind = _TERMINAL_STATE_MOD.TerminalStateKind
SUCCESS_KINDS = _TERMINAL_STATE_MOD.SUCCESS_KINDS

SCHEMA_VERSION = 1
CURRENT_REDACTION_VERSION = 1
ALLOWED_TRACE_STORAGE_PREFIX = "var/antares-traces/"


class ValidationError(ValueError):
    """A fail-closed schema/consistency rejection with a stable code.

    Raised only for shape and consistency problems the validator can decide
    from the in-memory artifact alone. It never performs I/O -- see the
    module docstring and `verify_trace_ref_roundtrip` for the one check
    that does need disk access, which is a writer-module concern instead.
    """

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


class DispositionState(Enum):
    """The four durable human-disposition states named in
    AGENT_WORKFLOW_GUIDE.md's Antares authority-boundary section."""

    NEEDS_HUMAN_REVIEW = "needs-human-review"
    ACCEPTED_NOW = "accepted-now"
    ACCEPTED_FOLLOW_UP = "accepted-follow-up"
    REJECTED = "rejected"


# Category partition of all 20 TerminalStateKind values. Each artifact's
# required fields depend on which category its `kind` falls into.
T2A_KINDS = frozenset(
    {
        TerminalStateKind.PARSED_TERMINAL_CALL,
        TerminalStateKind.SUBMITTED_VULNERABLE_FILES,
        TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
        TerminalStateKind.MALFORMED_TOOL_CALL,
        TerminalStateKind.UNSUPPORTED_TOOL_NAME,
        TerminalStateKind.MALFORMED_SUBMIT_PAYLOAD,
        TerminalStateKind.DUPLICATE_TERMINAL_SUBMISSION,
    }
)
T2B_KINDS = frozenset(
    {
        TerminalStateKind.COMMAND_PLAN_VALID,
        TerminalStateKind.PATH_CONTAINMENT_VALID,
        TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX,
        TerminalStateKind.COMMAND_REJECTED_EXECUTABLE_NOT_ALLOWED,
        TerminalStateKind.COMMAND_REJECTED_OPTION_NOT_ALLOWED,
        TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE,
    }
)
T2C1_KINDS = frozenset(
    {
        TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
        TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE,
        TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT,
    }
)
T2C2_KINDS = frozenset(
    {
        TerminalStateKind.SANDBOX_OUTPUT_CAP_EXCEEDED,
        TerminalStateKind.SANDBOX_WALL_BUDGET_EXCEEDED,
        TerminalStateKind.SANDBOX_BUDGET_EXHAUSTED,
        TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED,
    }
)

# Self-check, not just documentation: the four category sets must exactly
# partition all 20 TerminalStateKind values with no gap and no overlap.
assert len(T2A_KINDS) == 7
assert len(T2B_KINDS) == 6
assert len(T2C1_KINDS) == 3
assert len(T2C2_KINDS) == 4
assert T2A_KINDS | T2B_KINDS | T2C1_KINDS | T2C2_KINDS == frozenset(TerminalStateKind)
assert not (T2A_KINDS & T2B_KINDS & T2C1_KINDS & T2C2_KINDS)


@dataclass(frozen=True)
class Provenance:
    """Model/runtime/harness identity plus packet/snapshot hashes.

    All five fields are mandatory and unchanged from the pre-existing
    acceptance criterion; this dataclass only gives that requirement a
    concrete shape.
    """

    model_version: str
    runtime_version: str
    harness_version: str
    packet_hash: str
    snapshot_hash: str


@dataclass(frozen=True)
class TraceRef:
    """A redacted reference to raw trace content stored outside git.

    `content_hash` is SHA-256 (prefixed `sha256:`) over the raw,
    pre-redaction bytes -- computed by the writer, never by the validator.
    `storage_uri` must resolve under `ALLOWED_TRACE_STORAGE_PREFIX`, a
    gitignored root outside anything `make qa-docs` or git tracks.
    """

    content_hash: str
    storage_uri: str
    byte_length: int
    redaction_version: int = CURRENT_REDACTION_VERSION


@dataclass(frozen=True)
class Disposition:
    """The mandatory human-disposition contract. `state` defaults to
    `needs-human-review` at creation (EC-3): nothing is ever pre-closed."""

    state: DispositionState = DispositionState.NEEDS_HUMAN_REVIEW
    reviewer: str | None = None
    reviewed_at: str | None = None
    note: str | None = None


@dataclass(frozen=True)
class Budget:
    """A T2c-2 resource-accounting snapshot at the moment a budget kind fired."""

    limit: float
    consumed: float
    unit: str


@dataclass(frozen=True)
class Artifact:
    """One versioned, redacted, human-disposition-gated terminal-state record.

    `raw_stdout`/`raw_stderr` exist only as an in-memory pre-redaction
    staging area (e.g. immediately after a sandboxed run, before the writer
    redacts them into a `trace_ref`); `to_dict` never serializes them, and
    `validate_artifact` rejects any artifact where they are non-empty
    alongside a populated `trace_ref` (EC-2) -- the committed/validated form
    can never carry both raw content and its own redacted reference.
    """

    schema_version: int
    kind: Any  # TerminalStateKind
    finding_id: str
    artifact_id: str
    provenance: Provenance
    disposition: Disposition = field(default_factory=Disposition)
    supersedes: str | None = None
    trace_ref: TraceRef | None = None
    argv: tuple[str, ...] = field(default_factory=tuple)
    candidates: tuple[str, ...] = field(default_factory=tuple)
    detail: str = ""
    exit_code: int | None = None
    elapsed_seconds: float | None = None
    budget: Budget | None = None
    teardown_grace_seconds: float | None = None
    raw_stdout: str = ""
    raw_stderr: str = ""


def _category_of(kind: Any) -> str:
    if kind in T2A_KINDS:
        return "t2a_parser"
    if kind in T2B_KINDS:
        return "t2b_policy"
    if kind in T2C1_KINDS:
        return "t2c1_execution"
    if kind in T2C2_KINDS:
        return "t2c2_budget"
    raise ValidationError("unknown_kind_category", f"{kind!r} is not mapped to any category.")


def _validate_storage_uri(uri: str) -> None:
    if not uri.startswith("file://"):
        raise ValidationError("invalid_storage_uri", "trace_ref.storage_uri must start with 'file://'.")
    relative = uri[len("file://") :]
    if relative.startswith("/") or ".." in relative.split("/"):
        raise ValidationError(
            "storage_uri_escapes_root",
            "trace_ref.storage_uri must be a repo-relative path with no absolute or '..' segments.",
        )
    if not relative.startswith(ALLOWED_TRACE_STORAGE_PREFIX):
        raise ValidationError(
            "storage_uri_outside_allowed_root",
            f"trace_ref.storage_uri must resolve under {ALLOWED_TRACE_STORAGE_PREFIX!r}.",
        )


def _is_valid_sha256_hex(value: str) -> bool:
    if not value.startswith("sha256:"):
        return False
    digest = value[len("sha256:") :]
    return len(digest) == 64 and all(c in "0123456789abcdef" for c in digest)


def _validate_trace_ref_field(artifact: Artifact) -> None:
    if artifact.trace_ref is not None:
        _validate_storage_uri(artifact.trace_ref.storage_uri)
        if not _is_valid_sha256_hex(artifact.trace_ref.content_hash):
            raise ValidationError(
                "invalid_content_hash", "trace_ref.content_hash must be 'sha256:' followed by 64 lowercase hex chars."
            )
    # EC-2: a populated trace_ref and non-empty raw trace content may never
    # coexist -- that would mean redaction happened but the raw bytes were
    # never cleared from the record about to be committed.
    if artifact.trace_ref is not None and (artifact.raw_stdout or artifact.raw_stderr):
        raise ValidationError(
            "unredacted_trace_leak",
            "raw_stdout/raw_stderr must be empty once trace_ref is populated.",
        )


def _validate_disposition(artifact: Artifact) -> None:
    disposition = artifact.disposition
    if not isinstance(disposition.state, DispositionState):
        raise ValidationError("invalid_disposition_state", "disposition.state must be a DispositionState.")
    if disposition.state is DispositionState.NEEDS_HUMAN_REVIEW:
        if disposition.reviewer is not None or disposition.reviewed_at is not None:
            raise ValidationError(
                "premature_disposition_fields",
                "needs-human-review must not carry a reviewer/reviewed_at yet.",
            )
    else:
        if not disposition.reviewer or not disposition.reviewed_at:
            raise ValidationError(
                "incomplete_disposition",
                "Moving off needs-human-review requires reviewer and reviewed_at in the same write.",
            )


def _validate_category_fields(artifact: Artifact) -> None:
    kind = artifact.kind
    category = _category_of(kind)

    if category in ("t2a_parser", "t2b_policy"):
        if artifact.trace_ref is not None:
            raise ValidationError(
                "unexpected_trace_ref",
                f"{category} kinds must not carry a trace_ref (no subprocess trace exists yet).",
            )
        if kind not in SUCCESS_KINDS and not artifact.detail:
            raise ValidationError("missing_detail", f"{kind} is a rejection kind and requires non-empty detail.")
        if kind is TerminalStateKind.PARSED_TERMINAL_CALL and not artifact.argv:
            raise ValidationError("missing_argv", "PARSED_TERMINAL_CALL requires non-empty argv.")
        if kind is TerminalStateKind.COMMAND_PLAN_VALID and not artifact.argv:
            raise ValidationError("missing_argv", "COMMAND_PLAN_VALID requires non-empty argv.")
        if kind is TerminalStateKind.SUBMITTED_VULNERABLE_FILES and not artifact.candidates:
            # EC-4
            raise ValidationError(
                "empty_candidates",
                "SUBMITTED_VULNERABLE_FILES requires at least one candidate path.",
            )
        if kind is TerminalStateKind.PATH_CONTAINMENT_VALID and not artifact.candidates:
            raise ValidationError("missing_candidates", "PATH_CONTAINMENT_VALID requires non-empty candidates.")
        return

    if category == "t2c1_execution":
        if not artifact.argv:
            raise ValidationError("missing_argv", f"{kind} requires non-empty argv.")
        if artifact.elapsed_seconds is None:
            raise ValidationError("missing_elapsed_seconds", f"{kind} requires elapsed_seconds.")
        if kind is TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE:
            if artifact.trace_ref is not None:
                raise ValidationError(
                    "unexpected_trace_ref",
                    "SANDBOX_RUNTIME_UNAVAILABLE must not carry a trace_ref (nothing ran).",
                )
        else:
            if artifact.exit_code is None:
                raise ValidationError("missing_exit_code", f"{kind} requires exit_code.")
            if artifact.trace_ref is None:
                raise ValidationError("missing_trace_ref", f"{kind} requires a trace_ref.")
        return

    if category == "t2c2_budget":
        if artifact.elapsed_seconds is None:
            raise ValidationError("missing_elapsed_seconds", f"{kind} requires elapsed_seconds.")
        if artifact.budget is None:
            raise ValidationError("missing_budget", f"{kind} requires a budget snapshot.")
        if artifact.trace_ref is None:
            raise ValidationError("missing_trace_ref", f"{kind} requires a trace_ref.")
        if kind is TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED and artifact.teardown_grace_seconds is None:
            raise ValidationError(
                "missing_teardown_grace_seconds",
                "SANDBOX_TEARDOWN_UNCONFIRMED requires teardown_grace_seconds.",
            )
        return


def validate_artifact(artifact: Artifact) -> None:
    """Shape/consistency-only validation. Raises `ValidationError` on the
    first violation found; returns `None` on success. Never performs I/O --
    see `verify_trace_ref_roundtrip` for the one check that needs disk
    access, which is a writer-module concern, not a validator concern."""

    if artifact.schema_version != SCHEMA_VERSION:
        # EC-5: no best-effort parsing of an unrecognized version.
        raise ValidationError(
            "unrecognized_schema_version",
            f"schema_version {artifact.schema_version} is not the supported version {SCHEMA_VERSION}.",
        )
    if not artifact.finding_id:
        raise ValidationError("missing_finding_id", "finding_id must be a non-empty string.")
    if not artifact.artifact_id:
        raise ValidationError("missing_artifact_id", "artifact_id must be a non-empty string.")
    _validate_category_fields(artifact)
    _validate_trace_ref_field(artifact)
    _validate_disposition(artifact)


def validate_supersede_chain(chain: list[Artifact]) -> Artifact:
    """Validate a full revision chain for one finding and return its head.

    EC-6: every revision must share the same `finding_id`; `artifact_id`
    values must be unique within the chain; a non-null `supersedes` must
    name another artifact_id present in the same chain (no dangling
    reference); and exactly one revision must be the chain head (the one no
    other revision's `supersedes` names) -- zero heads means a cycle, more
    than one means a fork, and both are rejected.

    Each element is also individually validated with `validate_artifact`
    first (propagating its original error code unchanged): a chain built
    from individually-malformed artifacts must never report a clean head,
    consistent with this module's fail-closed intent.
    """

    if not chain:
        raise ValidationError("empty_chain", "A supersede chain must contain at least one artifact.")

    for artifact in chain:
        validate_artifact(artifact)

    finding_ids = {a.finding_id for a in chain}
    if len(finding_ids) != 1:
        raise ValidationError(
            "chain_finding_id_mismatch",
            "All artifacts in a supersede chain must share the same finding_id.",
        )

    artifact_ids = [a.artifact_id for a in chain]
    if len(artifact_ids) != len(set(artifact_ids)):
        raise ValidationError("duplicate_artifact_id", "artifact_id must be unique within a supersede chain.")

    id_set = set(artifact_ids)
    superseded_ids: set[str] = set()
    for artifact in chain:
        if artifact.supersedes is not None:
            if artifact.supersedes not in id_set:
                raise ValidationError(
                    "dangling_supersedes",
                    f"{artifact.artifact_id} supersedes {artifact.supersedes!r}, which is not in this chain.",
                )
            superseded_ids.add(artifact.supersedes)

    heads = [a for a in chain if a.artifact_id not in superseded_ids]
    if len(heads) != 1:
        raise ValidationError(
            "ambiguous_chain_head",
            f"Expected exactly one chain head, found {len(heads)} (cycle or fork).",
        )
    return heads[0]


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


def artifact_to_dict(artifact: Artifact) -> dict[str, Any]:
    """The committed/serialized form. Deliberately has no key for
    raw_stdout/raw_stderr -- those never leave the in-memory pre-redaction
    staging object, regardless of what `validate_artifact` catches."""

    return {
        "schema_version": artifact.schema_version,
        "kind": artifact.kind.value,
        "finding_id": artifact.finding_id,
        "artifact_id": artifact.artifact_id,
        "supersedes": artifact.supersedes,
        "provenance": asdict(artifact.provenance),
        "disposition": {
            "state": artifact.disposition.state.value,
            "reviewer": artifact.disposition.reviewer,
            "reviewed_at": artifact.disposition.reviewed_at,
            "note": artifact.disposition.note,
        },
        "trace_ref": None if artifact.trace_ref is None else asdict(artifact.trace_ref),
        "argv": list(artifact.argv),
        "candidates": list(artifact.candidates),
        "detail": artifact.detail,
        "exit_code": artifact.exit_code,
        "elapsed_seconds": artifact.elapsed_seconds,
        "budget": None if artifact.budget is None else asdict(artifact.budget),
        "teardown_grace_seconds": artifact.teardown_grace_seconds,
    }


def artifact_from_dict(data: dict[str, Any]) -> Artifact:
    disposition_data = data["disposition"]
    trace_ref_data = data.get("trace_ref")
    budget_data = data.get("budget")
    return Artifact(
        schema_version=data["schema_version"],
        kind=TerminalStateKind(data["kind"]),
        finding_id=data["finding_id"],
        artifact_id=data["artifact_id"],
        supersedes=data.get("supersedes"),
        provenance=Provenance(**data["provenance"]),
        disposition=Disposition(
            state=DispositionState(disposition_data["state"]),
            reviewer=disposition_data.get("reviewer"),
            reviewed_at=disposition_data.get("reviewed_at"),
            note=disposition_data.get("note"),
        ),
        trace_ref=None if trace_ref_data is None else TraceRef(**trace_ref_data),
        argv=tuple(data.get("argv", ())),
        candidates=tuple(data.get("candidates", ())),
        detail=data.get("detail", ""),
        exit_code=data.get("exit_code"),
        elapsed_seconds=data.get("elapsed_seconds"),
        budget=None if budget_data is None else Budget(**budget_data),
        teardown_grace_seconds=data.get("teardown_grace_seconds"),
    )


def _example_provenance() -> Provenance:
    return Provenance(
        model_version="fdtn-ai/antares-1b@example",
        runtime_version="scripts/antares@example",
        harness_version="antares-harness@example",
        packet_hash="sha256:" + "0" * 64,
        snapshot_hash="sha256:" + "1" * 64,
    )


def _example_trace_ref(kind: Any) -> TraceRef:
    return TraceRef(
        content_hash="sha256:" + "2" * 64,
        storage_uri=f"file://{ALLOWED_TRACE_STORAGE_PREFIX}{kind.value}.trace",
        byte_length=128,
        redaction_version=CURRENT_REDACTION_VERSION,
    )


def generate_example_artifacts() -> dict[Any, Artifact]:
    """One schema-valid, redacted example `Artifact` per TerminalStateKind.

    Every example carries `disposition.state = needs-human-review` (EC-3:
    nothing is created pre-closed, including negative/degraded/rejected
    results) and no raw trace content anywhere in the object graph.
    """

    examples: dict[Any, Artifact] = {}
    provenance = _example_provenance()

    for kind in TerminalStateKind:
        finding_id = f"example-{kind.value}"
        artifact_id = f"{finding_id}-r1"
        category = _category_of(kind)

        kwargs: dict[str, Any] = {
            "schema_version": SCHEMA_VERSION,
            "kind": kind,
            "finding_id": finding_id,
            "artifact_id": artifact_id,
            "provenance": provenance,
        }

        if category in ("t2a_parser", "t2b_policy"):
            if kind is TerminalStateKind.PARSED_TERMINAL_CALL:
                kwargs["argv"] = ("ls", "-la")
            elif kind is TerminalStateKind.COMMAND_PLAN_VALID:
                kwargs["argv"] = ("grep", "-n", "pattern", "file.py")
            elif kind is TerminalStateKind.SUBMITTED_VULNERABLE_FILES:
                kwargs["candidates"] = ("src/example.py",)
            elif kind is TerminalStateKind.PATH_CONTAINMENT_VALID:
                kwargs["candidates"] = ("src/example.py",)
            if kind not in SUCCESS_KINDS:
                kwargs["detail"] = f"example rejection detail for {kind.value}"
        elif category == "t2c1_execution":
            kwargs["argv"] = ("run-candidate-check",)
            kwargs["elapsed_seconds"] = 1.5
            if kind is not TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE:
                kwargs["exit_code"] = 0 if kind is TerminalStateKind.SANDBOX_EXECUTION_COMPLETE else 124
                kwargs["trace_ref"] = _example_trace_ref(kind)
            else:
                kwargs["detail"] = "example: sandbox runtime unavailable"
        elif category == "t2c2_budget":
            kwargs["elapsed_seconds"] = 42.0
            unit = {
                TerminalStateKind.SANDBOX_OUTPUT_CAP_EXCEEDED: "bytes",
                TerminalStateKind.SANDBOX_WALL_BUDGET_EXCEEDED: "seconds",
                TerminalStateKind.SANDBOX_BUDGET_EXHAUSTED: "commands",
                TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED: "commands",
            }[kind]
            kwargs["budget"] = Budget(limit=100.0, consumed=100.0, unit=unit)
            kwargs["trace_ref"] = _example_trace_ref(kind)
            if kind is TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED:
                kwargs["teardown_grace_seconds"] = 5.0

        artifact = Artifact(**kwargs)
        validate_artifact(artifact)
        examples[kind] = artifact

    return examples
