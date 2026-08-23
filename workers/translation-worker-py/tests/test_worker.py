"""Unit tests for the translation worker.

All tests force DUBBRIDGE_TRANSLATION_PROVIDER=fake, so no network call or
real translation model is needed.
"""

import json
import os
import sys
import urllib.error
from io import StringIO
from typing import Any, Dict, Optional, Tuple
from unittest.mock import MagicMock, patch

FAKE_ENV = {"DUBBRIDGE_TRANSLATION_PROVIDER": "fake"}


def run_main(stdin_text: str, env: Optional[Dict[str, str]] = None) -> Tuple[int, dict]:
    """Run main() with controlled stdin/env and capture stdout + exit code."""
    if "main" in sys.modules:
        del sys.modules["main"]

    sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

    captured_stdout = StringIO()
    exit_code = 0
    effective_env = dict(FAKE_ENV)
    if env:
        effective_env.update(env)

    with patch("sys.stdin", StringIO(stdin_text)), patch("sys.stdout", captured_stdout):
        with patch.dict(os.environ, effective_env):
            try:
                import main as worker_main

                worker_main.main()
            except SystemExit as e:
                exit_code = int(e.code) if e.code is not None else 0

    output_text = captured_stdout.getvalue()
    try:
        output = json.loads(output_text)
    except json.JSONDecodeError:
        output = {"_raw": output_text}

    return exit_code, output


def valid_input(job_id: str = "j1") -> Dict[str, Any]:
    return {
        "schema_version": 1,
        "job_id": job_id,
        "source_language": "en-US",
        "target_language": "es-ES",
        "segments": [
            {"segment_id": "s-0", "start_ms": 0, "end_ms": 1000, "source_text": "hello"},
            {"segment_id": "s-1", "start_ms": 1000, "end_ms": 2000, "source_text": "world"},
        ],
    }


# ---------------------------------------------------------------------------
# HP-1: valid input produces one translated segment per source segment,
# with segment_id/start_ms/end_ms preserved unchanged.
# ---------------------------------------------------------------------------


def test_valid_input_produces_translated_segments_with_preserved_identity_and_timing():
    exit_code, output = run_main(json.dumps(valid_input()))

    assert exit_code == 0
    assert output["schema_version"] == 1
    assert output["job_id"] == "j1"
    assert output["source_language"] == "en-US"
    assert output["target_language"] == "es-ES"
    assert output["status"] == "ok"
    assert len(output["segments"]) == 2

    first, second = output["segments"]
    assert first["segment_id"] == "s-0"
    assert first["start_ms"] == 0
    assert first["end_ms"] == 1000
    assert first["source_text"] == "hello"
    assert first["translated_text"] == "[es-ES] hello"

    assert second["segment_id"] == "s-1"
    assert second["start_ms"] == 1000
    assert second["end_ms"] == 2000
    assert second["translated_text"] == "[es-ES] world"


# ---------------------------------------------------------------------------
# EC-2: invalid/malformed input produces no success artifact.
# ---------------------------------------------------------------------------


def test_invalid_json_emits_error_and_exits_1():
    exit_code, output = run_main("this is not json")
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"
    assert "JSON" in output["message"]
    assert "segments" not in output


def test_missing_required_field_emits_error_and_exits_1():
    inp = valid_input()
    del inp["target_language"]
    exit_code, output = run_main(json.dumps(inp))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"
    assert output["job_id"] == "j1"


def test_unsupported_schema_version_emits_error_and_exits_1():
    inp = valid_input()
    inp["schema_version"] = 2
    exit_code, output = run_main(json.dumps(inp))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"


def test_empty_segments_emits_error_and_exits_1():
    inp = valid_input()
    inp["segments"] = []
    exit_code, output = run_main(json.dumps(inp))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"


def test_segment_missing_field_emits_error_and_exits_1():
    inp = valid_input()
    del inp["segments"][0]["source_text"]
    exit_code, output = run_main(json.dumps(inp))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"


def test_unresolved_source_language_emits_error_and_exits_1():
    inp = valid_input()
    inp["source_language"] = ""
    exit_code, output = run_main(json.dumps(inp))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"


def test_unresolved_target_language_emits_error_and_exits_1():
    inp = valid_input()
    inp["target_language"] = ""
    exit_code, output = run_main(json.dumps(inp))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"


def test_non_object_input_emits_error_and_exits_1():
    exit_code, output = run_main(json.dumps(["not", "an", "object"]))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"


def test_empty_string_job_id_emits_error_and_exits_1():
    inp = valid_input()
    inp["job_id"] = ""
    exit_code, output = run_main(json.dumps(inp))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"


def test_non_string_job_id_emits_error_and_exits_1():
    inp = valid_input()
    inp["job_id"] = 12345
    exit_code, output = run_main(json.dumps(inp))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"
    assert output["job_id"] == ""


def test_non_object_segment_emits_error_and_exits_1():
    inp = valid_input()
    inp["segments"] = ["not-a-segment-object"]
    exit_code, output = run_main(json.dumps(inp))
    assert exit_code == 1
    assert output["error_code"] == "invalid_input"


# ---------------------------------------------------------------------------
# EC-1: provider failure emits the error-schema payload and exits non-zero,
# with no partial success output.
# ---------------------------------------------------------------------------


