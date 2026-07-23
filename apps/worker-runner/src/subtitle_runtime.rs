use dubbridge_providers::SegmentationProvider;

#[allow(dead_code)]
pub(crate) async fn process_subtitle_envelope(
    pool: &sqlx::PgPool,
    storage: &(dyn dubbridge_storage::StorageAdapter + Send + Sync),
    envelope: dubbridge_jobs::JobEnvelope<dubbridge_jobs::SubtitleJob>,
) -> anyhow::Result<()> {
    if envelope.job_type != dubbridge_jobs::SubtitleJob::JOB_TYPE {
        anyhow::bail!(
            "unsupported subtitle job type '{}', expected '{}'",
            envelope.job_type,
            dubbridge_jobs::SubtitleJob::JOB_TYPE
        );
    }

    process_subtitle_job(pool, storage, envelope.payload).await
}

#[allow(dead_code)]
pub(crate) async fn process_subtitle_job(
    pool: &sqlx::PgPool,
    storage: &(dyn dubbridge_storage::StorageAdapter + Send + Sync),
    job: dubbridge_jobs::SubtitleJob,
) -> anyhow::Result<()> {
    let asset_id = dubbridge_domain::asset::AssetId(job.asset_id);

    let result = process_subtitle_job_inner(pool, storage, &job).await;
    if let Err(error) = result {
        let detail = format!("{error:#}");
        let _ = dubbridge_db::subtitle_repo::upsert_subtitle_status(
            pool,
            asset_id,
            dubbridge_domain::artifact::SubtitleStatus::Failed,
            Some(&detail),
        )
        .await;
        return Err(error);
    }

    Ok(())
}

async fn process_subtitle_job_inner(
    pool: &sqlx::PgPool,
    storage: &(dyn dubbridge_storage::StorageAdapter + Send + Sync),
    job: &dubbridge_jobs::SubtitleJob,
) -> anyhow::Result<()> {
    let asset_id = dubbridge_domain::asset::AssetId(job.asset_id);

    dubbridge_db::subtitle_repo::upsert_subtitle_status(
        pool,
        asset_id,
        dubbridge_domain::artifact::SubtitleStatus::InProgress,
        None,
    )
    .await?;

    let artifacts = dubbridge_db::preparation_repo::list_derived_artifacts(pool, asset_id).await?;
    let alignment_artifact = artifacts
        .into_iter()
        .rev()
        .find(|a| a.kind == dubbridge_domain::artifact::ArtifactKind::WordAlignment)
        .ok_or_else(|| {
            anyhow::anyhow!("missing upstream word alignment for asset {}", asset_id.0)
        })?;

    let bytes = storage.get(&alignment_artifact.storage_key).await?;
    let raw: RawAlignmentFile = serde_json::from_slice(&bytes)?;
    let words = raw_words_to_provider(&raw.words);
    let segments = dubbridge_providers::RustSegmentationProvider
        .segment(&words)
        .map_err(|e| anyhow::anyhow!("segmentation failed: {}", e.message))?;

    let subtitle_bytes = serde_json::to_vec(&segments)?;
    storage
        .put(
            &dubbridge_storage::subtitle_key(&job.asset_id.to_string()),
            subtitle_bytes.clone(),
        )
        .await?;

    dubbridge_db::subtitle_repo::insert_subtitle_artifact(
        pool,
        asset_id,
        alignment_artifact.id,
        &dubbridge_storage::subtitle_key(&job.asset_id.to_string()),
        "application/json",
        subtitle_bytes.len() as i64,
        &checksum_hex(&subtitle_bytes),
    )
    .await?;

    dubbridge_db::subtitle_repo::upsert_subtitle_status(
        pool,
        asset_id,
        dubbridge_domain::artifact::SubtitleStatus::Ready,
        None,
    )
    .await?;

    let ready =
        dubbridge_db::subtitle_repo::get_subtitle_readiness_evidence(pool, asset_id).await?;
    if !ready {
        anyhow::bail!("subtitle readiness evidence incomplete after Ready status write");
    }

    crate::review_enqueue::prepare_review_post_ready(
        pool,
        asset_id,
        dubbridge_domain::workspace::ProjectId(job.project_id),
        &job.target_language,
    )
    .await;

    Ok(())
}

#[derive(serde::Deserialize)]
pub(crate) struct RawAlignmentFile {
    pub(crate) words: Vec<RawWord>,
}

#[derive(serde::Deserialize)]
pub(crate) struct RawWord {
    word: String,
    start: f64,
    end: f64,
}

pub(crate) fn raw_words_to_provider(words: &[RawWord]) -> Vec<dubbridge_providers::WordAlignment> {
    words
        .iter()
        .map(|w| dubbridge_providers::WordAlignment {
            word: w.word.clone(),
            start_ms: (w.start * 1000.0).round() as u64,
            end_ms: (w.end * 1000.0).round() as u64,
        })
        .collect()
}

