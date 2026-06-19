// S-120-T2: integration tests for preparation repo — real DB, no mocks.
use std::env;

use dubbridge_db::{artifact_repo, preparation_repo};
use dubbridge_domain::{
    artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact, PreparationStatus},
    asset::AssetId,
};
use dubbridge_media::{canonical_ffprobe_json, validate_hls_outputs};
use dubbridge_storage::{
    LocalFsAdapter, StorageAdapter, hls_manifest_key, hls_segment_key, probe_metadata_key,
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tempfile::TempDir;
use uuid::Uuid;

async fn setup_pool() -> Option<PgPool> {
    let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("migrations");
    Some(pool)
}

async fn insert_asset(pool: &PgPool) -> AssetId {
    let asset_id = AssetId::new();
    sqlx::query(
        "INSERT INTO assets (id, uploader_subject_id, ingestion_status, created_at) VALUES ($1, $2, $3, now())"
    )
    .bind(asset_id.0)
    .bind(Uuid::new_v4())
    .bind("finalized")
    .execute(pool)
    .await
    .expect("insert asset");
    asset_id
}

async fn insert_source_artifact(pool: &PgPool, asset_id: AssetId) -> ArtifactRecord {
    let record = ArtifactRecord::new_original(
        asset_id,
        Uuid::new_v4(),
        format!("ingest/{}/source.mp4", asset_id),
        "video/mp4".into(),
        1_000_000,
        "deadbeef".into(),
    );
    artifact_repo::insert_artifact_record(pool, &record)
        .await
        .expect("insert source artifact");
    record
}

fn checksum_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn local_storage() -> (LocalFsAdapter, TempDir) {
    let dir = TempDir::new().expect("temp dir");
    (LocalFsAdapter::new(dir.path()), dir)
}

// HP-1: insert a derived artifact and list it back.
#[tokio::test]
async fn insert_and_list_derived_artifact() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;

    let derived = DerivedArtifact::new(
        asset_id,
        source.id,
        ArtifactKind::ProbeMetadata,
        format!("prepared/{}/probe.json", asset_id),
        "application/json".into(),
        512,
        "probesum".into(),
    );

    preparation_repo::insert_derived_artifact(&pool, &derived)
        .await
        .expect("insert derived artifact");

    let list = preparation_repo::list_derived_artifacts(&pool, asset_id)
        .await
        .expect("list derived artifacts");

    assert_eq!(list.len(), 1);
    let got = &list[0];
    assert_eq!(got.id, derived.id);
    assert_eq!(got.parent_artifact_id, source.id);
    assert_eq!(got.kind, ArtifactKind::ProbeMetadata);
    assert_eq!(got.storage_key, derived.storage_key);
    assert_eq!(got.checksum, "probesum");
}

// HP-2: multiple derived artifacts for one asset are all returned.
#[tokio::test]
async fn list_returns_all_derived_artifacts_for_asset() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;

    for (kind, key) in [
        (ArtifactKind::ProbeMetadata, "probe.json"),
        (ArtifactKind::HlsManifest, "index.m3u8"),
        (ArtifactKind::HlsSegment, "segment000.ts"),
    ] {
        let da = DerivedArtifact::new(
            asset_id,
            source.id,
            kind,
            format!("prepared/{}/{}", asset_id, key),
            "application/octet-stream".into(),
            100,
            format!("chk-{}", key),
        );
        preparation_repo::insert_derived_artifact(&pool, &da)
            .await
            .expect("insert");
    }

    let list = preparation_repo::list_derived_artifacts(&pool, asset_id)
        .await
        .expect("list");

    assert_eq!(list.len(), 3);
    let kinds: Vec<_> = list.iter().map(|a| &a.kind).collect();
    assert!(kinds.contains(&&ArtifactKind::ProbeMetadata));
    assert!(kinds.contains(&&ArtifactKind::HlsManifest));
    assert!(kinds.contains(&&ArtifactKind::HlsSegment));
}

// HP-3: list_derived_artifacts returns empty vec when no derived artifacts exist.
#[tokio::test]
async fn list_derived_artifacts_returns_empty_for_unknown_asset() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let unknown_asset = AssetId::new();
    let list = preparation_repo::list_derived_artifacts(&pool, unknown_asset)
        .await
        .expect("list");

    assert!(list.is_empty());
}

