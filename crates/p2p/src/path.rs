use std::fmt;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    AbsolutePath,
    Backslash,
    EmptySegment,
    DotSegment,
    ParentSegment,
    Empty,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::AbsolutePath => write!(f, "absolute path is not allowed"),
            PathError::Backslash => write!(f, "backslash is not allowed"),
            PathError::EmptySegment => write!(f, "empty segment is not allowed"),
            PathError::DotSegment => write!(f, "dot segment is not allowed"),
            PathError::ParentSegment => write!(f, "parent segment is not allowed"),
            PathError::Empty => write!(f, "empty path is not allowed"),
        }
    }
}

impl std::error::Error for PathError {}

pub fn normalize_path(input: &str) -> Result<String, PathError> {
    let normalized: String = input.nfc().collect();

    // 1. Reject empty input entirely
    if normalized.is_empty() {
        return Err(PathError::Empty);
    }

    // 2. Reject if the string contains any `\` character anywhere
    if normalized.contains('\\') {
        return Err(PathError::Backslash);
    }

    // 3. Reject if the string starts with `/`
    if normalized.starts_with('/') {
        return Err(PathError::AbsolutePath);
    }

    // 4. Split the (already NFC-normalized) string on `/` and reject:
    for segment in normalized.split('/') {
        if segment.is_empty() {
            return Err(PathError::EmptySegment);
        }
        if segment == "." {
            return Err(PathError::DotSegment);
        }
        if segment == ".." {
            return Err(PathError::ParentSegment);
        }
    }

    Ok(normalized)
}

pub fn sort_paths(paths: &mut [String]) {
    paths.sort();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_m3u8() {
        let result = normalize_path("index.m3u8");
        assert_eq!(result, Ok("index.m3u8".to_string()));
    }

    #[test]
    fn test_segments_000001_ts() {
        let result = normalize_path("segments/000001.ts");
        assert_eq!(result, Ok("segments/000001.ts".to_string()));
    }

    #[test]
    fn test_captions_cafe_nfd() {
        // NFD: e + combining acute U+0301
        let input = "captions/cafe\u{0301}.vtt";
        let result = normalize_path(input);
        assert_eq!(result, Ok("captions/café.vtt".to_string()));
    }

    #[test]
    fn test_parent_segment() {
        let result = normalize_path("../secret.key");
        assert_eq!(result, Err(PathError::ParentSegment));
    }

    #[test]
    fn test_absolute_path() {
        let result = normalize_path("/absolute/index.m3u8");
        assert_eq!(result, Err(PathError::AbsolutePath));
    }

    #[test]
    fn test_backslash() {
        let result = normalize_path("segments\\000001.ts");
        assert_eq!(result, Err(PathError::Backslash));
    }

    #[test]
    fn test_sort_paths() {
        let mut paths = vec![
            "b.txt".to_string(),
            "a.txt".to_string(),
            "segments/1.ts".to_string(),
        ];
        sort_paths(&mut paths);
        assert_eq!(
            paths,
            vec![
                "a.txt".to_string(),
                "b.txt".to_string(),
                "segments/1.ts".to_string(),
            ]
        );
    }
}
