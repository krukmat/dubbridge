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
from typing import Dict, List, NoReturn, Tuple


class WorkerError(Exception):
    """Base class for expected worker failures that map to structured error output."""

    error_code = "worker_error"

    def __init__(self, message: str, job_id: str = "") -> None:
        super().__init__(message)
        self.job_id = job_id


class InvalidInputError(WorkerError):
    """Raised when the subprocess input violates the required worker contract."""

    error_code = "invalid_input"


class AudioNotFoundError(WorkerError):
    """Raised when the requested local audio file does not exist."""

    error_code = "audio_not_found"


class TranscriptionError(WorkerError):
    """Raised when faster-whisper cannot load or transcribe the requested audio."""

    error_code = "transcription_failed"


def emit_error(job_id: str, error_code: str, message: str) -> NoReturn:
    json.dump({"job_id": job_id, "error_code": error_code, "message": message}, sys.stdout)
    sys.stdout.flush()
    sys.exit(1)


def parse_input(raw: str) -> Tuple[str, str, str]:
    """Return the validated (job_id, audio_uri, language_hint) tuple."""
    try:
        inp = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise InvalidInputError(f"failed to parse JSON: {exc}") from exc

    if not isinstance(inp, dict):
        raise InvalidInputError("input must be a JSON object")

    if "job_id" not in inp:
        raise InvalidInputError("missing required field: job_id")
    job_id = inp["job_id"]
    if not isinstance(job_id, str):
        raise InvalidInputError("field job_id must be a string")

    if "audio_uri" not in inp:
        raise InvalidInputError("missing required field: audio_uri", job_id)
    audio_uri = inp["audio_uri"]
    if not isinstance(audio_uri, str) or not audio_uri:
        raise InvalidInputError("field audio_uri must be a non-empty string", job_id)

    if "language_hint" not in inp:
        raise InvalidInputError("missing required field: language_hint", job_id)
    language_hint = inp["language_hint"]
    if not isinstance(language_hint, str):
        raise InvalidInputError("field language_hint must be a string", job_id)

    return job_id, audio_uri, language_hint


def require_audio_file(job_id: str, audio_uri: str) -> str:
    """Return the local path or raise a typed not-found error."""
    audio_path = audio_uri.removeprefix("file://")
    if not os.path.exists(audio_path):
        raise AudioNotFoundError(f"audio file not found: {audio_path}", job_id)
    return audio_path


def transcribe_audio(
    job_id: str, audio_path: str, language_hint: str
) -> Tuple[List[str], List[Dict]]:
    """Run faster-whisper and normalize transcript/alignment data."""
    try:
        from faster_whisper import WhisperModel  # type: ignore[import-untyped]
    except ImportError as exc:
        raise TranscriptionError("faster-whisper is unavailable", job_id) from exc

    model_size = os.environ.get("ASR_MODEL_SIZE", "large-v3")
    try:
        model = WhisperModel(model_size, device="auto", compute_type="auto")
        language = language_hint if language_hint else None
        segments, _ = model.transcribe(audio_path, language=language, word_timestamps=True)
    except (OSError, RuntimeError, ValueError) as exc:
        raise TranscriptionError(str(exc), job_id) from exc

    full_text_parts: List[str] = []
    word_timestamps: List[Dict] = []

    for segment in segments:
        full_text_parts.append(segment.text.strip())
        if segment.words:
            for word in segment.words:
                word_timestamps.append(
                    {"word": word.word.strip(), "start": word.start, "end": word.end}
                )

    return full_text_parts, word_timestamps


def main() -> None:
    try:
        job_id, audio_uri, language_hint = parse_input(sys.stdin.read())
        audio_path = require_audio_file(job_id, audio_uri)
        full_text_parts, word_timestamps = transcribe_audio(
            job_id, audio_path, language_hint
        )
    except WorkerError as exc:
        emit_error(exc.job_id, exc.error_code, str(exc))

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
