use std::{io::Write, process::Stdio, time::Duration};

use serde::{Deserialize, Serialize};

/// Input passed to the ASR subprocess via stdin (matches workers/asr-worker-py/input.schema.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrInput {
    pub job_id: String,
    pub audio_uri: String,
    pub language_hint: String,
}

/// Successful output returned from the ASR subprocess on stdout (output.schema.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrOutput {
    pub job_id: String,
    pub transcript_uri: String,
    pub alignment_uri: String,
    pub status: String,
}

/// Error envelope returned from the ASR subprocess when processing fails (error.schema.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrError {
    pub job_id: String,
    pub error_code: String,
    pub message: String,
}

impl std::fmt::Display for AsrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.error_code, self.message)
    }
}

impl std::error::Error for AsrError {}

/// Trait that abstracts over ASR worker communication.
pub trait AsrWorkerClient: Send + Sync {
    fn transcribe(&self, input: AsrInput) -> Result<AsrOutput, AsrError>;
}

/// Default subprocess timeout: 300 seconds.
pub const DEFAULT_ASR_TIMEOUT_SECS: u64 = 300;

/// Launches the ASR Python subprocess, sends `AsrInput` as JSON on stdin, and
/// reads `AsrOutput` or `AsrError` from stdout.
pub struct SubprocessAsrWorkerClient {
    pub command: Vec<String>,
    pub timeout: Duration,
}