fn checksum_hex(bytes: &[u8]) -> String {
    crate::checksum_hex(bytes)
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use dubbridge_db::{artifact_repo, preparation_repo, review_repo, subtitle_repo, workspace_repo};
    use dubbridge_domain::{
        artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact, SubtitleStatus},
        workspace::{OrgId, Organization, Project, ProjectId, TargetLanguage},
    };
    use dubbridge_jobs::JobEnvelope;
    use dubbridge_storage::{LocalFsAdapter, StorageAdapter};
    use time::OffsetDateTime;
    use uuid::Uuid;

    async fn setup_pool() -> Option<sqlx::PgPool> {
        let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
        let pool = sqlx::PgPool::connect(&url).await.expect("connect");
        sqlx::migrate!("../../infra/migrations")
            .run(&pool)
            .await
            .expect("migrations");
        sqlx::query(
            "TRUNCATE TABLE pending_ingestions, audit_events, artifact_records, rights_records, assets, asset_preparation_status, asset_transcription_status, asset_subtitle_status, review_tasks RESTART IDENTITY CASCADE",
        )
        .execute(&pool)
        .await
        .expect("truncate");
        Some(pool)
    }

    async fn insert_asset(pool: &sqlx::PgPool) -> dubbridge_domain::asset::AssetId {
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

    async fn insert_source_artifact(
        pool: &sqlx::PgPool,
        asset_id: dubbridge_domain::asset::AssetId,
    ) -> ArtifactRecord {
        let record = ArtifactRecord::new_original(
            asset_id,
            Uuid::new_v4(),
            format!("ingest/{}/source.mp4", asset_id.0),
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
        pool: &sqlx::PgPool,
        asset_id: dubbridge_domain::asset::AssetId,
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
        dubbridge_db::workspace_repo::insert_org(pool, &org)
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
        dubbridge_db::workspace_repo::insert_project(pool, &project)
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
            dubbridge_db::workspace_repo::upsert_target_language(
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

    #[test]
    fn alignment_seconds_to_ms_conversion_is_correct() {
        let words = vec![RawWord {
            word: "hi".into(),
            start: 1.5,
            end: 2.25,
        }];
        let result = raw_words_to_provider(&words);
        assert_eq!(result[0].start_ms, 1500);
        assert_eq!(result[0].end_ms, 2250);
    }

    #[tokio::test]
    async fn process_subtitle_job_marks_ready_and_stores_artifact_on_success() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let storage_workspace = tempfile::TempDir::new().expect("storage workspace");
        let storage = LocalFsAdapter::new(storage_workspace.path());

        // Insert WordAlignment derived artifact
        let alignment = DerivedArtifact::new(
            asset_id,
            source.id,
            ArtifactKind::WordAlignment,
            "words/alignment.json".into(),
            "application/json".into(),
            128,
            "alignsum".into(),
        );
        preparation_repo::insert_derived_artifact(&pool, &alignment)
            .await
            .expect("insert alignment artifact");

        // Store a 2-word non-overlapping alignment.json at that artifact's storage_key
        let alignment_json = br#"{"words":[{"word":"Hello","start":0.0,"end":1.0},{"word":"world","start":1.5,"end":2.5}]}"#.to_vec();
        storage
            .put("words/alignment.json", alignment_json)
            .await
            .expect("store alignment json");

        subtitle_repo::upsert_subtitle_status(&pool, asset_id, SubtitleStatus::Pending, None)
            .await
            .expect("set pending");
        insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;

        let job = dubbridge_jobs::SubtitleJob::new(asset_id.0, Uuid::new_v4(), "es");
        process_subtitle_job(&pool, &storage, job)
            .await
            .expect("process subtitle job");

        let status = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(status.status, SubtitleStatus::Ready);

        let ready = subtitle_repo::get_subtitle_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness");
        assert!(ready);

        // Verify review_tasks row was enqueued
        let review_rows: Vec<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM review_tasks ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .expect("get review tasks");
        assert_eq!(review_rows.len(), 1, "exactly one review_task should be enqueued on success");

        // Verify stored subtitle deserializes to 1 segment with expected joined text
        let key = dubbridge_storage::subtitle_key(&asset_id.0.to_string());
        let stored_bytes = storage.get(&key).await.expect("get stored subtitle");
        let segments: Vec<dubbridge_providers::SubtitleSegment> =
            serde_json::from_slice(&stored_bytes).expect("parse subtitle");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello world");
    }

    #[tokio::test]
    async fn process_subtitle_job_fails_when_alignment_missing() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        insert_source_artifact(&pool, asset_id).await;

        subtitle_repo::upsert_subtitle_status(&pool, asset_id, SubtitleStatus::Pending, None)
            .await
            .expect("set pending");
        insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;

        let job = dubbridge_jobs::SubtitleJob::new(asset_id.0, Uuid::new_v4(), "es");
        let storage_workspace = tempfile::TempDir::new().unwrap();
        let storage = LocalFsAdapter::new(storage_workspace.path());
        let err = process_subtitle_job(&pool, &storage, job)
            .await
            .expect_err("missing alignment must fail the job");

        assert!(err.to_string().contains("missing upstream word alignment"));
        let status = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(status.status, SubtitleStatus::Failed);
        assert!(
            status
                .error_detail
                .as_deref()
                .unwrap_or("")
                .contains("missing upstream word alignment")
        );

        // No review_tasks row should have been created
        let review_rows: Vec<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM review_tasks ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .expect("get review tasks");
        assert!(review_rows.is_empty());
    }

    #[tokio::test]
    async fn process_subtitle_job_fails_closed_on_invalid_segmentation_output() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let storage_workspace = tempfile::TempDir::new().expect("storage workspace");
        let storage = LocalFsAdapter::new(storage_workspace.path());

        // Insert WordAlignment derived artifact
        let alignment = DerivedArtifact::new(
            asset_id,
            source.id,
            ArtifactKind::WordAlignment,
            "words/overlapping.json".into(),
            "application/json".into(),
            128,
            "alignsum".into(),
        );
        preparation_repo::insert_derived_artifact(&pool, &alignment)
            .await
            .expect("insert alignment artifact");

        // Store an overlapping timing alignment (second word's start < first word's end)
        let alignment_json = br#"{"words":[{"word":"first","start":0.0,"end":1.5},{"word":"second","start":1.0,"end":2.0}]}"#.to_vec();
        storage
            .put("words/overlapping.json", alignment_json)
            .await
            .expect("store alignment json");

        subtitle_repo::upsert_subtitle_status(&pool, asset_id, SubtitleStatus::Pending, None)
            .await
            .expect("set pending");
        insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;

        let job = dubbridge_jobs::SubtitleJob::new(asset_id.0, Uuid::new_v4(), "es");
        let err = process_subtitle_job(&pool, &storage, job)
            .await
            .expect_err("segmentation must fail on overlapping timing");

        assert!(err.to_string().contains("segmentation failed"));
        let status = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(status.status, SubtitleStatus::Failed);

        // No review_tasks row should have been created
        let review_rows: Vec<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM review_tasks ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .expect("get review tasks");
        assert!(review_rows.is_empty());

        // No Subtitle artifact should have been inserted
        let derived = dubbridge_db::preparation_repo::list_derived_artifacts(&pool, asset_id)
            .await
            .expect("list derived artifacts");
        assert!(
            !derived.iter().any(|a| a.kind == ArtifactKind::Subtitle),
            "no Subtitle artifact should exist after segmentation failure"
        );

        // No subtitle.json should exist in storage
        let key = dubbridge_storage::subtitle_key(&asset_id.0.to_string());
        assert!(
            storage.get(&key).await.is_err(),
            "no subtitle file should exist"
        );
    }

    #[tokio::test]
    async fn process_subtitle_envelope_rejects_wrong_job_type() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;

        let err = process_subtitle_envelope(
            &pool,
            &LocalFsAdapter::new(tempfile::TempDir::new().unwrap().path()),
            JobEnvelope::new(
                "media_preparation",
                dubbridge_jobs::SubtitleJob::new(asset_id.0, Uuid::new_v4(), "es"),
            ),
        )
        .await
        .expect_err("wrong job type must fail the envelope");

        assert!(err.to_string().contains("unsupported subtitle job type"));
    }

    #[tokio::test]
    async fn process_subtitle_review_tasks_no_row_on_failure() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        insert_source_artifact(&pool, asset_id).await;

        subtitle_repo::upsert_subtitle_status(&pool, asset_id, SubtitleStatus::Pending, None)
            .await
            .expect("set pending");
        // No project/target languages inserted -> missing project case

        let job = dubbridge_jobs::SubtitleJob::new(asset_id.0, Uuid::new_v4(), "es");
        let storage_workspace = tempfile::TempDir::new().unwrap();
        let storage = LocalFsAdapter::new(storage_workspace.path());
        let err = process_subtitle_job(&pool, &storage, job)
            .await
            .expect_err("missing alignment must fail the job");

        assert!(err.to_string().contains("missing upstream word alignment"));

        // No review_tasks row should have been created
        let review_rows: Vec<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM review_tasks ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .expect("get review tasks");
        assert!(review_rows.is_empty());
    }
}
