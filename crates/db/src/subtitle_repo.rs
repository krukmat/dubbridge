// S-140-T1c-ii: repository for subtitle status and subtitle derived artifacts.
//
// Duplicate artifact rejection relies on the partial unique index from
// migration 0025 (artifact_records_subtitle_unique_asset_parent) rather than
// a check-then-insert, so concurrent inserts for the same
// (asset_id, parent_artifact_id) cannot both succeed (no TOCTOU window).
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::artifact::{
    ArtifactKind, DerivedArtifact, SubtitleStatus, SubtitleStatusRecord,
};
use dubbridge_domain::asset::AssetId;

use crate::error::DbError;

fn parse_subtitle_status(s: &str) -> Result<SubtitleStatus, DbError> {
    match s {
        "pending" => Ok(SubtitleStatus::Pending),
        "in_progress" => Ok(SubtitleStatus::InProgress),
        "ready" => Ok(SubtitleStatus::Ready),
        "failed" => Ok(SubtitleStatus::Failed),
        other => Err(DbError::UnknownStoredValue {
            field: "asset_subtitle_status.status",
            value: other.to_owned(),
        }),
    }
}

#[derive(sqlx::FromRow)]
struct SubtitleStatusRow {
    asset_id: Uuid,
    status: String,
    error_detail: Option<String>,
    updated_at: OffsetDateTime,
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .as_deref()
        == Some("23505")
}

/// Upsert the subtitle status for an asset (Pending/InProgress/Ready/Failed).
/// Mirrors `transcription_repo::upsert_transcription_status`. This is
/// distinct from the one-shot `Subtitle` derived-artifact row: status
/// transitions never touch `artifact_records`.
pub async fn upsert_subtitle_status(
    pool: &PgPool,
    asset_id: AssetId,
    status: SubtitleStatus,
    error_detail: Option<&str>,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO asset_subtitle_status (asset_id, status, error_detail, updated_at)
        VALUES ($1, $2, $3, now())
        ON CONFLICT (asset_id) DO UPDATE
            SET status       = EXCLUDED.status,
                error_detail = EXCLUDED.error_detail,
                updated_at   = EXCLUDED.updated_at
        "#,
    )
    .bind(asset_id.0)
    .bind(status.to_string())
    .bind(error_detail)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

