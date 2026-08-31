#!/usr/bin/env python3
"""Hash-bound primary route receipt validator for Med-high tasks (ADR-038 T2).

Pure, offline validation. Consumes the med-high-refinement-v1 artifact
produced by scripts/local-architect/run_analysis.py (ADR-038 section 2) and
a primary route receipt, and decides whether GO_LOCAL implementation may
start. No network, no filesystem access beyond the dicts passed in.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from typing import Any

MED_HIGH_PROFILE = "med-high-refinement-v1"
# ADR-037 Amendment 1 (T4b): the Local Architect / Complex Analyst binding
# moved from qwen3.6:27b-q4_K_M to GPT-OSS 20B.
REQUIRED_MODEL_TAG = "gpt-oss:20b"
MED_HIGH_RRI_MIN = 41
MED_HIGH_RRI_MAX = 55
MED_HIGH_BAND_LABEL = "Med-high"

ROUTE_GO_LOCAL = "GO_LOCAL"
ROUTE_CLOUD_REQUIRED = "CLOUD_REQUIRED"
VALID_ROUTES = (ROUTE_GO_LOCAL, ROUTE_CLOUD_REQUIRED)

REFINEMENT_REQUIRED_FIELDS = (
    "success",
    "profile",
    "packet",
    "model",
    "response",
)
RECEIPT_REQUIRED_FIELDS = (
    "primary_id",
    "decision",
    "rationale",
    "timestamp",
    "card_hash",
    "refinement_artifact_sha256",
)


class GateError(Exception):
    """A fail-closed gate rejection with a stable machine-readable code."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


@dataclass(frozen=True)
class GateDecision:
    route: str
    reason: str


def _canonical_json(data: Any) -> str:
    return json.dumps(data, sort_keys=True, separators=(",", ":"))


def sha256_of(data: Any) -> str:
    return hashlib.sha256(_canonical_json(data).encode("utf-8")).hexdigest()


def _require_fields(payload: dict[str, Any], fields: tuple[str, ...], label: str) -> None:
    for field in fields:
        if field not in payload:
            raise GateError("missing_field", f"{label} is missing required field {field!r}.")


def validate_refinement_artifact(
    artifact: dict[str, Any],
    *,
    expected_card_hash: str,
    expected_model_tag: str = REQUIRED_MODEL_TAG,
) -> dict[str, Any]:
    """Validate the GPT-OSS 20B med-high-refinement-v1 artifact in isolation.

    Returns the validated `response.validated` payload on success. Raises
    GateError on any mismatch -- an unavailable, malformed, stale, or
    hash-mismatched artifact is equivalent to CLOUD_REQUIRED (ADR-038 s.2).
    """
    if not isinstance(artifact, dict):
        raise GateError("invalid_artifact", "Refinement artifact must be a JSON object.")

    _require_fields(artifact, REFINEMENT_REQUIRED_FIELDS, "Refinement artifact")

    if artifact.get("profile") != MED_HIGH_PROFILE:
        raise GateError(
            "wrong_profile",
            f"Refinement artifact profile must be {MED_HIGH_PROFILE!r}, got {artifact.get('profile')!r}.",
        )
    if artifact.get("success") is not True:
        raise GateError("refinement_failed", "Refinement artifact does not report success.")

    packet = artifact.get("packet")
    if not isinstance(packet, dict) or not packet.get("sha256"):
        raise GateError("missing_field", "Refinement artifact packet is missing sha256.")
    if packet["sha256"] != expected_card_hash:
        raise GateError(
            "card_hash_mismatch",
            "Refinement artifact packet hash does not match the approved card hash.",
        )

    model = artifact.get("model")
    if not isinstance(model, dict):
        raise GateError("missing_field", "Refinement artifact is missing model provenance.")
    if model.get("tag") != expected_model_tag:
        raise GateError(
            "model_tag_mismatch",
            f"Refinement artifact model tag must be {expected_model_tag!r}, got {model.get('tag')!r}.",
        )
    resolved_digest = model.get("resolved_digest")
    expected_digest = model.get("expected_digest")
    if not resolved_digest or not expected_digest or resolved_digest != expected_digest:
        raise GateError(
            "model_digest_mismatch",
            "Refinement artifact model digest is missing or does not match the expected digest.",
        )

    response = artifact.get("response")
    if not isinstance(response, dict) or not isinstance(response.get("validated"), dict):
        raise GateError("missing_field", "Refinement artifact is missing a validated response payload.")
    validated = response["validated"]

    route_recommendation = validated.get("route_recommendation")
    if route_recommendation not in VALID_ROUTES:
        raise GateError(
            "invalid_route",
            f"Refinement route_recommendation {route_recommendation!r} is not one of {VALID_ROUTES}.",
        )

    return validated


