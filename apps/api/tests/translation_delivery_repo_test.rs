use std::env;

use dubbridge_db::{
    artifact_repo, subtitle_repo,
    transcription_repo::{self, TranscriptArtifactMeta},
    translation_delivery_repo::{
        self, TranslationDeliveryInput, TranslationDispatchAcknowledgementInput,
        TranslationDispatchAcknowledgementResult, TranslationDispatchDisposition,
        TranslationDispatchFailureInput, TranslationDispatchFailureResult,
    },
    translation_repo::TranslationClaimMode,
};
use dubbridge_domain::{
    artifact::{ArtifactRecord, DerivedArtifact, TranslationStatus},
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

struct Scope {
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
}

async fn insert_scope(pool: &PgPool) -> Scope {
    let org_id = OrgId(Uuid::new_v4());
    let project_id = ProjectId(Uuid::new_v4());
    let asset_id = AssetId(Uuid::new_v4());
    let target_language_id = Uuid::new_v4();

    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id.0)
        .bind("Translation Delivery Test Org")
        .execute(pool)
        .await
        .expect("insert org");
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project_id.0)
        .bind(org_id.0)
        .bind("Translation Delivery Project")
        .execute(pool)
        .await
        .expect("insert project");
    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(asset_id.0)
        .bind("translation-delivery-asset")
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
        .expect("insert project asset");
    insert_target_language(pool, project_id, target_language_id, "es").await;

    Scope {
        project_id,
        asset_id,
        target_language_id,
    }
}

async fn insert_target_language(
    pool: &PgPool,
    project_id: ProjectId,
    target_language_id: Uuid,
    target_lang: &str,
) {
    sqlx::query(
        "INSERT INTO target_languages (id, project_id, source_lang, target_lang) VALUES ($1, $2, $3, $4)",
    )
    .bind(target_language_id)
    .bind(project_id.0)
    .bind("en")
    .bind(target_lang)
    .execute(pool)
    .await
    .expect("insert target language");
}

async fn insert_original_artifact(pool: &PgPool, asset_id: AssetId, label: &str) -> ArtifactRecord {
    let artifact = ArtifactRecord::new_original(
        asset_id,
        Uuid::new_v4(),
        format!("ingest/{asset_id}/{label}.mp4"),
        "video/mp4".into(),
        1_000,
        format!("checksum-{label}"),
    );
    artifact_repo::insert_artifact_record(pool, &artifact)
        .await
        .expect("insert original");
    artifact
}

async fn insert_subtitle_source(pool: &PgPool, asset_id: AssetId, label: &str) -> DerivedArtifact {
    let source = insert_original_artifact(pool, asset_id, label).await;
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
    .expect("insert transcript and alignment");

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

fn delivery_input(
    scope: &Scope,
    target_language_id: Uuid,
    source_subtitle_artifact_id: Uuid,
    generation_request_id: Uuid,
) -> TranslationDeliveryInput {
    TranslationDeliveryInput {
        project_id: scope.project_id,
        asset_id: scope.asset_id,
        target_language_id,
        generation_request_id,
        source_subtitle_artifact_id,
        expected_initial_generation_request_id: generation_request_id,
        mode: TranslationClaimMode::InitialDelivery,
    }
}

fn failure_input(
    scope: &Scope,
    target_language_id: Uuid,
    generation_request_id: Uuid,
) -> TranslationDispatchFailureInput {
    TranslationDispatchFailureInput {
        project_id: scope.project_id,
        asset_id: scope.asset_id,
        target_language_id,
        generation_request_id,
        error_detail: "test-failure".to_string(),
    }
}

fn acknowledgement_input(
    scope: &Scope,
    target_language_id: Uuid,
    generation_request_id: Uuid,
) -> TranslationDispatchAcknowledgementInput {
    TranslationDispatchAcknowledgementInput {
        project_id: scope.project_id,
        asset_id: scope.asset_id,
        target_language_id,
        generation_request_id,
    }
}

async fn count_rows(pool: &PgPool, table: &str, input: TranslationDeliveryInput) -> i64 {
    let column = if table == "localization_generation_claims" {
        "localization_generation_claims"
    } else {
        "translation_dispatch_outbox"
    };
    sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM {column} WHERE operation = $1 AND project_id = $2 AND asset_id = $3 AND target_language_id = $4 AND generation_request_id = $5"
    ))
    .bind("translation")
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .fetch_one(pool)
    .await
    .expect("count persisted rows")
}

