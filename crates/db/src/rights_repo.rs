// T3: S1 repository — rights record insert
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::{
    asset::AssetId,
    rights::{LicenseType, RightsRecord, SourceType},
};

use crate::error::DbError;

#[derive(sqlx::FromRow)]
struct RightsRecordRow {
    id: Uuid,
    asset_id: Uuid,
    owner: String,
    license_type: String,
    source_type: String,
    proof_reference: String,
    created_at: OffsetDateTime,
}

fn parse_license_type(value: &str) -> Result<LicenseType, DbError> {
    match value {
        "all_rights_reserved" => Ok(LicenseType::AllRightsReserved),
        "creative_commons" => Ok(LicenseType::CreativeCommons),
        "public_domain" => Ok(LicenseType::PublicDomain),
        "licensed_distribution" => Ok(LicenseType::LicensedDistribution),
        "internal_only" => Ok(LicenseType::InternalOnly),
        other => Err(DbError::UnknownStoredValue {
            field: "rights_records.license_type",
            value: other.to_owned(),
        }),
    }
}

fn parse_source_type(value: &str) -> Result<SourceType, DbError> {
    match value {
        "direct_upload" => Ok(SourceType::DirectUpload),
        "authorized_s3" => Ok(SourceType::AuthorizedS3),
        "internal_feed" => Ok(SourceType::InternalFeed),
        "licensed_source" => Ok(SourceType::LicensedSource),
        "public_domain_with_proof" => Ok(SourceType::PublicDomainWithProof),
        other => Err(DbError::UnknownStoredValue {
            field: "rights_records.source_type",
            value: other.to_owned(),
        }),
    }
}

fn row_to_rights_record(row: RightsRecordRow) -> Result<RightsRecord, DbError> {
    Ok(RightsRecord {
        id: row.id,
        asset_id: AssetId(row.asset_id),
        owner: row.owner,
        license_type: parse_license_type(&row.license_type)?,
        source_type: parse_source_type(&row.source_type)?,
        proof_reference: row.proof_reference,
        created_at: row.created_at,
    })
}

// H1-T1: transaction-aware variant for atomic finalize.
pub async fn insert_rights_record_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    record: &RightsRecord,
) -> Result<(), DbError> {
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
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

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

/// Returns the rights ledger for an owned asset in chronological order.
/// Fails closed with `DbError::NotFound` when the asset does not exist or is not
/// owned by `owner_id`.
pub async fn list_rights_records_for_owned_asset(
    pool: &PgPool,
    asset_id: AssetId,
    owner_id: Uuid,
) -> Result<Vec<RightsRecord>, DbError> {
    let owned: Option<i32> =
        sqlx::query_scalar("SELECT 1 FROM assets WHERE id = $1 AND uploader_id = $2")
            .bind(asset_id.0)
            .bind(owner_id)
            .fetch_optional(pool)
            .await
            .map_err(DbError::QueryFailed)?;

    if owned.is_none() {
        return Err(DbError::NotFound);
    }

    let rows = sqlx::query_as::<_, RightsRecordRow>(
        r#"
        SELECT id, asset_id, owner, license_type, source_type, proof_reference, created_at
        FROM rights_records
        WHERE asset_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(asset_id.0)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter().map(row_to_rights_record).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_license_type_known_variants_succeed() {
        assert!(matches!(
            parse_license_type("all_rights_reserved"),
            Ok(LicenseType::AllRightsReserved)
        ));
        assert!(matches!(
            parse_license_type("internal_only"),
            Ok(LicenseType::InternalOnly)
        ));
    }

    #[test]
    fn parse_license_type_unknown_value_fails_closed() {
        let err = parse_license_type("temporary").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "rights_records.license_type",
                ..
            }
        ));
    }

    #[test]
    fn parse_source_type_known_variants_succeed() {
        assert!(matches!(
            parse_source_type("direct_upload"),
            Ok(SourceType::DirectUpload)
        ));
        assert!(matches!(
            parse_source_type("public_domain_with_proof"),
            Ok(SourceType::PublicDomainWithProof)
        ));
    }

    #[test]
    fn parse_source_type_unknown_value_fails_closed() {
        let err = parse_source_type("podcast_feed").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "rights_records.source_type",
                ..
            }
        ));
    }

    #[test]
    fn row_to_rights_record_round_trips_valid_values() {
        let asset_id = Uuid::new_v4();
        let row = RightsRecordRow {
            id: Uuid::new_v4(),
            asset_id,
            owner: "Acme".to_string(),
            license_type: "creative_commons".to_string(),
            source_type: "licensed_source".to_string(),
            proof_reference: "proof-001".to_string(),
            created_at: OffsetDateTime::now_utc(),
        };

        let record = row_to_rights_record(row).expect("record");
        assert_eq!(record.asset_id, AssetId(asset_id));
        assert_eq!(record.owner, "Acme");
        assert_eq!(record.license_type, LicenseType::CreativeCommons);
        assert_eq!(record.source_type, SourceType::LicensedSource);
    }
}
