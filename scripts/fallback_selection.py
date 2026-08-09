#!/usr/bin/env python3
"""Fail-closed model-selection checkpoint for local-to-cloud fallbacks.

This module records authorization evidence only. It never invokes a model,
opens a network connection, or starts a subprocess.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA_VERSION = "fallback-selection-v1"
AWAITING_STATUS = "awaiting_fallback_selection"
AUTHORIZED_STATUS = "fallback_authorized"
HUMAN_SELECTION_EXIT_CODE = 3

ROLE_D14 = "d14"
ROLE_CLOUD_IMPLEMENTER = "cloud-implementer"
VALID_ROLES = (ROLE_D14, ROLE_CLOUD_IMPLEMENTER)

MODE_HUMAN_SELECT = "human-select"
MODE_PREAUTHORIZED = "preauthorized"
VALID_SELECTION_MODES = (MODE_HUMAN_SELECT, MODE_PREAUTHORIZED)

TRIGGER_REVIEWER_UNUSABLE = "reviewer-chain-unusable"
TRIGGER_OPERATIONAL = "operational-only"
TRIGGER_CAPABILITY_RISK = "capability-risk"
VALID_TRIGGER_KINDS = (
    TRIGGER_REVIEWER_UNUSABLE,
    TRIGGER_OPERATIONAL,
    TRIGGER_CAPABILITY_RISK,
)

VALID_REASONING_EFFORTS = ("none", "low", "medium", "high", "xhigh", "max")


class FallbackSelectionError(ValueError):
    """A stable fail-closed validation error for fallback selection data."""


@dataclass(frozen=True)
class Recommendation:
    model: str
    reasoning_effort: str


def _require_non_empty(field: str, value: Any) -> str:
    if not isinstance(value, str) or not value.strip():
        raise FallbackSelectionError(f"{field}: must be a non-empty string")
    return value.strip()


def _canonical_json_bytes(value: Any) -> bytes:
    try:
        return json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
    except (TypeError, ValueError, UnicodeEncodeError) as exc:
        raise FallbackSelectionError(
            f"packet: must be bytes, UTF-8 text, or a JSON-serializable object ({exc})"
        ) from exc


def canonical_packet_bytes(packet: Any) -> bytes:
    if isinstance(packet, bytes):
        return packet
    if isinstance(packet, str):
        try:
            return packet.encode("utf-8")
        except UnicodeEncodeError as exc:
            raise FallbackSelectionError(
                f"packet: text must be UTF-8 encodable ({exc})"
            ) from exc
    return _canonical_json_bytes(packet)


def packet_sha256(packet: Any) -> str:
    return hashlib.sha256(canonical_packet_bytes(packet)).hexdigest()


def _canonical_sha256(value: Any) -> str:
    return hashlib.sha256(_canonical_json_bytes(value)).hexdigest()


def recommend_fallback(rri: int, role: str, trigger_kind: str) -> Recommendation:
    if role not in VALID_ROLES:
        raise FallbackSelectionError(
            f"role: must be one of {', '.join(VALID_ROLES)}"
        )
    if trigger_kind not in VALID_TRIGGER_KINDS:
        raise FallbackSelectionError(
            f"trigger_kind: must be one of {', '.join(VALID_TRIGGER_KINDS)}"
        )
    if not isinstance(rri, int) or isinstance(rri, bool) or rri < 0:
        raise FallbackSelectionError("rri: must be a non-negative integer")

    if role == ROLE_D14:
        return Recommendation("gpt-5.6-terra", "medium")
    if rri <= 25:
        return Recommendation("gpt-5.6-luna", "low")
    if rri <= 40:
        return Recommendation("gpt-5.6-terra", "medium")
    if rri <= 55 and trigger_kind == TRIGGER_OPERATIONAL:
        return Recommendation("gpt-5.6-terra", "high")
    if rri <= 70:
        return Recommendation("gpt-5.6-sol", "high")
    if rri <= 85:
        return Recommendation("gpt-5.6-sol", "xhigh")
    return Recommendation("gpt-5.6-sol", "max")


def _selection_state(
    *,
    selection_mode: str,
    selected_model: str | None,
    selected_reasoning_effort: str | None,
    selected_by: str | None,
) -> tuple[str | None, str | None, str | None]:
    if selection_mode not in VALID_SELECTION_MODES:
        raise FallbackSelectionError(
            f"selection_mode: must be one of {', '.join(VALID_SELECTION_MODES)}"
        )

    supplied = {
        "selected_model": selected_model,
        "selected_reasoning_effort": selected_reasoning_effort,
        "selected_by": selected_by,
    }
    present = {field: value is not None and str(value).strip() for field, value in supplied.items()}

    if not any(present.values()):
        if selection_mode == MODE_PREAUTHORIZED:
            raise FallbackSelectionError(
                "selected_model: required when selection_mode is preauthorized"
            )
        return None, None, None

    missing = [field for field, is_present in present.items() if not is_present]
    if missing:
        raise FallbackSelectionError(
            f"{missing[0]}: required when any fallback selection field is supplied"
        )

    model = _require_non_empty("selected_model", selected_model)
    effort = _require_non_empty(
        "selected_reasoning_effort", selected_reasoning_effort
    )
    selector = _require_non_empty("selected_by", selected_by)
    if effort not in VALID_REASONING_EFFORTS:
        raise FallbackSelectionError(
            "selected_reasoning_effort: must be one of "
            + ", ".join(VALID_REASONING_EFFORTS)
        )
    return model, effort, selector


def _utc_now() -> str:
    return (
        datetime.datetime.now(datetime.timezone.utc)
        .isoformat()
        .replace("+00:00", "Z")
    )


def build_checkpoint(
    *,
    task_id: str,
    phase: str,
    trigger: str,
    role: str,
    rri: int,
    packet: Any,
    trigger_kind: str,
    selection_mode: str = MODE_HUMAN_SELECT,
    selected_model: str | None = None,
    selected_reasoning_effort: str | None = None,
    selected_by: str | None = None,
    now: str | None = None,
) -> dict[str, Any]:
    task_id = _require_non_empty("task_id", task_id)
    phase = _require_non_empty("phase", phase)
    trigger = _require_non_empty("trigger", trigger)
    recommendation = recommend_fallback(rri, role, trigger_kind)
    model, effort, selector = _selection_state(
        selection_mode=selection_mode,
        selected_model=selected_model,
        selected_reasoning_effort=selected_reasoning_effort,
        selected_by=selected_by,
    )
    packet_digest = packet_sha256(packet)

    checkpoint: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "task_id": task_id,
        "phase": phase,
        "status": AWAITING_STATUS if model is None else AUTHORIZED_STATUS,
        "trigger": trigger,
        "trigger_kind": trigger_kind,
        "role": role,
        "rri": rri,
        "recommended_model": recommendation.model,
        "recommended_reasoning_effort": recommendation.reasoning_effort,
        "selection_mode": selection_mode,
        "selected_model": model,
        "selected_reasoning_effort": effort,
        "selected_by": selector,
        "packet_sha256": packet_digest,
    }

    if model is None:
        return checkpoint

    receipt = {
        "schema_version": SCHEMA_VERSION,
        "task_id": task_id,
        "phase": phase,
        "role": role,
        "selection_mode": selection_mode,
        "selected_model": model,
        "selected_reasoning_effort": effort,
        "selected_by": selector,
        "packet_sha256": packet_digest,
        "authorized_at": _require_non_empty("now", now or _utc_now()),
    }
    receipt["receipt_sha256"] = _canonical_sha256(receipt)
    checkpoint["authorization_receipt"] = receipt
    return checkpoint


def validate_authorized_checkpoint(checkpoint: dict[str, Any], packet: Any) -> None:
    if not isinstance(checkpoint, dict):
        raise FallbackSelectionError("checkpoint: must be a JSON object")
    if checkpoint.get("schema_version") != SCHEMA_VERSION:
        raise FallbackSelectionError("schema_version: unsupported checkpoint schema")
    if checkpoint.get("status") != AUTHORIZED_STATUS:
        raise FallbackSelectionError("status: fallback is not authorized")

    actual_packet_sha256 = packet_sha256(packet)
    if checkpoint.get("packet_sha256") != actual_packet_sha256:
        raise FallbackSelectionError(
            "packet_sha256: checkpoint does not match the current fallback packet"
        )

    receipt = checkpoint.get("authorization_receipt")
    if not isinstance(receipt, dict):
        raise FallbackSelectionError("authorization_receipt: missing or invalid")
    if receipt.get("schema_version") != SCHEMA_VERSION:
        raise FallbackSelectionError(
            "authorization_receipt.schema_version: unsupported receipt schema"
        )
    recorded_digest = receipt.get("receipt_sha256")
    unsigned_receipt = {
        key: value for key, value in receipt.items() if key != "receipt_sha256"
    }
    if not recorded_digest or recorded_digest != _canonical_sha256(unsigned_receipt):
        raise FallbackSelectionError("receipt_sha256: authorization receipt was modified")
    if receipt.get("packet_sha256") != actual_packet_sha256:
        raise FallbackSelectionError(
            "packet_sha256: authorization receipt does not match the current packet"
        )

    for field in (
        "task_id",
        "phase",
        "role",
        "selection_mode",
        "selected_model",
        "selected_reasoning_effort",
        "selected_by",
    ):
        if checkpoint.get(field) != receipt.get(field):
            raise FallbackSelectionError(
                f"{field}: checkpoint and authorization receipt differ"
            )
    _require_non_empty("task_id", checkpoint.get("task_id"))
    _require_non_empty("phase", checkpoint.get("phase"))
    recommend_fallback(
        checkpoint.get("rri"),
        checkpoint.get("role"),
        checkpoint.get("trigger_kind"),
    )
    _selection_state(
        selection_mode=checkpoint["selection_mode"],
        selected_model=checkpoint["selected_model"],
        selected_reasoning_effort=checkpoint["selected_reasoning_effort"],
        selected_by=checkpoint["selected_by"],
    )


def write_checkpoint(checkpoint: dict[str, Any], path: str | os.PathLike[str]) -> None:
    destination = Path(path)
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary_path: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            dir=destination.parent,
            prefix=f".{destination.name}.",
            suffix=".tmp",
            delete=False,
        ) as stream:
            temporary_path = stream.name
            json.dump(checkpoint, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_path, destination)
    except Exception:
        if temporary_path:
            try:
                os.unlink(temporary_path)
            except FileNotFoundError:
                pass
        raise


def default_checkpoint_path(source_artifact: str | os.PathLike[str]) -> str:
    source = Path(source_artifact)
    if source.suffix == ".json":
        return str(source.with_name(f"{source.stem}.fallback-selection.json"))
    return f"{source}.fallback-selection.json"


def add_cli_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--fallback-mode",
        choices=VALID_SELECTION_MODES,
        default=os.environ.get("DUBBRIDGE_FALLBACK_MODE", MODE_HUMAN_SELECT),
        help="Pause for human model selection (default) or use a complete preauthorization.",
    )
    parser.add_argument(
        "--fallback-model",
        default=os.environ.get("DUBBRIDGE_FALLBACK_MODEL"),
        help="Exact human-selected fallback model.",
    )
    parser.add_argument(
        "--fallback-reasoning-effort",
        choices=VALID_REASONING_EFFORTS,
        default=os.environ.get("DUBBRIDGE_FALLBACK_REASONING_EFFORT"),
        help="Exact human-selected reasoning effort.",
    )
    parser.add_argument(
        "--fallback-selected-by",
        default=os.environ.get("DUBBRIDGE_FALLBACK_SELECTED_BY"),
        help="Human or approval identity recorded in the authorization receipt.",
    )
    parser.add_argument(
        "--fallback-selection-artifact",
        default=os.environ.get("DUBBRIDGE_FALLBACK_SELECTION_ARTIFACT"),
        help="Optional explicit path for the fallback-selection-v1 artifact.",
    )


def build_checkpoint_from_args(
    args: argparse.Namespace,
    *,
    task_id: str,
    phase: str,
    trigger: str,
    role: str,
    rri: int,
    packet: Any,
    trigger_kind: str,
) -> dict[str, Any]:
    return build_checkpoint(
        task_id=task_id,
        phase=phase,
        trigger=trigger,
        role=role,
        rri=rri,
        packet=packet,
        trigger_kind=trigger_kind,
        selection_mode=args.fallback_mode,
        selected_model=args.fallback_model,
        selected_reasoning_effort=args.fallback_reasoning_effort,
        selected_by=args.fallback_selected_by,
    )
