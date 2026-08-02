use std::env;

use dubbridge_db::transcription_repo::TranscriptArtifactMeta;
use dubbridge_db::{
    artifact_repo, dubbing_repo, subtitle_repo, transcription_repo, translation_repo,
};
use dubbridge_domain::{
    artifact::{ArtifactRecord, DerivedArtifact, DubbingStatus, TranslationStatus},
    asset::AssetId,
    workspace::{OrgId, ProjectId},
};
use sqlx::PgPool;
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

struct LocalizationScope {
    project_id: ProjectId,
    asset_id: AssetId,
    other_asset_id: AssetId,
    target_language_id: Uuid,
}

async fn insert_scope(pool: &PgPool) -> LocalizationScope {
    let org_id = OrgId(Uuid::new_v4());
    let project_id = ProjectId(Uuid::new_v4());
    let asset_id = AssetId(Uuid::new_v4());
    let other_asset_id = AssetId(Uuid::new_v4());
    let target_language_id = Uuid::new_v4();

    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id.0)
        .bind("Localization Test Org")
        .execute(pool)
        .await
        .expect("insert org");

    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project_id.0)
        .bind(org_id.0)
        .bind("Localization Project")
        .execute(pool)
        .await
        .expect("insert project");

    for (asset_id, title) in [
        (asset_id.0, "localization-asset"),
        (other_asset_id.0, "localization-other-asset"),
    ] {
        sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
            .bind(asset_id)
            .bind(title)
            .bind(Uuid::new_v4())
            .bind("finalized")
            .execute(pool)
            .await
            .expect("insert asset");
    }

    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(project_id.0)
        .bind(asset_id.0)
        .execute(pool)
        .await
        .expect("insert project asset");

    sqlx::query(
        "INSERT INTO target_languages (id, project_id, source_lang, target_lang) VALUES ($1, $2, $3, $4)",
    )
    .bind(target_language_id)
    .bind(project_id.0)
    .bind("en")
    .bind("es")
    .execute(pool)
    .await
    .expect("insert target language");

    LocalizationScope {
        project_id,
        asset_id,
        other_asset_id,
        target_language_id,
    }
}

async fn insert_source_artifact(pool: &PgPool, asset_id: AssetId, label: &str) -> ArtifactRecord {
    let record = ArtifactRecord::new_original(
        asset_id,
        Uuid::new_v4(),
        format!("ingest/{asset_id}/{label}.mp4"),
        "video/mp4".into(),
        1_000,
        format!("checksum-{label}"),
    );
    artifact_repo::insert_artifact_record(pool, &record)
        .await
        .expect("insert source artifact");
    record
}

async fn insert_subtitle_source(pool: &PgPool, asset_id: AssetId, label: &str) -> DerivedArtifact {
    let source = insert_source_artifact(pool, asset_id, label).await;
    let (_transcript, alignment) = transcription_repo::insert_transcript_artifacts(
        pool,
        asset_id,
        source.id,
        TranscriptArtifactMeta {
            storage_key: &format!("transcripts/{asset_id}/{label}.json"),
            size_bytes: 128,
            checksum: &format!("transcript-{label}"),
        },
        TranscriptArtifactMeta {
            storage_key: &format!("alignments/{asset_id}/{label}.json"),
            size_bytes: 64,
            checksum: &format!("alignment-{label}"),
        },
    )
    .await
    .expect("insert transcript+alignment");

    subtitle_repo::insert_subtitle_artifact(
        pool,
        asset_id,
        alignment.id,
        &format!("subtitles/{asset_id}/{label}.vtt"),
        "text/vtt",
        96,
        &format!("subtitle-{label}"),
    )
    .await
    .expect("insert subtitle")
}

async fn insert_translated_source(
    pool: &PgPool,
    asset_id: AssetId,
    label: &str,
) -> DerivedArtifact {
    let subtitle = insert_subtitle_source(pool, asset_id, &format!("{label}-subtitle")).await;
    translation_repo::insert_translated_subtitle_artifact(
        pool,
        asset_id,
        subtitle.id,
        &format!("translated/{asset_id}/{label}.json"),
        "application/json",
        128,
        &format!("translated-{label}"),
    )
    .await
    .expect("insert translated subtitle source")
}

