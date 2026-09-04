use super::*;
use crate::translation_fanout::fan_out_localization;
use dubbridge_domain::artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact};
use dubbridge_domain::workspace::TargetLanguage;
use dubbridge_providers::translation::{
    StubTranslationWorkerClient, TRANSLATION_SCHEMA_VERSION, TranslatedSegment, TranslationError,
    TranslationOutput,
};
use dubbridge_storage::LocalFsAdapter;

async fn setup_pool() -> Option<PgPool> {
    let url = std::env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::migrate!("../../infra/migrations")
         .run(&pool)
         .await
         .expect("migrations");
    Some(pool)
}

async fn insert_asset(pool: &PgPool) -> AssetId {
    let asset_id = AssetId::new();
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

async fn insert_project_with_targets(
    pool: &PgPool,
    asset_id: AssetId,
    source_lang: &str,
    target_langs: &[&str],
) -> ProjectId {
    let project_id = ProjectId(Uuid::new_v4());
    let org_id = Uuid::new_v4();
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, 'test-org')")
        .bind(org_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, 'test-project')")
        .bind(project_id.0)
        .bind(org_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(project_id.0)
        .bind(asset_id.0)
        .execute(pool)
        .await
        .unwrap();

    for &code in target_langs {
        let tl = TargetLanguage {
            id: Uuid::new_v4(),
            project_id,
            source_lang: source_lang.to_string(),
            target_lang: code.to_string(),
            created_at: time::OffsetDateTime::now_utc(),
        };
        dubbridge_db::target_language_repo::upsert_target_language(pool, &tl)
            .await
            .unwrap();
    }
    project_id
}

async fn insert_word_alignment_and_subtitle(
    pool: &PgPool,
    asset_id: AssetId,
    storage: &LocalFsAdapter,
    legacy_segments: &[LegacySubtitleSegment],
) -> (Uuid, Uuid) {
    let original = ArtifactRecord::new_original(
        asset_id,
        Uuid::new_v4(),
        format!("ingest/{}/source.mp4", asset_id.0),
        "audio/mpeg".to_string(),
        1024,
        "orig-chk".to_string(),
    );
    dubbridge_db::artifact_repo::insert_artifact_record(pool, &original)
        .await
        .unwrap();

    let alignment = DerivedArtifact::new(
        asset_id,
        original.id,
        ArtifactKind::WordAlignment,
        format!("derived/{}/alignment.json", asset_id.0),
        "text/tab-separated-values".to_string(),
        512,
        "warp-chk".to_string(),
    );
    dubbridge_db::preparation_repo::insert_derived_artifact(pool, &alignment)
        .await
        .unwrap();

    let subtitle_key = dubbridge_storage::subtitle_key(&asset_id.to_string());
    let subtitle_bytes = serde_json::to_vec(legacy_segments).unwrap();
    storage
        .put(&subtitle_key, subtitle_bytes.clone())
        .await
        .unwrap();

    let subtitle = dubbridge_db::subtitle_repo::insert_subtitle_artifact(
        pool,
        asset_id,
        alignment.id,
        &subtitle_key,
        "application/json",
        subtitle_bytes.len() as i64,
        "sub-chk",
    )
    .await
    .unwrap();

    (alignment.id, subtitle.id)
}

fn sample_legacy_segments() -> Vec<LegacySubtitleSegment> {
    vec![
        LegacySubtitleSegment {
            start_ms: 0,
            end_ms: 1000,
            text: "Hello".into(),
        },
        LegacySubtitleSegment {
            start_ms: 1000,
            end_ms: 2000,
            text: "world".into(),
        },
    ]
}

fn stub_translation_output(
    job: &TranslationJob,
    segments: &[LegacySubtitleSegment],
) -> TranslationOutput {
    TranslationOutput {
        schema_version: TRANSLATION_SCHEMA_VERSION,
        job_id: job.generation_request_id.to_string(),
        source_language: "en".into(),
        target_language: "es".into(),
        segments: segments
            .iter()
            .enumerate()
            .map(|(ordinal, seg)| TranslatedSegment {
                segment_id: format!("{}:{ordinal}", job.source_subtitle_artifact_id),
                start_ms: seg.start_ms,
                end_ms: seg.end_ms,
                source_text: seg.text.clone(),
                translated_text: format!("[es] {}", seg.text),
            })
            .collect(),
        status: "ok".into(),
    }
}

/// One `TranslationJob` for the sole configured target language, produced by
/// the real fan-out path so the claim row already exists exactly as it would
/// in production (subtitle Ready -> fan_out_localization -> claim persisted).
async fn build_ready_job(
    pool: &PgPool,
    storage: &LocalFsAdapter,
    target_lang: &str,
) -> TranslationJob {
    let asset_id = insert_asset(pool).await;
    insert_project_with_targets(pool, asset_id, "en", &[target_lang]).await;
    let (alignment_id, _subtitle_id) =
        insert_word_alignment_and_subtitle(pool, asset_id, storage, &sample_legacy_segments())
            .await;

    let jobs = fan_out_localization(pool, asset_id, alignment_id)
        .await
        .expect("fan out localization");
    assert_eq!(jobs.len(), 1, "expected exactly one target-language job");
    jobs.into_iter().next().unwrap()
}

// ---------------- HP-1: valid worker result -> Ready, scoped storage key ----------------

#[tokio::test]
async fn process_translation_job_marks_ready_and_stores_scoped_artifact() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let storage_workspace = tempfile::TempDir::new().expect("storage workspace");
    let storage = LocalFsAdapter::new(storage_workspace.path());

    let job = build_ready_job(&pool, &storage, "es").await;
    let output = stub_translation_output(&job, &sample_legacy_segments());
    let client = StubTranslationWorkerClient::ok(output);

    process_translation_job(&pool, &storage, &client, job.clone())
        .await
        .expect("process translation job");

    let status = translation_repo::get_translation_status(
        &pool,
        ProjectId(job.project_id),
        AssetId(job.asset_id),
        job.target_language_id,
    )
    .await
    .expect("get status")
    .expect("status row");
    assert_eq!(
        status.status,
        dubbridge_domain::artifact::TranslationStatus::Ready
    );

    let expected_key = translated_subtitle_key(
        &AssetId(job.asset_id).to_string(),
        &job.target_language_id.to_string(),
    );
    let stored = storage
        .get(&expected_key)
        .await
        .expect("stored translated subtitle");
    let segments: Vec<TranslatedSegment> =
        serde_json::from_slice(&stored).expect("parse stored translated subtitle");
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].translated_text, "[es] Hello");

    let evidence = translation_repo::get_translation_readiness_evidence(
        &pool,
        ProjectId(job.project_id),
        AssetId(job.asset_id),
        job.target_language_id,
    )
    .await
    .expect("readiness evidence");
    assert!(evidence.is_ready());
}

