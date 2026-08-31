"""ASR worker — stdin/stdout JSON subprocess.

Reads one AsrWorkerInput JSON object from stdin, transcribes the audio
with faster-whisper, writes transcript.json and alignment.json to a
temp dir, and emits one AsrWorkerOutput (exit 0) or AsrWorkerError
(exit 1) JSON object to stdout.
"""

import json
import os
import signal
import sys
import tempfile
from pathlib import Path
from typing import Dict, List, NoReturn, Optional, Tuple

from jsonschema import Draft202012Validator, ValidationError

DEFAULT_TRANSCRIBE_TIMEOUT_SECONDS = 300.0
DEFAULT_MAX_AUDIO_BYTES = 500 * 1024 * 1024
SCHEMA_DIR = Path(__file__).resolve().parent


def _load_schema(filename: str) -> dict:
    with (SCHEMA_DIR / filename).open(encoding="utf-8") as schema_file:
        return json.load(schema_file)


INPUT_VALIDATOR = Draft202012Validator(_load_schema("input.schema.json"))
OUTPUT_VALIDATOR = Draft202012Validator(_load_schema("output.schema.json"))
ERROR_VALIDATOR = Draft202012Validator(_load_schema("error.schema.json"))

WHISPER_LANGUAGE_CODES = frozenset(
    {
        "af", "am", "ar", "as", "az", "ba", "be", "bg", "bn", "bo", "br", "bs",
        "ca", "cs", "cy", "da", "de", "el", "en", "es", "et", "eu", "fa", "fi",
        "fo", "fr", "gl", "gu", "ha", "haw", "he", "hi", "hr", "ht", "hu", "hy",
        "id", "is", "it", "ja", "jw", "ka", "kk", "km", "kn", "ko", "la", "lb",
        "ln", "lo", "lt", "lv", "mg", "mi", "mk", "ml", "mn", "mr", "ms", "mt",
        "my", "ne", "nl", "nn", "no", "oc", "pa", "pl", "ps", "pt", "ro", "ru",
        "sa", "sd", "si", "sk", "sl", "sn", "so", "sq", "sr", "su", "sv", "sw",
        "ta", "te", "tg", "th", "tk", "tl", "tr", "tt", "uk", "ur", "uz", "vi",
        "yi", "yo", "zh",
    }
)


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


class AudioTooLargeError(WorkerError):
    """Raised before model execution when the audio file exceeds the configured bound."""

    error_code = "audio_too_large"


class InvalidLanguageError(WorkerError):
    """Raised when language_hint is neither empty nor a supported Whisper language."""

    error_code = "invalid_language"


class TranscriptionError(WorkerError):
    """Raised when faster-whisper cannot load or transcribe the requested audio."""

    error_code = "transcription_failed"


class TranscriptionTimeoutError(WorkerError):
    """Raised when the configured transcription deadline is exceeded."""

    error_code = "transcription_timeout"


class WorkerConfigurationError(WorkerError):
    """Raised when a worker resource-bound setting is invalid."""

    error_code = "configuration_error"


class OutputContractError(WorkerError):
    """Raised if the worker attempts to emit a success payload outside its schema."""

    error_code = "output_schema_violation"


class _TranscriptionDeadlineExceeded(TimeoutError):
    """Internal signal used to interrupt a model call that exceeded its deadline."""


def emit_error(job_id: str, error_code: str, message: str) -> NoReturn:
    payload = {"job_id": job_id, "error_code": error_code, "message": message}
    ERROR_VALIDATOR.validate(payload)
    json.dump(payload, sys.stdout)
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

    try:
        INPUT_VALIDATOR.validate(inp)
    except ValidationError as exc:
        raise InvalidInputError(f"input schema violation: {exc.message}", job_id) from exc

    return job_id, audio_uri, language_hint


def _positive_float_setting(name: str, default: float, job_id: str) -> float:
    raw_value = os.environ.get(name)
    if raw_value is None:
        return default
    try:
        value = float(raw_value)
    except ValueError as exc:
        raise WorkerConfigurationError(f"{name} must be a positive number", job_id) from exc
    if value <= 0:
        raise WorkerConfigurationError(f"{name} must be a positive number", job_id)
    return value