async fn claim_dubbing_generation(
    pool: &PgPool,
    scope: &LocalizationScope,
    source_translated_subtitle_artifact_id: Uuid,
    generation_request_id: Uuid,
) {
    dubbing_repo::claim_dubbing_generation(
        pool,
        dubbing_repo::DubbingClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id,
            source_translated_subtitle_artifact_id,
        },
    )
    .await
    .expect("claim dubbing generation");
}

async fn insert_dubbing_outputs(
    pool: &PgPool,
    asset_id: AssetId,
    source_translated_subtitle_artifact_id: Uuid,
    label: &str,
) -> (DerivedArtifact, DerivedArtifact) {
    let manifest = dubbing_repo::insert_dubbing_manifest_artifact(
        pool,
        asset_id,
        source_translated_subtitle_artifact_id,
        &format!("dubbing/{asset_id}/{label}-manifest.json"),
        "application/json",
        90,
        &format!("{label}-manifest"),
    )
    .await
    .expect("insert manifest");

    let audio = dubbing_repo::insert_dubbed_audio_artifact(
        pool,
        asset_id,
        manifest.id,
        &format!("dubbing/{asset_id}/{label}-audio.mp3"),
        "audio/mpeg",
        390,
        &format!("{label}-audio"),
    )
    .await
    .expect("insert dubbed audio");

    (manifest, audio)
}

async fn assert_dubbing_promote_error(
    pool: &PgPool,
    scope: &LocalizationScope,
    generation_request_id: Uuid,
    manifest_artifact_id: Uuid,
    dubbed_audio_artifact_id: Uuid,
    expected: fn(&dubbridge_db::error::DbError) -> bool,
    message: &str,
) {
    let err = dubbing_repo::promote_dubbing_ready(
        pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        generation_request_id,
        manifest_artifact_id,
        dubbed_audio_artifact_id,
    )
    .await
    .expect_err(message);
    assert!(expected(&err), "unexpected error: {err:?}");
}

async fn count_claim_rows(
    pool: &PgPool,
    operation: &str,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
    generation_request_id: Uuid,
) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM localization_generation_claims
        WHERE operation = $1
          AND project_id = $2
          AND asset_id = $3
          AND target_language_id = $4
          AND generation_request_id = $5
        "#,
    )
    .bind(operation)
    .bind(project_id.0)
    .bind(asset_id.0)
    .bind(target_language_id)
    .bind(generation_request_id)
    .fetch_one(pool)
    .await
    .expect("count claims")
}

#[tokio::test]
async fn translation_claim_and_promote_ready_persists_exact_current_artifacts() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "translation-hp1").await;
    let initial_request_id = Uuid::new_v4();

    let claim = translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: initial_request_id,
            source_subtitle_artifact_id: subtitle.id,
            expected_initial_generation_request_id: initial_request_id,
            mode: translation_repo::TranslationClaimMode::InitialDelivery,
        },
    )
    .await
    .expect("claim translation");

    assert_eq!(claim.source_artifact_id, subtitle.id);

    let status = translation_repo::get_translation_status(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
    )
    .await
    .expect("get status")
    .expect("translation status");
    assert_eq!(status.status, TranslationStatus::InProgress);
    assert_eq!(
        status.current_generation_request_id,
        Some(initial_request_id)
    );
    assert_eq!(status.current_source_artifact_id, Some(subtitle.id));
    assert!(status.current_translated_subtitle_artifact_id.is_none());

    let before_ready = translation_repo::get_translation_readiness_evidence(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
    )
    .await
    .expect("readiness before output");
    assert!(!before_ready.is_ready());

    let translated = translation_repo::insert_translated_subtitle_artifact(
        &pool,
        scope.asset_id,
        subtitle.id,
        &format!("translated/{}/hp1.json", scope.asset_id),
        "application/json",
        150,
        "translated-hp1",
    )
    .await
    .expect("insert translated subtitle");

    let promoted = translation_repo::promote_translation_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        initial_request_id,
        translated.id,
    )
    .await
    .expect("promote ready");

    assert_eq!(promoted.status, TranslationStatus::Ready);
    assert_eq!(
        promoted.current_translated_subtitle_artifact_id,
        Some(translated.id)
    );

    let ready = translation_repo::get_translation_readiness_evidence(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
    )
    .await
    .expect("readiness after promote");
    assert!(ready.is_ready());
    assert_eq!(
        ready
            .current_translated_subtitle
            .as_ref()
            .map(|artifact| artifact.id),
        Some(translated.id)
    );
}

