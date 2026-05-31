// T3: S1 repository — rights record insert
use sqlx::PgPool;

use dubbridge_domain::rights::RightsRecord;

use crate::error::DbError;

pub async fn insert_rights_record(pool: &PgPool, record: &RightsRecord) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO rights_records (id, asset_id, owner, license_type, source_type, proof_reference, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(record.id)
    .bind(record.asset_id.0)
    .bind(&record.owner)
    .bind(record.license_type.to_string())
    .bind(record.source_type.to_string())
    .bind(&record.proof_reference)
    .bind(record.created_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}
