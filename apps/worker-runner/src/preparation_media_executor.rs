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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn path_to_string(path: &Path) -> anyhow::Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF-8 path '{}'", path.display()))
}