// HP-4: upsert then get preparation status round-trips correctly.
#[tokio::test]
async fn upsert_and_get_preparation_status() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    preparation_repo::upsert_preparation_status(&pool, asset_id, PreparationStatus::Pending, None)
        .await
        .expect("upsert pending");

    let record = preparation_repo::get_preparation_status(&pool, asset_id)
        .await
        .expect("get status")
        .expect("should be Some");

    assert_eq!(record.asset_id, asset_id);
    assert_eq!(record.status, PreparationStatus::Pending);
    assert!(record.error_detail.is_none());
}

// HP-5: upsert transitions status from Pending → InProgress → Ready.
#[tokio::test]
async fn preparation_status_transitions_succeed() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    for status in [
        PreparationStatus::Pending,
        PreparationStatus::InProgress,
        PreparationStatus::Ready,
    ] {
        preparation_repo::upsert_preparation_status(&pool, asset_id, status.clone(), None)
            .await
            .expect("upsert");

        let got = preparation_repo::get_preparation_status(&pool, asset_id)
            .await
            .expect("get")
            .expect("Some");

        assert_eq!(got.status, status);
    }
}

// EC-1: failed status persists error_detail.
#[tokio::test]
async fn failed_status_persists_error_detail() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    preparation_repo::upsert_preparation_status(
        &pool,
        asset_id,
        PreparationStatus::Failed,
        Some("ffprobe exited with code 1"),
    )
    .await
    .expect("upsert failed");

    let record = preparation_repo::get_preparation_status(&pool, asset_id)
        .await
        .expect("get")
        .expect("Some");

    assert_eq!(record.status, PreparationStatus::Failed);
    assert_eq!(
        record.error_detail.as_deref(),
        Some("ffprobe exited with code 1")
    );
}

// EC-2: source artifact's own list entry is not returned as a derived artifact.
#[tokio::test]
async fn source_artifact_not_included_in_derived_list() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let _source = insert_source_artifact(&pool, asset_id).await;

    let list = preparation_repo::list_derived_artifacts(&pool, asset_id)
        .await
        .expect("list");

    // source artifact has parent_artifact_id = NULL — must not appear.
    assert!(list.is_empty());
}

// EC-3: get_preparation_status returns None for asset with no status row.
#[tokio::test]
async fn get_preparation_status_returns_none_when_not_initialised() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    let result = preparation_repo::get_preparation_status(&pool, asset_id)
        .await
        .expect("get");

    assert!(result.is_none());
}

// HP-6: canonical probe metadata persists as a derived artifact linked to the source artifact.
#[tokio::test]
async fn insert_probe_metadata_artifact_links_to_source() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    let probe_bytes = canonical_ffprobe_json(
        r#"{
          "streams":[
            {"codec_type":"video","codec_name":"h264"},
            {"codec_type":"audio","codec_name":"aac"}
          ],
          "format":{"format_name":"mp4","duration":"6.200000"}
        }"#,
    )
    .expect("canonical probe bytes");
    let checksum = checksum_hex(&probe_bytes);
    let (storage, _dir) = local_storage();
    let storage_key = probe_metadata_key(&asset_id.to_string());

    storage
        .put(&storage_key, probe_bytes.clone())
        .await
        .expect("persist probe bytes");

    preparation_repo::upsert_preparation_status(
        &pool,
        asset_id,
        PreparationStatus::InProgress,
        None,
    )
    .await
    .expect("upsert in progress");

    let inserted = preparation_repo::insert_probe_metadata_artifact(
        &pool,
        asset_id,
        &storage_key,
        probe_bytes.len() as i64,
        &checksum,
    )
    .await
    .expect("insert probe metadata artifact");

    assert_eq!(inserted.parent_artifact_id, source.id);
    assert_eq!(inserted.kind, ArtifactKind::ProbeMetadata);
    assert_eq!(inserted.content_type, "application/json");
    assert_eq!(inserted.size_bytes, probe_bytes.len() as i64);
    assert_eq!(inserted.checksum, checksum);
    assert_eq!(
        storage.get(&storage_key).await.expect("stored probe bytes"),
        probe_bytes
    );

    let listed = preparation_repo::list_derived_artifacts(&pool, asset_id)
        .await
        .expect("list derived artifacts");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, inserted.id);

    let status = preparation_repo::get_preparation_status(&pool, asset_id)
        .await
        .expect("get status")
        .expect("status row");
    assert_eq!(status.status, PreparationStatus::InProgress);
}

