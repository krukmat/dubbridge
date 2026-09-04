use crate::translation_fanout::fan_out_localization;
use dubbridge_db::translation_delivery_repo::{
    MAX_TRANSLATION_DISPATCH_ATTEMPTS, TranslationDispatchFailureInput,
    TranslationDispatchFailureResult, translation_dispatch_enqueue_failure,
};
use dubbridge_domain::artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact};
use dubbridge_domain::asset::AssetId;
use dubbridge_domain::workspace::{ProjectId, TargetLanguage};
use dubbridge_jobs::TranslationJob;
use uuid::Uuid;

async fn setup_pool_for_test() -> Option<sqlx::PgPool> {
    let url = std::env::var("DUBBRIDGE_DATABASE_URL")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL").ok())?;
    let pool = sqlx::PgPool::connect_lazy(&url).ok()?;
    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .ok()?;
    Some(pool)
}

async fn insert_asset_for_test(pool: &sqlx::PgPool) -> dubbridge_domain::asset::AssetId {
    let asset_id = dubbridge_domain::asset::AssetId::new();
    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(asset_id.0)
        .bind("test-asset")
        .bind(uuid::Uuid::new_v4())
        .bind("finalized")
        .execute(pool)
        .await
        .expect("insert asset");
    asset_id
}

async fn insert_project_with_targets(
    pool: &sqlx::PgPool,
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
    pool: &sqlx::PgPool,
    asset_id: AssetId,
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

    let subtitle = dubbridge_db::subtitle_repo::insert_subtitle_artifact(
        pool,
        asset_id,
        alignment.id,
        &format!("derived/{}/subtitle.srt", asset_id.0),
        "text/srt",
        256_i64,
        "sub-chk",
    )
    .await
    .unwrap();

    (alignment.id, subtitle.id)
}

async fn mark_enqueue_failure(
    pool: &sqlx::PgPool,
    job: &TranslationJob,
    detail: &str,
) -> Result<TranslationDispatchFailureResult, String> {
    translation_dispatch_enqueue_failure(
        pool,
        TranslationDispatchFailureInput {
            project_id: ProjectId(job.project_id),
            asset_id: AssetId(job.asset_id),
            target_language_id: job.target_language_id,
            generation_request_id: job.generation_request_id,
            error_detail: detail.to_string(),
        },
    )
    .await
    .map_err(|error| error.to_string())
}

#[tokio::test]
async fn hp1_single_target_returns_one_translation_job() {
    let Some(pool) = setup_pool_for_test().await else {
        return;
    };
    let asset_id = insert_asset_for_test(&pool).await;
    let project_id = insert_project_with_targets(&pool, asset_id, "en", &["fr"]).await;
    let (alignment_id, subtitle_id) = insert_word_alignment_and_subtitle(&pool, asset_id).await;

    let result = fan_out_localization(&pool, asset_id, alignment_id).await;
    assert!(result.is_ok());
    let jobs = result.unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].project_id, project_id.0);
    assert_eq!(jobs[0].asset_id, asset_id.0);
    assert_eq!(jobs[0].source_subtitle_artifact_id, subtitle_id);
    assert_eq!(
        jobs[0].generation_request_id,
        dubbridge_jobs::initial_translation_generation_request_id(subtitle_id)
    );
}

#[tokio::test]
async fn ec1_partial_claim_leaves_other_target_working() -> Result<(), String> {
    let Some(pool) = setup_pool_for_test().await else {
        return Ok(());
    };
    let asset_id = insert_asset_for_test(&pool).await;
    let project_id = insert_project_with_targets(&pool, asset_id, "en", &["fr", "de"]).await;
    let (alignment_id, subtitle_id) = insert_word_alignment_and_subtitle(&pool, asset_id).await;

    let mut fr_id = Uuid::nil();
    let mut de_id = Uuid::nil();

    {
        let mut tx = pool.begin().await.unwrap();
        let candidates = dubbridge_db::target_language_repo::list_delivery_scope_candidates_tx(
            &mut tx,
            asset_id,
            subtitle_id,
        )
        .await
        .unwrap();
        for c in candidates {
            if c.target_language.target_lang == "fr" {
                fr_id = c.target_language.id;
            } else if c.target_language.target_lang == "de" {
                de_id = c.target_language.id;
            }
        }
        tx.rollback().await.unwrap();
    }

    let generation_request_id =
        dubbridge_jobs::initial_translation_generation_request_id(subtitle_id);
    let preclaimed = dubbridge_db::translation_delivery_repo::persist_translation_delivery(
        &pool,
        dubbridge_db::translation_delivery_repo::TranslationDeliveryInput {
            asset_id,
            project_id,
            target_language_id: fr_id,
            generation_request_id,
            source_subtitle_artifact_id: subtitle_id,
            expected_initial_generation_request_id: generation_request_id,
            mode: dubbridge_db::translation_repo::TranslationClaimMode::InitialDelivery,
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    assert_eq!(
        preclaimed.dispatch,
        dubbridge_db::translation_delivery_repo::TranslationDispatchDisposition::New
    );

    let jobs = fan_out_localization(&pool, asset_id, alignment_id).await?;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].target_language_id, de_id);
    Ok(())
}

#[tokio::test]
async fn retry_cap_allows_last_attempt_then_persists_terminal_failure() -> Result<(), String> {
    let Some(pool) = setup_pool_for_test().await else {
        return Ok(());
    };
    let asset_id = insert_asset_for_test(&pool).await;
    insert_project_with_targets(&pool, asset_id, "en", &["fr"]).await;
    let (alignment_id, _) = insert_word_alignment_and_subtitle(&pool, asset_id).await;

    let first = fan_out_localization(&pool, asset_id, alignment_id).await?;
    assert_eq!(first.len(), 1);
    let job = first[0].clone();
    assert_eq!(
        mark_enqueue_failure(&pool, &job, "attempt-1 enqueue failed").await?,
        TranslationDispatchFailureResult::Marked
    );

    let second = fan_out_localization(&pool, asset_id, alignment_id).await?;
    assert_eq!(second.len(), 1);
    assert_eq!(second[0], job);
    assert_eq!(
        mark_enqueue_failure(&pool, &job, "attempt-2 enqueue failed").await?,
        TranslationDispatchFailureResult::Marked
    );

    let last_permitted = fan_out_localization(&pool, asset_id, alignment_id).await?;
    assert_eq!(last_permitted.len(), 1);
    assert_eq!(last_permitted[0], job);
    assert_eq!(
        mark_enqueue_failure(&pool, &job, "attempt-3 enqueue failed").await?,
        TranslationDispatchFailureResult::Exhausted
    );

    let rejected = fan_out_localization(&pool, asset_id, alignment_id).await?;
    assert!(
        rejected.is_empty(),
        "attempt after the cap must not be scheduled"
    );

    let row: (String, i32, Option<String>) = sqlx::query_as(
        r#"
        SELECT delivery_state, attempt_count, error_detail
        FROM translation_dispatch_outbox
        WHERE operation = 'translation'
          AND project_id = $1
          AND asset_id = $2
          AND target_language_id = $3
          AND generation_request_id = $4
        "#,
    )
    .bind(job.project_id)
    .bind(job.asset_id)
    .bind(job.target_language_id)
    .bind(job.generation_request_id)
    .fetch_one(&pool)
    .await
    .map_err(|error| error.to_string())?;

    assert_eq!(row.0, "failed");
    assert_eq!(row.1, MAX_TRANSLATION_DISPATCH_ATTEMPTS);
    assert_eq!(row.2.as_deref(), Some("attempt-3 enqueue failed"));
    Ok(())
}
