pub fn ffprobe_command(input: &str) -> Vec<String> {
    vec![
        "ffprobe".to_string(),
        "-v".to_string(),
        "error".to_string(),
        input.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // T1-T3: verify argv contract so flag regressions are caught immediately
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
    fn ffprobe_command_input_is_last_arg() {
        let input = "/some/path/clip.mp4";
        let cmd = ffprobe_command(input);
        assert_eq!(cmd.last().unwrap(), input);
    }
}
