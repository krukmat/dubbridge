use dubbridge_db::{subtitle_repo, transcription_repo, workspace_repo};
use dubbridge_domain::{
    artifact::{SubtitleStatus, TranscriptionStatus},
    asset::AssetId,
};
use dubbridge_jobs::{SubtitleJob, SubtitleJobQueue, TranscriptionJob, TranscriptionJobQueue};
use sqlx::PgPool;

/// Prepare the transcription-ready post-processing flow.
///
/// On error it records `TranscriptionStatus::Failed` and never propagates back to
/// preparation readiness.
pub async fn prepare_transcription_post_ready(
    pool: &PgPool,
    queue: &dyn TranscriptionJobQueue,
    asset_id: AssetId,
    source_artifact_id: uuid::Uuid,
) {
    if let Err(error) = try_enqueue_transcription(pool, queue, asset_id, source_artifact_id).await {
        record_transcription_failure(pool, asset_id, &error).await;
    }
}

/// Prepare the subtitle-ready post-processing flow after transcription reaches `Ready`.
///
/// On error it records `SubtitleStatus::Failed` and never propagates back to
/// transcription readiness.
pub async fn prepare_subtitle_post_ready(
    pool: &PgPool,
    queue: &dyn SubtitleJobQueue,
    asset_id: AssetId,
) {
    if let Err(error) = try_enqueue_subtitle(pool, queue, asset_id).await {
        record_subtitle_failure(pool, asset_id, &error).await;
    }
}

async fn record_transcription_failure(pool: &PgPool, asset_id: AssetId, detail: &str) {
    let _ = transcription_repo::upsert_transcription_status(
        pool,
        asset_id,
        TranscriptionStatus::Failed,
        Some(detail),
    )
    .await;
}

async fn record_subtitle_failure(pool: &PgPool, asset_id: AssetId, detail: &str) {
    let _ =
        subtitle_repo::upsert_subtitle_status(pool, asset_id, SubtitleStatus::Failed, Some(detail))
            .await;
}

async fn try_enqueue_transcription(
    pool: &PgPool,
    queue: &dyn TranscriptionJobQueue,
    asset_id: AssetId,
    source_artifact_id: uuid::Uuid,
) -> Result<(), String> {
    let claimed = transcription_repo::try_claim_transcription_pending(pool, asset_id)
        .await
        .map_err(|error| {
            tracing::warn!(asset_id = %asset_id, error = %error, "failed to claim TranscriptionStatus::Pending");
            error.to_string()
        })?;
    if !claimed {
        return Ok(());
    }

    let source_language = resolve_source_language(pool, asset_id).await?;

    queue
        .enqueue(TranscriptionJob::new(
            asset_id.0,
            source_artifact_id,
            source_language,
        ))
        .map_err(|error| {
            tracing::warn!(asset_id = %asset_id, error = %error, "failed to enqueue TranscriptionJob");
            error.to_string()
        })
}

async fn try_enqueue_subtitle(
    pool: &PgPool,
    queue: &dyn SubtitleJobQueue,
    asset_id: AssetId,
) -> Result<(), String> {
    let claimed = subtitle_repo::try_claim_subtitle_pending(pool, asset_id)
        .await
        .map_err(|error| {
            tracing::warn!(asset_id = %asset_id, error = %error, "failed to claim SubtitleStatus::Pending");
            error.to_string()
        })?;
    if !claimed {
        return Ok(());
    }

    let route = resolve_subtitle_route(pool, asset_id).await?;
    queue
        .enqueue(SubtitleJob::new(
            asset_id.0,
            route.project_id.0,
            route.target_language,
        ))
        .map_err(|error| {
            tracing::warn!(asset_id = %asset_id, error = %error, "failed to enqueue SubtitleJob");
            error.to_string()
        })
}

async fn resolve_source_language(pool: &PgPool, asset_id: AssetId) -> Result<String, String> {
    let result = workspace_repo::get_source_language_for_asset(pool, asset_id)
        .await
        .map_err(|error| {
            tracing::warn!(asset_id = %asset_id, error = %error, "failed to resolve source language");
            error.to_string()
        })?;
    result.ok_or_else(|| {
        let detail = "no target_languages row found for asset project";
        tracing::warn!(asset_id = %asset_id, detail, "transcription enqueue failed");
        detail.to_string()
    })
}

