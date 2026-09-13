//! Behavioral tests for `crates/p2p/src/package_writer.rs::materialize`.
//!
//! Covers the acceptance criteria frozen in the T3c-S1b task packet:
//! HP-1 (fresh materialize), HP-2 (idempotent replay), EC-1 (containment
//! rejection), EC-2 (conflict rejection), EC-3 (IO failure propagation).

use dubbridge_p2p::package_builder::{PackageFileInput, build_package};
use dubbridge_p2p::package_writer::{MaterializeError, materialize};
use std::fs;
use std::path::PathBuf;

fn unique_root(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("p2p_package_writer_test-{label}-{pid}-{nanos}"));
    fs::create_dir_all(&dir).expect("failed to create test root");
    dir
}

fn sample_inputs() -> Vec<PackageFileInput> {
    vec![
        PackageFileInput {
            path: "index.m3u8".to_string(),
            plaintext: b"#EXTM3U".to_vec(),
        },
        PackageFileInput {
            path: "segments/000001.ts".to_string(),
            plaintext: b"segment-one-bytes".to_vec(),
        },
    ]
}

/// HP-1: materializing a valid multi-file SealedPackage under a fresh
/// publication_id creates the expected directory with correct manifest and
/// per-file ciphertext bytes, and returns a package_ref.
#[test]
fn hp1_fresh_materialize_creates_expected_directory() {
    let root = unique_root("hp1");
    let ck = [0u8; 32];
    let inputs = sample_inputs();
    let package = build_package("asset1", "pub-hp1", "line1", &ck, &inputs).expect("build_package");

    let package_ref = materialize(&root, &package).expect("materialize should succeed");
    assert_eq!(package_ref.publication_id, "pub-hp1");

    let manifest_bytes =
        fs::read(package_ref.root.join("manifest.json")).expect("manifest.json should exist");
    assert_eq!(manifest_bytes, package.manifest_canonical_json.as_bytes());

    for file in &package.files {
        let on_disk = fs::read(package_ref.root.join(&file.path)).expect("file should exist");
        assert_eq!(&on_disk, &file.ciphertext);
    }

    let _ = fs::remove_dir_all(&root);
}

/// HP-2: materializing the identical SealedPackage a second time (same
/// publication_id, byte-identical content) is a no-op success (idempotent
/// replay), not an error and not a rewrite (mtime/inode unchanged).
#[test]
fn hp2_idempotent_replay_is_noop_and_does_not_rewrite() {
    let root = unique_root("hp2");
    let ck = [0u8; 32];
    let inputs = sample_inputs();
    let package = build_package("asset1", "pub-hp2", "line1", &ck, &inputs).expect("build_package");

    let first = materialize(&root, &package).expect("first materialize should succeed");
    let manifest_path = first.root.join("manifest.json");
    let mtime_before = fs::metadata(&manifest_path)
        .expect("manifest metadata")
        .modified()
        .expect("mtime");

    #[cfg(unix)]
    let inode_before = {
        use std::os::unix::fs::MetadataExt;
        fs::metadata(&manifest_path)
            .expect("manifest metadata")
            .ino()
    };

    let second = materialize(&root, &package).expect("idempotent replay should succeed");
    assert_eq!(second.publication_id, first.publication_id);

    let mtime_after = fs::metadata(&manifest_path)
        .expect("manifest metadata")
        .modified()
        .expect("mtime");
    assert_eq!(
        mtime_before, mtime_after,
        "mtime must not change on idempotent replay"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let inode_after = fs::metadata(&manifest_path)
            .expect("manifest metadata")
            .ino();
        assert_eq!(
            inode_before, inode_after,
            "inode must not change on idempotent replay"
        );
    }

    let _ = fs::remove_dir_all(&root);
}

