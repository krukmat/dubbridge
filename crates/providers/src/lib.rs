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
            format!("echo '{json}'"),
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
}
