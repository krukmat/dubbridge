use dubbridge_domain::rights::{LicenseType, RightsBasis, SourceType};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;

// T1-T2: default TTL applied at insert time; expressed here so it is the single source of truth.
pub const PENDING_INGESTION_TTL_HOURS: i64 = 24;

#[derive(Debug, Clone)]
pub struct PendingIngestionRecord {
    pub ingest_token: Uuid,
    pub title: String,
    pub storage_key: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub checksum: String,
    pub rights_basis: Option<RightsBasis>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub expires_at: OffsetDateTime, // T1-T2
}

pub async fn insert_pending_ingestion(
    pool: &PgPool,
    record: &PendingIngestionRecord,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO pending_ingestions (
            ingest_token, title, storage_key, content_type, file_size_bytes, checksum,
            rights_owner, license_type, source_type, proof_reference,
            created_at, updated_at, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
    )
    .bind(record.ingest_token)
    .bind(&record.title)
    .bind(&record.storage_key)
    .bind(&record.content_type)
    .bind(record.file_size_bytes)
    .bind(&record.checksum)
    .bind(
        record
            .rights_basis
            .as_ref()
            .map(|basis| basis.owner.clone()),
    )
    .bind(
        record
            .rights_basis
            .as_ref()
            .map(|basis| basis.license_type.to_string()),
    )
    .bind(
        record
            .rights_basis
            .as_ref()
            .map(|basis| basis.source_type.to_string()),
    )
    .bind(
        record
            .rights_basis
            .as_ref()
            .map(|basis| basis.proof_reference.clone()),
    )
    .bind(record.created_at)
    .bind(record.updated_at)
    .bind(record.expires_at) // T1-T2
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn attach_rights(
    pool: &PgPool,
    ingest_token: Uuid,
    rights_basis: &RightsBasis,
) -> Result<bool, DbError> {
    let result = sqlx::query(
        r#"
        UPDATE pending_ingestions
        SET rights_owner = $1, license_type = $2, source_type = $3, proof_reference = $4, updated_at = $5
        WHERE ingest_token = $6
        "#,
    )
    .bind(&rights_basis.owner)
    .bind(rights_basis.license_type.to_string())
    .bind(rights_basis.source_type.to_string())
    .bind(&rights_basis.proof_reference)
    .bind(OffsetDateTime::now_utc())
    .bind(ingest_token)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(result.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
struct PendingIngestionRow {
    ingest_token: Uuid,
    title: String,
    storage_key: String,
    content_type: String,
    file_size_bytes: i64,
    checksum: String,
    rights_owner: Option<String>,
    license_type: Option<String>,
    source_type: Option<String>,
    proof_reference: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    expires_at: OffsetDateTime, // T1-T2
}

pub async fn find_pending_ingestion(
    pool: &PgPool,
    ingest_token: Uuid,
) -> Result<Option<PendingIngestionRecord>, DbError> {
    let row = sqlx::query_as::<_, PendingIngestionRow>(
        r#"
        SELECT
            ingest_token, title, storage_key, content_type, file_size_bytes, checksum,
            rights_owner, license_type, source_type, proof_reference,
            created_at, updated_at, expires_at
        FROM pending_ingestions
        WHERE ingest_token = $1
        "#,
    )
    .bind(ingest_token)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(TryInto::try_into).transpose()
}

// T1-T2: returns all sessions where expires_at is in the past.
pub async fn list_expired_pending_ingestions(
    pool: &PgPool,
) -> Result<Vec<PendingIngestionRecord>, DbError> {
    let rows = sqlx::query_as::<_, PendingIngestionRow>(
        r#"
        SELECT
            ingest_token, title, storage_key, content_type, file_size_bytes, checksum,
            rights_owner, license_type, source_type, proof_reference,
            created_at, updated_at, expires_at
        FROM pending_ingestions
        WHERE expires_at < NOW()
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn delete_pending_ingestion(pool: &PgPool, ingest_token: Uuid) -> Result<(), DbError> {
    sqlx::query(
        r#"
        DELETE FROM pending_ingestions WHERE ingest_token = $1
        "#,
    )
    .bind(ingest_token)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

impl TryFrom<PendingIngestionRow> for PendingIngestionRecord {
    type Error = DbError;

    fn try_from(row: PendingIngestionRow) -> Result<Self, Self::Error> {
        let rights_basis = match (
            row.rights_owner,
            row.license_type,
            row.source_type,
            row.proof_reference,
        ) {
            (Some(owner), Some(license_type), Some(source_type), Some(proof_reference)) => {
                Some(RightsBasis {
                    owner,
                    license_type: parse_license_type(&license_type)?,
                    source_type: parse_source_type(&source_type)?,
                    proof_reference,
                })
            }
            (None, None, None, None) => None,
            _ => {
                return Err(DbError::QueryFailed(sqlx::Error::Protocol(
                    "pending_ingestions rights columns are partially populated".into(),
                )));
            }
        };

        Ok(Self {
            ingest_token: row.ingest_token,
            title: row.title,
            storage_key: row.storage_key,
            content_type: row.content_type,
            file_size_bytes: row.file_size_bytes,
            checksum: row.checksum,
            rights_basis,
            created_at: row.created_at,
            updated_at: row.updated_at,
            expires_at: row.expires_at, // T1-T2
        })
    }
}

fn parse_license_type(value: &str) -> Result<LicenseType, DbError> {
    match value {
        "all_rights_reserved" => Ok(LicenseType::AllRightsReserved),
        "creative_commons" => Ok(LicenseType::CreativeCommons),
        "public_domain" => Ok(LicenseType::PublicDomain),
        "licensed_distribution" => Ok(LicenseType::LicensedDistribution),
        "internal_only" => Ok(LicenseType::InternalOnly),
        _ => Err(DbError::QueryFailed(sqlx::Error::Protocol(
            format!("unknown license_type '{value}'").into(),
        ))),
    }
}

fn parse_source_type(value: &str) -> Result<SourceType, DbError> {
    match value {
        "direct_upload" => Ok(SourceType::DirectUpload),
        "authorized_s3" => Ok(SourceType::AuthorizedS3),
        "internal_feed" => Ok(SourceType::InternalFeed),
        "licensed_source" => Ok(SourceType::LicensedSource),
        "public_domain_with_proof" => Ok(SourceType::PublicDomainWithProof),
        _ => Err(DbError::QueryFailed(sqlx::Error::Protocol(
            format!("unknown source_type '{value}'").into(),
        ))),
    }
}