def validate_primary_receipt(
    receipt: dict[str, Any],
    *,
    expected_card_hash: str,
    expected_refinement_sha256: str,
) -> None:
    """Validate the primary agent's hash-bound route receipt in isolation."""
    if not isinstance(receipt, dict):
        raise GateError("invalid_receipt", "Primary receipt must be a JSON object.")

    _require_fields(receipt, RECEIPT_REQUIRED_FIELDS, "Primary receipt")

    if receipt.get("card_hash") != expected_card_hash:
        raise GateError(
            "card_hash_mismatch",
            "Primary receipt card hash does not match the approved card hash.",
        )
    if receipt.get("refinement_artifact_sha256") != expected_refinement_sha256:
        raise GateError(
            "refinement_hash_mismatch",
            "Primary receipt does not bind to the exact refinement artifact evaluated by this gate.",
        )
    if receipt.get("decision") not in VALID_ROUTES:
        raise GateError(
            "invalid_receipt_decision",
            f"Primary receipt decision {receipt.get('decision')!r} is not one of {VALID_ROUTES}.",
        )
    if not receipt.get("primary_id"):
        raise GateError("missing_field", "Primary receipt is missing primary_id.")
    if not str(receipt.get("rationale", "")).strip():
        raise GateError("missing_field", "Primary receipt rationale must be a non-empty string.")


def evaluate_route(
    *,
    refinement_artifact: dict[str, Any],
    primary_receipt: dict[str, Any],
    card_hash: str,
    rri: int,
) -> GateDecision:
    """The single fail-closed entry point: validates both inputs and the RRI
    band, then applies the ADR-038 route rules.

    - GATE-1: GPT-OSS 20B GO_LOCAL plus a matching primary GO_LOCAL receipt
      is eligible for local implementation.
    - GATE-2: the primary may downgrade GO_LOCAL to cloud.
    - GATE-3: CLOUD_REQUIRED, any mismatch, or missing evidence never starts
      local implementation -- the primary may never upgrade CLOUD_REQUIRED to
      local, even if it attempts to.
    """
    if not (MED_HIGH_RRI_MIN <= rri <= MED_HIGH_RRI_MAX):
        raise GateError(
            "rri_out_of_band",
            f"RRI {rri} is outside the Med-high band [{MED_HIGH_RRI_MIN}, {MED_HIGH_RRI_MAX}]; "
            "this gate only routes Med-high tasks.",
        )

    validated_refinement = validate_refinement_artifact(
        refinement_artifact, expected_card_hash=card_hash
    )
    refinement_sha256 = sha256_of(refinement_artifact)
    validate_primary_receipt(
        primary_receipt,
        expected_card_hash=card_hash,
        expected_refinement_sha256=refinement_sha256,
    )

    architect_route = validated_refinement["route_recommendation"]
    primary_decision = primary_receipt["decision"]

    # The primary may never upgrade CLOUD_REQUIRED to local (ADR-038 s.3):
    # enforced structurally by requiring both sides to independently say
    # GO_LOCAL, never by trusting the primary's decision alone.
    if architect_route == ROUTE_CLOUD_REQUIRED:
        return GateDecision(
            route=ROUTE_CLOUD_REQUIRED,
            reason="GPT-OSS 20B recommended CLOUD_REQUIRED; the primary cannot upgrade this to local.",
        )
    if primary_decision == ROUTE_CLOUD_REQUIRED:
        return GateDecision(
            route=ROUTE_CLOUD_REQUIRED,
            reason="Primary receipt downgraded GO_LOCAL to cloud.",
        )
    # Both sides independently GO_LOCAL.
    return GateDecision(route=ROUTE_GO_LOCAL, reason="GPT-OSS 20B and primary both recommend GO_LOCAL.")


def main(argv: list[str] | None = None) -> int:
    import argparse
    from pathlib import Path

    parser = argparse.ArgumentParser(description="Evaluate the Med-high primary route gate.")
    parser.add_argument("--refinement-artifact", required=True, help="Path to the GPT-OSS 20B refinement artifact JSON.")
    parser.add_argument("--primary-receipt", required=True, help="Path to the primary route receipt JSON.")
    parser.add_argument("--card-hash", required=True, help="Expected approved-card SHA-256.")
    parser.add_argument("--rri", type=int, required=True, help="Final task RRI.")
    args = parser.parse_args(argv)

    try:
        refinement_artifact = json.loads(Path(args.refinement_artifact).read_text(encoding="utf-8"))
        primary_receipt = json.loads(Path(args.primary_receipt).read_text(encoding="utf-8"))
        decision = evaluate_route(
            refinement_artifact=refinement_artifact,
            primary_receipt=primary_receipt,
            card_hash=args.card_hash,
            rri=args.rri,
        )
    except GateError as exc:
        print(json.dumps({"route": ROUTE_CLOUD_REQUIRED, "error": {"code": exc.code, "message": str(exc)}}))
        return 1
    except (OSError, json.JSONDecodeError) as exc:
        print(json.dumps({"route": ROUTE_CLOUD_REQUIRED, "error": {"code": "io_error", "message": str(exc)}}))
        return 1

    print(json.dumps({"route": decision.route, "reason": decision.reason}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
