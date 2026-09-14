use std::sync::Arc;

use async_trait::async_trait;
use dubbridge_db::{preparation_repo, target_language_repo, workspace_repo};
use dubbridge_domain::{
    artifact::PreparationStatus,
    asset::AssetId,
    workspace::{OrgId, Organization, Project, ProjectId, TargetLanguage},
};
use dubbridge_jobs::{PreparationJob, QueueError, TranscriptionJob, TranscriptionJobQueue};
use dubbridge_storage::{LocalFsAdapter, StorageAdapter};
use sqlx::PgPool;
use tempfile::TempDir;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::preparation_runtime::process_preparation_job;

use super::support::{
    FakePreparationExecutor, assert_status, insert_asset, insert_source_artifact, setup_pool,
    valid_hls_output, valid_probe_bytes,
};

async fn insert_project_with_target_language(pool: &PgPool, asset_id: AssetId) {
    let org_id = OrgId(Uuid::new_v4());
    workspace_repo::insert_org(
        pool,
        &Organization {
            id: org_id,
            name: "p2p-activation-test-org".into(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        },
    )
    .await
    .expect("insert org");

    let project_id = ProjectId(Uuid::new_v4());
    workspace_repo::insert_project(
        pool,
        &Project {
            id: project_id,
            org_id,
            name: "p2p-activation-test-project".into(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        },
    )
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

    target_language_repo::upsert_target_language(
        pool,
        &TargetLanguage {
            id: Uuid::new_v4(),
            project_id,
            source_lang: "en".into(),
            target_lang: "es".into(),
            created_at: OffsetDateTime::now_utc(),
        },
    )
    .await
    .expect("insert target language");
}

struct ReadyObservingQueue {
    pool: PgPool,
    asset_id: AssetId,
    stage_log: Arc<Mutex<Vec<&'static str>>>,
}

#[async_trait]
impl TranscriptionJobQueue for ReadyObservingQueue {
    async fn enqueue(&self, _job: TranscriptionJob) -> Result<(), QueueError> {
        let status = preparation_repo::get_preparation_status(&self.pool, self.asset_id)
            .await
            .map_err(|error| QueueError::Unavailable(error.to_string()))?
            .ok_or_else(|| QueueError::Unavailable("preparation status missing".into()))?;

        assert_eq!(
            status.status,
            PreparationStatus::Ready,
            "transcription enqueue must only be attempted after S-120 Ready is durable"
        );
        self.stage_log.lock().await.push("transcription_enqueue");
        Ok(())
    }
}

#[tokio::test]
async fn t5a_s120_ready_is_durable_before_transcription_post_ready_enqueue() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    insert_project_with_target_language(&pool, asset_id).await;

    preparation_repo::upsert_preparation_status(&pool, asset_id, PreparationStatus::Pending, None)
        .await
        .expect("set preparation pending");

    let workspace = TempDir::new().expect("temp dir");
    let storage = LocalFsAdapter::new(workspace.path());
    storage
        .put(&source.storage_key, b"source-media-bytes".to_vec())
        .await
        .expect("persist source bytes");

    let stage_log = Arc::new(Mutex::new(Vec::new()));
    let executor = FakePreparationExecutor {
        pool: pool.clone(),
        asset_id,
        stage_log: stage_log.clone(),
        probe_result: Ok(valid_probe_bytes()),
        hls_result: Ok(valid_hls_output()),
    };
    let queue = ReadyObservingQueue {
        pool: pool.clone(),
        asset_id,
        stage_log: stage_log.clone(),
    };

    process_preparation_job(
        &pool,
        &storage,
        &executor,
        &queue,
        PreparationJob::new(asset_id.0, source.id, source.ingest_token),
    )
    .await
    .expect("process preparation");

    assert_status(&pool, asset_id, PreparationStatus::Ready).await;
    assert_eq!(
        *stage_log.lock().await,
        vec!["probe", "hls", "transcription_enqueue"],
        "the existing S-120 stages must complete before the post-ready transcription call"
    );
}
