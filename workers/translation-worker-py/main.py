"""Translation worker -- stdin/stdout JSON subprocess.

Reads one TranslationWorkerInput JSON object (D3 envelope, schema_version 1)
from stdin, translates every segment through a configurable provider, and
emits one TranslationWorkerOutput (exit 0) or TranslationWorkerError (exit 1)
JSON object to stdout. Mirrors workers/asr-worker-py's stdin/stdout contract
shape byte-for-byte in control flow.

Provider selection is env-driven (DUBBRIDGE_TRANSLATION_PROVIDER, default
"fake"): "fake" is the deterministic provider the test suite uses (no
network call); "http" calls a generic JSON translation endpoint configured
by DUBBRIDGE_TRANSLATION_API_URL, with credentials read only from the
injected DUBBRIDGE_TRANSLATION_API_KEY environment variable -- never
hardcoded, never logged.
"""

import json
import os
import sys
import urllib.error
import urllib.request
from typing import Any, Dict, List, NoReturn, Tuple

REQUIRED_INPUT_FIELDS = ("schema_version", "job_id", "source_language", "target_language", "segments")
REQUIRED_SEGMENT_FIELDS = ("segment_id", "start_ms", "end_ms", "source_text")


class ProviderError(Exception):
    """Raised by a TranslationProvider when translation cannot be completed."""

    def __init__(self, error_code: str, message: str) -> None:
        super().__init__(message)
        self.error_code = error_code
        self.message = message


class FakeTranslationProvider:
    """Deterministic provider used by the test suite. No network call."""

    def translate(self, segments: List[Dict[str, Any]], source_language: str, target_language: str) -> List[str]:
        return [f"[{target_language}] {segment['source_text']}" for segment in segments]


class HttpTranslationProvider:
    """Generic JSON-over-HTTP provider.

    Posts {source_language, target_language, texts: [...]} to
    DUBBRIDGE_TRANSLATION_API_URL with an Authorization bearer header from
    DUBBRIDGE_TRANSLATION_API_KEY, and expects back {translations: [...]}
    (one translated string per input text, same order).
    """

    def __init__(self, api_url: str, api_key: str, timeout_seconds: float = 60.0) -> None:
        self._api_url = api_url
        self._api_key = api_key
        self._timeout_seconds = timeout_seconds

    def translate(self, segments: List[Dict[str, Any]], source_language: str, target_language: str) -> List[str]:
        payload = json.dumps(
            {
                "source_language": source_language,
                "target_language": target_language,
                "texts": [segment["source_text"] for segment in segments],
            }
        ).encode("utf-8")

        request = urllib.request.Request(
            self._api_url,
            data=payload,
            headers={
                "Content-Type": "application/json",
                "User-Agent": "dubbridge-translation-worker/1",
                "Authorization": f"Bearer {self._api_key}",
            },
            method="POST",
        )

        try:
            with urllib.request.urlopen(request, timeout=self._timeout_seconds) as response:
                body = response.read()
        except urllib.error.HTTPError as exc:
            raise ProviderError("provider_http_error", f"translation API returned HTTP {exc.code}") from exc
        except (urllib.error.URLError, TimeoutError) as exc:
            raise ProviderError("provider_unreachable", f"failed to reach translation API: {exc}") from exc

        try:
            parsed = json.loads(body.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise ProviderError("provider_invalid_response", f"translation API returned invalid JSON: {exc}") from exc

        translations = parsed.get("translations") if isinstance(parsed, dict) else None
        if not isinstance(translations, list) or len(translations) != len(segments):
            raise ProviderError(
                "provider_invalid_response",
                "translation API response missing a 'translations' array matching the request size",
            )
        if not all(isinstance(text, str) for text in translations):
            raise ProviderError("provider_invalid_response", "translation API returned a non-string translation")

        return translations


def build_provider() -> Any:
    provider_kind = os.environ.get("DUBBRIDGE_TRANSLATION_PROVIDER", "fake")
    if provider_kind == "fake":
        return FakeTranslationProvider()
    if provider_kind == "http":
        api_url = os.environ.get("DUBBRIDGE_TRANSLATION_API_URL", "")
        api_key = os.environ.get("DUBBRIDGE_TRANSLATION_API_KEY", "")
        if not api_url or not api_key:
            raise ProviderError(
                "provider_misconfigured",
                "DUBBRIDGE_TRANSLATION_API_URL and DUBBRIDGE_TRANSLATION_API_KEY are required for the http provider",
            )
        return HttpTranslationProvider(api_url=api_url, api_key=api_key)
    raise ProviderError("provider_misconfigured", f"unknown DUBBRIDGE_TRANSLATION_PROVIDER: {provider_kind!r}")


def emit_error(job_id: str, error_code: str, message: str) -> NoReturn:
    json.dump({"job_id": job_id, "error_code": error_code, "message": message}, sys.stdout)
    sys.stdout.flush()
    sys.exit(1)


def parse_input(raw: str) -> Tuple[str, Dict[str, Any]]:
    """Returns (job_id, validated input dict) or calls emit_error."""
    try:
        inp = json.loads(raw)
    except json.JSONDecodeError as exc:
        emit_error("", "invalid_input", f"failed to parse JSON: {exc}")

    if not isinstance(inp, dict):
        emit_error("", "invalid_input", "input must be a JSON object")

    job_id = inp.get("job_id", "")
    if not isinstance(job_id, str):
        job_id = ""

    for field in REQUIRED_INPUT_FIELDS:
        if field not in inp:
            emit_error(job_id, "invalid_input", f"missing required field: {field}")

    if inp.get("schema_version") != 1:
        emit_error(job_id, "invalid_input", f"unsupported schema_version: {inp.get('schema_version')!r}")

    if not job_id:
        emit_error(job_id, "invalid_input", "missing required field: job_id")

    if not inp.get("source_language"):
        emit_error(job_id, "invalid_input", "missing required field: source_language")

    if not inp.get("target_language"):
        emit_error(job_id, "invalid_input", "missing required field: target_language")

    segments = inp.get("segments")
    if not isinstance(segments, list) or not segments:
        emit_error(job_id, "invalid_input", "segments must be a non-empty array")

    for segment in segments:
        if not isinstance(segment, dict):
            emit_error(job_id, "invalid_input", "each segment must be an object")
        for field in REQUIRED_SEGMENT_FIELDS:
            if field not in segment:
                emit_error(job_id, "invalid_input", f"segment missing required field: {field}")

    return job_id, inp


def main() -> None:
    raw = sys.stdin.read()
    job_id, inp = parse_input(raw)

    source_language = inp["source_language"]
    target_language = inp["target_language"]
    segments = inp["segments"]

    try:
        provider = build_provider()
        translated_texts = provider.translate(segments, source_language, target_language)
    except ProviderError as exc:
        emit_error(job_id, exc.error_code, exc.message)

    output_segments = [
        {
            "segment_id": segment["segment_id"],
            "start_ms": segment["start_ms"],
            "end_ms": segment["end_ms"],
            "source_text": segment["source_text"],
            "translated_text": translated_text,
        }
        for segment, translated_text in zip(segments, translated_texts)
    ]

    result = {
        "schema_version": 1,
        "job_id": job_id,
        "source_language": source_language,
        "target_language": target_language,
        "segments": output_segments,
        "status": "ok",
    }
    json.dump(result, sys.stdout)
    sys.stdout.flush()
    sys.exit(0)


if __name__ == "__main__":
    main()
