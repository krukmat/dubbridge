use std::env;

use dubbridge_db::{artifact_repo, subtitle_repo, target_language_repo};
use dubbridge_domain::{
    artifact::ArtifactRecord,
    asset::AssetId,
    workspace::{ProjectId, TargetLanguage},
};
use sqlx::PgPool;
use time::OffsetDateTime;
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

struct Scope {
    project_id: ProjectId,
    asset_id: AssetId,
    original_id: Uuid,
    subtitle_id: Uuid,
    targets: Vec<TargetLanguage>,
}

async fn seed_scope(pool: &PgPool, codes: &[&str]) -> Scope {
    let project_id = ProjectId::new();
    let asset_id = AssetId::new();
    let org_id = Uuid::new_v4();

    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind("delivery-scope-test-org")
        .execute(pool)
        .await
        .expect("insert organization");

    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project_id.0)
        .bind(org_id)
        .bind("delivery-scope-test-project")
        .execute(pool)
        .await
        .expect("insert project");

    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(asset_id.0)
        .bind("delivery-scope-test-asset")
        .bind(Uuid::new_v4())
        .bind("finalized")
        .execute(pool)
        .await
        .expect("insert asset");

    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(project_id.0)
        .bind(asset_id.0)
        .execute(pool)
        .await
        .expect("insert project_assets");

    let mut targets = Vec::new();
    for (index, code) in codes.iter().enumerate() {
        let lang_id = Uuid::new_v4();
        let created_at = OffsetDateTime::from_unix_timestamp(1700000000 + index as i64)
            .expect("valid timestamp");
        let target = TargetLanguage {
            id: lang_id,
            project_id,
            source_lang: "en".into(),
            target_lang: (*code).into(),
            created_at,
        };
        target_language_repo::upsert_target_language(pool, &target)
            .await
            .expect("insert target language");
        targets.push(target);
    }

    let original = ArtifactRecord::new_original(
        asset_id,
        Uuid::new_v4(),
        format!("delivery-scope/{asset_id}/source.mp4"),
        "video/mp4".into(),
        1,
        format!("source-{asset_id}"),
    );
    artifact_repo::insert_artifact_record(pool, &original)
        .await
        .expect("insert original artifact");

    let subtitle = subtitle_repo::insert_subtitle_artifact(
        pool,
        asset_id,
        original.id,
        &format!("delivery-scope/{asset_id}/subtitle.vtt"),
        "text/vtt",
        1,
        &format!("subtitle-{asset_id}"),
    )
    .await
    .expect("insert subtitle artifact");

    Scope {
        project_id,
        asset_id,
        original_id: original.id,
        subtitle_id: subtitle.id,
        targets,
    }
}

#[tokio::test]
async fn delivery_scope_candidates_decode_all_targets_in_deterministic_order() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let scope = seed_scope(&pool, &["fr", "es", "de"]).await;
    let mut tx = pool.begin().await.expect("begin transaction");

    let candidates = target_language_repo::list_delivery_scope_candidates_tx(
        &mut tx,
        scope.asset_id,
        scope.subtitle_id,
    )
    .await
    .expect("list candidates");

    assert_eq!(candidates.len(), 3);
    for (candidate, expected_code) in candidates.iter().zip(["de", "es", "fr"]) {
        let persisted = scope
            .targets
            .iter()
            .find(|target| target.target_lang == expected_code)
            .expect("persisted target");
        assert_eq!(candidate.project_id, scope.project_id);
        assert_eq!(candidate.target_language.id, persisted.id);
        assert_eq!(candidate.target_language.project_id, persisted.project_id);
        assert_eq!(candidate.target_language.source_lang, persisted.source_lang);
        assert_eq!(candidate.target_language.target_lang, persisted.target_lang);
        assert_eq!(candidate.target_language.created_at, persisted.created_at);
    }

    tx.rollback().await.expect("rollback transaction");
}

#[tokio::test]
async fn delivery_scope_candidates_fail_closed_for_missing_or_mismatched_scope() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let scope = seed_scope(&pool, &["es"]).await;
    let mut tx = pool.begin().await.expect("begin transaction");

    let non_subtitle = target_language_repo::list_delivery_scope_candidates_tx(
        &mut tx,
        scope.asset_id,
        scope.original_id,
    )
    .await
    .expect("query non-subtitle source");
    assert!(non_subtitle.is_empty());

    let mismatched_asset = target_language_repo::list_delivery_scope_candidates_tx(
        &mut tx,
        AssetId(Uuid::new_v4()),
        scope.subtitle_id,
    )
    .await
    .expect("query mismatched asset");
    assert!(mismatched_asset.is_empty());

    sqlx::query("DELETE FROM target_languages WHERE project_id = $1")
        .bind(scope.project_id.0)
        .execute(&mut *tx)
        .await
        .expect("delete target configuration");
    let missing_target = target_language_repo::list_delivery_scope_candidates_tx(
        &mut tx,
        scope.asset_id,
        scope.subtitle_id,
    )
    .await
    .expect("query missing target configuration");
    assert!(missing_target.is_empty());

    tx.rollback().await.expect("rollback transaction");
}
