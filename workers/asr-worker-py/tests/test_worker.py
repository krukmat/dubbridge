"""Unit tests for the ASR worker.

All tests mock faster_whisper so no GPU, model download, or real audio is needed.
"""

import importlib
import json
import os
import sys
import tempfile
from io import StringIO
from typing import Dict, Optional, Tuple
from unittest.mock import MagicMock, patch

import pytest


def load_worker_module():
    """Import the worker module with module-level state reset."""
    if "main" in sys.modules:
        del sys.modules["main"]

    worker_path = os.path.join(os.path.dirname(__file__), "..")
    if worker_path not in sys.path:
        sys.path.insert(0, worker_path)

    return importlib.import_module("main")


def run_main(stdin_text: str, env: Optional[Dict[str, str]] = None) -> Tuple[int, dict]:
    """Run main() with controlled stdin and capture stdout + exit code."""
    worker_main = load_worker_module()
    captured_stdout = StringIO()
    exit_code = 0

    with patch("sys.stdin", StringIO(stdin_text)), patch("sys.stdout", captured_stdout):
        if env:
            with patch.dict(os.environ, env):
                try:
                    worker_main.main()
                except SystemExit as exc:
                    exit_code = int(exc.code) if exc.code is not None else 0
        else:
            try:
                worker_main.main()
            except SystemExit as exc:
                exit_code = int(exc.code) if exc.code is not None else 0

    output_text = captured_stdout.getvalue()
    try:
        output = json.loads(output_text)
    except json.JSONDecodeError:
        output = {"_raw": output_text}

    return exit_code, output


def make_audio_file() -> str:
    """Write a minimal WAV file to a temp path and return its path."""
    with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
        f.write(b"RIFF\x24\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00")
        f.write(b"\x44\xac\x00\x00\x88\x58\x01\x00\x02\x00\x10\x00data\x00\x00\x00\x00")
        return f.name


def make_whisper_mock(words: Optional[list] = None) -> MagicMock:
    """Return a mocked WhisperModel that produces one segment with optional words."""
    word_objects = []
    if words:
        for word_spec in words:
            word = MagicMock()
            word.word = word_spec["word"]
            word.start = word_spec["start"]
            word.end = word_spec["end"]
            word_objects.append(word)

    segment = MagicMock()
    segment.text = "hello world"
    segment.words = word_objects if word_objects else None

    model_instance = MagicMock()
    model_instance.transcribe.return_value = ([segment], MagicMock())
    return MagicMock(return_value=model_instance)


def test_invalid_json_emits_error_and_exits_1():
    exit_code, output = run_main("this is not json")
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"
    assert "JSON" in output["message"]


@pytest.mark.parametrize(
    ("payload", "missing_field"),
    [
        ({"audio_uri": "file:///tmp/audio.wav", "language_hint": "en"}, "job_id"),
        ({"job_id": "j-missing-audio", "language_hint": "en"}, "audio_uri"),
        ({"job_id": "j-missing-language", "audio_uri": "file:///tmp/audio.wav"}, "language_hint"),
    ],
)
def test_missing_required_field_raises_named_invalid_input(payload, missing_field):
    worker_main = load_worker_module()

    with pytest.raises(worker_main.InvalidInputError) as exc_info:
        worker_main.parse_input(json.dumps(payload))

    assert exc_info.value.error_code == "invalid_input"
    assert missing_field in str(exc_info.value)


def test_audio_not_found_emits_error_and_exits_1():
    inp = json.dumps(
        {"job_id": "j1", "audio_uri": "file:///nonexistent/audio.wav", "language_hint": "en"}
    )
    exit_code, output = run_main(inp)
    assert exit_code == 1
    assert output["error_code"] == "audio_not_found"
    assert output["job_id"] == "j1"


def test_audio_not_found_uses_named_exception():
    worker_main = load_worker_module()

    with pytest.raises(worker_main.AudioNotFoundError) as exc_info:
        worker_main.require_audio_file("j-audio", "file:///nonexistent/audio.wav")

    assert exc_info.value.error_code == "audio_not_found"
    assert exc_info.value.job_id == "j-audio"


