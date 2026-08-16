use dubbridge_db::translation_delivery_repo::{self, TranslationDeliveryInput};
use dubbridge_db::translation_repo::TranslationClaimMode;
use dubbridge_jobs::{TranslationJob, initial_translation_generation_request_id};
use sqlx::PgPool;
use tracing::{debug, warn};
use uuid::Uuid;

pub async fn fan_out_localization(
    pool: &PgPool,
    asset_id: dubbridge_domain::asset::AssetId,
    word_alignment_parent_artifact_id: Uuid,
) -> Result<Vec<TranslationJob>, String> {
    // 1. Resolve the source subtitle artifact id
    let source_subtitle_artifact_id =
        dubbridge_db::subtitle_repo::find_subtitle_for_asset_and_word_alignment_parent(
            pool,
            asset_id,
            word_alignment_parent_artifact_id,
        )
        .await
        .map_err(|e| e.to_string())?
        .id;

    // 2. List target-language candidates via a short read-only transaction
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let candidates = dubbridge_db::target_language_repo::list_delivery_scope_candidates_tx(
        &mut tx,
        asset_id,
        source_subtitle_artifact_id,
    )
    .await
    .map_err(|e| e.to_string())?;
    tx.rollback().await.map_err(|e| e.to_string())?;

    // 3. Derive generation_request_id
    let generation_request_id =
        initial_translation_generation_request_id(source_subtitle_artifact_id);

    let mut jobs = Vec::new();

    // 4. Iterate through candidates and persist delivery inputs independently
    for candidate in candidates {
        let input = TranslationDeliveryInput {
            project_id: candidate.project_id,
            asset_id,
            target_language_id: candidate.target_language.id,
            generation_request_id,
            source_subtitle_artifact_id,
            expected_initial_generation_request_id: generation_request_id,
            mode: TranslationClaimMode::InitialDelivery,
        };

        let outcome = translation_delivery_repo::persist_translation_delivery(pool, input).await;
        if let Some(job) = job_for_delivery_outcome(
            outcome,
            asset_id,
            candidate.project_id,
            candidate.target_language.id,
            source_subtitle_artifact_id,
            generation_request_id,
        ) {
            jobs.push(job);
        }
    }

    Ok(jobs)
}

/// Whether a delivery dispatch disposition means "queue delivery is due".
fn dispatch_is_due(disposition: translation_delivery_repo::TranslationDispatchDisposition) -> bool {
    matches!(
        disposition,
        translation_delivery_repo::TranslationDispatchDisposition::New
            | translation_delivery_repo::TranslationDispatchDisposition::Retryable
    )
}

fn log_persist_failure(
    asset_id: dubbridge_domain::asset::AssetId,
    target_language_id: Uuid,
    error: &dubbridge_db::error::DbError,
) {
    warn!(
        "Failed to persist translation delivery for asset {:?} and target language {:?}: {:?}",
        asset_id, target_language_id, error
    );
}

/// Map one candidate's `persist_translation_delivery` outcome to an optional
/// `TranslationJob` to enqueue. A dispatch already `Active`/`Acknowledged` is a
/// no-op skip; an `Err` is logged and skipped so one target's persistence
/// failure never aborts or discards sibling targets.
fn job_for_delivery_outcome(
    outcome: Result<
        translation_delivery_repo::TranslationDeliveryPersistence,
        dubbridge_db::error::DbError,
    >,
    asset_id: dubbridge_domain::asset::AssetId,
    project_id: dubbridge_domain::workspace::ProjectId,
    target_language_id: Uuid,
    source_subtitle_artifact_id: Uuid,
    generation_request_id: Uuid,
) -> Option<TranslationJob> {
    let persistence = match outcome {
        Ok(persistence) => persistence,
        Err(e) => {
            log_persist_failure(asset_id, target_language_id, &e);
            return None;
        }
    };

    if !dispatch_is_due(persistence.dispatch) {
        debug!(
            "Skipping target {target_language_id}: dispatch is {:?}",
            persistence.dispatch
        );
        return None;
    }

    Some(TranslationJob::new(
        project_id.0,
        asset_id.0,
        target_language_id,
        source_subtitle_artifact_id,
        generation_request_id,
    ))
}