async fn set_dispatch_state(pool: &PgPool, input: TranslationDeliveryInput, state: &str) {
    sqlx::query(
        r#"
        UPDATE translation_dispatch_outbox
        SET delivery_state = $1, updated_at = now()
        WHERE operation = 'translation'
          AND project_id = $2
          AND asset_id = $3
          AND target_language_id = $4
          AND generation_request_id = $5
        "#,
    )
    .bind(state)
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .execute(pool)
    .await
    .expect("set dispatch state");
}

#[tokio::test]
async fn persistence_creates_one_claim_and_pending_dispatch_for_each_selected_target() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let french_target = Uuid::new_v4();
    insert_target_language(&pool, scope.project_id, french_target, "fr").await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "fanout").await;
    let request_id = Uuid::new_v4();

    for target_language_id in [scope.target_language_id, french_target] {
        let input = delivery_input(&scope, target_language_id, subtitle.id, request_id);
        let persisted = translation_delivery_repo::persist_translation_delivery(&pool, input)
            .await
            .expect("persist selected target delivery");
        assert_eq!(persisted.claim.source_artifact_id, subtitle.id);
        assert_eq!(persisted.dispatch, TranslationDispatchDisposition::New);
        assert_eq!(
            count_rows(&pool, "localization_generation_claims", input).await,
            1
        );
        assert_eq!(
            count_rows(&pool, "translation_dispatch_outbox", input).await,
            1
        );
    }

    let status = dubbridge_db::translation_repo::get_translation_status(
        &pool,
        scope.project_id,
        scope.asset_id,
        french_target,
    )
    .await
    .expect("get status")
    .expect("status for selected target");
    assert_eq!(status.status, TranslationStatus::InProgress);
    assert_eq!(status.current_source_artifact_id, Some(subtitle.id));
}

#[tokio::test]
async fn redelivery_classifies_existing_dispatch_state_without_mutation() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "redelivery").await;
    let input = delivery_input(
        &scope,
        scope.target_language_id,
        subtitle.id,
        Uuid::new_v4(),
    );

    assert_eq!(
        translation_delivery_repo::persist_translation_delivery(&pool, input)
            .await
            .expect("first persistence")
            .dispatch,
        TranslationDispatchDisposition::New
    );
    assert_eq!(
        translation_delivery_repo::persist_translation_delivery(&pool, input)
            .await
            .expect("pending redelivery")
            .dispatch,
        TranslationDispatchDisposition::Active
    );

    set_dispatch_state(&pool, input, "enqueue_failed").await;
    assert_eq!(
        translation_delivery_repo::persist_translation_delivery(&pool, input)
            .await
            .expect("retryable redelivery")
            .dispatch,
        TranslationDispatchDisposition::Retryable
    );

    set_dispatch_state(&pool, input, "acknowledged").await;
    assert_eq!(
        translation_delivery_repo::persist_translation_delivery(&pool, input)
            .await
            .expect("acknowledged redelivery")
            .dispatch,
        TranslationDispatchDisposition::Acknowledged
    );
    assert_eq!(
        count_rows(&pool, "localization_generation_claims", input).await,
        1
    );
    assert_eq!(
        count_rows(&pool, "translation_dispatch_outbox", input).await,
        1
    );
}

#[tokio::test]
async fn invalid_requested_scope_fails_before_claim_or_dispatch_write() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "invalid-scope").await;
    let input = TranslationDeliveryInput {
        project_id: ProjectId(Uuid::new_v4()),
        ..delivery_input(
            &scope,
            scope.target_language_id,
            subtitle.id,
            Uuid::new_v4(),
        )
    };

    let err = translation_delivery_repo::persist_translation_delivery(&pool, input)
        .await
        .expect_err("wrong project must fail closed");
    assert!(matches!(err, dubbridge_db::error::DbError::NotFound));
    assert_eq!(
        count_rows(&pool, "localization_generation_claims", input).await,
        0
    );
    assert_eq!(
        count_rows(&pool, "translation_dispatch_outbox", input).await,
        0
    );

    let unknown_target = delivery_input(&scope, Uuid::new_v4(), subtitle.id, Uuid::new_v4());
    let err = translation_delivery_repo::persist_translation_delivery(&pool, unknown_target)
        .await
        .expect_err("unknown target must fail closed");
    assert!(matches!(err, dubbridge_db::error::DbError::NotFound));
    assert_eq!(
        count_rows(&pool, "localization_generation_claims", unknown_target).await,
        0
    );
    assert_eq!(
        count_rows(&pool, "translation_dispatch_outbox", unknown_target).await,
        0
    );

    let original = insert_original_artifact(&pool, scope.asset_id, "wrong-kind").await;
    let wrong_kind = delivery_input(
        &scope,
        scope.target_language_id,
        original.id,
        Uuid::new_v4(),
    );
    let err = translation_delivery_repo::persist_translation_delivery(&pool, wrong_kind)
        .await
        .expect_err("non-subtitle source must fail closed");
    assert!(matches!(err, dubbridge_db::error::DbError::NotFound));
    assert_eq!(
        count_rows(&pool, "localization_generation_claims", wrong_kind).await,
        0
    );
    assert_eq!(
        count_rows(&pool, "translation_dispatch_outbox", wrong_kind).await,
        0
    );
}

