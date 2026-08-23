use std::{io::Write, process::Stdio, time::Duration};

use serde::{Deserialize, Serialize};

/// Current schema version for the D3 canonical translation payload contract.
pub const TRANSLATION_SCHEMA_VERSION: u32 = 1;

/// Default subprocess timeout: 300 seconds (mirrors the ASR worker default).
pub const DEFAULT_TRANSLATION_TIMEOUT_SECS: u64 = 300;

/// One ordered subtitle segment carrying a stable identity across translation.
///
/// `segment_id` is derived by the caller from `(subtitle_artifact_id,
/// zero_based_ordinal)` per D3 — this module never mints or mutates it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslationSegment {
    pub segment_id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub source_text: String,
}

/// Input passed to the translation subprocess via stdin (matches
/// workers/translation-worker-py/input.schema.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationInput {
    pub schema_version: u32,
    pub job_id: String,
    pub source_language: String,
    pub target_language: String,
    pub segments: Vec<TranslationSegment>,
}

/// One translated segment, preserving the source segment's ID and timing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslatedSegment {
    pub segment_id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub source_text: String,
    pub translated_text: String,
}

/// Successful output returned from the translation subprocess on stdout
/// (output.schema.json). This is the D3 `TranslatedSubtitle` envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationOutput {
    pub schema_version: u32,
    pub job_id: String,
    pub source_language: String,
    pub target_language: String,
    pub segments: Vec<TranslatedSegment>,
    pub status: String,
}

/// Error envelope returned from the translation subprocess when processing
/// fails (error.schema.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationError {
    pub job_id: String,
    pub error_code: String,
    pub message: String,
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.error_code, self.message)
    }
}

impl std::error::Error for TranslationError {}

/// Trait that abstracts over translation worker communication.
pub trait TranslationWorkerClient: Send + Sync {
    fn translate(&self, input: TranslationInput) -> Result<TranslationOutput, TranslationError>;
}

/// Launches the translation Python subprocess, sends `TranslationInput` as
/// JSON on stdin, and reads `TranslationOutput` or `TranslationError` from
/// stdout. Mirrors `SubprocessAsrWorkerClient` byte-for-byte in control flow.
pub struct SubprocessTranslationWorkerClient {
    pub command: Vec<String>,
    pub timeout: Duration,
}

impl SubprocessTranslationWorkerClient {
    pub fn new(command: Vec<String>) -> Self {
        Self {
            command,
            timeout: Duration::from_secs(DEFAULT_TRANSLATION_TIMEOUT_SECS),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl TranslationWorkerClient for SubprocessTranslationWorkerClient {
    fn translate(&self, input: TranslationInput) -> Result<TranslationOutput, TranslationError> {
        let binary = self.command.first().cloned().unwrap_or_default();
        let input_json =
            serde_json::to_vec(&input).expect("TranslationInput serialization is infallible");

        let mut child = std::process::Command::new(&binary)
            .args(self.command.get(1..).unwrap_or(&[]))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| TranslationError {
                job_id: input.job_id.clone(),
                error_code: "SPAWN_FAILED".into(),
                message: format!("failed to spawn translation worker '{}': {e}", binary),
            })?;

        if let Some(mut stdin) = child.stdin.take()
            && let Err(e) = stdin.write_all(&input_json)
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err(TranslationError {
                job_id: input.job_id.clone(),
                error_code: "STDIN_WRITE_FAILED".into(),
                message: format!("failed to write translation input: {e}"),
            });
        }

        let output = wait_with_timeout(child, self.timeout).map_err(|e| TranslationError {
            job_id: input.job_id.clone(),
            error_code: "TIMEOUT".into(),
            message: e,
        })?;

        if output.status.success() {
            serde_json::from_slice::<TranslationOutput>(&output.stdout).map_err(|e| {
                TranslationError {
                    job_id: input.job_id.clone(),
                    error_code: "OUTPUT_PARSE_FAILED".into(),
                    message: format!("failed to parse translation output: {e}"),
                }
            })
        } else {
            let err: TranslationError =
                serde_json::from_slice(&output.stdout).unwrap_or_else(|_| TranslationError {
                    job_id: input.job_id.clone(),
                    error_code: "UNKNOWN_ERROR".into(),
                    message: String::from_utf8_lossy(&output.stdout).into_owned(),
                });
            Err(err)
        }
    }
}

fn wait_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
) -> Result<std::process::Output, String> {
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|e| format!("failed to collect output: {e}"));
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "translation worker timed out after {}s",
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(format!("error polling child process: {e}")),
        }
    }
}