#[tokio::test]
async fn translation_redelivery_same_request_reuses_existing_claim() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "translation-hp3").await;
    let initial_request_id = Uuid::new_v4();

    let first = translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: initial_request_id,
            source_subtitle_artifact_id: subtitle.id,
            expected_initial_generation_request_id: initial_request_id,
            mode: translation_repo::TranslationClaimMode::InitialDelivery,
        },
    )
    .await
    .expect("first claim");

    let translated = translation_repo::insert_translated_subtitle_artifact(
        &pool,
        scope.asset_id,
        subtitle.id,
        &format!("translated/{}/hp3.json", scope.asset_id),
        "application/json",
        120,
        "translated-hp3",
    )
    .await
    .expect("insert translated");

    translation_repo::promote_translation_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        initial_request_id,
        translated.id,
    )
    .await
    .expect("promote ready");

    let second = translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: initial_request_id,
            source_subtitle_artifact_id: subtitle.id,
            expected_initial_generation_request_id: initial_request_id,
            mode: translation_repo::TranslationClaimMode::InitialDelivery,
        },
    )
    .await
    .expect("second claim");

    assert_eq!(second.generation_request_id, first.generation_request_id);
    assert_eq!(second.source_artifact_id, first.source_artifact_id);
    assert_eq!(
        count_claim_rows(
            &pool,
            "translation",
            scope.project_id,
            scope.asset_id,
            scope.target_language_id,
            initial_request_id,
        )
        .await,
        1
    );

    let status = translation_repo::get_translation_status(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
    )
    .await
    .expect("get status")
    .expect("translation status");
    assert_eq!(status.status, TranslationStatus::Ready);
    assert_eq!(
        status.current_translated_subtitle_artifact_id,
        Some(translated.id)
    );
}

#[tokio::test]
async fn translation_claim_rejects_wrong_kind_and_other_asset() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let wrong_kind = insert_source_artifact(&pool, scope.asset_id, "translation-ec1").await;
    let other_asset_subtitle =
        insert_subtitle_source(&pool, scope.other_asset_id, "translation-other").await;

    let wrong_kind_err = translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: Uuid::new_v4(),
            source_subtitle_artifact_id: wrong_kind.id,
            expected_initial_generation_request_id: Uuid::new_v4(),
            mode: translation_repo::TranslationClaimMode::ExplicitRegeneration,
        },
    )
    .await
    .expect_err("wrong source kind must fail");
    assert!(matches!(
        wrong_kind_err,
        dubbridge_db::error::DbError::NotFound
    ));

    let other_asset_err = translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: Uuid::new_v4(),
            source_subtitle_artifact_id: other_asset_subtitle.id,
            expected_initial_generation_request_id: Uuid::new_v4(),
            mode: translation_repo::TranslationClaimMode::ExplicitRegeneration,
        },
    )
    .await
    .expect_err("other asset source must fail");
    assert!(matches!(
        other_asset_err,
        dubbridge_db::error::DbError::NotFound
    ));
}