#[tokio::test]
async fn same_generation_with_different_source_rolls_back_without_second_dispatch() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let first_source = insert_subtitle_source(&pool, scope.asset_id, "conflict-first").await;
    let conflicting_source = insert_subtitle_source(&pool, scope.asset_id, "conflict-second").await;
    let request_id = Uuid::new_v4();
    let first = delivery_input(
        &scope,
        scope.target_language_id,
        first_source.id,
        request_id,
    );
    translation_delivery_repo::persist_translation_delivery(&pool, first)
        .await
        .expect("persist first source");
    let conflicting = delivery_input(
        &scope,
        scope.target_language_id,
        conflicting_source.id,
        request_id,
    );

    let err = translation_delivery_repo::persist_translation_delivery(&pool, conflicting)
        .await
        .expect_err("different source must conflict");
    assert!(matches!(err, dubbridge_db::error::DbError::Conflict));
    assert_eq!(
        count_rows(&pool, "localization_generation_claims", first).await,
        1
    );
    assert_eq!(
        count_rows(&pool, "translation_dispatch_outbox", first).await,
        1
    );
}

#[tokio::test]
async fn enqueue_failure_marks_pending_returns_marked_and_sets_error_detail() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "fail-mark").await;
    let request_id = Uuid::new_v4();
    let input = delivery_input(&scope, scope.target_language_id, subtitle.id, request_id);

    translation_delivery_repo::persist_translation_delivery(&pool, input)
        .await
        .expect("create pending dispatch");

    let fail_input = failure_input(&scope, scope.target_language_id, request_id);
    assert_eq!(
        translation_delivery_repo::translation_dispatch_enqueue_failure(&pool, fail_input.clone())
            .await
            .expect("mark enqueue_failed"),
        TranslationDispatchFailureResult::Marked
    );

    let updated: String = sqlx::query_scalar(
        r#"SELECT delivery_state FROM translation_dispatch_outbox
           WHERE operation = $1 AND project_id = $2 AND asset_id = $3
             AND target_language_id = $4 AND generation_request_id = $5"#,
    )
    .bind("translation")
    .bind(scope.project_id.0)
    .bind(scope.asset_id.0)
    .bind(scope.target_language_id)
    .bind(request_id)
    .fetch_one(&pool)
    .await
    .expect("read back state");
    assert_eq!(updated, "enqueue_failed");

    let error_detail: String = sqlx::query_scalar(
        r#"SELECT error_detail FROM translation_dispatch_outbox
           WHERE operation = $1 AND project_id = $2 AND asset_id = $3
             AND target_language_id = $4 AND generation_request_id = $5"#,
    )
    .bind("translation")
    .bind(scope.project_id.0)
    .bind(scope.asset_id.0)
    .bind(scope.target_language_id)
    .bind(request_id)
    .fetch_one(&pool)
    .await
    .expect("read back error_detail");
    assert_eq!(error_detail, "test-failure");
}

#[tokio::test]
async fn enqueue_failure_returns_already_failed_when_enqueue_failed() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "fail-already").await;
    let request_id = Uuid::new_v4();
    let fail_input = failure_input(&scope, scope.target_language_id, request_id);
    translation_delivery_repo::persist_translation_delivery(
        &pool,
        delivery_input(&scope, scope.target_language_id, subtitle.id, request_id),
    )
    .await
    .expect("create pending dispatch");
    assert_eq!(
        translation_delivery_repo::translation_dispatch_enqueue_failure(&pool, fail_input.clone())
            .await
            .expect("mark enqueue failed"),
        TranslationDispatchFailureResult::Marked
    );

    assert_eq!(
        translation_delivery_repo::translation_dispatch_enqueue_failure(&pool, fail_input.clone())
            .await
            .expect("already failed"),
        TranslationDispatchFailureResult::AlreadyFailed
    );
}