// EC-4: missing source artifact prevents probe persistence and preserves fail-closed status handling.
#[tokio::test]
async fn insert_probe_metadata_artifact_requires_source_artifact() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;

    let err = preparation_repo::insert_probe_metadata_artifact(
        &pool,
        asset_id,
        "prepared/asset/probe.json",
        123,
        "deadbeef",
    )
    .await
    .expect_err("missing source artifact must fail");

    assert!(matches!(err, dubbridge_db::error::DbError::NotFound));
}

// HP-7: HLS manifest and segments persist through storage + derived-artifact lineage.
#[tokio::test]
// End-to-end persistence test with sequential setup + assertions; splitting it
// would fragment a single lineage scenario.
#[allow(clippy::too_many_lines)]
async fn insert_hls_artifacts_persists_manifest_and_segments() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    let (storage, _dir) = local_storage();

    let manifest = b"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:6
#EXT-X-PLAYLIST-TYPE:VOD
#EXTINF:6.0,
segment_00000.ts
#EXTINF:6.0,
segment_00001.ts
#EXT-X-ENDLIST
"
    .to_vec();
    let segment_names = ["segment_00000.ts", "segment_00001.ts"];
    validate_hls_outputs(
        std::str::from_utf8(&manifest).expect("utf8 manifest"),
        &segment_names,
    )
    .expect("valid HLS outputs");

    let manifest_key = hls_manifest_key(&asset_id.to_string());
    storage
        .put(&manifest_key, manifest.clone())
        .await
        .expect("persist manifest");

    let mut segment_metadata = Vec::new();
    for (index, name) in segment_names.iter().enumerate() {
        let bytes = format!("segment-{index}-bytes").into_bytes();
        let key = hls_segment_key(&asset_id.to_string(), name);
        storage
            .put(&key, bytes.clone())
            .await
            .expect("persist segment");
        segment_metadata.push((key, bytes.len() as i64, checksum_hex(&bytes)));
    }

    preparation_repo::upsert_preparation_status(
        &pool,
        asset_id,
        PreparationStatus::InProgress,
        None,
    )
    .await
    .expect("upsert in progress");

    let (manifest_artifact, segment_artifacts) = preparation_repo::insert_hls_artifacts(
        &pool,
        asset_id,
        &manifest_key,
        manifest.len() as i64,
        &checksum_hex(&manifest),
        &segment_metadata,
    )
    .await
    .expect("insert hls artifacts");

    assert_eq!(manifest_artifact.parent_artifact_id, source.id);
    assert_eq!(manifest_artifact.kind, ArtifactKind::HlsManifest);
    assert_eq!(segment_artifacts.len(), 2);
    assert!(
        segment_artifacts
            .iter()
            .all(|artifact| artifact.parent_artifact_id == source.id)
    );
    assert!(
        segment_artifacts
            .iter()
            .all(|artifact| artifact.kind == ArtifactKind::HlsSegment)
    );
    assert_eq!(
        storage.get(&manifest_key).await.expect("stored manifest"),
        manifest
    );
    assert_eq!(
        storage
            .get(&segment_metadata[0].0)
            .await
            .expect("stored segment"),
        b"segment-0-bytes".to_vec()
    );

    let listed = preparation_repo::list_derived_artifacts(&pool, asset_id)
        .await
        .expect("list derived artifacts");
    assert_eq!(listed.len(), 3);
    assert_eq!(
        listed
            .iter()
            .filter(|artifact| artifact.kind == ArtifactKind::HlsManifest)
            .count(),
        1
    );
    assert_eq!(
        listed
            .iter()
            .filter(|artifact| artifact.kind == ArtifactKind::HlsSegment)
            .count(),
        2
    );

    let status = preparation_repo::get_preparation_status(&pool, asset_id)
        .await
        .expect("get status")
        .expect("status row");
    assert_eq!(status.status, PreparationStatus::InProgress);
}

