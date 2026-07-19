#!/usr/bin/env python3
"""Print the effective Codex model from the newest local rollout session."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def default_sessions_dir() -> Path:
    return Path.home() / ".codex" / "sessions"


def latest_rollout_file(sessions_dir: Path) -> Path:
    files = list(sessions_dir.rglob("rollout-*.jsonl"))
    if not files:
        raise SystemExit("No se encontraron sesiones de Codex.")
    return max(files, key=lambda path: path.stat().st_mtime)


def load_last_turn_context(rollout_path: Path) -> dict[str, object] | None:
    last_context: dict[str, object] | None = None
    with rollout_path.open(encoding="utf-8", errors="ignore") as file:
        for line in file:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            if entry.get("type") == "turn_context":
                payload = entry.get("payload", {})
                if isinstance(payload, dict):
                    last_context = payload
    return last_context


def render_output(rollout_path: Path, last_context: dict[str, object] | None) -> str:
    lines = [f"Sesión: {rollout_path}"]
    if last_context:
        lines.append(f"Modelo efectivo: {last_context.get('model', 'no encontrado')}")
        lines.append(f"Reasoning: {last_context.get('effort', 'no encontrado')}")
    else:
        lines.append("No se encontró turn_context.")
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Show the effective model and reasoning from the newest Codex rollout session."
    )
    parser.add_argument(
        "--sessions-dir",
        default=str(default_sessions_dir()),
        help="Root directory containing Codex rollout session files.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    sessions_dir = Path(args.sessions_dir).expanduser()
    rollout_path = latest_rollout_file(sessions_dir)
    last_context = load_last_turn_context(rollout_path)
    print(render_output(rollout_path, last_context))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