// ---------------- HP-1 continued: sibling target languages unaffected ----------------

#[tokio::test]
async fn process_translation_job_leaves_sibling_target_language_untouched() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let storage_workspace = tempfile::TempDir::new().expect("storage workspace");
    let storage = LocalFsAdapter::new(storage_workspace.path());

    let asset_id = insert_asset(&pool).await;
    insert_project_with_targets(&pool, asset_id, "en", &["es", "fr"]).await;
    let (alignment_id, _subtitle_id) =
        insert_word_alignment_and_subtitle(&pool, asset_id, &storage, &sample_legacy_segments())
            .await;

    let jobs = fan_out_localization(&pool, asset_id, alignment_id)
        .await
        .expect("fan out localization");
    assert_eq!(jobs.len(), 2);

    let project_id = ProjectId(jobs[0].project_id);
    let target_languages =
        dubbridge_db::target_language_repo::list_target_languages(&pool, project_id)
            .await
            .expect("list target languages");
    let es_target_language_id = target_languages
        .iter()
        .find(|tl| tl.target_lang == "es")
        .expect("es target language")
        .id;

    let es_job = jobs
        .iter()
        .find(|j| j.target_language_id == es_target_language_id)
        .cloned()
        .expect("es job");
    let fr_job = jobs
        .iter()
        .find(|j| j.target_language_id != es_job.target_language_id)
        .cloned()
        .expect("fr job");

    let output = stub_translation_output(&es_job, &sample_legacy_segments());
    let client = StubTranslationWorkerClient::ok(output);

    process_translation_job(&pool, &storage, &client, es_job.clone())
        .await
        .expect("process es translation job");

    let es_status = translation_repo::get_translation_status(
        &pool,
        ProjectId(es_job.project_id),
        AssetId(es_job.asset_id),
        es_job.target_language_id,
    )
    .await
    .expect("get es status")
    .expect("es status row");
    assert_eq!(
        es_status.status,
        dubbridge_domain::artifact::TranslationStatus::Ready
    );

    let fr_status = translation_repo::get_translation_status(
        &pool,
        ProjectId(fr_job.project_id),
        AssetId(fr_job.asset_id),
        fr_job.target_language_id,
    )
    .await
    .expect("get fr status")
    .expect("fr status row");
    assert_eq!(
        fr_status.status,
        dubbridge_domain::artifact::TranslationStatus::InProgress
    );
}

// ---------------- EC-1: worker failure leaves non-Ready with error detail ----------------