async fn resolve_subtitle_route(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<workspace_repo::AssetSubtitleRoute, String> {
    let route = workspace_repo::get_asset_subtitle_route(pool, asset_id)
        .await
        .map_err(|error| {
            tracing::warn!(asset_id = %asset_id, error = %error, "failed to resolve subtitle route");
            error.to_string()
        })?;
    route.ok_or_else(|| {
        let detail = "no subtitle project/target route found for asset";
        tracing::warn!(asset_id = %asset_id, detail, "subtitle enqueue failed");
        detail.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_db::{artifact_repo, preparation_repo, workspace_repo};
    use dubbridge_domain::{
        artifact::{ArtifactRecord, PreparationStatus},
        workspace::{OrgId, Organization, Project, ProjectId, TargetLanguage},
    };
    use dubbridge_jobs::{InMemorySubtitleJobQueue, InMemoryTranscriptionJobQueue, QueueError};
    use dubbridge_storage::StorageAdapter;
    use tempfile::TempDir;
    use time::OffsetDateTime;
    use uuid::Uuid;

    async fn setup_pool_for_test() -> Option<PgPool> {
        let url = std::env::var("DUBBRIDGE_DATABASE_URL").ok()?;
        let pool = PgPool::connect(&url).await.expect("connect");
        sqlx::migrate!("../../infra/migrations")
            .run(&pool)
            .await
            .expect("migrations");
        sqlx::query(
            "TRUNCATE TABLE target_languages, project_assets, projects, org_members, organizations, pending_ingestions, audit_events, artifact_records, rights_records, assets, asset_preparation_status, asset_transcription_status, asset_subtitle_status RESTART IDENTITY CASCADE",
        )
        .execute(&pool)
        .await
        .expect("truncate");
        Some(pool)
    }

    async fn insert_asset_for_test(pool: &PgPool) -> AssetId {
        let asset_id = dubbridge_domain::asset::AssetId::new();
        sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
            .bind(asset_id.0)
            .bind("test-asset")
            .bind(Uuid::new_v4())
            .bind("finalized")
            .execute(pool)
            .await
            .expect("insert asset");
        asset_id
    }

    async fn insert_source_artifact_for_test(pool: &PgPool, asset_id: AssetId) -> ArtifactRecord {
        let record = ArtifactRecord::new_original(
            asset_id,
            Uuid::new_v4(),
            format!("ingest/{asset_id}/source.mp4"),
            "video/mp4".into(),
            1024,
            "sourcesum".into(),
        );
        artifact_repo::insert_artifact_record(pool, &record)
            .await
            .expect("insert source artifact");
        record
    }

    async fn insert_project_with_targets(
        pool: &PgPool,
        asset_id: AssetId,
        source_lang: &str,
        target_langs: &[&str],
    ) -> ProjectId {
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

        for target_lang in target_langs {
            workspace_repo::upsert_target_language(
                pool,
                &TargetLanguage {
                    id: Uuid::new_v4(),
                    project_id,
                    source_lang: source_lang.into(),
                    target_lang: (*target_lang).into(),
                    created_at: OffsetDateTime::now_utc(),
                },
            )
            .await
            .expect("insert target language");
        }

        project_id
    }

    #[tokio::test]
    async fn prepare_transcription_post_ready_enqueues_transcription_job() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        let asset_id = insert_asset_for_test(&pool).await;
        let source = insert_source_artifact_for_test(&pool, asset_id).await;
        insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;

        let workspace = TempDir::new().expect("temp dir");
        let storage = dubbridge_storage::LocalFsAdapter::new(workspace.path());
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

        let queue = InMemoryTranscriptionJobQueue::default();
        prepare_transcription_post_ready(&pool, &queue, asset_id, source.id).await;

        let jobs = queue.queued_jobs();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].asset_id, asset_id.0);
        assert_eq!(jobs[0].source_artifact_id, source.id);
        assert_eq!(jobs[0].source_language, "en");
    }

    #[tokio::test]
    async fn prepare_subtitle_post_ready_enqueues_first_target_in_c_order() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        let asset_id = insert_asset_for_test(&pool).await;
        let project_id = insert_project_with_targets(&pool, asset_id, "en", &["fr", "de"]).await;

        let queue = InMemorySubtitleJobQueue::default();
        prepare_subtitle_post_ready(&pool, &queue, asset_id).await;

        let jobs = queue.queued_jobs();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].asset_id, asset_id.0);
        assert_eq!(jobs[0].project_id, project_id.0);
        assert_eq!(jobs[0].target_language, "de");

        let status = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .expect("get subtitle status")
            .expect("subtitle row");
        assert_eq!(status.status, SubtitleStatus::Pending);
        assert!(status.error_detail.is_none());
    }

    #[tokio::test]
    async fn prepare_subtitle_post_ready_is_idempotent_after_pending_claim() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        let asset_id = insert_asset_for_test(&pool).await;
        insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;
        let queue = InMemorySubtitleJobQueue::default();

        prepare_subtitle_post_ready(&pool, &queue, asset_id).await;
        prepare_subtitle_post_ready(&pool, &queue, asset_id).await;

        assert_eq!(queue.queued_jobs().len(), 1);
    }

    #[tokio::test]
    async fn prepare_subtitle_post_ready_records_failed_when_route_missing() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        let asset_id = insert_asset_for_test(&pool).await;
        let queue = InMemorySubtitleJobQueue::default();

        prepare_subtitle_post_ready(&pool, &queue, asset_id).await;

        let status = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .expect("get subtitle status")
            .expect("subtitle row");
        assert_eq!(status.status, SubtitleStatus::Failed);
        assert!(
            status
                .error_detail
                .as_deref()
                .unwrap_or("")
                .contains("no subtitle project/target route found")
        );
        assert!(queue.queued_jobs().is_empty());
    }

    #[tokio::test]
    async fn prepare_subtitle_post_ready_records_failed_when_queue_fails() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        struct FailingSubtitleQueue;
        impl SubtitleJobQueue for FailingSubtitleQueue {
            fn enqueue(&self, _job: SubtitleJob) -> Result<(), QueueError> {
                Err(QueueError::Unavailable("subtitle queue down".into()))
            }
        }

        let asset_id = insert_asset_for_test(&pool).await;
        insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;

        prepare_subtitle_post_ready(&pool, &FailingSubtitleQueue, asset_id).await;

        let status = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .expect("get subtitle status")
            .expect("subtitle row");
        assert_eq!(status.status, SubtitleStatus::Failed);
        assert!(
            status
                .error_detail
                .as_deref()
                .unwrap_or("")
                .contains("subtitle queue down")
        );
    }
}
