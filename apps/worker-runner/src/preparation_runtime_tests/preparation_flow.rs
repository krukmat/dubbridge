use std::sync::Arc;

use dubbridge_db::preparation_repo;
use dubbridge_domain::artifact::{ArtifactKind, PreparationStatus};
use dubbridge_jobs::{InMemoryTranscriptionJobQueue, JobEnvelope, PreparationJob};
use dubbridge_storage::{LocalFsAdapter, StorageAdapter};
use tempfile::TempDir;
use tokio::sync::Mutex;

use crate::preparation_runtime::{HlsPackageOutput, process_preparation_envelope, process_preparation_job};

use super::support::{
    FakePreparationExecutor, assert_status, insert_asset, insert_source_artifact, setup_pool,
    valid_hls_output, valid_probe_bytes,
};

#[tokio::test]
async fn process_preparation_job_marks_ready_when_probe_and_hls_exist() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    let workspace = TempDir::new().expect("temp dir");
    let storage = LocalFsAdapter::new(workspace.path());
    storage
        .put(&source.storage_key, b"source-media-bytes".to_vec())
        .await
        .expect("persist source bytes");
    preparation_repo::upsert_preparation_status(
        &pool,
        asset_id,
        PreparationStatus::Pending,
        None,
    )
    .await
    .expect("set pending");

    let stage_log = Arc::new(Mutex::new(Vec::new()));
    let executor = FakePreparationExecutor {
        pool: pool.clone(),
        asset_id,
        stage_log: stage_log.clone(),
        probe_result: Ok(valid_probe_bytes()),
        hls_result: Ok(valid_hls_output()),
    };

    let queue = InMemoryTranscriptionJobQueue::default();
    process_preparation_job(
        &pool,
        &storage,
        &executor,
        &queue,
        PreparationJob::new(asset_id.0, source.id, source.ingest_token),
    )
    .await
    .expect("process preparation job");

    assert_status(&pool, asset_id, PreparationStatus::Ready).await;
    let readiness = preparation_repo::get_preparation_readiness_evidence(&pool, asset_id)
        .await
        .expect("readiness evidence");
    let derived = preparation_repo::list_derived_artifacts(&pool, asset_id)
        .await
        .expect("derived artifacts");

    assert!(readiness.is_ready());
    assert_eq!(readiness.probe_metadata_count, 1);
    assert_eq!(readiness.hls_manifest_count, 1);
    assert_eq!(readiness.hls_segment_count, 1);
    assert_eq!(derived.len(), 3);
    assert_eq!(stage_log.lock().await.as_slice(), &["probe", "hls"]);
}

#[tokio::test]
async fn process_preparation_job_marks_failed_when_hls_stage_fails() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    let workspace = TempDir::new().expect("temp dir");
    let storage = LocalFsAdapter::new(workspace.path());
    storage
        .put(&source.storage_key, b"source-media-bytes".to_vec())
        .await
        .expect("persist source bytes");
    preparation_repo::upsert_preparation_status(
        &pool,
        asset_id,
        PreparationStatus::Pending,
        None,
    )
    .await
    .expect("set pending");

    let executor = FakePreparationExecutor {
        pool: pool.clone(),
        asset_id,
        stage_log: Arc::new(Mutex::new(Vec::new())),
        probe_result: Ok(valid_probe_bytes()),
        hls_result: Err("ffmpeg transcode failed in fake executor".to_string()),
    };

    let queue = InMemoryTranscriptionJobQueue::default();
    let error = process_preparation_job(
        &pool,
        &storage,
        &executor,
        &queue,
        PreparationJob::new(asset_id.0, source.id, source.ingest_token),
    )
    .await
    .expect_err("HLS failure must fail job");

    assert!(error.to_string().contains("HLS stage failed"));
    let status = preparation_repo::get_preparation_status(&pool, asset_id)
        .await
        .expect("get status")
        .expect("status row");
    let readiness = preparation_repo::get_preparation_readiness_evidence(&pool, asset_id)
        .await
        .expect("readiness evidence");
    let derived = preparation_repo::list_derived_artifacts(&pool, asset_id)
        .await
        .expect("derived artifacts");

    assert_eq!(status.status, PreparationStatus::Failed);
    assert!(
        status
            .error_detail
            .as_deref()
            .expect("error detail")
            .contains("ffmpeg transcode failed in fake executor")
    );
    assert!(!readiness.is_ready());
    assert_eq!(readiness.probe_metadata_count, 1);
    assert_eq!(readiness.hls_manifest_count, 0);
    assert_eq!(readiness.hls_segment_count, 0);
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].kind, ArtifactKind::ProbeMetadata);
}

#[tokio::test]
async fn process_preparation_job_does_not_mark_ready_when_hls_output_is_invalid() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    let workspace = TempDir::new().expect("temp dir");
    let storage = LocalFsAdapter::new(workspace.path());
    storage
        .put(&source.storage_key, b"source-media-bytes".to_vec())
        .await
        .expect("persist source bytes");
    preparation_repo::upsert_preparation_status(
        &pool,
        asset_id,
        PreparationStatus::Pending,
        None,
    )
    .await
    .expect("set pending");

    let executor = FakePreparationExecutor {
        pool: pool.clone(),
        asset_id,
        stage_log: Arc::new(Mutex::new(Vec::new())),
        probe_result: Ok(valid_probe_bytes()),
        hls_result: Ok(HlsPackageOutput {
            manifest_bytes: b"#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-ENDLIST\n".to_vec(),
            segments: Vec::new(),
        }),
    };

    let queue = InMemoryTranscriptionJobQueue::default();
    let error = process_preparation_job(
        &pool,
        &storage,
        &executor,
        &queue,
        PreparationJob::new(asset_id.0, source.id, source.ingest_token),
    )
    .await
    .expect_err("invalid HLS output must fail readiness gate");

    assert!(error.to_string().contains("HLS output validation failed"));
    let status = preparation_repo::get_preparation_status(&pool, asset_id)
        .await
        .expect("get status")
        .expect("status row");
    let readiness = preparation_repo::get_preparation_readiness_evidence(&pool, asset_id)
        .await
        .expect("readiness evidence");

    assert_eq!(status.status, PreparationStatus::Failed);
    assert!(!readiness.is_ready());
    assert_eq!(readiness.probe_metadata_count, 1);
    assert_eq!(readiness.hls_manifest_count, 0);
    assert_eq!(readiness.hls_segment_count, 0);
}

#[tokio::test]
async fn process_preparation_envelope_rejects_wrong_job_type() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    let workspace = TempDir::new().expect("temp dir");
    let storage = LocalFsAdapter::new(workspace.path());
    let executor = FakePreparationExecutor {
        pool: pool.clone(),
        asset_id,
        stage_log: Arc::new(Mutex::new(Vec::new())),
        probe_result: Ok(valid_probe_bytes()),
        hls_result: Ok(valid_hls_output()),
    };

    let queue = InMemoryTranscriptionJobQueue::default();
    let error = process_preparation_envelope(
        &pool,
        &storage,
        &executor,
        &queue,
        JobEnvelope::new(
            "other-job",
            PreparationJob::new(asset_id.0, source.id, source.ingest_token),
        ),
    )
    .await
    .expect_err("unexpected job type must fail");

    assert!(
        error
            .to_string()
            .contains("unsupported preparation job type")
    );
}