def _positive_int_setting(name: str, default: int, job_id: str) -> int:
    raw_value = os.environ.get(name)
    if raw_value is None:
        return default
    try:
        value = int(raw_value)
    except ValueError as exc:
        raise WorkerConfigurationError(f"{name} must be a positive integer", job_id) from exc
    if value <= 0:
        raise WorkerConfigurationError(f"{name} must be a positive integer", job_id)
    return value


def require_audio_file(job_id: str, audio_uri: str) -> str:
    """Return the local path or raise a typed not-found/size error."""
    audio_path = audio_uri.removeprefix("file://")
    if not os.path.exists(audio_path):
        raise AudioNotFoundError(f"audio file not found: {audio_path}", job_id)

    max_audio_bytes = _positive_int_setting("ASR_MAX_AUDIO_BYTES", DEFAULT_MAX_AUDIO_BYTES, job_id)
    audio_bytes = os.path.getsize(audio_path)
    if audio_bytes > max_audio_bytes:
        raise AudioTooLargeError(
            f"audio file exceeds ASR_MAX_AUDIO_BYTES ({audio_bytes} > {max_audio_bytes})",
            job_id,
        )
    return audio_path


def validate_language_hint(job_id: str, language_hint: str) -> Optional[str]:
    """Return a validated Whisper language code, or None for auto-detection."""
    if language_hint == "":
        return None
    if language_hint not in WHISPER_LANGUAGE_CODES:
        raise InvalidLanguageError(f"unsupported language_hint: {language_hint}", job_id)
    return language_hint


def _deadline_handler(_signum, _frame) -> NoReturn:
    raise _TranscriptionDeadlineExceeded("transcription deadline exceeded")


def _transcribe_with_timeout(model, job_id: str, timeout_seconds: float, audio_path: str, language):
    previous_handler = signal.signal(signal.SIGALRM, _deadline_handler)
    signal.setitimer(signal.ITIMER_REAL, timeout_seconds)
    try:
        return model.transcribe(audio_path, language=language, word_timestamps=True)
    except _TranscriptionDeadlineExceeded as exc:
        raise TranscriptionTimeoutError(
            f"transcription exceeded {timeout_seconds:g}s timeout", job_id
        ) from exc
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous_handler)


def transcribe_audio(
    job_id: str, audio_path: str, language_hint: str
) -> Tuple[List[str], List[Dict]]:
    """Run faster-whisper within the configured deadline and normalize output."""
    try:
        from faster_whisper import WhisperModel  # type: ignore[import-untyped]
    except ImportError as exc:
        raise TranscriptionError("faster-whisper is unavailable", job_id) from exc

    language = validate_language_hint(job_id, language_hint)
    timeout_seconds = _positive_float_setting(
        "ASR_TRANSCRIBE_TIMEOUT_SECONDS", DEFAULT_TRANSCRIBE_TIMEOUT_SECONDS, job_id
    )
    model_size = os.environ.get("ASR_MODEL_SIZE", "large-v3")
    try:
        model = WhisperModel(model_size, device="auto", compute_type="auto")
        segments, _ = _transcribe_with_timeout(
            model, job_id, timeout_seconds, audio_path, language
        )
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



def validate_output(payload: dict) -> dict:
    """Validate the success payload immediately before emission."""
    try:
        OUTPUT_VALIDATOR.validate(payload)
    except ValidationError as exc:
        job_id = payload.get("job_id", "") if isinstance(payload, dict) else ""
        raise OutputContractError(f"output schema violation: {exc.message}", job_id) from exc
    return payload


def main() -> None:
    try:
        job_id, audio_uri, language_hint = parse_input(sys.stdin.read())
        audio_path = require_audio_file(job_id, audio_uri)
        full_text_parts, word_timestamps = transcribe_audio(job_id, audio_path, language_hint)
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
    try:
        validate_output(result)
    except WorkerError as exc:
        emit_error(exc.job_id, exc.error_code, str(exc))
    json.dump(result, sys.stdout)
    sys.stdout.flush()
    sys.exit(0)


if __name__ == "__main__":
    main()