/// EC-1: a SealedPackage whose publication_id is engineered to escape the
/// shared root via `../` is rejected with the containment error before any
/// write occurs.
#[test]
fn ec1_publication_id_traversal_is_rejected_before_any_write() {
    let root = unique_root("ec1");
    let ck = [0u8; 32];
    let inputs = sample_inputs();
    let package =
        build_package("asset1", "../escape", "line1", &ck, &inputs).expect("build_package");

    let result = materialize(&root, &package);
    assert!(matches!(result, Err(MaterializeError::Containment(_))));

    let entries: Vec<_> = fs::read_dir(&root)
        .expect("root should still exist")
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        entries.is_empty(),
        "no write should occur on containment rejection"
    );

    let _ = fs::remove_dir_all(&root);
}

/// EC-1b: a publication_id that resolves through a symlink escaping the
/// shared root is rejected with the containment error.
#[cfg(unix)]
#[test]
fn ec1b_symlink_escape_publication_id_is_rejected() {
    let root = unique_root("ec1b");
    let outside = unique_root("ec1b-outside");

    let link_path = root.join("escape-link");
    std::os::unix::fs::symlink(&outside, &link_path).expect("symlink");

    let ck = [0u8; 32];
    let inputs = sample_inputs();
    let package =
        build_package("asset1", "escape-link", "line1", &ck, &inputs).expect("build_package");

    let result = materialize(&root, &package);
    assert!(matches!(result, Err(MaterializeError::Containment(_))));

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
}

/// EC-2: materializing a SealedPackage for a publication_id that already has
/// a directory with DIFFERENT content is rejected as a conflict, and the
/// pre-existing directory is left byte-for-byte unchanged.
#[test]
fn ec2_conflicting_content_is_rejected_and_existing_left_unchanged() {
    let root = unique_root("ec2");
    let ck = [0u8; 32];
    let inputs = sample_inputs();
    let package_v1 =
        build_package("asset1", "pub-ec2", "line1", &ck, &inputs).expect("build_package v1");
    materialize(&root, &package_v1).expect("first materialize should succeed");

    let manifest_path = root.join("pub-ec2").join("manifest.json");
    let manifest_before = fs::read(&manifest_path).expect("manifest exists");

    let different_inputs = vec![
        PackageFileInput {
            path: "index.m3u8".to_string(),
            plaintext: b"#EXTM3U-DIFFERENT".to_vec(),
        },
        PackageFileInput {
            path: "segments/000001.ts".to_string(),
            plaintext: b"different-segment-bytes".to_vec(),
        },
    ];
    let package_v2 = build_package("asset1", "pub-ec2", "line1", &ck, &different_inputs)
        .expect("build_package v2");

    let result = materialize(&root, &package_v2);
    assert!(matches!(result, Err(MaterializeError::Conflict)));

    let manifest_after = fs::read(&manifest_path).expect("manifest still exists");
    assert_eq!(
        manifest_before, manifest_after,
        "existing directory must be untouched on conflict"
    );

    let _ = fs::remove_dir_all(&root);
}

/// EC-3: an underlying IO failure during materialization propagates as an
/// IO error rather than panicking or silently succeeding.
#[cfg(unix)]
#[test]
fn ec3_io_failure_propagates_as_io_error() {
    let root = unique_root("ec3");
    let ck = [0u8; 32];
    let inputs = sample_inputs();
    let package = build_package("asset1", "pub-ec3", "line1", &ck, &inputs).expect("build_package");

    // Pre-create the package directory as read-only so the atomic write's
    // temp-file creation inside it fails with a permission error.
    let package_dir = root.join("pub-ec3");
    fs::create_dir_all(&package_dir).expect("create package dir");
    let mut perms = fs::metadata(&package_dir).expect("metadata").permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o555);
    fs::set_permissions(&package_dir, perms).expect("set readonly");

    let result = materialize(&root, &package);
    assert!(matches!(result, Err(MaterializeError::Io(_))));

    // Restore permissions so the temp directory can be cleaned up.
    let mut perms = fs::metadata(&package_dir).expect("metadata").permissions();
    perms.set_mode(0o755);
    let _ = fs::set_permissions(&package_dir, perms);

    let _ = fs::remove_dir_all(&root);
}
