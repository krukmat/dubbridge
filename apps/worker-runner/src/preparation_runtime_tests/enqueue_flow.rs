use std::sync::Arc;

use dubbridge_db::{preparation_repo, transcription_repo, workspace_repo};
use dubbridge_domain::{
    artifact::{ArtifactRecord, PreparationStatus, TranscriptionStatus},
    asset::AssetId,
    workspace::{OrgId, Organization, Project, ProjectId, TargetLanguage},
};
use dubbridge_jobs::{
    InMemoryTranscriptionJobQueue, PreparationJob, QueueError, TranscriptionJob,
    TranscriptionJobQueue,
};
use dubbridge_storage::{LocalFsAdapter, StorageAdapter};
use sqlx::PgPool;
use tempfile::TempDir;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{preparation_runtime::process_preparation_job, subtitle_enqueue::prepare_transcription_post_ready};

use super::support::{
    FakePreparationExecutor, assert_status, insert_asset, insert_source_artifact, setup_pool,
    valid_hls_output, valid_probe_bytes,
};

async fn insert_project_with_target_language(pool: &PgPool, asset_id: AssetId, source_lang: &str) {
    let org_id = OrgId(Uuid::new_v4());
    let org = Organization {
        id: org_id,
        name: "test-org".into(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };
    workspace_repo::insert_org(pool, &org)
        .await
        .expect("insert org");

    let project_id = ProjectId(Uuid::new_v4());
    let project = Project {
        id: project_id,
        org_id,
        name: "test-project".into(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };
    workspace_repo::insert_project(pool, &project)
        .await
        .expect("insert project");

    sqlx::query(
        "INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(project_id.0)
    .bind(asset_id.0)
    .execute(pool)
    .await
    .expect("link asset to project");

    let tl = TargetLanguage {
        id: Uuid::new_v4(),
        project_id,
        source_lang: source_lang.into(),
        target_lang: "es".into(),
        created_at: OffsetDateTime::now_utc(),
    };
    workspace_repo::upsert_target_language(pool, &tl)
        .await
        .expect("insert target language");
}

async fn run_full_preparation(
    pool: &PgPool,
    asset_id: AssetId,
    source: &ArtifactRecord,
    queue: &dyn TranscriptionJobQueue,
) {
    let workspace = TempDir::new().expect("temp dir");
    let storage = LocalFsAdapter::new(workspace.path());
    storage
        .put(&source.storage_key, b"source-media-bytes".to_vec())
        .await
        .expect("persist source bytes");
    preparation_repo::upsert_preparation_status(
        pool,
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
        hls_result: Ok(valid_hls_output()),
    };

    process_preparation_job(
        pool,
        &storage,
        &executor,
        queue,
        PreparationJob::new(asset_id.0, source.id, source.ingest_token),
    )
    .await
    .expect("process preparation job");
}

#[tokio::test]
async fn preparation_ready_enqueues_transcription_job_with_source_language() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    insert_project_with_target_language(&pool, asset_id, "en").await;

    let queue = InMemoryTranscriptionJobQueue::default();
    run_full_preparation(&pool, asset_id, &source, &queue).await;

    let jobs = queue.queued_jobs();
    assert_eq!(jobs.len(), 1, "exactly one job should be enqueued");
    assert_eq!(jobs[0].asset_id, asset_id.0);
    assert_eq!(jobs[0].source_artifact_id, source.id);
    assert_eq!(jobs[0].source_language, "en");
}

#[tokio::test]
async fn preparation_ready_writes_transcription_pending_status() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    insert_project_with_target_language(&pool, asset_id, "pt-BR").await;

    let queue = InMemoryTranscriptionJobQueue::default();
    run_full_preparation(&pool, asset_id, &source, &queue).await;

    let status = transcription_repo::get_transcription_status(&pool, asset_id)
        .await
        .expect("get transcription status")
        .expect("status row present");

    assert_eq!(status.status, TranscriptionStatus::Pending);
    assert!(status.error_detail.is_none());
}

#[tokio::test]
async fn enqueued_source_language_matches_target_languages_row() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    insert_project_with_target_language(&pool, asset_id, "fr").await;

    let queue = InMemoryTranscriptionJobQueue::default();
    run_full_preparation(&pool, asset_id, &source, &queue).await;

    let jobs = queue.queued_jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].source_language, "fr");
}

#[tokio::test]
async fn enqueue_failure_records_transcription_failed_status() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    struct FailingQueue;

    impl TranscriptionJobQueue for FailingQueue {
        fn enqueue(&self, _job: TranscriptionJob) -> Result<(), QueueError> {
            Err(QueueError::Unavailable("queue is down".into()))
        }
    }

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    insert_project_with_target_language(&pool, asset_id, "en").await;

    run_full_preparation(&pool, asset_id, &source, &FailingQueue).await;

    assert_status(&pool, asset_id, PreparationStatus::Ready).await;

    let ts = transcription_repo::get_transcription_status(&pool, asset_id)
        .await
        .expect("get status")
        .expect("status row");

    assert_eq!(ts.status, TranscriptionStatus::Failed);
    assert!(
        ts.error_detail
            .as_deref()
            .unwrap_or("")
            .contains("queue is down")
    );
}

#[tokio::test]
async fn missing_target_languages_row_records_transcription_failed() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    let queue = InMemoryTranscriptionJobQueue::default();
    run_full_preparation(&pool, asset_id, &source, &queue).await;

    assert_status(&pool, asset_id, PreparationStatus::Ready).await;

    let ts = transcription_repo::get_transcription_status(&pool, asset_id)
        .await
        .expect("get status")
        .expect("status row");

    assert_eq!(ts.status, TranscriptionStatus::Failed);
    assert!(
        ts.error_detail
            .as_deref()
            .unwrap_or("")
            .contains("no target_languages row")
    );
    assert!(queue.queued_jobs().is_empty());
}

#[tokio::test]
async fn duplicate_preparation_completion_does_not_enqueue_second_job() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    insert_project_with_target_language(&pool, asset_id, "en").await;

    let queue = InMemoryTranscriptionJobQueue::default();
    run_full_preparation(&pool, asset_id, &source, &queue).await;
    assert_eq!(queue.queued_jobs().len(), 1);

    prepare_transcription_post_ready(&pool, &queue, asset_id, source.id).await;

    assert_eq!(
        queue.queued_jobs().len(),
        1,
        "no second job should be enqueued"
    );
}
