"""Opt-in real-audio / real-model smoke test for the ASR subprocess.

Set DUBBRIDGE_ASR_REAL_MODEL_SMOKE=1 to enable it. The default model is
`tiny`, whose first run downloads roughly 78 MB into the Hugging Face cache.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest
from jsonschema import Draft202012Validator

WORKER_DIR = Path(__file__).resolve().parents[1]
FIXTURE = Path(__file__).resolve().parent / "fixtures" / "smoke-tone.wav"
SMOKE_ENV = "DUBBRIDGE_ASR_REAL_MODEL_SMOKE"


@pytest.mark.skipif(
    os.environ.get(SMOKE_ENV) != "1",
    reason=f"set {SMOKE_ENV}=1 to download/use a real faster-whisper model",
)
def test_real_audio_real_model_smoke_matches_output_schema():
    """Exercise the actual subprocess, decoder, model, and output contract."""
    env = os.environ.copy()
    env["ASR_MODEL_SIZE"] = env.get("DUBBRIDGE_ASR_SMOKE_MODEL", "tiny")
    env.setdefault("ASR_TRANSCRIBE_TIMEOUT_SECONDS", "300")

    payload = {
        "job_id": "real-model-smoke",
        "audio_uri": f"file://{FIXTURE}",
        "language_hint": "en",
    }
    completed = subprocess.run(
        [sys.executable, str(WORKER_DIR / "main.py")],
        input=json.dumps(payload),
        capture_output=True,
        text=True,
        env=env,
        timeout=360,
        check=False,
    )

    assert completed.returncode == 0, completed.stdout or completed.stderr
    output = json.loads(completed.stdout)
    with (WORKER_DIR / "output.schema.json").open(encoding="utf-8") as schema_file:
        Draft202012Validator(json.load(schema_file)).validate(output)