#[tokio::test]
async fn translation_promote_ready_rejects_wrong_kind_wrong_parent_and_other_asset_outputs() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let subtitle_a = insert_subtitle_source(&pool, scope.asset_id, "translation-promote-a").await;
    let subtitle_b = insert_subtitle_source(&pool, scope.asset_id, "translation-promote-b").await;
    let other_asset_subtitle =
        insert_subtitle_source(&pool, scope.other_asset_id, "translation-promote-other").await;
    let request_id = Uuid::new_v4();

    translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: request_id,
            source_subtitle_artifact_id: subtitle_a.id,
            expected_initial_generation_request_id: request_id,
            mode: translation_repo::TranslationClaimMode::InitialDelivery,
        },
    )
    .await
    .expect("claim translation");

    let wrong_kind_err = translation_repo::promote_translation_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        request_id,
        subtitle_a.id,
    )
    .await
    .expect_err("source subtitle is the wrong output kind");
    assert!(matches!(
        wrong_kind_err,
        dubbridge_db::error::DbError::NotFound
    ));

    let wrong_parent_output = translation_repo::insert_translated_subtitle_artifact(
        &pool,
        scope.asset_id,
        subtitle_b.id,
        &format!("translated/{}/wrong-parent.json", scope.asset_id),
        "application/json",
        120,
        "translation-wrong-parent",
    )
    .await
    .expect("insert wrong-parent translated subtitle");

    let wrong_parent_err = translation_repo::promote_translation_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        request_id,
        wrong_parent_output.id,
    )
    .await
    .expect_err("translated subtitle from another source must fail");
    assert!(matches!(
        wrong_parent_err,
        dubbridge_db::error::DbError::Conflict
    ));

    let other_asset_output = translation_repo::insert_translated_subtitle_artifact(
        &pool,
        scope.other_asset_id,
        other_asset_subtitle.id,
        &format!("translated/{}/other-asset.json", scope.other_asset_id),
        "application/json",
        120,
        "translation-other-asset",
    )
    .await
    .expect("insert other-asset translated subtitle");

    let other_asset_err = translation_repo::promote_translation_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        request_id,
        other_asset_output.id,
    )
    .await
    .expect_err("other asset output must fail");
    assert!(matches!(
        other_asset_err,
        dubbridge_db::error::DbError::NotFound
    ));
}

#[tokio::test]
async fn translation_reused_request_id_with_different_source_conflicts() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let subtitle_a = insert_subtitle_source(&pool, scope.asset_id, "translation-ec5-a").await;
    let subtitle_b = insert_subtitle_source(&pool, scope.asset_id, "translation-ec5-b").await;
    let request_id = Uuid::new_v4();
    let reserved = Uuid::new_v4();

    translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: request_id,
            source_subtitle_artifact_id: subtitle_a.id,
            expected_initial_generation_request_id: reserved,
            mode: translation_repo::TranslationClaimMode::ExplicitRegeneration,
        },
    )
    .await
    .expect("first claim");

    let err = translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: request_id,
            source_subtitle_artifact_id: subtitle_b.id,
            expected_initial_generation_request_id: reserved,
            mode: translation_repo::TranslationClaimMode::ExplicitRegeneration,
        },
    )
    .await
    .expect_err("same request with different source must fail");

    assert!(matches!(err, dubbridge_db::error::DbError::Conflict));
}

#[tokio::test]
async fn translation_explicit_regeneration_cannot_use_reserved_initial_request_id() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "translation-ec6").await;
    let reserved = Uuid::new_v4();

    let err = translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: reserved,
            source_subtitle_artifact_id: subtitle.id,
            expected_initial_generation_request_id: reserved,
            mode: translation_repo::TranslationClaimMode::ExplicitRegeneration,
        },
    )
    .await
    .expect_err("explicit regeneration must reject reserved initial id");

    assert!(matches!(err, dubbridge_db::error::DbError::Conflict));
}