#[tokio::test]
async fn process_translation_job_marks_failed_on_worker_error() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let storage_workspace = tempfile::TempDir::new().expect("storage workspace");
    let storage = LocalFsAdapter::new(storage_workspace.path());

    let job = build_ready_job(&pool, &storage, "es").await;
    let client = StubTranslationWorkerClient::err(TranslationError {
        job_id: job.generation_request_id.to_string(),
        error_code: "PROVIDER_FAILED".into(),
        message: "translation provider unavailable".into(),
    });

    let err = process_translation_job(&pool, &storage, &client, job.clone())
        .await
        .expect_err("worker error must fail the job");
    assert!(err.to_string().contains("translation worker error"));

    let status = translation_repo::get_translation_status(
        &pool,
        ProjectId(job.project_id),
        AssetId(job.asset_id),
        job.target_language_id,
    )
    .await
    .expect("get status")
    .expect("status row");
    assert_eq!(
        status.status,
        dubbridge_domain::artifact::TranslationStatus::Failed
    );
    assert!(
        status
            .error_detail
            .as_deref()
            .unwrap_or("")
            .contains("translation worker error")
    );
}

// ---------------- EC-2: stale/replayed generation cannot overwrite current Ready ----------------

#[tokio::test]
async fn process_translation_job_rejects_stale_generation_after_ready() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let storage_workspace = tempfile::TempDir::new().expect("storage workspace");
    let storage = LocalFsAdapter::new(storage_workspace.path());

    let job = build_ready_job(&pool, &storage, "es").await;
    let output = stub_translation_output(&job, &sample_legacy_segments());
    let client = StubTranslationWorkerClient::ok(output);

    process_translation_job(&pool, &storage, &client, job.clone())
        .await
        .expect("first process succeeds");

    // Simulate a stale/replayed delivery of the same job after an explicit
    // regeneration has superseded it: forge a job carrying a
    // generation_request_id that no longer matches asset_translation_status's
    // current_generation_request_id.
    let mut stale_job = job.clone();
    stale_job.generation_request_id = Uuid::new_v4();

    let stale_output = stub_translation_output(&stale_job, &sample_legacy_segments());
    let stale_client = StubTranslationWorkerClient::ok(stale_output);

    let err = process_translation_job(&pool, &storage, &stale_client, stale_job.clone())
        .await
        .expect_err("stale generation must not overwrite current Ready state");
    assert!(err.to_string().contains("promote translation to ready"));

    let status = translation_repo::get_translation_status(
        &pool,
        ProjectId(job.project_id),
        AssetId(job.asset_id),
        job.target_language_id,
    )
    .await
    .expect("get status")
    .expect("status row");
    assert_eq!(
        status.status,
        dubbridge_domain::artifact::TranslationStatus::Ready
    );
    assert_eq!(
        status.current_generation_request_id,
        Some(job.generation_request_id)
    );
}

// ---------------- EC-3: worker-local URI never persisted as canonical storage_key ----------------

#[tokio::test]
async fn process_translation_job_never_persists_worker_local_uri_as_storage_key() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let storage_workspace = tempfile::TempDir::new().expect("storage workspace");
    let storage = LocalFsAdapter::new(storage_workspace.path());

    let job = build_ready_job(&pool, &storage, "es").await;
    let output = stub_translation_output(&job, &sample_legacy_segments());
    let client = StubTranslationWorkerClient::ok(output);

    process_translation_job(&pool, &storage, &client, job.clone())
        .await
        .expect("process translation job");

    let evidence = translation_repo::get_translation_readiness_evidence(
        &pool,
        ProjectId(job.project_id),
        AssetId(job.asset_id),
        job.target_language_id,
    )
    .await
    .expect("readiness evidence");
    let translated = evidence
        .current_translated_subtitle
        .expect("translated subtitle artifact");

    assert!(!translated.storage_key.starts_with("file://"));
    assert_eq!(
        translated.storage_key,
        translated_subtitle_key(
            &AssetId(job.asset_id).to_string(),
            &job.target_language_id.to_string()
        )
    );
}

// ---------------- Envelope job-type guard ----------------

#[tokio::test]
async fn process_translation_envelope_rejects_wrong_job_type() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let storage_workspace = tempfile::TempDir::new().expect("storage workspace");
    let storage = LocalFsAdapter::new(storage_workspace.path());

    let job = build_ready_job(&pool, &storage, "es").await;
    let output = stub_translation_output(&job, &sample_legacy_segments());
    let client = StubTranslationWorkerClient::ok(output);

    let err = process_translation_envelope(
        &pool,
        &storage,
        &client,
        JobEnvelope::new("subtitle_generation", job),
    )
    .await
    .expect_err("wrong job type must fail");

    assert!(err.to_string().contains("unsupported translation job type"));
}
