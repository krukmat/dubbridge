#!/usr/bin/env python3
"""Session preflight for DubBridge coding agents."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import threading
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


SCRIPT_VERSION = 1
SENTINEL_RELATIVE = Path(".agent") / "session-preflight.json"

RECEIPT_SCHEMA_VERSION = 2
RECEIPTS_DIR_RELATIVE = Path(".agent") / "receipts" / "v2"
V2_VALID_PROVIDERS = frozenset({"codex", "claude"})
V2_VALID_LIFECYCLE_EVENTS = frozenset(
    {
        "startup",
        "resume",
        "clear",
        "compact",
        "fork",
        "subagent",
    }
)

SUMMARY_LINES = [
    "DubBridge agent preflight",
    "",
    "Authority:",
    "- docs/playbooks/AGENT_WORKFLOW_GUIDE.md is the highest-authority workflow source.",
    "- CLAUDE.md and AGENTS.md are summaries for topics not overridden by the workflow guide.",
    "",
    "Before implementation:",
    "- Identify and read affected files plus material governing docs.",
    "- Include docs/architecture.md, applicable ADRs, docs/plan/roadmap.md, slice plan/task ledger, BDD/product docs, and relevant policies/configs when they constrain the task.",
    "- Ensure plan/task ledger exists for staged work.",
    "- Run scripts/rri.py before presenting or delegating a task.",
    "- RRI 0-25: no full approval packet; use Gemma only for eligible simple code patches.",
    "- RRI 26+: present the task and wait for explicit approval before editing.",
    "- Mobile UI/presentation work under mobile/ must read root DESIGN.md first.",
    "",
    "Before closure:",
    "- For development tasks, evaluate Gemma Reviewer / D14 review before coverage or Done status.",
    "- Sync materially affected status docs before reporting completion.",
]


class PreflightError(Exception):
    """Raised when the session preflight has not been satisfied."""


def find_repo_root(start: Path | None = None) -> Path:
    """Return the git repository root, falling back to the script parent."""
    if start is None:
        start = Path.cwd()
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=start,
            capture_output=True,
            check=True,
            text=True,
        )
        return Path(result.stdout.strip()).resolve()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return Path(__file__).resolve().parents[1]


def sentinel_path(repo_root: Path) -> Path:
    return repo_root / SENTINEL_RELATIVE


def preflight_summary() -> str:
    return "\n".join(SUMMARY_LINES) + "\n"


def sentinel_payload(repo_root: Path) -> Dict[str, Any]:
    return {
        "version": SCRIPT_VERSION,
        "repo_root": str(repo_root.resolve()),
        "marked_at": datetime.now(timezone.utc).isoformat(),
        "requirements": [
            "read AGENT_WORKFLOW_GUIDE.md before staged work",
            "identify architecture, ADR, roadmap, plan, task, BDD/product, policy, and config docs that materially constrain the task",
            "run scripts/rri.py before implementation",
            "wait for approval when RRI is 26 or higher",
            "read DESIGN.md for mobile UI/presentation work",
            "run Gemma Reviewer or D14 before development closure when required",
        ],
    }


def mark_preflight(repo_root: Path) -> Path:
    path = sentinel_path(repo_root)
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_suffix(f"{path.suffix}.tmp")
    tmp_path.write_text(json.dumps(sentinel_payload(repo_root), indent=2) + "\n", encoding="utf-8")
    os.replace(tmp_path, path)
    return path


def load_sentinel(path: Path) -> Dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise PreflightError(
            f"Missing {path}. Run scripts/agent-preflight.py --mark before editing."
        ) from exc
    except json.JSONDecodeError as exc:
        raise PreflightError(
            f"Invalid {path}. Re-run scripts/agent-preflight.py --mark before editing."
        ) from exc
    if not isinstance(data, dict):
        raise PreflightError(
            f"Invalid {path}: expected a JSON object. Re-run scripts/agent-preflight.py --mark."
        )
    return data


def check_preflight(repo_root: Path) -> Dict[str, Any]:
    path = sentinel_path(repo_root)
    data = load_sentinel(path)
    marked_root = data.get("repo_root")
    if marked_root != str(repo_root.resolve()):
        raise PreflightError(
            f"{path} was marked for {marked_root!r}, not {str(repo_root.resolve())!r}. "
            "Run scripts/agent-preflight.py --mark in this repository."
        )
    if data.get("version") != SCRIPT_VERSION:
        raise PreflightError(
            f"{path} has version {data.get('version')!r}; expected {SCRIPT_VERSION}. "
            "Run scripts/agent-preflight.py --mark again."
        )
    return data


class ReceiptValidationError(PreflightError):
    """Raised when a v2 receipt payload or its inputs fail validation."""


def validate_provider(provider: str) -> None:
    if provider not in V2_VALID_PROVIDERS:
        raise ReceiptValidationError(
            f"Unsupported provider {provider!r}; expected one of {sorted(V2_VALID_PROVIDERS)}."
        )


def validate_lifecycle_event(hook_event_name: str) -> None:
    if hook_event_name not in V2_VALID_LIFECYCLE_EVENTS:
        raise ReceiptValidationError(
            f"Unsupported lifecycle hook_event_name {hook_event_name!r}; "
            f"expected one of {sorted(V2_VALID_LIFECYCLE_EVENTS)}."
        )


def validate_opaque_id(label: str, value: str) -> None:
    if not value:
        raise ReceiptValidationError(f"{label} must not be empty.")
    if "\x00" in value:
        raise ReceiptValidationError(f"{label} must not contain a NUL byte.")
    if "/" in value or "\\" in value:
        raise ReceiptValidationError(f"{label} must not contain a path separator.")
    if ".." in value:
        raise ReceiptValidationError(f"{label} must not contain a '..' segment.")


def compute_receipt_identity(provider: str, session_id: str, actor_id: str) -> str:
    validate_provider(provider)
    validate_opaque_id("session_id", session_id)
    validate_opaque_id("actor_id", actor_id)
    digest_input = "\x00".join([provider, session_id, actor_id]).encode("utf-8")
    return hashlib.sha256(digest_input).hexdigest()


def hash_source_file(repo_root: Path, relative_path: str) -> Dict[str, Any]:
    full_path = repo_root / relative_path
    try:
        data = full_path.read_bytes()
    except OSError as exc:
        raise ReceiptValidationError(
            f"Could not read source file {relative_path!r} under {repo_root}: {exc}"
        ) from exc
    return {
        "path": relative_path,
        "sha256": hashlib.sha256(data).hexdigest(),
        "bytes": len(data),
    }


def build_v2_receipt_payload(
    *,
    provider: str,
    session_id: str,
    actor_id: str,
    repo_root: Path,
    hook_event_name: str,
    source: str,
    transcript_path: str,
    native_instruction_mechanism: str,
    native_instruction_path: str,
    document_paths: List[str],
) -> Dict[str, Any]:
    validate_provider(provider)
    validate_lifecycle_event(hook_event_name)
    validate_opaque_id("session_id", session_id)
    validate_opaque_id("actor_id", actor_id)

    identity = compute_receipt_identity(provider, session_id, actor_id)

    native_instruction = hash_source_file(repo_root, native_instruction_path)
    native_instruction["mechanism"] = native_instruction_mechanism

    documents = [hash_source_file(repo_root, doc_path) for doc_path in document_paths]

    return {
        "schema_version": RECEIPT_SCHEMA_VERSION,
        "provider": provider,
        "session_id": session_id,
        "actor_id": actor_id,
        "receipt_identity": identity,
        "repo_root": str(repo_root.resolve()),
        "lifecycle": {
            "hook_event_name": hook_event_name,
            "source": source,
            "transcript_path": transcript_path,
        },
        "loaded_at": datetime.now(timezone.utc).isoformat(),
        "native_instruction": native_instruction,
        "documents": documents,
    }


def validate_v2_receipt_payload(payload: Dict[str, Any]) -> None:
    if not isinstance(payload, dict):
        raise ReceiptValidationError("v2 receipt payload must be a JSON object.")
    if "version" in payload and "schema_version" not in payload:
        raise ReceiptValidationError(
            "Payload looks like a legacy v1 sentinel (has 'version', missing 'schema_version')."
        )
    if payload.get("schema_version") != RECEIPT_SCHEMA_VERSION:
        raise ReceiptValidationError(
            f"Unsupported receipt schema_version {payload.get('schema_version')!r}; "
            f"expected {RECEIPT_SCHEMA_VERSION}."
        )
    validate_provider(payload.get("provider"))
    validate_opaque_id("session_id", payload.get("session_id", ""))
    validate_opaque_id("actor_id", payload.get("actor_id", ""))
    lifecycle = payload.get("lifecycle")
    if not isinstance(lifecycle, dict):
        raise ReceiptValidationError("v2 receipt payload requires a 'lifecycle' object.")
    validate_lifecycle_event(lifecycle.get("hook_event_name"))
    for required_key in ("native_instruction", "documents"):
        if required_key not in payload:
            raise ReceiptValidationError(f"v2 receipt payload missing {required_key!r}.")


def v2_receipts_dir(repo_root: Path) -> Path:
    return repo_root / RECEIPTS_DIR_RELATIVE


def v2_receipt_path(repo_root: Path, provider: str, session_id: str, actor_id: str) -> Path:
    identity = compute_receipt_identity(provider, session_id, actor_id)
    return v2_receipts_dir(repo_root) / f"{identity}.json"


def _secure_mkdir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)
    try:
        os.chmod(path, 0o700)
    except OSError:
        pass


def _invalidate_prior_receipt(target_path: Path) -> None:
    if not target_path.exists() and not target_path.is_symlink():
        return
    try:
        target_path.unlink()
    except FileNotFoundError:
        pass


def publish_v2_receipt(
    repo_root: Path,
    payload: Dict[str, Any],
) -> Path:
    """Atomically publish a v2 receipt, invalidating any prior receipt for the
    same provider/session/actor identity first. A crash or error at any point
    before the final `os.replace` leaves no authorizing receipt on disk."""
    validate_v2_receipt_payload(payload)

    target_path = v2_receipt_path(
        repo_root, payload["provider"], payload["session_id"], payload["actor_id"]
    )
    receipts_dir = target_path.parent
    _secure_mkdir(receipts_dir)

    _invalidate_prior_receipt(target_path)

    tmp_path = (
        receipts_dir
        / f".{target_path.name}.{os.getpid()}.{threading.get_ident()}.{uuid.uuid4().hex}.tmp"
    )
    try:
        fd = os.open(str(tmp_path), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                handle.write(json.dumps(payload, indent=2) + "\n")
                handle.flush()
                os.fsync(handle.fileno())
        except BaseException:
            try:
                os.close(fd)
            except OSError:
                pass
            raise
        try:
            os.chmod(tmp_path, 0o600)
        except OSError:
            pass
        os.replace(tmp_path, target_path)
    except BaseException:
        try:
            tmp_path.unlink()
        except FileNotFoundError:
            pass
        raise

    return target_path


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Print and validate the DubBridge agent-session workflow preflight."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=None,
        help="Repository root override, mainly for tests and hook wrappers.",
    )
    parser.add_argument(
        "--print-summary",
        action="store_true",
        help="Print the compact workflow startup summary.",
    )
    parser.add_argument(
        "--mark",
        action="store_true",
        help="Write the session preflight sentinel.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Fail unless the session preflight sentinel is present and valid.",
    )
    return parser


def resolve_repo_root(raw: Path | None) -> Path:
    if raw is not None:
        return raw.resolve()
    return find_repo_root()


def main(argv: List[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if not (args.print_summary or args.mark or args.check):
        args.print_summary = True

    repo_root = resolve_repo_root(args.repo_root)

    if args.print_summary:
        sys.stdout.write(preflight_summary())

    if args.mark:
        path = mark_preflight(repo_root)
        print(f"agent preflight marked: {path}")

    if args.check:
        try:
            data = check_preflight(repo_root)
        except PreflightError as exc:
            print(f"agent preflight failed: {exc}", file=sys.stderr)
            return 1
        print(f"agent preflight ok: {data.get('marked_at', 'unknown time')}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