#[tokio::test]
async fn translation_stale_generation_cannot_overwrite_new_current_output() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "translation-ec3").await;
    let initial_request_id = Uuid::new_v4();

    translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: initial_request_id,
            source_subtitle_artifact_id: subtitle.id,
            expected_initial_generation_request_id: initial_request_id,
            mode: translation_repo::TranslationClaimMode::InitialDelivery,
        },
    )
    .await
    .expect("initial claim");

    let stale_output = translation_repo::insert_translated_subtitle_artifact(
        &pool,
        scope.asset_id,
        subtitle.id,
        &format!("translated/{}/stale.json", scope.asset_id),
        "application/json",
        100,
        "stale-output",
    )
    .await
    .expect("insert stale output");

    let regen_request_id = Uuid::new_v4();
    translation_repo::claim_translation_generation(
        &pool,
        translation_repo::TranslationClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: regen_request_id,
            source_subtitle_artifact_id: subtitle.id,
            expected_initial_generation_request_id: initial_request_id,
            mode: translation_repo::TranslationClaimMode::ExplicitRegeneration,
        },
    )
    .await
    .expect("regen claim");

    let stale_err = translation_repo::promote_translation_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        initial_request_id,
        stale_output.id,
    )
    .await
    .expect_err("stale generation must not overwrite current pointers");
    assert!(matches!(stale_err, dubbridge_db::error::DbError::Conflict));

    let current_output = translation_repo::insert_translated_subtitle_artifact(
        &pool,
        scope.asset_id,
        subtitle.id,
        &format!("translated/{}/current.json", scope.asset_id),
        "application/json",
        100,
        "current-output",
    )
    .await
    .expect("insert current output");

    let status = translation_repo::promote_translation_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        regen_request_id,
        current_output.id,
    )
    .await
    .expect("promote current generation");

    assert_eq!(status.status, TranslationStatus::Ready);
    assert_eq!(status.current_generation_request_id, Some(regen_request_id));
    assert_eq!(
        status.current_translated_subtitle_artifact_id,
        Some(current_output.id)
    );
}

#[tokio::test]
async fn dubbing_claim_and_promote_ready_persists_exact_manifest_and_audio() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let translated = insert_translated_source(&pool, scope.asset_id, "dubbing-hp2").await;
    let request_id = Uuid::new_v4();

    let claim = dubbing_repo::claim_dubbing_generation(
        &pool,
        dubbing_repo::DubbingClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: request_id,
            source_translated_subtitle_artifact_id: translated.id,
        },
    )
    .await
    .expect("claim dubbing");

    assert_eq!(claim.source_artifact_id, translated.id);

    let status = dubbing_repo::get_dubbing_status(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
    )
    .await
    .expect("get status")
    .expect("dubbing status");
    assert_eq!(status.status, DubbingStatus::InProgress);
    assert_eq!(status.current_source_artifact_id, Some(translated.id));
    assert!(status.current_manifest_artifact_id.is_none());
    assert!(status.current_dubbed_audio_artifact_id.is_none());

    let before_ready = dubbing_repo::get_dubbing_readiness_evidence(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
    )
    .await
    .expect("readiness before outputs");
    assert!(!before_ready.is_ready());

    let manifest = dubbing_repo::insert_dubbing_manifest_artifact(
        &pool,
        scope.asset_id,
        translated.id,
        &format!("dubbing/{}/manifest.json", scope.asset_id),
        "application/json",
        90,
        "manifest-hp2",
    )
    .await
    .expect("insert manifest");

    let still_not_ready = dubbing_repo::get_dubbing_readiness_evidence(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
    )
    .await
    .expect("readiness with manifest only");
    assert!(!still_not_ready.is_ready());

    let dubbed_audio = dubbing_repo::insert_dubbed_audio_artifact(
        &pool,
        scope.asset_id,
        manifest.id,
        &format!("dubbing/{}/audio.mp3", scope.asset_id),
        "audio/mpeg",
        400,
        "audio-hp2",
    )
    .await
    .expect("insert dubbed audio");

    let promoted = dubbing_repo::promote_dubbing_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        request_id,
        manifest.id,
        dubbed_audio.id,
    )
    .await
    .expect("promote dubbing ready");

    assert_eq!(promoted.status, DubbingStatus::Ready);
    assert_eq!(promoted.current_manifest_artifact_id, Some(manifest.id));
    assert_eq!(
        promoted.current_dubbed_audio_artifact_id,
        Some(dubbed_audio.id)
    );

    let ready = dubbing_repo::get_dubbing_readiness_evidence(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
    )
    .await
    .expect("final readiness");
    assert!(ready.is_ready());
}