def test_unknown_provider_kind_emits_error_and_exits_1():
    exit_code, output = run_main(
        json.dumps(valid_input()), env={"DUBBRIDGE_TRANSLATION_PROVIDER": "not-a-real-provider"}
    )
    assert exit_code == 1
    assert output["error_code"] == "provider_misconfigured"
    assert "segments" not in output


def test_http_provider_without_credentials_emits_error_and_exits_1():
    exit_code, output = run_main(json.dumps(valid_input()), env={"DUBBRIDGE_TRANSLATION_PROVIDER": "http"})
    assert exit_code == 1
    assert output["error_code"] == "provider_misconfigured"
    assert "segments" not in output


def test_http_provider_credentials_never_appear_in_output():
    env = {
        "DUBBRIDGE_TRANSLATION_PROVIDER": "http",
        "DUBBRIDGE_TRANSLATION_API_URL": "https://example.invalid/translate",
        "DUBBRIDGE_TRANSLATION_API_KEY": "super-secret-key",
    }
    fake_response = MagicMock()
    fake_response.__enter__.return_value.read.return_value = json.dumps(
        {"translations": ["hola", "mundo"]}
    ).encode("utf-8")
    fake_response.__exit__.return_value = False

    with patch("main.urllib.request.urlopen", return_value=fake_response) as mock_urlopen:
        exit_code, output = run_main(json.dumps(valid_input()), env=env)

    assert exit_code == 0
    assert output["segments"][0]["translated_text"] == "hola"
    assert output["segments"][1]["translated_text"] == "mundo"
    assert "super-secret-key" not in json.dumps(output)

    sent_request = mock_urlopen.call_args[0][0]
    assert sent_request.get_header("Authorization") == "Bearer super-secret-key"


def test_http_provider_unreachable_emits_error_and_exits_1():
    env = {
        "DUBBRIDGE_TRANSLATION_PROVIDER": "http",
        "DUBBRIDGE_TRANSLATION_API_URL": "https://example.invalid/translate",
        "DUBBRIDGE_TRANSLATION_API_KEY": "key",
    }
    with patch("main.urllib.request.urlopen", side_effect=urllib.error.URLError("connection refused")):
        exit_code, output = run_main(json.dumps(valid_input()), env=env)

    assert exit_code == 1
    assert output["error_code"] == "provider_unreachable"
    assert "segments" not in output


def test_http_provider_mismatched_translation_count_emits_error_and_exits_1():
    env = {
        "DUBBRIDGE_TRANSLATION_PROVIDER": "http",
        "DUBBRIDGE_TRANSLATION_API_URL": "https://example.invalid/translate",
        "DUBBRIDGE_TRANSLATION_API_KEY": "key",
    }
    fake_response = MagicMock()
    fake_response.__enter__.return_value.read.return_value = json.dumps(
        {"translations": ["hola"]}
    ).encode("utf-8")
    fake_response.__exit__.return_value = False

    with patch("main.urllib.request.urlopen", return_value=fake_response):
        exit_code, output = run_main(json.dumps(valid_input()), env=env)

    assert exit_code == 1
    assert output["error_code"] == "provider_invalid_response"


def test_http_provider_non_string_translation_emits_error_and_exits_1():
    env = {
        "DUBBRIDGE_TRANSLATION_PROVIDER": "http",
        "DUBBRIDGE_TRANSLATION_API_URL": "https://example.invalid/translate",
        "DUBBRIDGE_TRANSLATION_API_KEY": "key",
    }
    fake_response = MagicMock()
    fake_response.__enter__.return_value.read.return_value = json.dumps(
        {"translations": ["hola", 123]}
    ).encode("utf-8")
    fake_response.__exit__.return_value = False

    with patch("main.urllib.request.urlopen", return_value=fake_response):
        exit_code, output = run_main(json.dumps(valid_input()), env=env)

    assert exit_code == 1
    assert output["error_code"] == "provider_invalid_response"


def test_http_provider_invalid_json_response_emits_error_and_exits_1():
    env = {
        "DUBBRIDGE_TRANSLATION_PROVIDER": "http",
        "DUBBRIDGE_TRANSLATION_API_URL": "https://example.invalid/translate",
        "DUBBRIDGE_TRANSLATION_API_KEY": "key",
    }
    fake_response = MagicMock()
    fake_response.__enter__.return_value.read.return_value = b"not json"
    fake_response.__exit__.return_value = False

    with patch("main.urllib.request.urlopen", return_value=fake_response):
        exit_code, output = run_main(json.dumps(valid_input()), env=env)

    assert exit_code == 1
    assert output["error_code"] == "provider_invalid_response"


def test_http_provider_http_error_status_emits_error_and_exits_1():
    env = {
        "DUBBRIDGE_TRANSLATION_PROVIDER": "http",
        "DUBBRIDGE_TRANSLATION_API_URL": "https://example.invalid/translate",
        "DUBBRIDGE_TRANSLATION_API_KEY": "key",
    }
    http_error = urllib.error.HTTPError(
        url="https://example.invalid/translate", code=503, msg="Service Unavailable", hdrs=None, fp=None
    )

    with patch("main.urllib.request.urlopen", side_effect=http_error):
        exit_code, output = run_main(json.dumps(valid_input()), env=env)

    assert exit_code == 1
    assert output["error_code"] == "provider_http_error"
