//! Generic atomic file write primitive.
//!
//! This module provides a filesystem-level atomic write operation that is
//! independent of any P2P-specific semantics. It is intended to be composed
//! with higher-level path-safety and package-writing logic, but contains no
//! knowledge of packages, manifests, or P2P protocols.

use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// A process-wide monotonic counter used to generate unique temp-file suffixes.
///
/// Combined with the process ID and a nanosecond timestamp, this ensures that
/// concurrent calls for different targets do not collide, and that a leftover
/// temp file from a crashed previous attempt does not collide with a fresh one.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Atomically write `contents` to `target`.
///
/// The write is performed by:
/// 1. Creating a temporary file in the **same directory** as `target` (so the
///    final rename is guaranteed atomic on the same filesystem/volume).
/// 2. Writing `contents` in full to the temp file.
/// 3. Flushing and `fsync`-ing the temp file via `File::sync_all()` so the
///    data is durable before it becomes visible under the target name.
/// 4. Renaming the temp file to `target` using `std::fs::rename` (atomic
///    replace on POSIX; atomic on the same filesystem on Windows).
///
/// If any step fails, the leftover temp file is removed (best-effort; cleanup
/// errors are ignored) and the original `std::io::Error` from the failing step
/// is returned. The target path is only ever created by the final rename, so a
/// partially-written file is never visible at `target` on failure.
///
/// # Errors
///
/// Returns an `std::io::Error` if any of the underlying filesystem operations
/// (create, write, sync, rename) fail.
pub fn write_atomic(target: &Path, contents: &[u8]) -> io::Result<()> {
    // Derive the parent directory of the target.
    let parent = target.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    // Derive the target's file name for the temp-file base.
    let target_name = target
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target path has no file name"))?
        .to_string_lossy()
        .into_owned();

    // Generate a unique suffix: process id + monotonic counter + nanosecond
    // timestamp. This is std-only and avoids any external dependency.
    let pid = std::process::id();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let suffix = format!("{pid}-{counter}-{nanos}");

    // Build the temp file path in the same directory as the target.
    let temp_path = parent.join(format!(".{target_name}.tmp.{suffix}"));

    // Create the temp file.
    let mut file = match std::fs::File::create(&temp_path) {
        Ok(f) => f,
        Err(e) => {
            // Best-effort cleanup (the file may not exist, but ignore errors).
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }
    };

    // Write the contents in full.
    if let Err(e) = file.write_all(contents) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }

    // Flush and fsync the temp file so the data is durable before rename.
    if let Err(e) = file.sync_all() {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }

    // Rename the temp file to the target (atomic replace on the same
    // filesystem).
    if let Err(e) = std::fs::rename(&temp_path, target) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Create a unique per-test subdirectory under `std::env::temp_dir()`.
    fn unique_test_dir() -> PathBuf {
        let pid = std::process::id();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("atomic_write_test-{pid}-{counter}-{nanos}"));
        fs::create_dir_all(&dir).expect("failed to create test directory");
        dir
    }

    /// HP-1: writing to a target that does not yet exist creates it with the
    /// exact given contents.
    #[test]
    fn hp1_create_new_target() {
        let dir = unique_test_dir();
        let target = dir.join("new_file.txt");
        let contents = b"hello atomic world";

        assert!(!target.exists(), "target should not exist before write");

        write_atomic(&target, contents).expect("write_atomic failed");

        assert!(target.exists(), "target should exist after write");
        let read_back = fs::read(&target).expect("failed to read target");
        assert_eq!(read_back, contents, "contents do not match");

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    /// HP-2: writing to a target that already exists with different content
    /// fully replaces its content (no leftover bytes from the old version).
    #[test]
    fn hp2_replace_existing_target() {
        let dir = unique_test_dir();
        let target = dir.join("existing_file.txt");
        let old_contents = b"old data that is longer than new";
        let new_contents = b"new";

        // Create the target with old content.
        fs::write(&target, old_contents).expect("failed to create target");
        assert_eq!(fs::read(&target).unwrap(), old_contents);

        // Atomically replace with new (shorter) content.
        write_atomic(&target, new_contents).expect("write_atomic failed");

        let read_back = fs::read(&target).expect("failed to read target");
        assert_eq!(
            read_back, new_contents,
            "target was not fully replaced; leftover bytes detected"
        );

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    /// EC-1: after a successful write, no stray temp file (matching the
    /// temp-name pattern) remains in the directory.
    #[test]
    fn ec1_no_stray_temp_files() {
        let dir = unique_test_dir();
        let target = dir.join("clean_file.txt");
        let contents = b"clean contents";

        write_atomic(&target, contents).expect("write_atomic failed");

        // List all entries in the directory and verify no temp files remain.
        let entries: Vec<_> = fs::read_dir(&dir)
            .expect("failed to read directory")
            .filter_map(|e| e.ok())
            .collect();

        let target_name = target
            .file_name()
            .expect("target has no file name")
            .to_string_lossy()
            .into_owned();

        for entry in &entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            // The temp-file pattern is: .{target_name}.tmp.{suffix}
            let is_temp = name.starts_with(&format!(".{target_name}.tmp."));
            assert!(!is_temp, "stray temp file found: {name}");
        }

        // Verify the target itself exists with correct content.
        assert!(target.exists(), "target should exist after write");
        assert_eq!(fs::read(&target).expect("failed to read target"), contents);

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }
}