impl SubprocessAsrWorkerClient {
    pub fn new(command: Vec<String>) -> Self {
        Self {
            command,
            timeout: Duration::from_secs(DEFAULT_ASR_TIMEOUT_SECS),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl AsrWorkerClient for SubprocessAsrWorkerClient {
    fn transcribe(&self, input: AsrInput) -> Result<AsrOutput, AsrError> {
        let binary = self.command.first().cloned().unwrap_or_default();
        let input_json = serde_json::to_vec(&input).expect("AsrInput serialization is infallible");

        let mut child = std::process::Command::new(&binary)
            .args(self.command.get(1..).unwrap_or(&[]))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| AsrError {
                job_id: input.job_id.clone(),
                error_code: "SPAWN_FAILED".into(),
                message: format!("failed to spawn ASR worker '{}': {e}", binary),
            })?;

        if let Some(mut stdin) = child.stdin.take()
            && let Err(e) = stdin.write_all(&input_json)
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err(AsrError {
                job_id: input.job_id.clone(),
                error_code: "STDIN_WRITE_FAILED".into(),
                message: format!("failed to write ASR input: {e}"),
            });
        }

        let output = wait_with_timeout(child, self.timeout).map_err(|e| AsrError {
            job_id: input.job_id.clone(),
            error_code: "TIMEOUT".into(),
            message: e,
        })?;

        if output.status.success() {
            serde_json::from_slice::<AsrOutput>(&output.stdout).map_err(|e| AsrError {
                job_id: input.job_id.clone(),
                error_code: "OUTPUT_PARSE_FAILED".into(),
                message: format!("failed to parse ASR output: {e}"),
            })
        } else {
            let err: AsrError =
                serde_json::from_slice(&output.stdout).unwrap_or_else(|_| AsrError {
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
                    return Err(format!("ASR worker timed out after {}s", timeout.as_secs()));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(format!("error polling child process: {e}")),
        }
    }
}

/// Test stub: returns a configurable `Result<AsrOutput, AsrError>` without spawning a subprocess.
pub struct StubAsrWorkerClient {
    pub result: Result<AsrOutput, AsrError>,
}

impl StubAsrWorkerClient {
    pub fn ok(output: AsrOutput) -> Self {
        Self { result: Ok(output) }
    }

    pub fn err(error: AsrError) -> Self {
        Self { result: Err(error) }
    }
}

impl AsrWorkerClient for StubAsrWorkerClient {
    fn transcribe(&self, _input: AsrInput) -> Result<AsrOutput, AsrError> {
        self.result.clone()
    }
}

// ============================================================
// D1a: Segmentation types (Subtitle generation)
// ============================================================

/// A single word with timing information from the ASR alignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordAlignment {
    pub word: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

/// One subtitle segment to be rendered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubtitleSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

/// Error type for segmentation failures.
#[derive(Debug, Clone)]
pub struct SegmentationError {
    pub message: String,
}

impl std::fmt::Display for SegmentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SegmentationError {}

/// Maximum characters per subtitle line.
pub const MAX_CHARS_PER_LINE: usize = 42;

/// Maximum segment duration in milliseconds.
pub const MAX_SEGMENT_DURATION_MS: u64 = 7000;

/// Trait abstracting over segmentation implementations.
pub trait SegmentationProvider: Send + Sync {
    fn segment(&self, words: &[WordAlignment]) -> Result<Vec<SubtitleSegment>, SegmentationError>;
}

/// Default pure-Rust segmentation provider.
pub struct RustSegmentationProvider;

impl SegmentationProvider for RustSegmentationProvider {
    fn segment(&self, words: &[WordAlignment]) -> Result<Vec<SubtitleSegment>, SegmentationError> {
        if words.is_empty() {
            return Ok(vec![]);
        }

        // Validate timing in order
        let mut prev_end_ms: Option<u64> = None;
        for (i, w) in words.iter().enumerate() {
            if w.end_ms < w.start_ms {
                return Err(SegmentationError {
                    message: format!(
                        "word[{}]: end_ms ({}) < start_ms ({})",
                        i, w.end_ms, w.start_ms
                    ),
                });
            }
            if let Some(prev_end) = prev_end_ms
                && w.start_ms < prev_end
            {
                return Err(SegmentationError {
                    message: format!(
                        "word[{}]: start_ms ({}) overlaps with previous word end_ms ({})",
                        i, w.start_ms, prev_end
                    ),
                });
            }
            prev_end_ms = Some(w.end_ms);
        }

        let mut result: Vec<SubtitleSegment> = vec![];
        let mut seg_start_ms: u64 = 0;
        let mut seg_end_ms: u64 = 0;
        let mut seg_text: String = String::new();

        for w in words {
            let separator_len = if seg_text.is_empty() { 0 } else { 1 };
            let candidate_text_len = seg_text.len() + separator_len + w.word.len();
            let candidate_duration = w.end_ms - seg_start_ms;

            let would_exceed_chars = candidate_text_len > MAX_CHARS_PER_LINE;
            let would_exceed_duration = candidate_duration > MAX_SEGMENT_DURATION_MS;

            if (would_exceed_chars || would_exceed_duration) && !seg_text.is_empty() {
                result.push(SubtitleSegment {
                    start_ms: seg_start_ms,
                    end_ms: seg_end_ms,
                    text: std::mem::take(&mut seg_text),
                });
                seg_start_ms = w.start_ms;
                seg_text = w.word.clone();
            } else if !seg_text.is_empty() {
                seg_text.push(' ');
                seg_text.push_str(&w.word);
            } else {
                seg_start_ms = w.start_ms;
                seg_text.push_str(&w.word);
            }
            seg_end_ms = w.end_ms;
        }

        // Push any remaining segment
        if !seg_text.is_empty() {
            result.push(SubtitleSegment {
                start_ms: seg_start_ms,
                end_ms: seg_end_ms,
                text: seg_text.clone(),
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> AsrInput {
        AsrInput {
            job_id: "job-1".into(),
            audio_uri: "file:///tmp/audio.wav".into(),
            language_hint: "en".into(),
        }
    }

    fn sample_output() -> AsrOutput {
        AsrOutput {
            job_id: "job-1".into(),
            transcript_uri: "file:///tmp/transcript.json".into(),
            alignment_uri: "file:///tmp/alignment.json".into(),
            status: "ok".into(),
        }
    }

    fn sample_error() -> AsrError {
        AsrError {
            job_id: "job-1".into(),
            error_code: "MODEL_LOAD_FAILED".into(),
            message: "whisper model not found".into(),
        }
    }

    #[test]
    fn stub_ok_returns_output() {
        let client = StubAsrWorkerClient::ok(sample_output());
        let result = client.transcribe(sample_input());
        assert!(result.is_ok());
        let out = result.unwrap();
        assert_eq!(out.status, "ok");
        assert_eq!(out.job_id, "job-1");
    }

    #[test]
    fn stub_err_returns_error() {
        let client = StubAsrWorkerClient::err(sample_error());
        let result = client.transcribe(sample_input());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code, "MODEL_LOAD_FAILED");
    }

    #[test]
    fn asr_error_display_includes_code_and_message() {
        let err = sample_error();
        let s = err.to_string();
        assert!(s.contains("MODEL_LOAD_FAILED"));
        assert!(s.contains("whisper model not found"));
    }

    #[test]
    fn subprocess_client_returns_spawn_failed_for_nonexistent_binary() {
        let client = SubprocessAsrWorkerClient::new(vec!["/nonexistent/binary".into()]);
        let result = client.transcribe(sample_input());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code, "SPAWN_FAILED");
    }

    #[test]
    fn subprocess_client_parses_valid_output_json() {
        // Use `echo` to emit a valid AsrOutput JSON to stdout and exit 0.
        let output = sample_output();
        let json = serde_json::to_string(&output).unwrap();
        let client = SubprocessAsrWorkerClient::new(vec![
            "sh".into(),
            "-c".into(),
            format!("read _; echo '{json}'"),
        ]);
        let result = client.transcribe(sample_input());
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        assert_eq!(result.unwrap().job_id, "job-1");
    }

    #[test]
    fn subprocess_client_returns_error_on_nonzero_exit_with_json() {
        let err = sample_error();
        let json = serde_json::to_string(&err).unwrap();
        let client = SubprocessAsrWorkerClient::new(vec![
            "sh".into(),
            "-c".into(),
            format!("read _; echo '{json}'; exit 1"),
        ]);
        let result = client.transcribe(sample_input());
        assert!(result.is_err());
        let e = result.unwrap_err();
        assert_eq!(e.error_code, "MODEL_LOAD_FAILED");
    }

    #[test]
    fn subprocess_client_timeout_kills_and_returns_error() {
        let client =
            SubprocessAsrWorkerClient::new(vec!["sh".into(), "-c".into(), "sleep 60".into()])
                .with_timeout(Duration::from_millis(200));
        let result = client.transcribe(sample_input());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error_code, "TIMEOUT");
        assert!(err.message.contains("timed out"));
    }

    #[test]
    fn default_timeout_is_300_seconds() {
        let client = SubprocessAsrWorkerClient::new(vec!["sh".into()]);
        assert_eq!(
            client.timeout,
            Duration::from_secs(DEFAULT_ASR_TIMEOUT_SECS)
        );
    }

    // ============================================================
    // D1a segmentation tests
    // ============================================================

    #[test]
    fn segment_groups_words_into_ordered_non_overlapping_segments() {
        let provider = RustSegmentationProvider;
        let words = vec![
            WordAlignment {
                word: "Hello".into(),
                start_ms: 0,
                end_ms: 500,
            },
            WordAlignment {
                word: "world".into(),
                start_ms: 500,
                end_ms: 1000,
            },
        ];
        let result = provider.segment(&words).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "Hello world");
        assert_eq!(result[0].start_ms, 0);
        assert_eq!(result[0].end_ms, 1000);
    }

    #[test]
    fn segment_splits_when_max_chars_per_line_exceeded() {
        let provider = RustSegmentationProvider;
        // Words whose combined text exceeds 42 chars
        let long_words: Vec<&str> = vec![
            "short",
            "words",
            "that",
            "together",
            "exceed",
            "the",
            "forty-two",
            "character",
            "limit",
        ];
        let words: Vec<WordAlignment> = long_words
            .iter()
            .enumerate()
            .map(|(i, w)| WordAlignment {
                word: w.to_string(),
                start_ms: (i as u64) * 100,
                end_ms: (i as u64 + 1) * 100,
            })
            .collect();
        let result = provider.segment(&words).unwrap();
        assert!(
            result.len() >= 2,
            "expected 2+ segments but got {}",
            result.len()
        );
        for seg in &result {
            assert!(
                seg.text.len() <= MAX_CHARS_PER_LINE,
                "segment text length {} exceeds MAX_CHARS_PER_LINE",
                seg.text.len()
            );
        }
    }

    #[test]
    fn segment_splits_when_max_duration_exceeded() {
        let provider = RustSegmentationProvider;
        // Words spanning more than 7000ms combined
        let words: Vec<WordAlignment> = (0..15)
            .map(|i| WordAlignment {
                word: format!("w{}", i),
                start_ms: i * 800,
                end_ms: i * 800 + 100,
            })
            .collect();
        let result = provider.segment(&words).unwrap();
        assert!(
            result.len() >= 2,
            "expected 2+ segments but got {}",
            result.len()
        );
        for seg in &result {
            assert!(
                (seg.end_ms - seg.start_ms) <= MAX_SEGMENT_DURATION_MS,
                "segment duration {} exceeds MAX_SEGMENT_DURATION_MS",
                seg.end_ms - seg.start_ms
            );
        }
    }

    #[test]
    fn segment_empty_input_returns_empty_vec() {
        let provider = RustSegmentationProvider;
        let result = provider.segment(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn segment_single_word_exceeds_limits() {
        let provider = RustSegmentationProvider;
        let long_word = "a".repeat(MAX_CHARS_PER_LINE + 10);
        let words = vec![WordAlignment {
            word: long_word.clone(),
            start_ms: 0,
            end_ms: MAX_SEGMENT_DURATION_MS + 5000,
        }];
        let result = provider.segment(&words).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, long_word);
        assert_eq!(result[0].start_ms, 0);
        assert_eq!(result[0].end_ms, MAX_SEGMENT_DURATION_MS + 5000);
    }

    #[test]
    fn segment_fails_closed_on_overlapping_timing() {
        let provider = RustSegmentationProvider;
        let words = vec![
            WordAlignment {
                word: "first".into(),
                start_ms: 0,
                end_ms: 1000,
            },
            WordAlignment {
                word: "second".into(),
                start_ms: 500,
                end_ms: 1500,
            }, // overlaps
        ];
        let result = provider.segment(&words);
        assert!(result.is_err());
    }

    #[test]
    fn segment_fails_closed_on_end_before_start() {
        let provider = RustSegmentationProvider;
        let words = vec![
            WordAlignment {
                word: "invalid".into(),
                start_ms: 1000,
                end_ms: 500,
            }, // end < start
        ];
        let result = provider.segment(&words);
        assert!(result.is_err());
    }

    #[test]
    fn segments_never_overlap() {
        let provider = RustSegmentationProvider;
        // Build a word list that forces multiple segment splits due to both constraints
        let words: Vec<WordAlignment> = (0..20)
            .map(|i| WordAlignment {
                word: format!("word{}", i),
                start_ms: i * 500,
                end_ms: i * 500 + 100,
            })
            .collect();
        let result = provider.segment(&words).unwrap();
        // Check that consecutive segments don't overlap
        for i in 0..result.len().saturating_sub(1) {
            assert!(
                result[i].end_ms <= result[i + 1].start_ms,
                "segment[{}].end_ms ( {}) > segment[{}].start_ms ({})",
                i,
                result[i].end_ms,
                i + 1,
                result[i + 1].start_ms
            );
        }
    }

    #[test]
    fn segmentation_error_display_includes_message() {
        let err = SegmentationError {
            message: "test error message".into(),
        };
        assert!(err.to_string().contains("test error message"));
    }
}
