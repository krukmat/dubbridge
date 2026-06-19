use anyhow::{Context, bail};
use serde_json::Value;
use std::collections::BTreeSet;

pub const HLS_MANIFEST_FILE_NAME: &str = "index.m3u8";
pub const HLS_SEGMENT_FILE_PATTERN: &str = "segment_%05d.ts";
pub const HLS_SEGMENT_FILE_EXTENSION: &str = ".ts";

pub fn ffprobe_command(input: &str) -> Vec<String> {
    vec![
        "ffprobe".to_string(),
        "-v".to_string(),
        "error".to_string(),
        "-show_format".to_string(),
        "-show_streams".to_string(),
        "-of".to_string(),
        "json".to_string(),
        input.to_string(),
    ]
}

pub fn parse_ffprobe_output(raw: &str) -> anyhow::Result<Value> {
    let parsed: Value = serde_json::from_str(raw).context("ffprobe output is not valid JSON")?;

    let format = parsed
        .get("format")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("ffprobe output missing format object"))?;
    let format_name = format
        .get("format_name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("ffprobe output missing format.format_name"))?;
    let duration = format
        .get("duration")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("ffprobe output missing format.duration"))?;

    if format_name.is_empty() || duration.is_empty() {
        bail!("ffprobe output contains empty required format fields");
    }

    let streams = parsed
        .get("streams")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("ffprobe output missing streams array"))?;

    if streams.is_empty() {
        bail!("ffprobe output streams array is empty");
    }

    for (index, stream) in streams.iter().enumerate() {
        let codec_type = stream
            .get("codec_type")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow::anyhow!("ffprobe output stream[{index}] missing codec_type"))?;

        if codec_type != "audio" && codec_type != "video" {
            bail!("ffprobe output stream[{index}] has unsupported codec_type '{codec_type}'");
        }
    }

    Ok(parsed)
}

pub fn canonical_ffprobe_json(raw: &str) -> anyhow::Result<Vec<u8>> {
    let parsed = parse_ffprobe_output(raw)?;
    serde_json::to_vec(&parsed).context("failed to serialize canonical ffprobe JSON")
}

pub fn ffmpeg_hls_command(input: &str, output_dir: &str) -> Vec<String> {
    vec![
        "ffmpeg".to_string(),
        "-v".to_string(),
        "error".to_string(),
        "-i".to_string(),
        input.to_string(),
        "-f".to_string(),
        "hls".to_string(),
        "-hls_time".to_string(),
        "6".to_string(),
        "-hls_playlist_type".to_string(),
        "vod".to_string(),
        "-hls_segment_filename".to_string(),
        format!("{output_dir}/{HLS_SEGMENT_FILE_PATTERN}"),
        format!("{output_dir}/{HLS_MANIFEST_FILE_NAME}"),
    ]
}

