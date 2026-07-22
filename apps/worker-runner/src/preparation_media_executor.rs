use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::{Context, bail};
use async_trait::async_trait;
use dubbridge_media::{
    HLS_MANIFEST_FILE_NAME, HLS_SEGMENT_FILE_EXTENSION, canonical_ffprobe_json, ffmpeg_hls_command,
    ffprobe_command, validate_hls_outputs,
};
use tempfile::TempDir;
use tokio::{fs, process::Command};

use crate::preparation_runtime::{HlsPackageOutput, HlsSegmentOutput, PreparationExecutor};

pub(crate) struct SubprocessPreparationExecutor;

#[async_trait]
impl PreparationExecutor for SubprocessPreparationExecutor {
    async fn extract_probe_metadata(&self, source_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
        let (_workspace, input_path) = write_source_workspace(source_bytes).await?;
        let command = ffprobe_command(path_to_string(&input_path)?);
        let output = run_command(&command).await?;
        let stdout =
            String::from_utf8(output.stdout).context("ffprobe stdout is not valid UTF-8")?;
        canonical_ffprobe_json(&stdout)
    }

    async fn transcode_hls(&self, source_bytes: &[u8]) -> anyhow::Result<HlsPackageOutput> {
        let (workspace, input_path) = write_source_workspace(source_bytes).await?;
        let output_dir = workspace.path().join("hls");
        fs::create_dir_all(&output_dir)
            .await
            .context("failed to create HLS output directory")?;

        let command =
            ffmpeg_hls_command(path_to_string(&input_path)?, path_to_string(&output_dir)?);
        run_command(&command).await?;

        let manifest_path = output_dir.join(HLS_MANIFEST_FILE_NAME);
        let manifest_bytes = fs::read(&manifest_path).await.with_context(|| {
            format!("failed to read HLS manifest at {}", manifest_path.display())
        })?;
        let manifest_raw =
            std::str::from_utf8(&manifest_bytes).context("HLS manifest is not valid UTF-8")?;
        let segments = read_hls_segments(&output_dir).await?;
        let segment_names = segments
            .iter()
            .map(|segment| segment.file_name.as_str())
            .collect::<Vec<_>>();
        validate_hls_outputs(manifest_raw, &segment_names)?;

        Ok(HlsPackageOutput {
            manifest_bytes,
            segments,
        })
    }
}

async fn write_source_workspace(source_bytes: &[u8]) -> anyhow::Result<(TempDir, PathBuf)> {
    let workspace = TempDir::new().context("failed to create temporary preparation workspace")?;
    let input_path = workspace.path().join("source-media.bin");
    fs::write(&input_path, source_bytes)
        .await
        .with_context(|| {
            format!(
                "failed to write temporary source file at {}",
                input_path.display()
            )
        })?;
    Ok((workspace, input_path))
}

async fn run_command(command: &[String]) -> anyhow::Result<std::process::Output> {
    let binary = command
        .first()
        .ok_or_else(|| anyhow::anyhow!("empty command"))?;
    let output = Command::new(binary)
        .args(&command[1..])
        .stdin(Stdio::null())
        .output()
        .await
        .with_context(|| format!("failed to spawn command '{}'", binary))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "command '{}' failed with status {}: {}",
            binary,
            output.status,
            stderr.trim()
        );
    }

    Ok(output)
}

async fn read_hls_segments(output_dir: &Path) -> anyhow::Result<Vec<HlsSegmentOutput>> {
    let mut entries = fs::read_dir(output_dir).await.with_context(|| {
        format!(
            "failed to read HLS output directory {}",
            output_dir.display()
        )
    })?;
    let mut segment_paths = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .context("failed while iterating HLS output directory")?
    {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| format!(".{ext}") == HLS_SEGMENT_FILE_EXTENSION)
        {
            segment_paths.push(path);
        }
    }
    segment_paths.sort();

    let mut segments = Vec::with_capacity(segment_paths.len());
    for path in segment_paths {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "HLS segment path '{}' has no valid file name",
                    path.display()
                )
            })?;
        let bytes = fs::read(&path)
            .await
            .with_context(|| format!("failed to read HLS segment at {}", path.display()))?;
        segments.push(HlsSegmentOutput { file_name, bytes });
    }

    Ok(segments)
}

