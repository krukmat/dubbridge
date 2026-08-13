//! S-150-T2b-ii-b: atomically persist one caller-selected translation delivery.

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use dubbridge_domain::{asset::AssetId, workspace::ProjectId};

use crate::{
    error::DbError,
    target_language_repo::list_delivery_scope_candidates_tx,
    translation_repo::{
        TranslationClaimInput, TranslationClaimMode, TranslationGenerationClaim,
        claim_translation_generation_tx,
    },
};

const OPERATION: &str = "translation";

#[derive(Debug, Clone, Copy)]
pub struct TranslationDeliveryInput {
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub generation_request_id: Uuid,
    pub source_subtitle_artifact_id: Uuid,
    pub expected_initial_generation_request_id: Uuid,
    pub mode: TranslationClaimMode,
}

/// The persisted dispatch meaning used by T2c to decide whether queue work is due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationDispatchDisposition {
    New,
    Retryable,
    Active,
    Acknowledged,
}

#[derive(Debug, Clone)]
pub struct TranslationDeliveryPersistence {
    pub claim: TranslationGenerationClaim,
    pub dispatch: TranslationDispatchDisposition,
}

#[derive(sqlx::FromRow)]
struct DispatchRow {
    delivery_state: String,
}

fn dispatch_disposition(state: &str) -> Result<TranslationDispatchDisposition, DbError> {
    match state {
        "enqueue_failed" => Ok(TranslationDispatchDisposition::Retryable),
        "pending" => Ok(TranslationDispatchDisposition::Active),
        "acknowledged" => Ok(TranslationDispatchDisposition::Acknowledged),
        other => Err(DbError::UnknownStoredValue {
            field: "translation_dispatch_outbox.delivery_state",
            value: other.to_owned(),
        }),
    }
}

async fn insert_dispatch_if_absent_tx(
    tx: &mut Transaction<'_, Postgres>,
    input: TranslationDeliveryInput,
) -> Result<bool, DbError> {
    let inserted: Option<String> = sqlx::query_scalar(
        r#"
        INSERT INTO translation_dispatch_outbox (
            operation, project_id, asset_id, target_language_id, generation_request_id,
            delivery_state, error_detail
        )
        VALUES ($1, $2, $3, $4, $5, 'pending', NULL)
        ON CONFLICT DO NOTHING
        RETURNING delivery_state
        "#,
    )
    .bind(OPERATION)
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(inserted.is_some())
}

async fn get_dispatch_tx(
    tx: &mut Transaction<'_, Postgres>,
    input: TranslationDeliveryInput,
) -> Result<DispatchRow, DbError> {
    sqlx::query_as(
        r#"
        SELECT delivery_state
        FROM translation_dispatch_outbox
        WHERE operation = $1
          AND project_id = $2
          AND asset_id = $3
          AND target_language_id = $4
          AND generation_request_id = $5
        "#,
    )
    .bind(OPERATION)
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)
}

// ── enqueue-failure repair dispatch ──────────────────────────────────────

/// Public failure-input carrying the exact dispatch identity.
#[derive(Debug, Clone)]
pub struct TranslationDispatchFailureInput {
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub generation_request_id: Uuid,
    pub error_detail: String,
}

/// Public result contract distinguishing all outcomes of the repair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationDispatchFailureResult {
    Marked,
    AlreadyFailed,
    Rejected,
    NotFound,
}

/// Public acknowledgement input carrying the exact dispatch identity.
#[derive(Debug, Clone, Copy)]
pub struct TranslationDispatchAcknowledgementInput {
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub generation_request_id: Uuid,
}

/// Public result contract distinguishing all acknowledgement outcomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationDispatchAcknowledgementResult {
    Marked,
    AlreadyAcknowledged,
    Rejected,
    NotFound,
}

async fn update_enqueue_failed_tx(
    tx: &mut Transaction<'_, Postgres>,
    input: &TranslationDispatchFailureInput,
) -> Result<bool, DbError> {
    let updated = sqlx::query(
        r#"
        UPDATE translation_dispatch_outbox
        SET delivery_state = 'enqueue_failed',
            error_detail = $2,
            updated_at = now()
        WHERE operation = $1
          AND project_id = $3
          AND asset_id = $4
          AND target_language_id = $5
          AND generation_request_id = $6
          AND delivery_state = 'pending'
        RETURNING 1
        "#,
    )
    .bind(OPERATION)
    .bind(&input.error_detail)
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(!updated.is_empty())
}

async fn update_acknowledged_tx(
    tx: &mut Transaction<'_, Postgres>,
    input: TranslationDispatchAcknowledgementInput,
) -> Result<bool, DbError> {
    let updated = sqlx::query(
        r#"
        UPDATE translation_dispatch_outbox
        SET delivery_state = 'acknowledged',
            updated_at = now()
        WHERE operation = $1
          AND project_id = $2
          AND asset_id = $3
          AND target_language_id = $4
          AND generation_request_id = $5
          AND delivery_state = 'pending'
        RETURNING 1
        "#,
    )
    .bind(OPERATION)
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(!updated.is_empty())
}

