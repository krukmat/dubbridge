"""Shape/consistency validation for the versioned artifact schema (T2d).

Split out of artifact_schema.py (T2e-pre, pure refactor, zero intended
behavior change). `_validate_category_fields`'s original if/elif dispatch on
category string is now a Strategy dispatch table (`_CATEGORY_VALIDATORS`)
keyed by the same category strings `_category_of` already returns.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from typing import Callable


def _load_sibling_module(module_name: str, filename: str):
    """Load `filename` as `module_name`, reusing an already-loaded copy from
    `sys.modules` if one exists.

    Required once a single concern is split across sibling files that must
    share one class identity for Enum/dataclass types defined elsewhere
    (e.g. `TerminalStateKind`): `importlib.util.module_from_spec` +
    `exec_module` always re-executes a file from scratch and does not
    consult `sys.modules` on its own, so two independent loads of the same
    file produce two distinct, non-`==`-comparable class objects. See
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
Artifact = _ARTIFACT_SCHEMA_MOD.Artifact
ValidationError = _ARTIFACT_SCHEMA_MOD.ValidationError
SCHEMA_VERSION = _ARTIFACT_SCHEMA_MOD.SCHEMA_VERSION
SUCCESS_KINDS = _ARTIFACT_SCHEMA_MOD.SUCCESS_KINDS
DispositionState = _ARTIFACT_SCHEMA_MOD.DispositionState
TerminalStateKind = _ARTIFACT_SCHEMA_MOD.TerminalStateKind
ALLOWED_TRACE_STORAGE_PREFIX = _ARTIFACT_SCHEMA_MOD.ALLOWED_TRACE_STORAGE_PREFIX
_category_of = _ARTIFACT_SCHEMA_MOD._category_of


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


def _validate_t2a_t2b_fields(artifact: Artifact) -> None:
    category = _category_of(artifact.kind)
    kind = artifact.kind
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


def _validate_t2c1_fields(artifact: Artifact) -> None:
    kind = artifact.kind
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


def _validate_t2c2_fields(artifact: Artifact) -> None:
    kind = artifact.kind
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


def _validate_t2cli_fields(artifact: Artifact) -> None:
    """Element 3 Subtask B: antares-cli subprocess dispatch outcomes.

    No argv/candidates come from a parsed tool call here (this path bypasses
    the T2a parser entirely) -- candidates, when present, come from the
    CLI's own findings array. CLI_EXECUTION_COMPLETE and CLI_EXECUTION_FAILED
    ran a real subprocess and so require a trace_ref (stdout/stderr
    captured, same redaction discipline as t2c1_execution); CLI_BINARY_UNAVAILABLE
    fails closed before any subprocess spawns, so -- like SANDBOX_RUNTIME_UNAVAILABLE
    -- it must not carry one. CLI_OUTPUT_MALFORMED did run the CLI (exit 0,
    unparseable stdout), so it requires a trace_ref like the other two
    post-execution kinds.
    """
    kind = artifact.kind
    if kind is TerminalStateKind.CLI_BINARY_UNAVAILABLE:
        if artifact.trace_ref is not None:
            raise ValidationError(
                "unexpected_trace_ref",
                "CLI_BINARY_UNAVAILABLE must not carry a trace_ref (nothing ran).",
            )
        if not artifact.detail:
            raise ValidationError("missing_detail", "CLI_BINARY_UNAVAILABLE requires non-empty detail.")
        return
    if artifact.trace_ref is None:
        raise ValidationError("missing_trace_ref", f"{kind} requires a trace_ref.")
    if kind is not TerminalStateKind.CLI_EXECUTION_COMPLETE and not artifact.detail:
        raise ValidationError("missing_detail", f"{kind} is a rejection kind and requires non-empty detail.")


# Strategy: dispatch table keyed by the same category strings `_category_of`
# returns, replacing the original if/elif chain. t2a_parser and t2b_policy
# share one strategy, exactly as the original if/elif's combined branch did.
_CATEGORY_VALIDATORS: dict[str, Callable[[Artifact], None]] = {
    "t2a_parser": _validate_t2a_t2b_fields,
    "t2b_policy": _validate_t2a_t2b_fields,
    "t2c1_execution": _validate_t2c1_fields,
    "t2c2_budget": _validate_t2c2_fields,
    "t2cli_execution": _validate_t2cli_fields,
}


def _validate_category_fields(artifact: Artifact) -> None:
    category = _category_of(artifact.kind)
    _CATEGORY_VALIDATORS[category](artifact)


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