#[tokio::test]
async fn enqueue_failure_returns_rejected_when_acknowledged() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "fail-reject").await;
    let request_id = Uuid::new_v4();
    let fail_input = failure_input(&scope, scope.target_language_id, request_id);
    let input = delivery_input(&scope, scope.target_language_id, subtitle.id, request_id);
    translation_delivery_repo::persist_translation_delivery(&pool, input)
        .await
        .expect("create pending dispatch");
    translation_delivery_repo::translation_dispatch_acknowledge(
        &pool,
        acknowledgement_input(&scope, scope.target_language_id, request_id),
    )
    .await
    .expect("acknowledge dispatch");

    assert_eq!(
        translation_delivery_repo::translation_dispatch_enqueue_failure(&pool, fail_input)
            .await
            .expect("rejected"),
        TranslationDispatchFailureResult::Rejected
    );
    let state: String = sqlx::query_scalar(
        r#"SELECT delivery_state FROM translation_dispatch_outbox
           WHERE operation = $1 AND project_id = $2 AND asset_id = $3
             AND target_language_id = $4 AND generation_request_id = $5"#,
    )
    .bind("translation")
    .bind(scope.project_id.0)
    .bind(scope.asset_id.0)
    .bind(scope.target_language_id)
    .bind(request_id)
    .fetch_one(&pool)
    .await
    .expect("read back acknowledged state");
    assert_eq!(state, "acknowledged");
}

#[tokio::test]
async fn enqueue_failure_returns_not_found_for_absent_identity() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let fail_input = TranslationDispatchFailureInput {
        project_id: ProjectId(Uuid::new_v4()),
        asset_id: scope.asset_id,
        target_language_id: scope.target_language_id,
        generation_request_id: Uuid::new_v4(),
        error_detail: "no-op".to_string(),
    };

    assert_eq!(
        translation_delivery_repo::translation_dispatch_enqueue_failure(&pool, fail_input)
            .await
            .expect("not found"),
        TranslationDispatchFailureResult::NotFound
    );
}

#[tokio::test]
async fn acknowledgement_marks_pending_dispatch_and_is_idempotent() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "ack-mark").await;
    let request_id = Uuid::new_v4();
    let input = delivery_input(&scope, scope.target_language_id, subtitle.id, request_id);
    translation_delivery_repo::persist_translation_delivery(&pool, input)
        .await
        .expect("create pending dispatch");

    let acknowledgement = acknowledgement_input(&scope, scope.target_language_id, request_id);
    assert_eq!(
        translation_delivery_repo::translation_dispatch_acknowledge(&pool, acknowledgement)
            .await
            .expect("mark acknowledged"),
        TranslationDispatchAcknowledgementResult::Marked
    );
    assert_eq!(
        translation_delivery_repo::translation_dispatch_acknowledge(&pool, acknowledgement)
            .await
            .expect("duplicate acknowledgement"),
        TranslationDispatchAcknowledgementResult::AlreadyAcknowledged
    );

    let state: String = sqlx::query_scalar(
        r#"SELECT delivery_state FROM translation_dispatch_outbox
           WHERE operation = $1 AND project_id = $2 AND asset_id = $3
             AND target_language_id = $4 AND generation_request_id = $5"#,
    )
    .bind("translation")
    .bind(scope.project_id.0)
    .bind(scope.asset_id.0)
    .bind(scope.target_language_id)
    .bind(request_id)
    .fetch_one(&pool)
    .await
    .expect("read back acknowledged state");
    assert_eq!(state, "acknowledged");
}

#[tokio::test]
async fn acknowledgement_rejects_failed_dispatch_without_mutation() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "ack-reject").await;
    let request_id = Uuid::new_v4();
    let input = delivery_input(&scope, scope.target_language_id, subtitle.id, request_id);
    translation_delivery_repo::persist_translation_delivery(&pool, input)
        .await
        .expect("create pending dispatch");
    translation_delivery_repo::translation_dispatch_enqueue_failure(
        &pool,
        failure_input(&scope, scope.target_language_id, request_id),
    )
    .await
    .expect("mark enqueue failed");

    assert_eq!(
        translation_delivery_repo::translation_dispatch_acknowledge(
            &pool,
            acknowledgement_input(&scope, scope.target_language_id, request_id),
        )
        .await
        .expect("reject acknowledgement after failure"),
        TranslationDispatchAcknowledgementResult::Rejected
    );
}

#[tokio::test]
async fn acknowledgement_returns_not_found_for_distinct_dispatch_identity() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    let scope = insert_scope(&pool).await;
    let subtitle = insert_subtitle_source(&pool, scope.asset_id, "ack-not-found").await;
    let request_id = Uuid::new_v4();
    translation_delivery_repo::persist_translation_delivery(
        &pool,
        delivery_input(&scope, scope.target_language_id, subtitle.id, request_id),
    )
    .await
    .expect("create sibling dispatch");

    assert_eq!(
        translation_delivery_repo::translation_dispatch_acknowledge(
            &pool,
            acknowledgement_input(&scope, scope.target_language_id, Uuid::new_v4()),
        )
        .await
        .expect("distinct identity is not found"),
        TranslationDispatchAcknowledgementResult::NotFound
    );
}
