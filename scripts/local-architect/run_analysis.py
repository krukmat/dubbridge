#!/usr/bin/env python3
"""Run one tool-free Local Architect analysis against Ollama."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import socket
import tempfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable
from urllib import error, request


ARTIFACT_SCHEMA_VERSION = "adr037-local-architect-artifact-v1"
PROMPT_VERSION = "adr037-local-architect-prompt-v1"
EXPECTED_CLAIM_LABELS = {"SUPPORTED", "INFERRED", "UNKNOWN"}
REQUIRED_RESPONSE_FIELDS = {
    "objective": str,
    "current_state": str,
    "constraints": list,
    "risks": list,
    "recommendations": list,
    "open_questions": list,
    "evidence_gaps": list,
    "claims": list,
}


class AnalysisError(RuntimeError):
    """A fail-closed execution error with structured context."""

    def __init__(self, code: str, message: str, context: dict[str, Any] | None = None) -> None:
        super().__init__(message)
        self.code = code
        self.context = context or {}


@dataclass(frozen=True)
class Config:
    packet_path: Path
    expected_packet_sha256: str
    output_path: Path
    model_tag: str
    expected_model_digest: str
    ollama_url: str
    timeout_seconds: float
    temperature: float
    num_predict: int
    overwrite: bool


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def build_prompt(packet: dict[str, Any]) -> str:
    packet_json = json.dumps(packet, ensure_ascii=True, indent=2, sort_keys=True)
    return (
        "You are the DubBridge Local Architect / Complex Analyst. "
        "You are advisory-only, read-only, and must not claim authority.\n\n"
        "Return JSON only. Do not use Markdown fences.\n"
        "Required schema:\n"
        "{\n"
        '  "objective": string,\n'
        '  "current_state": string,\n'
        '  "constraints": [string],\n'
        '  "risks": [string],\n'
        '  "recommendations": [string],\n'
        '  "open_questions": [string],\n'
        '  "evidence_gaps": [string],\n'
        '  "claims": [{"statement": string, "label": "SUPPORTED|INFERRED|UNKNOWN"}]\n'
        "}\n\n"
        "Use only facts supported by the packet. Mark uncertainty explicitly.\n\n"
        "Project packet:\n"
        f"{packet_json}\n"
    )


def parse_packet(packet_bytes: bytes) -> dict[str, Any]:
    try:
        packet = json.loads(packet_bytes.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise AnalysisError("invalid_packet", f"Packet must be valid UTF-8 JSON: {exc}") from exc
    if not isinstance(packet, dict):
        raise AnalysisError("invalid_packet", "Packet root must be a JSON object.")
    return packet


def fetch_json(
    url: str, payload: dict[str, Any], timeout_seconds: float, method: str = "POST"
) -> dict[str, Any]:
    data = json.dumps(payload).encode("utf-8") if method != "GET" else None
    req = request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
        method=method,
    )
    try:
        with request.urlopen(req, timeout=timeout_seconds) as response:
            body = response.read()
    except error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        raise AnalysisError("http_error", f"{url} returned HTTP {exc.code}: {body}") from exc
    except (socket.timeout, TimeoutError) as exc:
        raise AnalysisError(
            "timeout", f"{url} did not respond within {timeout_seconds}s."
        ) from exc
    except error.URLError as exc:
        raise AnalysisError("connection_error", f"Failed to reach {url}: {exc.reason}") from exc
    try:
        parsed = json.loads(body.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise AnalysisError("invalid_ollama_json", f"{url} returned invalid JSON: {exc}") from exc
    if not isinstance(parsed, dict):
        raise AnalysisError("invalid_ollama_json", f"{url} returned a non-object JSON payload.")
    return parsed


def resolve_model_digest(
    ollama_url: str,
    model_tag: str,
    timeout_seconds: float,
    fetcher: Callable[..., dict[str, Any]] = fetch_json,
) -> str:
    payload = fetcher(f"{ollama_url}/api/tags", {}, timeout_seconds, method="GET")
    models = payload.get("models")
    if not isinstance(models, list):
        raise AnalysisError("invalid_model_registry", "Ollama /api/tags payload is missing the models list.")
    for model in models:
        if isinstance(model, dict) and model.get("name") == model_tag:
            digest = model.get("digest")
            if not isinstance(digest, str) or not digest:
                raise AnalysisError("invalid_model_registry", f"Model {model_tag} has no digest in /api/tags.")
            return digest
    raise AnalysisError("model_not_found", f"Model {model_tag} was not found in /api/tags.")


def strip_json_fence(response_text: str) -> str:
    trimmed = response_text.strip()
    if trimmed.startswith("```"):
        lines = trimmed.splitlines()
        if len(lines) >= 3 and lines[-1].strip() == "```":
            return "\n".join(lines[1:-1]).strip()
    return trimmed


def validate_response_text(response_text: str) -> dict[str, Any]:
    candidate = strip_json_fence(response_text)
    try:
        payload = json.loads(candidate)
    except json.JSONDecodeError as exc:
        raise AnalysisError("invalid_response", f"Model response is not valid JSON: {exc}") from exc
    if not isinstance(payload, dict):
        raise AnalysisError("invalid_response", "Model response root must be a JSON object.")
    for field, expected_type in REQUIRED_RESPONSE_FIELDS.items():
        value = payload.get(field)
        if not isinstance(value, expected_type):
            raise AnalysisError("invalid_response", f"Field {field!r} is missing or has the wrong type.")
    for field in ("constraints", "risks", "recommendations", "open_questions", "evidence_gaps"):
        if not all(isinstance(item, str) and item.strip() for item in payload[field]):
            raise AnalysisError("invalid_response", f"Field {field!r} must contain only non-empty strings.")
    claims = payload["claims"]
    if not claims:
        raise AnalysisError("invalid_response", "Field 'claims' must contain at least one claim label.")
    for claim in claims:
        if not isinstance(claim, dict):
            raise AnalysisError("invalid_response", "Each claim must be an object.")
        statement = claim.get("statement")
        label = claim.get("label")
        if not isinstance(statement, str) or not statement.strip():
            raise AnalysisError("invalid_response", "Each claim requires a non-empty statement.")
        if label not in EXPECTED_CLAIM_LABELS:
            raise AnalysisError("invalid_response", f"Claim label {label!r} is not allowed.")
    return payload


def write_json_atomic(output_path: Path, payload: dict[str, Any], overwrite: bool) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    if output_path.exists() and not overwrite:
        raise AnalysisError("output_exists", f"Output path already exists: {output_path}")
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=output_path.parent, delete=False) as tmp:
        json.dump(payload, tmp, ensure_ascii=True, indent=2, sort_keys=True)
        tmp.write("\n")
        temp_name = tmp.name
    os.replace(temp_name, output_path)


def artifact_base(config: Config, packet_sha256: str | None = None) -> dict[str, Any]:
    return {
        "schema_version": ARTIFACT_SCHEMA_VERSION,
        "success": False,
        "status": "failed",
        "started_at": utc_now(),
        "finished_at": None,
        "packet": {
            "path": str(config.packet_path),
            "sha256": packet_sha256,
            "expected_sha256": config.expected_packet_sha256,
        },
        "model": {
            "tag": config.model_tag,
            "expected_digest": config.expected_model_digest,
            "resolved_digest": None,
        },
        "prompt": {
            "version": PROMPT_VERSION,
            "sha256": None,
        },
        "runtime": {
            "ollama_url": config.ollama_url,
            "timeout_seconds": config.timeout_seconds,
            "temperature": config.temperature,
            "num_predict": config.num_predict,
        },
        "generation": None,
        "response": None,
        "error": None,
    }


def run_analysis(
    config: Config,
    fetcher: Callable[..., dict[str, Any]] = fetch_json,
) -> dict[str, Any]:
    packet_bytes = config.packet_path.read_bytes()
    packet_sha256 = sha256_bytes(packet_bytes)
    artifact = artifact_base(config, packet_sha256=packet_sha256)

    if packet_sha256 != config.expected_packet_sha256:
        raise AnalysisError(
            "packet_hash_mismatch",
            "Packet SHA-256 does not match the expected value.",
            context=artifact,
        )

    packet = parse_packet(packet_bytes)

    try:
        resolved_digest = resolve_model_digest(config.ollama_url, config.model_tag, config.timeout_seconds, fetcher)
    except AnalysisError as exc:
        raise AnalysisError(exc.code, str(exc), context=artifact) from exc
    artifact["model"]["resolved_digest"] = resolved_digest
    if resolved_digest != config.expected_model_digest:
        raise AnalysisError(
            "model_digest_mismatch",
            "Resolved model digest does not match the expected value.",
            context=artifact,
        )

    prompt = build_prompt(packet)
    artifact["prompt"]["sha256"] = sha256_bytes(prompt.encode("utf-8"))
    request_payload = {
        "model": config.model_tag,
        "prompt": prompt,
        "stream": False,
        # Disable interleaved reasoning so /api/generate returns the final
        # response without buffering an unbounded chain-of-thought first
        # (ADR-037 T4 timeout root cause). Ollama relocates any deliberation to
        # a separate "thinking" field, which we capture below for provenance.
        "think": False,
        "options": {
            "temperature": config.temperature,
            "num_predict": config.num_predict,
        },
    }
    try:
        raw_response = fetcher(f"{config.ollama_url}/api/generate", request_payload, config.timeout_seconds)
    except AnalysisError as exc:
        raise AnalysisError(exc.code, str(exc), context=artifact) from exc
    response_text = raw_response.get("response")
    if not isinstance(response_text, str) or not response_text.strip():
        raise AnalysisError("invalid_response", "Ollama generate response is missing the response text.", context=artifact)

    validated = validate_response_text(response_text)
    artifact["success"] = True
    artifact["status"] = "ok"
    artifact["finished_at"] = utc_now()
    artifact["generation"] = {
        key: raw_response.get(key)
        for key in (
            "created_at",
            "done",
            "total_duration",
            "load_duration",
            "prompt_eval_count",
            "prompt_eval_duration",
            "eval_count",
            "eval_duration",
        )
    }
    thinking = raw_response.get("thinking")
    artifact["generation"]["think_disabled"] = True
    artifact["generation"]["thinking_present"] = isinstance(thinking, str) and bool(thinking.strip())
    artifact["generation"]["thinking_sha256"] = (
        sha256_bytes(thinking.encode("utf-8")) if isinstance(thinking, str) and thinking.strip() else None
    )
    artifact["response"] = {
        "validated": validated,
        "raw_text_sha256": sha256_bytes(response_text.encode("utf-8")),
    }
    return artifact


def build_failure_artifact(config: Config, exc: AnalysisError) -> dict[str, Any]:
    context = exc.context if isinstance(exc.context, dict) else {}
    artifact = artifact_base(
        config,
        packet_sha256=context.get("packet", {}).get("sha256") if isinstance(context.get("packet"), dict) else None,
    )
    artifact["started_at"] = context.get("started_at", artifact["started_at"])
    if isinstance(context.get("packet"), dict):
        artifact["packet"].update(context["packet"])
    if isinstance(context.get("model"), dict):
        artifact["model"].update(context["model"])
    if isinstance(context.get("prompt"), dict):
        artifact["prompt"].update(context["prompt"])
    artifact["finished_at"] = utc_now()
    artifact["generation"] = context.get("generation")
    artifact["response"] = context.get("response")
    artifact["error"] = {"code": exc.code, "message": str(exc)}
    return artifact


def parse_args() -> Config:
    parser = argparse.ArgumentParser(description="Run one Local Architect analysis with a frozen packet.")
    parser.add_argument("--packet", required=True, help="Path to the immutable JSON packet.")
    parser.add_argument("--expected-packet-sha256", required=True, help="Expected SHA-256 for the packet bytes.")
    parser.add_argument("--output", required=True, help="Artifact path to write atomically.")
    parser.add_argument("--model-tag", default="qwen3.6:27b-q4_K_M", help="Exact Ollama model tag to verify and run.")
    parser.add_argument(
        "--expected-model-digest",
        default="a50eda8ed977ab48a12431878896b27ffd5cef552c17af3317d9623b939a7f1e",
        help="Expected Ollama digest for the exact model tag.",
    )
    parser.add_argument("--ollama-url", default="http://127.0.0.1:11434", help="Base URL for the local Ollama API.")
    parser.add_argument("--timeout-seconds", type=float, default=120.0, help="Per-request timeout in seconds.")
    parser.add_argument("--temperature", type=float, default=0.0, help="Generation temperature.")
    parser.add_argument("--num-predict", type=int, default=4096, help="Maximum tokens to generate.")
    parser.add_argument("--overwrite", action="store_true", help="Allow replacing an existing artifact.")
    args = parser.parse_args()
    return Config(
        packet_path=Path(args.packet),
        expected_packet_sha256=args.expected_packet_sha256,
        output_path=Path(args.output),
        model_tag=args.model_tag,
        expected_model_digest=args.expected_model_digest,
        ollama_url=args.ollama_url.rstrip("/"),
        timeout_seconds=args.timeout_seconds,
        temperature=args.temperature,
        num_predict=args.num_predict,
        overwrite=args.overwrite,
    )


def main() -> int:
    config = parse_args()
    try:
        artifact = run_analysis(config)
        write_json_atomic(config.output_path, artifact, overwrite=config.overwrite)
    except AnalysisError as exc:
        if exc.code == "output_exists":
            print(f"ERROR [{exc.code}]: {exc}")
            return 1
        artifact = build_failure_artifact(config, exc)
        write_json_atomic(config.output_path, artifact, overwrite=config.overwrite)
        print(f"ERROR [{exc.code}]: {exc}")
        return 1

    print(json.dumps({"status": artifact["status"], "output": str(config.output_path)}, ensure_ascii=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
