#!/usr/bin/env python3
"""Unit tests for show-codex-session-model.py."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


_SCRIPT = Path(__file__).with_name("show-codex-session-model.py")
_SPEC = importlib.util.spec_from_file_location("show_codex_session_model", _SCRIPT)
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_SCRIPT}")
_MOD = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MOD)


class ShowCodexSessionModelTest(unittest.TestCase):
    def test_hp1_reads_newest_rollout_and_extracts_last_turn_context(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            sessions_dir = Path(tmp)
            older = sessions_dir / "2026" / "07" / "18" / "rollout-older.jsonl"
            newer = sessions_dir / "2026" / "07" / "19" / "rollout-newer.jsonl"
            older.parent.mkdir(parents=True, exist_ok=True)
            newer.parent.mkdir(parents=True, exist_ok=True)

            older.write_text(
                json.dumps({"type": "turn_context", "payload": {"model": "gpt-old", "effort": "low"}})
                + "\n",
                encoding="utf-8",
            )
            newer.write_text(
                "\n".join(
                    [
                        "{bad json",
                        json.dumps({"type": "other", "payload": {}}),
                        json.dumps({"type": "turn_context", "payload": {"model": "gpt-5.4", "effort": "medium"}}),
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            latest = _MOD.latest_rollout_file(sessions_dir)
            context = _MOD.load_last_turn_context(latest)
            output = _MOD.render_output(latest, context)

            self.assertEqual(latest, newer)
            self.assertEqual(context, {"model": "gpt-5.4", "effort": "medium"})
            self.assertIn(f"Sesión: {newer}", output)
            self.assertIn("Modelo efectivo: gpt-5.4", output)
            self.assertIn("Reasoning: medium", output)

    def test_ec1_fails_closed_when_no_rollouts_exist(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(SystemExit) as ctx:
                _MOD.latest_rollout_file(Path(tmp))
            self.assertEqual(str(ctx.exception), "No se encontraron sesiones de Codex.")

    def test_ec2_reports_missing_turn_context_without_crashing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            rollout = Path(tmp) / "rollout-only.jsonl"
            rollout.write_text(
                json.dumps({"type": "other", "payload": {"model": "ignored"}}) + "\n",
                encoding="utf-8",
            )

            context = _MOD.load_last_turn_context(rollout)
            output = _MOD.render_output(rollout, context)

            self.assertIsNone(context)
            self.assertIn("No se encontró turn_context.", output)


if __name__ == "__main__":
    unittest.main()
