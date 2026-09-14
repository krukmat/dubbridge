// MVP0-P2P P2.T5c: durable ready evidence + authoritative descriptor read model.

use dubbridge_domain::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId},
    p2p_ready_descriptor::{
        P2pReadyDescriptor, P2pReadyDescriptorInput, is_lower_hex_sha256,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DbError;

#[derive(sqlx::FromRow)]
struct ReadyDescriptorRow {
    asset_id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
    manifest_digest_sha256: String,
    external_publication_id: String,
    sealed_kek_id: String,
    sealed_kek_version: i32,
    ready_at: String,
}

/// Persist the manifest digest only after Availability Node evidence has already
/// matched the exact request identity and digest. Repeating the same digest is
/// idempotent; a conflicting digest or lineage fails closed.
pub async fn persist_confirmed_manifest_digest(
    pool: &PgPool,
    publication_id: P2pPublicationId,
    lineage_id: K1LineageId,
    manifest_digest_sha256: &str,
) -> Result<(), DbError> {
    if !is_lower_hex_sha256(manifest_digest_sha256) {
        return Err(DbError::Conflict);
    }

    let result = sqlx::query(
        r#"
        UPDATE p2p_publications
           SET manifest_digest_sha256 = $3,
               updated_at = now()
         WHERE id = $1
           AND lineage_id = $2
           AND state IN ('publishing', 'reconciling')
           AND (
                manifest_digest_sha256 IS NULL
                OR manifest_digest_sha256 = $3
           )
        "#,
    )
    .bind(publication_id.0)
    .bind(lineage_id.0)
    .bind(manifest_digest_sha256)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    if result.rows_affected() == 1 {
        return Ok(());
    }

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM p2p_publications WHERE id = $1)",
    )
    .bind(publication_id.0)
    .fetch_one(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    if exists {
        Err(DbError::Conflict)
    } else {
        Err(DbError::NotFound)
    }
}

/// Return the downstream handoff only from authoritative PostgreSQL Ready state
/// after same-lineage confirmation, durable manifest evidence, sealed K1 metadata
/// and completed delivery of the matching outbox obligation. Any partial/legacy
/// row intentionally remains invisible.
pub async fn get_ready_descriptor_by_asset(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<Option<P2pReadyDescriptor>, DbError> {
    let row = sqlx::query_as::<_, ReadyDescriptorRow>(
        r#"
        SELECT p.asset_id,
               p.id AS publication_id,
               p.lineage_id,
               p.manifest_digest_sha256,
               p.external_publication_id,
               p.sealed_kek_id,
               p.sealed_kek_version,
               to_char(
                   p.updated_at AT TIME ZONE 'UTC',
                   'YYYY-MM-DD"T"HH24:MI:SS.US"Z"'
               ) AS ready_at
          FROM p2p_publications p
          JOIN p2p_publication_outbox o
            ON o.publication_id = p.id
           AND o.lineage_id = p.lineage_id
         WHERE p.asset_id = $1
           AND p.state = 'ready'
           AND p.confirmed_lineage_id = p.lineage_id
           AND p.external_publication_id IS NOT NULL
           AND p.external_confirmed_at IS NOT NULL
           AND p.manifest_digest_sha256 IS NOT NULL
           AND p.sealed_kek_id IS NOT NULL
           AND p.sealed_kek_version IS NOT NULL
           AND p.sealed_nonce IS NOT NULL
           AND p.sealed_wrapped_ck IS NOT NULL
           AND p.sealed_at IS NOT NULL
           AND o.delivery_state = 'delivered'
           AND o.delivered_at IS NOT NULL
        "#,
    )
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(descriptor_from_row).transpose()
}

fn descriptor_from_row(row: ReadyDescriptorRow) -> Result<P2pReadyDescriptor, DbError> {
    P2pReadyDescriptor::try_from(P2pReadyDescriptorInput {
        asset_id: AssetId(row.asset_id),
        publication_id: P2pPublicationId(row.publication_id),
        lineage_id: K1LineageId(row.lineage_id),
        manifest_digest_sha256: row.manifest_digest_sha256,
        external_publication_id: row.external_publication_id,
        kek_id: row.sealed_kek_id,
        kek_version: row.sealed_kek_version,
        ready_at: row.ready_at,
    })
    .map_err(|_| DbError::Conflict)
}