// EC-5: malformed HLS output fails closed and leaves the asset without derived HLS artifacts.
#[tokio::test]
async fn malformed_hls_output_does_not_persist_artifacts() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let _source = insert_source_artifact(&pool, asset_id).await;
    let invalid_manifest = "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-ENDLIST\n";

    let err = validate_hls_outputs(invalid_manifest, &["segment_00000.ts"]).unwrap_err();
    assert!(err.to_string().contains("no segment references"));

    preparation_repo::upsert_preparation_status(
        &pool,
        asset_id,
        PreparationStatus::Failed,
        Some("HLS manifest contains no segment references"),
    )
    .await
    .expect("upsert failed");

    let listed = preparation_repo::list_derived_artifacts(&pool, asset_id)
        .await
        .expect("list derived artifacts");
    assert!(listed.is_empty());

    let status = preparation_repo::get_preparation_status(&pool, asset_id)
        .await
        .expect("get status")
        .expect("status row");
    assert_eq!(status.status, PreparationStatus::Failed);
    assert_eq!(
        status.error_detail.as_deref(),
        Some("HLS manifest contains no segment references")
    );
}

// HP-8: readiness evidence is complete only when probe + manifest + segment artifacts exist.
#[tokio::test]
async fn preparation_readiness_evidence_is_ready_when_required_artifacts_exist() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let _source = insert_source_artifact(&pool, asset_id).await;
    let probe_bytes = canonical_ffprobe_json(
        r#"{
          "streams":[{"codec_type":"video","codec_name":"h264"}],
          "format":{"format_name":"mp4","duration":"4.000000"}
        }"#,
    )
    .expect("canonical probe bytes");

    preparation_repo::insert_probe_metadata_artifact(
        &pool,
        asset_id,
        &probe_metadata_key(&asset_id.to_string()),
        probe_bytes.len() as i64,
        &checksum_hex(&probe_bytes),
    )
    .await
    .expect("insert probe");

    let segments = vec![(
        hls_segment_key(&asset_id.to_string(), "segment_00000.ts"),
        5,
        "segchk".to_string(),
    )];
    preparation_repo::insert_hls_artifacts(
        &pool,
        asset_id,
        &hls_manifest_key(&asset_id.to_string()),
        42,
        "manifestchk",
        &segments,
    )
    .await
    .expect("insert hls");

    let evidence = preparation_repo::get_preparation_readiness_evidence(&pool, asset_id)
        .await
        .expect("readiness evidence");

    assert_eq!(evidence.probe_metadata_count, 1);
    assert_eq!(evidence.hls_manifest_count, 1);
    assert_eq!(evidence.hls_segment_count, 1);
    assert!(evidence.is_ready());
}

// EC-6: readiness evidence stays incomplete when required derived artifacts are missing.
#[tokio::test]
async fn preparation_readiness_evidence_is_incomplete_without_hls() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let _source = insert_source_artifact(&pool, asset_id).await;
    let probe_bytes = canonical_ffprobe_json(
        r#"{
          "streams":[{"codec_type":"audio","codec_name":"aac"}],
          "format":{"format_name":"mp4","duration":"2.000000"}
        }"#,
    )
    .expect("canonical probe bytes");

    preparation_repo::insert_probe_metadata_artifact(
        &pool,
        asset_id,
        &probe_metadata_key(&asset_id.to_string()),
        probe_bytes.len() as i64,
        &checksum_hex(&probe_bytes),
    )
    .await
    .expect("insert probe");

    let evidence = preparation_repo::get_preparation_readiness_evidence(&pool, asset_id)
        .await
        .expect("readiness evidence");

    assert_eq!(evidence.probe_metadata_count, 1);
    assert_eq!(evidence.hls_manifest_count, 0);
    assert_eq!(evidence.hls_segment_count, 0);
    assert!(!evidence.is_ready());
}
