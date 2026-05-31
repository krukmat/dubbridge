// T3: S1 repository — asset insert and lookup
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::asset::{Asset, AssetId, IngestionStatus};

use crate::error::DbError;

pub async fn insert_asset(pool: &PgPool, asset: &Asset) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO assets (id, title, uploader_id, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(asset.id.0)
    .bind(&asset.title)
    .bind(asset.uploader_id)
    .bind(asset.status.to_string())
    .bind(asset.created_at)
    .bind(asset.updated_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn update_asset_status(
    pool: &PgPool,
    asset_id: AssetId,
    status: &IngestionStatus,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        UPDATE assets SET status = $1, updated_at = $2 WHERE id = $3
        "#,
    )
    .bind(status.to_string())
    .bind(OffsetDateTime::now_utc())
    .bind(asset_id.0)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct AssetRow {
    id: Uuid,
    title: String,
    uploader_id: Uuid,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

// H1-T2: fail-closed — unknown stored status must never silently coerce to Pending (ADR-008).
fn parse_status(s: &str) -> Result<IngestionStatus, DbError> {
    match s {
        "pending" => Ok(IngestionStatus::Pending),
        "finalized" => Ok(IngestionStatus::Finalized),
        "rejected_missing_rights" => Ok(IngestionStatus::RejectedMissingRights),
        "rejected_missing_uploader_context" => Ok(IngestionStatus::RejectedMissingUploaderContext),
        other => Err(DbError::UnknownStoredValue {
            field: "assets.status",
            value: other.to_owned(),
        }),
    }
}

// H1-T1: transaction-aware variants for atomic finalize.
pub async fn insert_asset_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    asset: &Asset,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO assets (id, title, uploader_id, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(asset.id.0)
    .bind(&asset.title)
    .bind(asset.uploader_id)
    .bind(asset.status.to_string())
    .bind(asset.created_at)
    .bind(asset.updated_at)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn update_asset_status_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    asset_id: AssetId,
    status: &IngestionStatus,
) -> Result<(), DbError> {
    sqlx::query("UPDATE assets SET status = $1, updated_at = $2 WHERE id = $3")
        .bind(status.to_string())
        .bind(OffsetDateTime::now_utc())
        .bind(asset_id.0)
        .execute(&mut **tx)
        .await
        .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn find_asset_by_id(pool: &PgPool, asset_id: AssetId) -> Result<Option<Asset>, DbError> {
    let row = sqlx::query_as::<_, AssetRow>(
        r#"
        SELECT id, title, uploader_id, status, created_at, updated_at
        FROM assets WHERE id = $1
        "#,
    )
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(|r| {
        Ok(Asset {
            id: AssetId(r.id),
            title: r.title,
            uploader_id: r.uploader_id,
            status: parse_status(&r.status)?,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    // H1-T2: parse_status must succeed for every known variant and fail for unknown values.
    #[test]
    fn parse_status_known_variants_succeed() {
        assert!(matches!(
            parse_status("pending"),
            Ok(IngestionStatus::Pending)
        ));
        assert!(matches!(
            parse_status("finalized"),
            Ok(IngestionStatus::Finalized)
        ));
        assert!(matches!(
            parse_status("rejected_missing_rights"),
            Ok(IngestionStatus::RejectedMissingRights)
        ));
        assert!(matches!(
            parse_status("rejected_missing_uploader_context"),
            Ok(IngestionStatus::RejectedMissingUploaderContext)
        ));
    }

    #[test]
    fn parse_status_unknown_value_fails_closed() {
        let err = parse_status("processing").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "assets.status",
                ..
            }
        ));
        assert!(err.to_string().contains("processing"));
    }
}