#[tokio::test]
async fn dubbing_redelivery_same_request_reuses_existing_claim() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let translated = insert_translated_source(&pool, scope.asset_id, "dubbing-hp3").await;
    let request_id = Uuid::new_v4();

    let first = dubbing_repo::claim_dubbing_generation(
        &pool,
        dubbing_repo::DubbingClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: request_id,
            source_translated_subtitle_artifact_id: translated.id,
        },
    )
    .await
    .expect("first claim");

    let manifest = dubbing_repo::insert_dubbing_manifest_artifact(
        &pool,
        scope.asset_id,
        translated.id,
        &format!("dubbing/{}/hp3-manifest.json", scope.asset_id),
        "application/json",
        100,
        "manifest-hp3",
    )
    .await
    .expect("insert manifest");

    let audio = dubbing_repo::insert_dubbed_audio_artifact(
        &pool,
        scope.asset_id,
        manifest.id,
        &format!("dubbing/{}/hp3-audio.mp3", scope.asset_id),
        "audio/mpeg",
        420,
        "audio-hp3",
    )
    .await
    .expect("insert audio");

    dubbing_repo::promote_dubbing_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        request_id,
        manifest.id,
        audio.id,
    )
    .await
    .expect("promote ready");

    let second = dubbing_repo::claim_dubbing_generation(
        &pool,
        dubbing_repo::DubbingClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: request_id,
            source_translated_subtitle_artifact_id: translated.id,
        },
    )
    .await
    .expect("second claim");

    assert_eq!(second.generation_request_id, first.generation_request_id);
    assert_eq!(second.source_artifact_id, first.source_artifact_id);
    assert_eq!(
        count_claim_rows(
            &pool,
            "dubbing",
            scope.project_id,
            scope.asset_id,
            scope.target_language_id,
            request_id,
        )
        .await,
        1
    );

    let status = dubbing_repo::get_dubbing_status(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
    )
    .await
    .expect("get status")
    .expect("dubbing status");
    assert_eq!(status.status, DubbingStatus::Ready);
    assert_eq!(status.current_manifest_artifact_id, Some(manifest.id));
    assert_eq!(status.current_dubbed_audio_artifact_id, Some(audio.id));
}

#[tokio::test]
async fn dubbing_claim_rejects_wrong_kind_and_other_asset() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let wrong_kind = insert_subtitle_source(&pool, scope.asset_id, "dubbing-ec1").await;
    let other_asset_translated =
        insert_translated_source(&pool, scope.other_asset_id, "dubbing-other").await;

    let wrong_kind_err = dubbing_repo::claim_dubbing_generation(
        &pool,
        dubbing_repo::DubbingClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: Uuid::new_v4(),
            source_translated_subtitle_artifact_id: wrong_kind.id,
        },
    )
    .await
    .expect_err("subtitle source must fail for dubbing claim");
    assert!(matches!(
        wrong_kind_err,
        dubbridge_db::error::DbError::NotFound
    ));

    let other_asset_err = dubbing_repo::claim_dubbing_generation(
        &pool,
        dubbing_repo::DubbingClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: Uuid::new_v4(),
            source_translated_subtitle_artifact_id: other_asset_translated.id,
        },
    )
    .await
    .expect_err("other asset translated subtitle must fail");
    assert!(matches!(
        other_asset_err,
        dubbridge_db::error::DbError::NotFound
    ));
}

