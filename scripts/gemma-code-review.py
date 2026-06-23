#!/usr/bin/env python3
"""Run a read-only local Gemma code review over a pre-built packet.

The caller builds the packet, including the diff to review. This wrapper owns
only transport, contract parsing, out-of-scope labeling, and result writing.
It never applies patches or approves task completion.
"""

import argparse
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gemma_local


STATUS_VALUES = {"PASS": "pass", "FINDINGS": "findings", "BLOCKED": "blocked"}
SEVERITY_VALUES = {"blocking", "major", "minor", "nit"}
FINDING_START_MARKER = "=== FINDING START ==="
FINDING_END_MARKER = "=== FINDING END ==="
PATCH_LIKE_PATTERNS = (
    "diff --git ",
    "--- ",
    "+++ ",
    "@@ ",
    "=== FILE START ===",
    "=== REPLACEMENT START ===",
    "ACTION: ",
)


def parse_args():
    parser = argparse.ArgumentParser(
        description="Send a read-only code review packet to local Ollama/Gemma.",
    )
    parser.add_argument(
        "packet",
        nargs="?",
        help="Packet file to send. Reads stdin when omitted or set to '-'.",
    )
    parser.add_argument(
        "--host",
        default=os.environ.get("OLLAMA_HOST", gemma_local.DEFAULT_HOST),
        help=f"Ollama host; defaults to OLLAMA_HOST or {gemma_local.DEFAULT_HOST}.",
    )
    parser.add_argument(
        "--model",
        default=os.environ.get(
            "DUBBRIDGE_REVIEW_MODEL",
            os.environ.get("DUBBRIDGE_LOW_RRI_MODEL", gemma_local.DEFAULT_MODEL),
        ),
        help=(
            "Review model; defaults to DUBBRIDGE_REVIEW_MODEL, then "
            "DUBBRIDGE_LOW_RRI_MODEL, then the repo local Gemma default."
        ),
    )
    parser.add_argument(
        "--idle-timeout",
        type=int,
        dest="idle_timeout",
        default=int(
            os.environ.get(
                "DUBBRIDGE_REVIEW_IDLE_TIMEOUT_SECONDS",
                os.environ.get(
                    "DUBBRIDGE_LOW_RRI_IDLE_TIMEOUT_SECONDS",
                    str(gemma_local.DEFAULT_IDLE_TIMEOUT_SECONDS),
                ),
            )
        ),
        help="Seconds without a new token before treating Gemma as stalled.",
    )
    parser.add_argument(
        "--max-wall",
        type=int,
        dest="max_wall",
        default=int(
            os.environ.get(
                "DUBBRIDGE_REVIEW_MAX_WALL_SECONDS",
                os.environ.get(
                    "DUBBRIDGE_LOW_RRI_MAX_WALL_SECONDS",
                    str(gemma_local.DEFAULT_MAX_WALL_SECONDS),
                ),
            )
        ),
        help="Hard wall-time cap in seconds.",
    )
    parser.add_argument(
        "--num-ctx",
        type=int,
        dest="num_ctx",
        default=int(
            os.environ.get(
                "DUBBRIDGE_REVIEW_NUM_CTX",
                os.environ.get("DUBBRIDGE_LOW_RRI_NUM_CTX", str(gemma_local.DEFAULT_NUM_CTX)),
            )
        ),
        help="Context window size for Ollama.",
    )
    parser.add_argument(
        "--num-predict",
        type=int,
        dest="num_predict",
        default=int(
            os.environ.get(
                "DUBBRIDGE_REVIEW_NUM_PREDICT",
                os.environ.get(
                    "DUBBRIDGE_LOW_RRI_NUM_PREDICT",
                    str(gemma_local.DEFAULT_NUM_PREDICT),
                ),
            )
        ),
        help="Max tokens Ollama may generate.",
    )
    parser.add_argument(
        "--temperature",
        type=float,
        default=float(
            os.environ.get(
                "DUBBRIDGE_REVIEW_TEMPERATURE",
                os.environ.get(
                    "DUBBRIDGE_LOW_RRI_TEMPERATURE",
                    str(gemma_local.DEFAULT_TEMPERATURE),
                ),
            )
        ),
        help="Sampling temperature for Ollama.",
    )
    parser.add_argument(
        "--think",
        action="store_true",
        default=gemma_local.bool_from_env(
            "DUBBRIDGE_REVIEW_THINK",
            gemma_local.bool_from_env("DUBBRIDGE_LOW_RRI_THINK", gemma_local.DEFAULT_THINK),
        ),
        help="Enable Ollama thinking mode for models that support it.",
    )
    parser.add_argument(
        "--no-think",
        action="store_false",
        dest="think",
        help="Disable thinking mode for this invocation, overriding the environment.",
    )
    parser.add_argument(
        "--out",
        default=None,
        metavar="FILE",
        help="Write the validated result JSON atomically to FILE instead of stdout.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the Ollama request payload without sending it.",
    )
    return parser.parse_args()


