//! Materializes a [`SealedPackage`] to a shared ciphertext filesystem root.
//!
//! This module composes three already-delivered primitives —
//! [`crate::path::verify_contained_realpath`] (containment/traversal
//! rejection), [`crate::atomic_write::write_atomic`] (crash-safe single-file
//! writes), and [`crate::package_builder::SealedPackage`] (trusted,
//! already-encrypted package content) — into one directory-materialization
//! operation. It does not perform network IO, database writes, or any
//! `P2P_READY` state transition; those belong to later leaves.

use std::path::{Path, PathBuf};

use crate::package_builder::SealedPackage;
use crate::path::{PathError, verify_contained_realpath};

const MANIFEST_FILE_NAME: &str = "manifest.json";

/// Identifies the on-disk location of a materialized package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRef {
    pub publication_id: String,
    pub root: PathBuf,
}

#[derive(Debug)]
pub enum MaterializeError {
    /// The publication_id or a file path would escape the shared root.
    Containment(PathError),
    /// An existing directory for this publication_id has different content
    /// than the package being written now.
    Conflict,
    /// An underlying filesystem operation failed.
    Io(std::io::Error),
}

impl std::fmt::Display for MaterializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaterializeError::Containment(e) => write!(f, "containment check failed: {e}"),
            MaterializeError::Conflict => write!(
                f,
                "an existing package directory has different content for this publication_id"
            ),
            MaterializeError::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for MaterializeError {}

impl From<std::io::Error> for MaterializeError {
    fn from(e: std::io::Error) -> Self {
        MaterializeError::Io(e)
    }
}

/// Materialize `package` under `root` as a directory named by its
/// `publication_id`, atomically and idempotently.
///
/// - If no directory exists yet for this `publication_id`, it is created and
///   every file (manifest + ciphertext files) is written atomically.
/// - If a directory already exists with byte-identical content, this is a
///   successful no-op.
/// - If a directory already exists with *different* content, this returns
///   [`MaterializeError::Conflict`] and the existing directory is left
///   untouched.
pub fn materialize(root: &Path, package: &SealedPackage) -> Result<PackageRef, MaterializeError> {
    let publication_id = &package.manifest.publication_id;

    let package_dir =
        verify_contained_realpath(root, publication_id).map_err(MaterializeError::Containment)?;

    if package_dir.is_dir() {
        match diff_existing(&package_dir, package) {
            ExistingState::Identical => {
                return Ok(PackageRef {
                    publication_id: publication_id.clone(),
                    root: package_dir,
                });
            }
            ExistingState::Different => return Err(MaterializeError::Conflict),
            ExistingState::Missing => {
                // Partially-written or corrupt directory from a prior crash:
                // treat as absent content for that slot and proceed to
                // (re-)write below via write_atomic's own replace semantics.
            }
        }
    }

    std::fs::create_dir_all(&package_dir)?;

    // Manifest and ciphertext file paths are joined directly under the
    // already-containment-verified `package_dir`: `MANIFEST_FILE_NAME` is a
    // fixed constant (no traversal surface), and each `file.path` is already
    // `normalize_path`-clean by construction from `build_package` (no `.`,
    // `..`, absolute, or backslash segments) -- re-validating it here would
    // duplicate a check `package_builder` already enforces, which this
    // leaf's scope explicitly excludes.
    let manifest_target = package_dir.join(MANIFEST_FILE_NAME);
    crate::atomic_write::write_atomic(
        &manifest_target,
        package.manifest_canonical_json.as_bytes(),
    )?;

    for file in &package.files {
        let file_target = package_dir.join(&file.path);
        if let Some(parent) = file_target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        crate::atomic_write::write_atomic(&file_target, &file.ciphertext)?;
    }

    Ok(PackageRef {
        publication_id: publication_id.clone(),
        root: package_dir,
    })
}

enum ExistingState {
    Identical,
    Different,
    Missing,
}

/// Compare an existing on-disk package directory against `package`'s
/// expected content, byte-for-byte.
fn diff_existing(package_dir: &Path, package: &SealedPackage) -> ExistingState {
    let manifest_path = package_dir.join(MANIFEST_FILE_NAME);
    match std::fs::read(&manifest_path) {
        Ok(existing_manifest) => {
            if existing_manifest != package.manifest_canonical_json.as_bytes() {
                return ExistingState::Different;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return ExistingState::Missing,
        Err(_) => return ExistingState::Different,
    }

    for file in &package.files {
        let file_path = package_dir.join(&file.path);
        match std::fs::read(&file_path) {
            Ok(existing_bytes) => {
                if existing_bytes != file.ciphertext {
                    return ExistingState::Different;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return ExistingState::Missing,
            Err(_) => return ExistingState::Different,
        }
    }

    ExistingState::Identical
}