/// Return the current subtitle status for an asset, or `None` if not yet initialised.
pub async fn get_subtitle_status(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<Option<SubtitleStatusRecord>, DbError> {
    let row = sqlx::query_as::<_, SubtitleStatusRow>(
        r#"
        SELECT asset_id, status, error_detail, updated_at
        FROM asset_subtitle_status
        WHERE asset_id = $1
        "#,
    )
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(|r| {
        Ok(SubtitleStatusRecord {
            asset_id: AssetId(r.asset_id),
            status: parse_subtitle_status(&r.status)?,
            error_detail: r.error_detail,
            updated_at: r.updated_at,
        })
    })
    .transpose()
}

/// Atomically claim `SubtitleStatus::Pending` for the caller that gets to enqueue.
///
/// Returns `true` only when the row was absent or previously `Failed`.
/// Returns `false` when the asset is already `Pending`, `InProgress`, or `Ready`.
pub async fn try_claim_subtitle_pending(pool: &PgPool, asset_id: AssetId) -> Result<bool, DbError> {
    let claimed: Option<Uuid> = sqlx::query_scalar(
        r#"
        INSERT INTO asset_subtitle_status (asset_id, status, error_detail, updated_at)
        VALUES ($1, 'pending', NULL, now())
        ON CONFLICT (asset_id) DO UPDATE
            SET status = 'pending',
                error_detail = NULL,
                updated_at = now()
        WHERE asset_subtitle_status.status = 'failed'
        RETURNING asset_id
        "#,
    )
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(claimed.is_some())
}

/// Persist a `Subtitle` derived artifact linked to `parent_artifact_id`
/// (the asset's `WordAlignment` artifact, per D1a).
///
/// Returns `DbError::Conflict` if a subtitle artifact already exists for this
/// `(asset_id, parent_artifact_id)` pair (migration 0025's unique index,
/// SQLSTATE 23505). Returns `DbError::QueryFailed` for any other failure,
/// including a `parent_artifact_id` that does not reference an existing
/// artifact row (SQLSTATE 23503) — there is no dedicated foreign-key-violation
/// variant in this crate; the wrapped `sqlx::Error` still carries the
/// SQLSTATE detail for callers who need to distinguish it.
pub async fn insert_subtitle_artifact(
    pool: &PgPool,
    asset_id: AssetId,
    parent_artifact_id: Uuid,
    storage_key: &str,
    content_type: &str,
    size_bytes: i64,
    checksum: &str,
) -> Result<DerivedArtifact, DbError> {
    let artifact = DerivedArtifact::new(
        asset_id,
        parent_artifact_id,
        ArtifactKind::Subtitle,
        storage_key.to_string(),
        content_type.to_string(),
        size_bytes,
        checksum.to_string(),
    );

    sqlx::query(
        r#"
        INSERT INTO artifact_records
            (id, asset_id, kind, parent_artifact_id, storage_key, content_type, size_bytes, checksum, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(artifact.id)
    .bind(artifact.asset_id.0)
    .bind(artifact.kind.to_string())
    .bind(artifact.parent_artifact_id)
    .bind(&artifact.storage_key)
    .bind(&artifact.content_type)
    .bind(artifact.size_bytes)
    .bind(&artifact.checksum)
    .bind(artifact.created_at)
    .execute(pool)
    .await
    .map_err(|error| {
        if is_unique_violation(&error) {
            DbError::Conflict
        } else {
            DbError::QueryFailed(error)
        }
    })?;

    Ok(artifact)
}

/// Return `true` only when a `Subtitle` derived artifact exists for the asset
/// AND the persisted subtitle status is `Ready` (artifact existence alone is
/// not sufficient). Fail-closed per ADR-018: any DB error propagates.
///
/// `asset_subtitle_status` has one row per `asset_id` (migration 0024,
/// `PRIMARY KEY (asset_id)`) — it tracks a single generation cycle, not a
/// status per subtitle artifact/language. If an asset ever has more than one
/// `Subtitle` artifact (e.g. distinct `WordAlignment` parents for different
/// languages, as `insert_subtitle_artifact` structurally allows), this
/// function cannot distinguish "this specific artifact is ready" from "some
/// other artifact exists while the shared asset-level status says Ready" —
/// per-parent readiness would require per-parent status tracking, which is
/// out of scope for this task; multi-language subtitle generation is not
/// exercised anywhere else in this slice yet either. A timestamp-ordering
/// guard (artifact `created_at` vs. status `updated_at`) was tried and
/// rejected: `created_at` is set in application code
/// (`OffsetDateTime::now_utc()`) while `updated_at` is set by Postgres'
/// `now()` in a separate round-trip, so ordering between the two clock reads
/// is not guaranteed and would make this function flaky rather than correct.
/// Flagged by phase-2 review (peer-code-review-S-140-T1c-ii-v7.json, MEDIUM);
/// deliberately deferred pending a real per-language status design.
pub async fn get_subtitle_readiness_evidence(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<bool, DbError> {
    let ready: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM artifact_records
            WHERE asset_id = $1 AND kind = 'subtitle'
        ) AND EXISTS (
            SELECT 1
            FROM asset_subtitle_status
            WHERE asset_id = $1 AND status = 'ready'
        )
        "#,
    )
    .bind(asset_id.0)
    .fetch_one(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(ready)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The public repo functions are thin SQL wrappers whose meaningful behavior
    // depends on live Postgres constraints and SQLSTATE mapping, so their
    // contract is exercised in `apps/api/tests/subtitle_repo_test.rs` rather
    // than duplicated with brittle unit-level stubs here. Keep this module for
    // pure, DB-free helpers only.
    #[test]
    fn parse_subtitle_status_all_known_variants() {
        assert_eq!(
            parse_subtitle_status("pending").unwrap(),
            SubtitleStatus::Pending
        );
        assert_eq!(
            parse_subtitle_status("in_progress").unwrap(),
            SubtitleStatus::InProgress
        );
        assert_eq!(
            parse_subtitle_status("ready").unwrap(),
            SubtitleStatus::Ready
        );
        assert_eq!(
            parse_subtitle_status("failed").unwrap(),
            SubtitleStatus::Failed
        );
    }

    #[test]
    fn parse_subtitle_status_unknown_fails_closed() {
        let err = parse_subtitle_status("skipped").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "asset_subtitle_status.status",
                ..
            }
        ));
        assert!(err.to_string().contains("skipped"));
    }
}