def build_review_payload(model, packet, num_ctx, num_predict, temperature, think):
    system_prompt = (
        "You are Gemma Reviewer for DubBridge. You are read-only.\n"
        "Review the supplied packet and diff. Do not approve, close tasks, "
        "modify files, emit patches, emit unified diffs, or output file bodies.\n"
        "Return ONLY tagged text in this exact shape:\n"
        "STATUS: PASS\n"
        "SUMMARY: short review summary\n"
        "=== FINDING START ===\n"
        "PATH: repo/relative/path.ext\n"
        "LINE: 123\n"
        "SEVERITY: blocking|major|minor|nit\n"
        "DETAIL: concrete bug or risk\n"
        "SUGGESTION: concise fix direction\n"
        "=== FINDING END ===\n"
        "Rules: use exactly one STATUS value: PASS, FINDINGS, or BLOCKED. "
        "Use PASS with no finding blocks when no issues are found. Use FINDINGS "
        "with one or more finding blocks. Use BLOCKED only when the packet is not "
        "reviewable. No markdown fences, no JSON, no diff, no patch, no extra text."
    )
    return gemma_local.build_chat_payload(
        model=model,
        system_prompt=system_prompt,
        packet=packet,
        num_ctx=num_ctx,
        num_predict=num_predict,
        temperature=temperature,
        think=think,
    )


def changed_paths_from_packet(packet):
    paths = set()
    for line in packet.splitlines():
        if line.startswith("diff --git "):
            match = re.match(r"diff --git a/(.+?) b/(.+)$", line)
            if match:
                paths.add(match.group(1))
                paths.add(match.group(2))
        elif line.startswith("+++ b/"):
            paths.add(line[len("+++ b/"):].strip())
        elif line.startswith("--- a/"):
            paths.add(line[len("--- a/"):].strip())
    paths.discard("/dev/null")
    return sorted(paths)