pub fn validate_hls_outputs(
    manifest_raw: &str,
    segment_files: &[impl AsRef<str>],
) -> anyhow::Result<Vec<String>> {
    if !manifest_raw.lines().any(|line| line.trim() == "#EXTM3U") {
        bail!("HLS manifest missing #EXTM3U header");
    }

    let mut manifest_segments = Vec::new();
    let mut saw_extinf = false;
    for line in manifest_raw.lines().map(str::trim) {
        if line.is_empty() {
            continue;
        }
        if line.starts_with("#EXTINF:") {
            saw_extinf = true;
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        if line.contains('/') || line.contains('\\') {
            bail!("HLS manifest segment reference must be a file name, got '{line}'");
        }
        if !line.ends_with(HLS_SEGMENT_FILE_EXTENSION) {
            bail!("HLS manifest segment reference must end with '.ts', got '{line}'");
        }
        manifest_segments.push(line.to_string());
    }

    if !saw_extinf {
        bail!("HLS manifest missing #EXTINF entries");
    }
    if manifest_segments.is_empty() {
        bail!("HLS manifest contains no segment references");
    }

    let provided_segments = segment_files
        .iter()
        .map(|segment| segment.as_ref().trim())
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();

    if provided_segments.is_empty() {
        bail!("HLS output contains no segment files");
    }

    let manifest_set = manifest_segments.iter().cloned().collect::<BTreeSet<_>>();
    if manifest_set != provided_segments {
        bail!("HLS manifest segment references do not match provided segment files");
    }

    Ok(manifest_segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_probe_json() -> &'static str {
        r#"{
          "streams": [
            {
              "index": 0,
              "codec_name": "h264",
              "codec_type": "video"
            },
            {
              "index": 1,
              "codec_name": "aac",
              "codec_type": "audio"
            }
          ],
          "format": {
            "filename": "clip.mp4",
            "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
            "duration": "12.345000"
          }
        }"#
    }

    fn valid_hls_manifest() -> &'static str {
        r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:6
#EXT-X-PLAYLIST-TYPE:VOD
#EXTINF:6.0,
segment_00000.ts
#EXTINF:6.0,
segment_00001.ts
#EXT-X-ENDLIST
"#
    }

    #[test]
    fn ffprobe_command_starts_with_ffprobe() {
        let cmd = ffprobe_command("/media/file.mp4");
        assert_eq!(cmd[0], "ffprobe");
    }

    #[test]
    fn ffprobe_command_includes_verbosity_flag() {
        let cmd = ffprobe_command("/media/file.mp4");
        let v_pos = cmd.iter().position(|s| s == "-v").expect("-v flag missing");
        assert_eq!(cmd[v_pos + 1], "error", "-v must be followed by 'error'");
    }

    #[test]
    fn ffprobe_command_requests_json_format_and_streams() {
        let cmd = ffprobe_command("/media/file.mp4");
        assert!(cmd.iter().any(|arg| arg == "-show_format"));
        assert!(cmd.iter().any(|arg| arg == "-show_streams"));
        let of_pos = cmd
            .iter()
            .position(|s| s == "-of")
            .expect("-of flag missing");
        assert_eq!(cmd[of_pos + 1], "json");
    }

    #[test]
    fn ffprobe_command_input_is_last_arg() {
        let input = "/some/path/clip.mp4";
        let cmd = ffprobe_command(input);
        assert_eq!(cmd.last().unwrap(), input);
    }

    #[test]
    fn parse_ffprobe_output_accepts_valid_payload() {
        let parsed = parse_ffprobe_output(valid_probe_json()).expect("valid probe output");
        assert_eq!(
            parsed["format"]["format_name"].as_str(),
            Some("mov,mp4,m4a,3gp,3g2,mj2")
        );
        assert_eq!(parsed["streams"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn canonical_ffprobe_json_round_trips_valid_payload() {
        let bytes = canonical_ffprobe_json(valid_probe_json()).expect("canonical bytes");
        let reparsed: Value = serde_json::from_slice(&bytes).expect("reparse canonical JSON");
        assert_eq!(reparsed["format"]["duration"].as_str(), Some("12.345000"));
    }

    #[test]
    fn parse_ffprobe_output_rejects_non_json() {
        let err = parse_ffprobe_output("not-json").unwrap_err();
        assert!(err.to_string().contains("not valid JSON"));
    }

    #[test]
    fn parse_ffprobe_output_rejects_missing_format() {
        let err = parse_ffprobe_output(r#"{"streams":[{"codec_type":"video"}]}"#).unwrap_err();
        assert!(err.to_string().contains("missing format object"));
    }

    #[test]
    fn parse_ffprobe_output_rejects_empty_streams() {
        let err = parse_ffprobe_output(
            r#"{"streams":[],"format":{"format_name":"mp4","duration":"1.0"}}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("streams array is empty"));
    }

    #[test]
    fn parse_ffprobe_output_rejects_streams_without_codec_type() {
        let err = parse_ffprobe_output(
            r#"{"streams":[{"codec_name":"h264"}],"format":{"format_name":"mp4","duration":"1.0"}}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("missing codec_type"));
    }

    #[test]
    fn parse_ffprobe_output_rejects_unsupported_codec_type() {
        let err = parse_ffprobe_output(
            r#"{"streams":[{"codec_type":"subtitle"}],"format":{"format_name":"mp4","duration":"1.0"}}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("unsupported codec_type"));
    }

    #[test]
    fn ffmpeg_hls_command_starts_with_ffmpeg() {
        let cmd = ffmpeg_hls_command("/media/file.mp4", "/tmp/hls");
        assert_eq!(cmd[0], "ffmpeg");
    }

    #[test]
    fn ffmpeg_hls_command_requests_hls_outputs() {
        let cmd = ffmpeg_hls_command("/media/file.mp4", "/tmp/hls");
        assert!(cmd.iter().any(|arg| arg == "hls"));
        assert!(cmd.iter().any(|arg| arg == "-hls_segment_filename"));
        assert_eq!(cmd.last().map(String::as_str), Some("/tmp/hls/index.m3u8"));
        assert!(cmd.iter().any(|arg| arg == "/tmp/hls/segment_%05d.ts"));
    }

    #[test]
    fn validate_hls_outputs_accepts_matching_manifest_and_segments() {
        let segments = validate_hls_outputs(
            valid_hls_manifest(),
            &["segment_00000.ts", "segment_00001.ts"],
        )
        .expect("valid HLS outputs");
        assert_eq!(segments.len(), 2);
    }

    #[test]
    fn validate_hls_outputs_rejects_missing_header() {
        let err = validate_hls_outputs("#EXTINF:6.0,\nsegment_00000.ts\n", &["segment_00000.ts"])
            .unwrap_err();
        assert!(err.to_string().contains("missing #EXTM3U"));
    }

    #[test]
    fn validate_hls_outputs_rejects_empty_segments() {
        let err = validate_hls_outputs(valid_hls_manifest(), &[] as &[&str]).unwrap_err();
        assert!(err.to_string().contains("no segment files"));
    }

    #[test]
    fn validate_hls_outputs_rejects_mismatched_segments() {
        let err = validate_hls_outputs(valid_hls_manifest(), &["segment_00000.ts"]).unwrap_err();
        assert!(
            err.to_string()
                .contains("do not match provided segment files")
        );
    }
}
