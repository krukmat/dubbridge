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
    SymlinkEscape,
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
            PathError::SymlinkEscape => write!(f, "path escapes the configured root via a symlink"),
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

pub fn verify_contained_realpath(
    root: &std::path::Path,
    relative: &str,
) -> Result<std::path::PathBuf, PathError> {
    // 1. Canonicalize the root
    let canonical_root = std::fs::canonicalize(root).map_err(|_| PathError::SymlinkEscape)?;

    // 2. Join root and relative
    let candidate = root.join(relative);

    // 3. Walk components, rejecting any intermediate or final symlink
    let components: Vec<&std::path::Path> = relative
        .split('/')
        .filter(|s| !s.is_empty())
        .map(std::path::Path::new)
        .collect();

    let mut current = canonical_root.clone();

    for comp in &components {
        current.push(comp);

        // Use `symlink_metadata` (lstat) unconditionally rather than gating
        // on `current.exists()`: `exists()` follows symlinks to stat their
        // target, so it silently returns `false` for a dangling symlink
        // (one whose target doesn't exist) — which would skip this check
        // entirely and let a dangling symlink pointing outside `root` go
        // undetected. `symlink_metadata` never follows the final link, so
        // it correctly reports `is_symlink() == true` regardless of whether
        // the target exists.
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(PathError::SymlinkEscape);
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // Component genuinely does not exist yet (not a dangling
                // symlink, since that case is handled above) — fine for a
                // to-be-created target file.
            }
            Err(_) => return Err(PathError::SymlinkEscape),
        }
    }

    // 4. Canonicalize the deepest existing ancestor of the final candidate
    // path and confirm it is still inside the canonicalized root. Walk
    // upward from the parent until an existing, non-symlink directory is
    // found — every component between it and `current` was already
    // confirmed in step 3 to either not exist or not be a symlink (dangling
    // or otherwise), so it cannot itself introduce an escape.
    let mut ancestor = current.parent().ok_or(PathError::SymlinkEscape)?;
    loop {
        match std::fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                ancestor = ancestor.parent().ok_or(PathError::SymlinkEscape)?;
            }
            Err(_) => return Err(PathError::SymlinkEscape),
        }
    }
    let canonical_ancestor =
        std::fs::canonicalize(ancestor).map_err(|_| PathError::SymlinkEscape)?;

    if !canonical_ancestor.starts_with(&canonical_root) {
        return Err(PathError::SymlinkEscape);
    }

    // 5. Return the non-canonicalized joined path (the final component may
    // not exist yet for a to-be-created file).
    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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

    #[test]
    fn test_verify_contained_realpath_valid() {
        let root = std::env::temp_dir().join(format!("p2p_test_valid_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        let result = verify_contained_realpath(&root, "subdir/file.txt");
        assert!(result.is_ok());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_verify_contained_realpath_symlink_escape_intermediate() {
        let root =
            std::env::temp_dir().join(format!("p2p_test_intermediate_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        let outside =
            std::env::temp_dir().join(format!("p2p_outside_intermediate_{}", std::process::id()));
        fs::create_dir_all(&outside).unwrap();

        let link_path = root.join("link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link_path).unwrap();

        let result = verify_contained_realpath(&root, "link/file.txt");
        assert!(matches!(result, Err(PathError::SymlinkEscape)));

        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&outside).unwrap();
    }

    #[test]
    fn test_verify_contained_realpath_symlink_escape_final() {
        let root = std::env::temp_dir().join(format!("p2p_test_final_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        let outside =
            std::env::temp_dir().join(format!("p2p_outside_final_{}", std::process::id()));
        fs::create_dir_all(&outside).unwrap();

        let link_path = root.join("file.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link_path).unwrap();

        let result = verify_contained_realpath(&root, "file.txt");
        assert!(matches!(result, Err(PathError::SymlinkEscape)));

        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&outside).unwrap();
    }

    #[test]
    fn test_verify_contained_realpath_dangling_symlink_escape() {
        let root = std::env::temp_dir().join(format!("p2p_test_dangling_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        // A symlink whose target does not exist anywhere ("dangling"),
        // pointing outside root. `Path::exists()` follows symlinks and
        // returns false for a dangling link (since the target can't be
        // stat'd), which previously let this bypass detection entirely.
        let outside_nonexistent =
            std::env::temp_dir().join(format!("p2p_outside_dangling_{}", std::process::id()));
        let link_path = root.join("danglink");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside_nonexistent, &link_path).unwrap();

        let result = verify_contained_realpath(&root, "danglink/file.txt");
        assert!(matches!(result, Err(PathError::SymlinkEscape)));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_verify_contained_realpath_nonexistent_file() {
        let root =
            std::env::temp_dir().join(format!("p2p_test_nonexistent_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        let result = verify_contained_realpath(&root, "new_file.txt");
        assert!(result.is_ok());

        fs::remove_dir_all(&root).unwrap();
    }
}