def parse_review_response(content, changed_paths):
    content = gemma_local.normalize_tagged_content(content, "review")
    for pattern in PATCH_LIKE_PATTERNS:
        if pattern in content:
            raise RuntimeError(
                f"invalid review response: patch-like output is forbidden ({pattern!r})"
            )

    lines = content.split("\n")
    status = None
    summary = None
    findings = []
    idx = 0

    def skip_blank(i):
        while i < len(lines) and not lines[i].strip():
            i += 1
        return i

    idx = skip_blank(idx)
    while idx < len(lines):
        line = lines[idx]
        if line == FINDING_START_MARKER:
            finding, idx = parse_finding_block(lines, idx)
            finding["scope"] = (
                "in-scope" if finding["path"] in changed_paths else "out-of-scope"
            )
            findings.append(finding)
            idx = skip_blank(idx)
            continue
        if line.startswith("STATUS: ") or line.strip() in STATUS_VALUES:
            if status is not None:
                raise RuntimeError("invalid review response: duplicate STATUS header")
            raw = (line[len("STATUS: "):] if line.startswith("STATUS: ") else line).strip()
            if raw not in STATUS_VALUES:
                raise RuntimeError(f"invalid review response: unknown STATUS {raw!r}")
            status = STATUS_VALUES[raw]
        elif line.startswith("SUMMARY: "):
            if summary is not None:
                raise RuntimeError("invalid review response: duplicate SUMMARY header")
            summary = line[len("SUMMARY: "):].strip()
        else:
            raise RuntimeError(
                "invalid review response: unexpected text outside sections: "
                f"{line!r}"
            )
        idx += 1
        idx = skip_blank(idx)

    if status is None:
        raise RuntimeError("invalid review response: missing STATUS header")
    if summary is None:
        raise RuntimeError("invalid review response: missing SUMMARY header")
    if status == "pass" and findings:
        raise RuntimeError("invalid review response: STATUS PASS cannot include findings")
    if status == "findings" and not findings:
        raise RuntimeError("invalid review response: STATUS FINDINGS requires findings")

    return {
        "status": status,
        "summary": summary,
        "changed_paths": changed_paths,
        "findings": findings,
    }


def parse_finding_block(lines, start_idx):
    idx = start_idx + 1
    fields = {}
    while idx < len(lines) and lines[idx] != FINDING_END_MARKER:
        line = lines[idx]
        if not line.strip():
            idx += 1
            continue
        for label in ("PATH", "LINE", "SEVERITY", "DETAIL", "SUGGESTION"):
            prefix = f"{label}: "
            if line.startswith(prefix):
                key = label.lower()
                if key in fields:
                    raise RuntimeError(
                        f"invalid review response: duplicate {label} in finding"
                    )
                fields[key] = line[len(prefix):].strip()
                break
        else:
            raise RuntimeError(
                f"invalid review response: unexpected finding text: {line!r}"
            )
        idx += 1

    if idx >= len(lines):
        raise RuntimeError(
            "invalid review response: missing finding end marker "
            "(response may be truncated)"
        )
    idx += 1

    required = ["path", "line", "severity", "detail", "suggestion"]
    missing = [key for key in required if not fields.get(key)]
    if missing:
        raise RuntimeError(f"invalid review response: finding missing {missing}")
    if fields["severity"] not in SEVERITY_VALUES:
        raise RuntimeError(
            f"invalid review response: invalid severity {fields['severity']!r}"
        )
    try:
        fields["line"] = int(fields["line"])
    except ValueError as exc:
        raise RuntimeError("invalid review response: LINE must be an integer") from exc
    if fields["line"] < 1:
        raise RuntimeError("invalid review response: LINE must be >= 1")
    return fields, idx


def main():
    args = parse_args()
    packet = gemma_local.read_packet(args.packet).strip()
    if not packet:
        raise RuntimeError("review packet is empty")

    payload = build_review_payload(
        args.model,
        packet,
        args.num_ctx,
        args.num_predict,
        args.temperature,
        args.think,
    )
    if args.dry_run:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    gemma_local.ensure_model_available(args.host, args.model, args.idle_timeout)
    content = gemma_local.stream_chat(
        gemma_local.endpoint(args.host, "/api/chat"),
        payload,
        idle_timeout=args.idle_timeout,
        max_wall=args.max_wall,
        progress_label="review",
    )
    result = parse_review_response(content, changed_paths_from_packet(packet))

    if args.out:
        gemma_local.write_result(result, args.out)
        print(f"[review] result written to {args.out}", file=sys.stderr)
    else:
        print(json.dumps(result, indent=2, sort_keys=True))

    return 2 if result["status"] == "blocked" else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except gemma_local.GemmaIdleTimeout as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(exc.exit_code)
    except gemma_local.GemmaWallTimeout as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(exc.exit_code)
    except (RuntimeError, OSError) as exc:
        print(f"Gemma review failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