/// Atomically acknowledge a pending dispatch, or inspect the same row.
pub async fn translation_dispatch_acknowledge(
    pool: &PgPool,
    input: TranslationDispatchAcknowledgementInput,
) -> Result<TranslationDispatchAcknowledgementResult, DbError> {
    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;

    if update_acknowledged_tx(&mut tx, input).await? {
        tx.commit().await.map_err(DbError::QueryFailed)?;
        return Ok(TranslationDispatchAcknowledgementResult::Marked);
    }

    let maybe_state: Option<String> = sqlx::query_scalar(
        r#"
        SELECT delivery_state
        FROM translation_dispatch_outbox
        WHERE operation = $1
          AND project_id = $2
          AND asset_id = $3
          AND target_language_id = $4
          AND generation_request_id = $5
        "#,
    )
    .bind(OPERATION)
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;

    let result = match maybe_state.as_deref() {
        Some("acknowledged") => TranslationDispatchAcknowledgementResult::AlreadyAcknowledged,
        Some("enqueue_failed") => TranslationDispatchAcknowledgementResult::Rejected,
        None => TranslationDispatchAcknowledgementResult::NotFound,
        Some(other) => {
            return Err(DbError::UnknownStoredValue {
                field: "translation_dispatch_outbox.delivery_state",
                value: other.to_owned(),
            });
        }
    };

    tx.commit().await.map_err(DbError::QueryFailed)?;
    Ok(result)
}

/// Atomically mark a pending dispatch as enqueue_failed, or inspect the same row.
pub async fn translation_dispatch_enqueue_failure(
    pool: &PgPool,
    input: TranslationDispatchFailureInput,
) -> Result<TranslationDispatchFailureResult, DbError> {
    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;

    let updated_rows = update_enqueue_failed_tx(&mut tx, &input).await?;

    if updated_rows {
        tx.commit().await.map_err(DbError::QueryFailed)?;
        return Ok(TranslationDispatchFailureResult::Marked);
    }

    // No update means the row was not pending; inspect exact identity.
    let maybe_state: Option<String> = sqlx::query_scalar(
        r#"
        SELECT delivery_state
        FROM translation_dispatch_outbox
        WHERE operation = $1
          AND project_id = $2
          AND asset_id = $3
          AND target_language_id = $4
          AND generation_request_id = $5
        "#,
    )
    .bind(OPERATION)
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?;

    let delivery_state = match maybe_state {
        Some(s) => s,
        None => {
            tx.commit().await.map_err(DbError::QueryFailed)?;
            return Ok(TranslationDispatchFailureResult::NotFound);
        }
    };

    tx.commit().await.map_err(DbError::QueryFailed)?;

    match delivery_state.as_str() {
        "enqueue_failed" => Ok(TranslationDispatchFailureResult::AlreadyFailed),
        "acknowledged" => Ok(TranslationDispatchFailureResult::Rejected),
        _ => Err(DbError::UnknownStoredValue {
            field: "translation_dispatch_outbox.delivery_state",
            value: delivery_state,
        }),
    }
}

/// Persist or reuse one configured target-language delivery in one transaction.
///
/// This is intentionally one target at a time. T2c owns fan-out and calls this
/// API for each persisted candidate; this boundary never selects a first target.
pub async fn persist_translation_delivery(
    pool: &PgPool,
    input: TranslationDeliveryInput,
) -> Result<TranslationDeliveryPersistence, DbError> {
    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;

    let candidates = list_delivery_scope_candidates_tx(
        &mut tx,
        input.asset_id,
        input.source_subtitle_artifact_id,
    )
    .await?;
    let scope_matches = candidates.into_iter().any(|candidate| {
        candidate.project_id == input.project_id
            && candidate.target_language.id == input.target_language_id
    });
    if !scope_matches {
        return Err(DbError::NotFound);
    }

    let claim = claim_translation_generation_tx(
        &mut tx,
        TranslationClaimInput {
            project_id: input.project_id,
            asset_id: input.asset_id,
            target_language_id: input.target_language_id,
            generation_request_id: input.generation_request_id,
            source_subtitle_artifact_id: input.source_subtitle_artifact_id,
            expected_initial_generation_request_id: input.expected_initial_generation_request_id,
            mode: input.mode,
        },
    )
    .await?;

    let dispatch = if insert_dispatch_if_absent_tx(&mut tx, input).await? {
        TranslationDispatchDisposition::New
    } else {
        dispatch_disposition(&get_dispatch_tx(&mut tx, input).await?.delivery_state)?
    };

    tx.commit().await.map_err(DbError::QueryFailed)?;
    Ok(TranslationDeliveryPersistence { claim, dispatch })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_disposition_is_explicit_and_fail_closed() {
        assert_eq!(
            dispatch_disposition("enqueue_failed").unwrap(),
            TranslationDispatchDisposition::Retryable
        );
        assert_eq!(
            dispatch_disposition("pending").unwrap(),
            TranslationDispatchDisposition::Active
        );
        assert_eq!(
            dispatch_disposition("acknowledged").unwrap(),
            TranslationDispatchDisposition::Acknowledged
        );
        assert!(matches!(
            dispatch_disposition("unknown"),
            Err(DbError::UnknownStoredValue {
                field: "translation_dispatch_outbox.delivery_state",
                ..
            })
        ));
    }
}