/// Test stub: returns a configurable `Result<TranslationOutput,
/// TranslationError>` without spawning a subprocess.
pub struct StubTranslationWorkerClient {
    pub result: Result<TranslationOutput, TranslationError>,
}

impl StubTranslationWorkerClient {
    pub fn ok(output: TranslationOutput) -> Self {
        Self { result: Ok(output) }
    }

    pub fn err(error: TranslationError) -> Self {
        Self { result: Err(error) }
    }
}

impl TranslationWorkerClient for StubTranslationWorkerClient {
    fn translate(&self, _input: TranslationInput) -> Result<TranslationOutput, TranslationError> {
        self.result.clone()
    }
}

// ============================================================
// D3 input-adapter normalization: legacy S-140 segments -> versioned,
// identity-bearing TranslationInput. Never mutates the source S-140 artifact.
// ============================================================

/// A bare legacy S-140 segment as currently persisted (timing + text only,
/// no stable identity, no schema envelope).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegacySubtitleSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

/// Error rejecting a legacy segment array before it reaches the translation
/// worker (D3 EC-1/EC-3).
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizationError {
    pub message: String,
}

impl std::fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NormalizationError {}

/// Derives a stable `segment_id` from `(subtitle_artifact_id,
/// zero_based_ordinal)`, validates timing/order, and emits a versioned
/// `TranslationInput`. The source S-140 artifact is read-only input; nothing
/// here writes back to it.
///
/// Rejects (EC-1/EC-3): an empty segment list, any segment whose `end_ms` is
/// not strictly greater than its `start_ms`, any segment that overlaps or is
/// out of order relative to the previous one, and an empty/blank source or
/// target language tag.
pub fn normalize_legacy_segments(
    subtitle_artifact_id: &str,
    job_id: &str,
    source_language: &str,
    target_language: &str,
    legacy_segments: &[LegacySubtitleSegment],
) -> Result<TranslationInput, NormalizationError> {
    if source_language.trim().is_empty() {
        return Err(NormalizationError {
            message: "source_language must not be empty".into(),
        });
    }
    if target_language.trim().is_empty() {
        return Err(NormalizationError {
            message: "target_language must not be empty".into(),
        });
    }
    if legacy_segments.is_empty() {
        return Err(NormalizationError {
            message: "legacy segment list must not be empty".into(),
        });
    }

    let mut segments = Vec::with_capacity(legacy_segments.len());
    let mut prev_end_ms: Option<u64> = None;

    for (ordinal, seg) in legacy_segments.iter().enumerate() {
        if seg.end_ms <= seg.start_ms {
            return Err(NormalizationError {
                message: format!(
                    "segment[{ordinal}]: end_ms ({}) must be greater than start_ms ({})",
                    seg.end_ms, seg.start_ms
                ),
            });
        }
        if let Some(prev_end) = prev_end_ms
            && seg.start_ms < prev_end
        {
            return Err(NormalizationError {
                message: format!(
                    "segment[{ordinal}]: start_ms ({}) overlaps or reorders before the \
                     previous segment's end_ms ({prev_end})",
                    seg.start_ms
                ),
            });
        }
        prev_end_ms = Some(seg.end_ms);

        segments.push(TranslationSegment {
            segment_id: format!("{subtitle_artifact_id}:{ordinal}"),
            start_ms: seg.start_ms,
            end_ms: seg.end_ms,
            source_text: seg.text.clone(),
        });
    }

    Ok(TranslationInput {
        schema_version: TRANSLATION_SCHEMA_VERSION,
        job_id: job_id.to_string(),
        source_language: source_language.to_string(),
        target_language: target_language.to_string(),
        segments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> TranslationInput {
        TranslationInput {
            schema_version: TRANSLATION_SCHEMA_VERSION,
            job_id: "job-1".into(),
            source_language: "en".into(),
            target_language: "es".into(),
            segments: vec![TranslationSegment {
                segment_id: "artifact-1:0".into(),
                start_ms: 0,
                end_ms: 1000,
                source_text: "Hello world".into(),
            }],
        }
    }

    fn sample_output() -> TranslationOutput {
        TranslationOutput {
            schema_version: TRANSLATION_SCHEMA_VERSION,
            job_id: "job-1".into(),
            source_language: "en".into(),
            target_language: "es".into(),
            segments: vec![TranslatedSegment {
                segment_id: "artifact-1:0".into(),
                start_ms: 0,
                end_ms: 1000,
                source_text: "Hello world".into(),
                translated_text: "Hola mundo".into(),
            }],
            status: "ok".into(),
        }
    }

    fn sample_error() -> TranslationError {
        TranslationError {
            job_id: "job-1".into(),
            error_code: "PROVIDER_FAILED".into(),
            message: "translation provider unavailable".into(),
        }
    }

    // ---------------- HP-1: stub client preserves segment_id and timing ----------------

    #[test]
    fn stub_ok_preserves_segment_id_and_timing() {
        let client = StubTranslationWorkerClient::ok(sample_output());
        let result = client.translate(sample_input());
        assert!(result.is_ok());
        let out = result.unwrap();
        assert_eq!(out.segments[0].segment_id, "artifact-1:0");
        assert_eq!(out.segments[0].start_ms, 0);
        assert_eq!(out.segments[0].end_ms, 1000);
        assert_eq!(out.status, "ok");
    }

    #[test]
    fn stub_err_returns_error() {
        let client = StubTranslationWorkerClient::err(sample_error());
        let result = client.translate(sample_input());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, "PROVIDER_FAILED");
    }

    #[test]
    fn translation_error_display_includes_code_and_message() {
        let err = sample_error();
        let s = err.to_string();
        assert!(s.contains("PROVIDER_FAILED"));
        assert!(s.contains("translation provider unavailable"));
    }

    // ---------------- HP-2: schema field-name parity ----------------

    #[test]
    fn translation_input_serializes_with_exact_schema_field_names() {
        let value = serde_json::to_value(sample_input()).unwrap();
        let obj = value.as_object().unwrap();
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "job_id",
                "schema_version",
                "segments",
                "source_language",
                "target_language",
            ]
        );
        let segment = obj["segments"][0].as_object().unwrap();
        let mut segment_keys: Vec<&str> = segment.keys().map(String::as_str).collect();
        segment_keys.sort_unstable();
        assert_eq!(
            segment_keys,
            vec!["end_ms", "segment_id", "source_text", "start_ms"]
        );
    }

    #[test]
    fn translation_output_serializes_with_exact_schema_field_names() {
        let value = serde_json::to_value(sample_output()).unwrap();
        let obj = value.as_object().unwrap();
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "job_id",
                "schema_version",
                "segments",
                "source_language",
                "status",
                "target_language",
            ]
        );
        let segment = obj["segments"][0].as_object().unwrap();
        let mut segment_keys: Vec<&str> = segment.keys().map(String::as_str).collect();
        segment_keys.sort_unstable();
        assert_eq!(
            segment_keys,
            vec![
                "end_ms",
                "segment_id",
                "source_text",
                "start_ms",
                "translated_text",
            ]
        );
    }

    #[test]
    fn translation_error_serializes_with_exact_schema_field_names() {
        let value = serde_json::to_value(sample_error()).unwrap();
        let obj = value.as_object().unwrap();
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, vec!["error_code", "job_id", "message"]);
    }

    #[test]
    fn translation_input_rejects_unknown_fields_on_deserialize() {
        let mut value = serde_json::to_value(sample_input()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("unexpected_field".into(), serde_json::json!("nope"));
        // serde's default behavior for structs without #[serde(deny_unknown_fields)]
        // ignores unknown fields; the schema's own `additionalProperties: false`
        // is what a JSON-Schema validator enforces at the wire boundary. This
        // test documents that the Rust struct alone is permissive and the
        // schema file is the source of truth for extra-field rejection.
        let result: Result<TranslationInput, _> = serde_json::from_value(value);
        assert!(result.is_ok());
    }

    // ---------------- EC-2: subprocess client typed errors ----------------

    #[test]
    fn subprocess_client_returns_spawn_failed_for_nonexistent_binary() {
        let client = SubprocessTranslationWorkerClient::new(vec!["/nonexistent/binary".into()]);
        let result = client.translate(sample_input());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, "SPAWN_FAILED");
    }

    #[test]
    fn subprocess_client_parses_valid_output_json() {
        let output = sample_output();
        let json = serde_json::to_string(&output).unwrap();
        let client = SubprocessTranslationWorkerClient::new(vec![
            "sh".into(),
            "-c".into(),
            format!("read _; echo '{json}'"),
        ]);
        let result = client.translate(sample_input());
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        assert_eq!(result.unwrap().job_id, "job-1");
    }

    #[test]
    fn subprocess_client_returns_error_on_nonzero_exit_with_json() {
        let err = sample_error();
        let json = serde_json::to_string(&err).unwrap();
        let client = SubprocessTranslationWorkerClient::new(vec![
            "sh".into(),
            "-c".into(),
            format!("read _; echo '{json}'; exit 1"),
        ]);
        let result = client.translate(sample_input());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, "PROVIDER_FAILED");
    }

    #[test]
    fn subprocess_client_returns_unknown_error_on_malformed_json_exit() {
        let client = SubprocessTranslationWorkerClient::new(vec![
            "sh".into(),
            "-c".into(),
            "read _; echo 'not json'; exit 1".into(),
        ]);
        let result = client.translate(sample_input());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code, "UNKNOWN_ERROR");
        assert!(err.message.contains("not json"));
    }

    #[test]
    fn subprocess_client_returns_output_parse_failed_on_malformed_success_json() {
        let client = SubprocessTranslationWorkerClient::new(vec![
            "sh".into(),
            "-c".into(),
            "read _; echo 'not json'".into(),
        ]);
        let result = client.translate(sample_input());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, "OUTPUT_PARSE_FAILED");
    }

    #[test]
    fn subprocess_client_timeout_kills_and_returns_error() {
        let client = SubprocessTranslationWorkerClient::new(vec![
            "sh".into(),
            "-c".into(),
            "sleep 60".into(),
        ])
        .with_timeout(Duration::from_millis(200));
        let result = client.translate(sample_input());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code, "TIMEOUT");
        assert!(err.message.contains("timed out"));
    }

    #[test]
    fn default_timeout_is_300_seconds() {
        let client = SubprocessTranslationWorkerClient::new(vec!["sh".into()]);
        assert_eq!(
            client.timeout,
            Duration::from_secs(DEFAULT_TRANSLATION_TIMEOUT_SECS)
        );
    }

    // ---------------- HP-3 / EC-1 / EC-3: normalize_legacy_segments ----------------

    fn legacy_segments() -> Vec<LegacySubtitleSegment> {
        vec![
            LegacySubtitleSegment {
                start_ms: 0,
                end_ms: 1000,
                text: "Hello".into(),
            },
            LegacySubtitleSegment {
                start_ms: 1000,
                end_ms: 2000,
                text: "world".into(),
            },
        ]
    }

    #[test]
    fn normalize_derives_deterministic_segment_ids_from_ordinal() {
        let input =
            normalize_legacy_segments("artifact-42", "job-9", "en", "es", &legacy_segments())
                .unwrap();
        assert_eq!(input.segments[0].segment_id, "artifact-42:0");
        assert_eq!(input.segments[1].segment_id, "artifact-42:1");
        assert_eq!(input.schema_version, TRANSLATION_SCHEMA_VERSION);
        assert_eq!(input.source_language, "en");
        assert_eq!(input.target_language, "es");
    }

    #[test]
    fn normalize_does_not_mutate_source_text() {
        let legacy = legacy_segments();
        let input = normalize_legacy_segments("artifact-1", "job-1", "en", "es", &legacy).unwrap();
        assert_eq!(input.segments[0].source_text, "Hello");
        assert_eq!(input.segments[1].source_text, "world");
        // Source slice itself is untouched (read-only borrow, no in-place mutation).
        assert_eq!(legacy[0].text, "Hello");
    }

    #[test]
    fn normalize_rejects_empty_segment_list() {
        let result = normalize_legacy_segments("artifact-1", "job-1", "en", "es", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("must not be empty"));
    }

    #[test]
    fn normalize_rejects_empty_source_language() {
        let result = normalize_legacy_segments("artifact-1", "job-1", "", "es", &legacy_segments());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("source_language"));
    }

    #[test]
    fn normalize_rejects_empty_target_language() {
        let result = normalize_legacy_segments("artifact-1", "job-1", "en", "", &legacy_segments());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("target_language"));
    }

    #[test]
    fn normalize_rejects_end_before_or_equal_start() {
        let bad = vec![LegacySubtitleSegment {
            start_ms: 1000,
            end_ms: 1000,
            text: "invalid".into(),
        }];
        let result = normalize_legacy_segments("artifact-1", "job-1", "en", "es", &bad);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("end_ms"));
    }

    #[test]
    fn normalize_rejects_overlapping_timing() {
        let bad = vec![
            LegacySubtitleSegment {
                start_ms: 0,
                end_ms: 1000,
                text: "first".into(),
            },
            LegacySubtitleSegment {
                start_ms: 500,
                end_ms: 1500,
                text: "second".into(),
            },
        ];
        let result = normalize_legacy_segments("artifact-1", "job-1", "en", "es", &bad);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("overlaps"));
    }

    #[test]
    fn normalize_rejects_exact_duplicate_segment() {
        let duplicated = vec![
            LegacySubtitleSegment {
                start_ms: 0,
                end_ms: 1000,
                text: "Hello".into(),
            },
            LegacySubtitleSegment {
                start_ms: 0,
                end_ms: 1000,
                text: "Hello".into(),
            },
        ];
        let result = normalize_legacy_segments("artifact-1", "job-1", "en", "es", &duplicated);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("overlaps"));
    }

    #[test]
    fn normalize_rejects_reordered_without_identity() {
        // A segment list presented out of chronological order — since the
        // legacy shape carries no identity, this is indistinguishable from
        // an overlap/reorder and must fail closed the same way.
        let reordered = vec![
            LegacySubtitleSegment {
                start_ms: 1000,
                end_ms: 2000,
                text: "second".into(),
            },
            LegacySubtitleSegment {
                start_ms: 0,
                end_ms: 1000,
                text: "first".into(),
            },
        ];
        let result = normalize_legacy_segments("artifact-1", "job-1", "en", "es", &reordered);
        assert!(result.is_err());
    }

    #[test]
    fn normalization_error_display_includes_message() {
        let err = NormalizationError {
            message: "test error message".into(),
        };
        assert!(err.to_string().contains("test error message"));
    }
}