def test_transcription_exception_emits_error_and_exits_1():
    audio_path = make_audio_file()
    try:
        inp = json.dumps(
            {"job_id": "j2", "audio_uri": f"file://{audio_path}", "language_hint": "en"}
        )

        model_class = make_whisper_mock()
        model_class.return_value.transcribe.side_effect = RuntimeError("transcribe failed")
        with patch.dict("sys.modules", {"faster_whisper": MagicMock(WhisperModel=model_class)}):
            exit_code, output = run_main(inp)

        assert exit_code == 1
        assert output["error_code"] == "transcription_failed"
        assert "transcribe failed" in output["message"]
        assert output["job_id"] == "j2"
    finally:
        os.unlink(audio_path)


def test_transcription_runtime_error_uses_named_exception():
    worker_main = load_worker_module()
    model_class = make_whisper_mock()
    model_class.return_value.transcribe.side_effect = RuntimeError("transcribe failed")

    with patch.dict("sys.modules", {"faster_whisper": MagicMock(WhisperModel=model_class)}):
        with pytest.raises(worker_main.TranscriptionError) as exc_info:
            worker_main.transcribe_audio("j-transcribe", "/tmp/audio.wav", "en")

    assert exc_info.value.error_code == "transcription_failed"
    assert exc_info.value.job_id == "j-transcribe"


def test_successful_transcription_emits_output_and_exits_0():
    audio_path = make_audio_file()
    try:
        inp = json.dumps(
            {"job_id": "j3", "audio_uri": f"file://{audio_path}", "language_hint": "en"}
        )

        words = [
            {"word": "hello", "start": 0.0, "end": 0.5},
            {"word": "world", "start": 0.6, "end": 1.0},
        ]
        model_class = make_whisper_mock(words=words)

        with patch.dict("sys.modules", {"faster_whisper": MagicMock(WhisperModel=model_class)}):
            exit_code, output = run_main(inp)

        assert exit_code == 0
        assert output["job_id"] == "j3"
        assert output["status"] == "ok"

        transcript_path = output["transcript_uri"].removeprefix("file://")
        alignment_path = output["alignment_uri"].removeprefix("file://")

        with open(transcript_path, encoding="utf-8") as f:
            transcript = json.load(f)
        with open(alignment_path, encoding="utf-8") as f:
            alignment = json.load(f)

        assert "hello world" in transcript["text"]
        assert len(alignment["words"]) == 2
        assert alignment["words"][0]["word"] == "hello"
    finally:
        os.unlink(audio_path)


def test_unknown_language_hint_does_not_fail():
    audio_path = make_audio_file()
    try:
        inp = json.dumps(
            {
                "job_id": "j4",
                "audio_uri": f"file://{audio_path}",
                "language_hint": "xx-unknown",
            }
        )

        model_class = make_whisper_mock()
        with patch.dict("sys.modules", {"faster_whisper": MagicMock(WhisperModel=model_class)}):
            exit_code, output = run_main(inp)

        assert exit_code == 0
        _, call_kwargs = model_class.return_value.transcribe.call_args
        assert call_kwargs.get("language") == "xx-unknown"
    finally:
        os.unlink(audio_path)


def test_model_size_env_var_is_used():
    audio_path = make_audio_file()
    try:
        inp = json.dumps(
            {"job_id": "j5", "audio_uri": f"file://{audio_path}", "language_hint": "en"}
        )

        model_class = make_whisper_mock()
        with patch.dict(
            "sys.modules", {"faster_whisper": MagicMock(WhisperModel=model_class)}
        ), patch.dict(os.environ, {"ASR_MODEL_SIZE": "base"}):
            exit_code, output = run_main(inp)

        assert exit_code == 0
        call_args, _ = model_class.call_args
        assert call_args[0] == "base"
    finally:
        os.unlink(audio_path)