#[tokio::test]
async fn dubbing_promote_ready_rejects_wrong_kind_wrong_parent_and_other_asset_outputs() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let translated_a = insert_translated_source(&pool, scope.asset_id, "dubbing-promote-a").await;
    let translated_b = insert_translated_source(&pool, scope.asset_id, "dubbing-promote-b").await;
    let other_asset_translated =
        insert_translated_source(&pool, scope.other_asset_id, "dubbing-promote-other").await;
    let request_id = Uuid::new_v4();

    claim_dubbing_generation(&pool, &scope, translated_a.id, request_id).await;
    assert_dubbing_promote_error(
        &pool,
        &scope,
        request_id,
        translated_a.id,
        translated_a.id,
        |err| matches!(err, dubbridge_db::error::DbError::NotFound),
        "translated subtitle cannot be used as manifest",
    )
    .await;

    let manifest_a = dubbing_repo::insert_dubbing_manifest_artifact(
        &pool,
        scope.asset_id,
        translated_a.id,
        &format!("dubbing/{}/promote-a-manifest.json", scope.asset_id),
        "application/json",
        90,
        "promote-a-manifest",
    )
    .await
    .expect("insert correct manifest");
    let (_manifest_b, wrong_parent_audio) =
        insert_dubbing_outputs(&pool, scope.asset_id, translated_b.id, "promote-b").await;
    assert_dubbing_promote_error(
        &pool,
        &scope,
        request_id,
        manifest_a.id,
        wrong_parent_audio.id,
        |err| matches!(err, dubbridge_db::error::DbError::Conflict),
        "audio from another manifest must fail",
    )
    .await;

    let (other_asset_manifest, other_asset_audio) = insert_dubbing_outputs(
        &pool,
        scope.other_asset_id,
        other_asset_translated.id,
        "other-asset",
    )
    .await;
    assert_dubbing_promote_error(
        &pool,
        &scope,
        request_id,
        other_asset_manifest.id,
        other_asset_audio.id,
        |err| matches!(err, dubbridge_db::error::DbError::NotFound),
        "other-asset manifest/audio must fail",
    )
    .await;
}

#[tokio::test]
async fn dubbing_reused_request_id_with_different_source_conflicts() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let translated_a = insert_translated_source(&pool, scope.asset_id, "dubbing-ec5-a").await;
    let translated_b = insert_translated_source(&pool, scope.asset_id, "dubbing-ec5-b").await;
    let request_id = Uuid::new_v4();

    dubbing_repo::claim_dubbing_generation(
        &pool,
        dubbing_repo::DubbingClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: request_id,
            source_translated_subtitle_artifact_id: translated_a.id,
        },
    )
    .await
    .expect("first claim");

    let err = dubbing_repo::claim_dubbing_generation(
        &pool,
        dubbing_repo::DubbingClaimInput {
            project_id: scope.project_id,
            asset_id: scope.asset_id,
            target_language_id: scope.target_language_id,
            generation_request_id: request_id,
            source_translated_subtitle_artifact_id: translated_b.id,
        },
    )
    .await
    .expect_err("same request with different source must fail");

    assert!(matches!(err, dubbridge_db::error::DbError::Conflict));
}

#[tokio::test]
async fn dubbing_stale_generation_cannot_overwrite_new_current_outputs() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let scope = insert_scope(&pool).await;
    let translated = insert_translated_source(&pool, scope.asset_id, "dubbing-ec3").await;
    let request_a = Uuid::new_v4();
    claim_dubbing_generation(&pool, &scope, translated.id, request_a).await;
    let (manifest_a, audio_a) =
        insert_dubbing_outputs(&pool, scope.asset_id, translated.id, "stale").await;

    let request_b = Uuid::new_v4();
    claim_dubbing_generation(&pool, &scope, translated.id, request_b).await;

    let stale_err = dubbing_repo::promote_dubbing_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        request_a,
        manifest_a.id,
        audio_a.id,
    )
    .await
    .expect_err("stale dubbing generation must fail");
    assert!(matches!(stale_err, dubbridge_db::error::DbError::Conflict));

    let (manifest_b, audio_b) =
        insert_dubbing_outputs(&pool, scope.asset_id, translated.id, "current").await;

    let status = dubbing_repo::promote_dubbing_ready(
        &pool,
        scope.project_id,
        scope.asset_id,
        scope.target_language_id,
        request_b,
        manifest_b.id,
        audio_b.id,
    )
    .await
    .expect("promote B");

    assert_eq!(status.status, DubbingStatus::Ready);
    assert_eq!(status.current_generation_request_id, Some(request_b));
    assert_eq!(status.current_manifest_artifact_id, Some(manifest_b.id));
    assert_eq!(status.current_dubbed_audio_artifact_id, Some(audio_b.id));
}