fn path_to_string(path: &Path) -> anyhow::Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF-8 path '{}'", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_command_returns_output_on_success() {
        let command = vec!["echo".to_string(), "hello".to_string()];

        let output = run_command(&command).await.expect("command should succeed");

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    }

    #[tokio::test]
    async fn run_command_fails_closed_on_nonzero_exit() {
        let command = vec!["sh".to_string(), "-c".to_string(), "exit 7".to_string()];

        let result = run_command(&command).await;

        let err = result.expect_err("nonzero exit should be an error");
        assert!(err.to_string().contains("failed with status"));
    }

    #[tokio::test]
    async fn run_command_includes_stderr_in_error_message() {
        let command = vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo boom 1>&2; exit 1".to_string(),
        ];

        let err = run_command(&command)
            .await
            .expect_err("nonzero exit should be an error");

        assert!(err.to_string().contains("boom"));
    }

    #[tokio::test]
    async fn run_command_rejects_empty_command() {
        let command: Vec<String> = Vec::new();

        let err = run_command(&command)
            .await
            .expect_err("empty command should be rejected");

        assert!(err.to_string().contains("empty command"));
    }

    #[tokio::test]
    async fn run_command_errors_when_binary_is_missing() {
        let command = vec!["dubbridge-definitely-not-a-real-binary".to_string()];

        let result = run_command(&command).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn read_hls_segments_filters_by_extension_and_sorts_by_name() {
        let workspace = TempDir::new().expect("failed to create temp dir");
        let dir = workspace.path();

        fs::write(dir.join("segment2.ts"), b"second")
            .await
            .expect("write segment2");
        fs::write(dir.join("segment1.ts"), b"first")
            .await
            .expect("write segment1");
        fs::write(dir.join("manifest.m3u8"), b"not a segment")
            .await
            .expect("write manifest");

        let segments = read_hls_segments(dir)
            .await
            .expect("reading segments should succeed");

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].file_name, "segment1.ts");
        assert_eq!(segments[0].bytes, b"first");
        assert_eq!(segments[1].file_name, "segment2.ts");
        assert_eq!(segments[1].bytes, b"second");
    }

    #[tokio::test]
    async fn read_hls_segments_ignores_subdirectories() {
        let workspace = TempDir::new().expect("failed to create temp dir");
        let dir = workspace.path();

        fs::create_dir(dir.join("nested.ts"))
            .await
            .expect("create dir with .ts-like name");
        fs::write(dir.join("segment.ts"), b"payload")
            .await
            .expect("write segment");

        let segments = read_hls_segments(dir)
            .await
            .expect("reading segments should succeed");

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].file_name, "segment.ts");
    }

    #[tokio::test]
    async fn read_hls_segments_returns_empty_vec_for_empty_directory() {
        let workspace = TempDir::new().expect("failed to create temp dir");

        let segments = read_hls_segments(workspace.path())
            .await
            .expect("reading segments should succeed");

        assert!(segments.is_empty());
    }

    #[tokio::test]
    async fn read_hls_segments_fails_closed_on_missing_directory() {
        let workspace = TempDir::new().expect("failed to create temp dir");
        let missing = workspace.path().join("does-not-exist");

        let result = read_hls_segments(&missing).await;

        assert!(result.is_err());
    }

    #[test]
    fn path_to_string_returns_str_for_utf8_path() {
        let path = PathBuf::from("/tmp/some/valid/path.txt");

        let value = path_to_string(&path).expect("valid UTF-8 path should succeed");

        assert_eq!(value, "/tmp/some/valid/path.txt");
    }

    #[cfg(unix)]
    #[test]
    fn path_to_string_fails_closed_on_non_utf8_path() {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

        let invalid = OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f]);
        let path = PathBuf::from(invalid);

        let err = path_to_string(&path).expect_err("non-UTF-8 path should be rejected");

        assert!(err.to_string().contains("non-UTF-8 path"));
    }

    #[tokio::test]
    async fn write_source_workspace_writes_bytes_to_temp_file() {
        let source_bytes = b"source media payload";

        let (workspace, input_path) = write_source_workspace(source_bytes)
            .await
            .expect("writing workspace should succeed");

        assert!(input_path.starts_with(workspace.path()));
        let written = fs::read(&input_path).await.expect("read written file");
        assert_eq!(written, source_bytes);
    }
}
