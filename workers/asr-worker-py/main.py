"""ASR worker — stdin/stdout JSON subprocess.

Reads one AsrWorkerInput JSON object from stdin, transcribes the audio
with faster-whisper, writes transcript.json and alignment.json to a
temp dir, and emits one AsrWorkerOutput (exit 0) or AsrWorkerError
(exit 1) JSON object to stdout.
"""

import json
import os
import sys
import tempfile
from typing import Dict, List, NoReturn, Optional, Tuple


def emit_error(job_id: str, error_code: str, message: str) -> NoReturn:
    json.dump({"job_id": job_id, "error_code": error_code, "message": message}, sys.stdout)
    sys.stdout.flush()
    sys.exit(1)


def parse_input(raw: str) -> Tuple[str, str, str]:
    """Returns (job_id, audio_uri, language_hint) or calls emit_error."""
    try:
        inp = json.loads(raw)
    except json.JSONDecodeError as exc:
        emit_error("", "invalid_input", f"failed to parse JSON: {exc}")

    if not isinstance(inp, dict):
        emit_error("", "invalid_input", "input must be a JSON object")

    job_id: str = inp.get("job_id", "")
    audio_uri = inp.get("audio_uri")
    language_hint: str = inp.get("language_hint", "")

    if not audio_uri:
        emit_error(job_id, "invalid_input", "missing required field: audio_uri")

    return job_id, audio_uri, language_hint


def main() -> None:
    raw = sys.stdin.read()
    job_id, audio_uri, language_hint = parse_input(raw)

    audio_path = audio_uri.removeprefix("file://")
    if not os.path.exists(audio_path):
        emit_error(job_id, "audio_not_found", f"audio file not found: {audio_path}")

    model_size = os.environ.get("ASR_MODEL_SIZE", "large-v3")

    try:
        from faster_whisper import WhisperModel  # type: ignore[import-untyped]

        model = WhisperModel(model_size, device="auto", compute_type="auto")
        language = language_hint if language_hint else None
        segments, _ = model.transcribe(audio_path, language=language, word_timestamps=True)

        full_text_parts: List[str] = []
        word_timestamps: List[Dict] = []

        for segment in segments:
            full_text_parts.append(segment.text.strip())
            if segment.words:
                for word in segment.words:
                    word_timestamps.append(
                        {"word": word.word.strip(), "start": word.start, "end": word.end}
                    )

    except Exception as exc:
        emit_error(job_id, "transcription_failed", str(exc))

    tmp = tempfile.mkdtemp(prefix="asr-")
    transcript_path = os.path.join(tmp, "transcript.json")
    alignment_path = os.path.join(tmp, "alignment.json")

    with open(transcript_path, "w", encoding="utf-8") as f:
        json.dump({"job_id": job_id, "text": " ".join(full_text_parts)}, f)

    with open(alignment_path, "w", encoding="utf-8") as f:
        json.dump({"job_id": job_id, "words": word_timestamps}, f)

    result = {
        "job_id": job_id,
        "transcript_uri": f"file://{transcript_path}",
        "alignment_uri": f"file://{alignment_path}",
        "status": "ok",
    }
    json.dump(result, sys.stdout)
    sys.stdout.flush()
    sys.exit(0)


if __name__ == "__main__":
    main()
