use crate::path;
use dubbridge_domain::artifact::{ArtifactKind, DerivedArtifact};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFile {
    pub path: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSnapshot {
    pub asset_id: String,
    pub files: Vec<SnapshotFile>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SnapshotError {
    MissingManifest,
    MultipleManifests,
    NoSegments,
    InvalidPath(crate::path::PathError),
}

impl From<crate::path::PathError> for SnapshotError {
    fn from(err: crate::path::PathError) -> Self {
        SnapshotError::InvalidPath(err)
    }
}

pub fn build_snapshot(
    asset_id: &dubbridge_domain::asset::AssetId,
    artifacts: &[dubbridge_domain::artifact::DerivedArtifact],
) -> Result<PackageSnapshot, SnapshotError> {
    // 1. Filter artifacts to only HlsManifest or HlsSegment
    let filtered: Vec<&DerivedArtifact> = artifacts
        .iter()
        .filter(|a| matches!(a.kind, ArtifactKind::HlsManifest | ArtifactKind::HlsSegment))
        .collect();

    // 2. Count manifests
    let manifest_count = filtered
        .iter()
        .filter(|a| matches!(a.kind, ArtifactKind::HlsManifest))
        .count();

    if manifest_count == 0 {
        return Err(SnapshotError::MissingManifest);
    }
    if manifest_count > 1 {
        return Err(SnapshotError::MultipleManifests);
    }

    // 3. Count segments
    let segment_count = filtered
        .iter()
        .filter(|a| matches!(a.kind, ArtifactKind::HlsSegment))
        .count();

    if segment_count == 0 {
        return Err(SnapshotError::NoSegments);
    }

    // 4. Normalize paths for all filtered rows
    let mut manifest_file: Option<SnapshotFile> = None;
    let mut segment_files: Vec<SnapshotFile> = Vec::new();

    for artifact in &filtered {
        let normalized_path = path::normalize_path(&artifact.storage_key)?;
        let file = SnapshotFile {
            path: normalized_path,
            size_bytes: artifact.size_bytes as u64,
            checksum: artifact.checksum.clone(),
        };

        if matches!(artifact.kind, ArtifactKind::HlsManifest) {
            manifest_file = Some(file);
        } else {
            segment_files.push(file);
        }
    }

    // 5. Sort segment files by path
    // We need to sort the segment_files by their path field.
    // Since sort_paths takes &mut [String], we can extract paths, sort them,
    // and then reorder segment_files accordingly, or just sort segment_files directly.
    // The requirement says "sort a Vec<String> of paths in parallel with the segment data...
    // the point is the FINAL segment order in files must match what path::sort_paths would produce on their paths".
    // Easiest way: sort segment_files by path.
    segment_files.sort_by(|a, b| a.path.cmp(&b.path));

    // 6. Build final files list: manifest first, then sorted segments
    let mut files = Vec::new();
    if let Some(mf) = manifest_file {
        files.push(mf);
    }
    files.extend(segment_files);

    // 7. Return Ok
    Ok(PackageSnapshot {
        asset_id: asset_id.to_string(),
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_domain::asset::AssetId;
    use time::OffsetDateTime;
    use uuid::Uuid;

    fn make_artifact(
        kind: ArtifactKind,
        storage_key: &str,
        size_bytes: i64,
        checksum: &str,
    ) -> DerivedArtifact {
        DerivedArtifact {
            id: Uuid::new_v4(),
            asset_id: AssetId(Uuid::new_v4()),
            parent_artifact_id: Uuid::new_v4(),
            kind,
            storage_key: storage_key.to_string(),
            content_type: "application/octet-stream".to_string(),
            size_bytes,
            checksum: checksum.to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn hp_t2b_1_one_manifest_and_segments_produces_ordered_snapshot() {
        let asset_id = AssetId(Uuid::new_v4());
        let manifest = make_artifact(ArtifactKind::HlsManifest, "index.m3u8", 100, "chk_m");
        let seg3 = make_artifact(ArtifactKind::HlsSegment, "segments/000003.ts", 100, "chk_3");
        let seg1 = make_artifact(ArtifactKind::HlsSegment, "segments/000001.ts", 100, "chk_1");
        let seg2 = make_artifact(ArtifactKind::HlsSegment, "segments/000002.ts", 100, "chk_2");

        let artifacts = vec![manifest, seg3, seg1, seg2];
        let snapshot = build_snapshot(&asset_id, &artifacts).unwrap();

        assert_eq!(snapshot.files.len(), 4);
        assert_eq!(snapshot.files[0].path, "index.m3u8");
        assert_eq!(snapshot.files[1].path, "segments/000001.ts");
        assert_eq!(snapshot.files[2].path, "segments/000002.ts");
        assert_eq!(snapshot.files[3].path, "segments/000003.ts");
    }

    #[test]
    fn hp_t2b_2_unrelated_artifact_kinds_are_excluded() {
        let asset_id = AssetId(Uuid::new_v4());
        let manifest = make_artifact(ArtifactKind::HlsManifest, "index.m3u8", 100, "chk_m");
        let seg1 = make_artifact(ArtifactKind::HlsSegment, "segments/000001.ts", 100, "chk_1");
        let seg2 = make_artifact(ArtifactKind::HlsSegment, "segments/000002.ts", 100, "chk_2");
        let seg3 = make_artifact(ArtifactKind::HlsSegment, "segments/000003.ts", 100, "chk_3");
        let probe = make_artifact(ArtifactKind::ProbeMetadata, "probe.json", 50, "chk_p");

        let artifacts = vec![manifest, seg1, seg2, seg3, probe];
        let snapshot = build_snapshot(&asset_id, &artifacts).unwrap();

        assert_eq!(snapshot.files.len(), 4);
        for file in &snapshot.files {
            assert_ne!(file.path, "probe.json");
        }
    }

    #[test]
    fn ec_t2b_1_missing_manifest_is_rejected() {
        let asset_id = AssetId(Uuid::new_v4());
        let seg1 = make_artifact(ArtifactKind::HlsSegment, "segments/000001.ts", 100, "chk_1");
        let artifacts = vec![seg1];
        let result = build_snapshot(&asset_id, &artifacts);
        assert_eq!(result, Err(SnapshotError::MissingManifest));
    }

    #[test]
    fn ec_t2b_1b_multiple_manifests_are_rejected() {
        let asset_id = AssetId(Uuid::new_v4());
        let m1 = make_artifact(ArtifactKind::HlsManifest, "index.m3u8", 100, "chk_m1");
        let m2 = make_artifact(ArtifactKind::HlsManifest, "index2.m3u8", 100, "chk_m2");
        let seg1 = make_artifact(ArtifactKind::HlsSegment, "segments/000001.ts", 100, "chk_1");
        let artifacts = vec![m1, m2, seg1];
        let result = build_snapshot(&asset_id, &artifacts);
        assert_eq!(result, Err(SnapshotError::MultipleManifests));
    }

    #[test]
    fn ec_t2b_2_no_segments_is_rejected() {
        let asset_id = AssetId(Uuid::new_v4());
        let m1 = make_artifact(ArtifactKind::HlsManifest, "index.m3u8", 100, "chk_m");
        let artifacts = vec![m1];
        let result = build_snapshot(&asset_id, &artifacts);
        assert_eq!(result, Err(SnapshotError::NoSegments));
    }

    #[test]
    fn ec_t2b_3_invalid_path_is_rejected() {
        let asset_id = AssetId(Uuid::new_v4());
        let m1 = make_artifact(ArtifactKind::HlsManifest, "index.m3u8", 100, "chk_m");
        let bad_seg = make_artifact(ArtifactKind::HlsSegment, "../escape.ts", 100, "chk_bad");
        let artifacts = vec![m1, bad_seg];
        let result = build_snapshot(&asset_id, &artifacts);
        assert!(matches!(result, Err(SnapshotError::InvalidPath(_))));
    }
}
